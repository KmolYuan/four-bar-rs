use super::{Syn, SynCfg};
use four_bar::{cb::Codebook, mh, plot, syn, FourBar};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{
    path::{Path, PathBuf},
    time::Instant,
};

type AnyResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

enum SynErr {
    // Unsupported format
    Format,
    // Reading file error
    Io,
    // Serialization error
    Ser,
    // Invalid linkage
    Linkage,
}

impl std::fmt::Display for SynErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            Self::Format => "unsupported format",
            Self::Io => "reading file error",
            Self::Ser => "serialization error",
            Self::Linkage => "invalid linkage",
        };
        f.write_str(s)
    }
}

struct Info<'a> {
    target: Vec<[f64; 2]>,
    title: &'a str,
    mode: syn::Mode,
}

pub(super) fn syn(syn: Syn) {
    let Syn { files, no_parallel, cfg, cb } = syn;
    let cb = cb
        .map(|cb| std::env::split_paths(&cb).collect())
        .unwrap_or_default();
    let cb = load_codebook(cb).expect("Load codebook failed!");
    let mpb = MultiProgress::new();
    let run = |file: PathBuf| run(&mpb, file, &cfg, &cb);
    if no_parallel {
        files.into_iter().for_each(run);
    } else {
        use mh::rayon::prelude::*;
        files.into_par_iter().for_each(run);
    }
}

fn load_codebook(cb: Vec<PathBuf>) -> AnyResult<Codebook> {
    if !cb.is_empty() {
        println!("Loading codebook database...");
    }
    cb.into_iter()
        .map(|path| Ok(Codebook::read(std::fs::File::open(path)?)?))
        .collect()
}

fn run(mpb: &MultiProgress, file: PathBuf, cfg: &SynCfg, cb: &Codebook) {
    let file = file.canonicalize().unwrap();
    let pb = mpb.add(ProgressBar::new(cfg.gen));
    let Info { target, title, mode } = match info(&file, cfg.n) {
        Ok(info) => info,
        Err(e) => {
            if !matches!(e, SynErr::Format) {
                let title = file.to_str().unwrap().to_string();
                const STYLE: &str = "[{prefix}] {msg}";
                pb.set_style(ProgressStyle::with_template(STYLE).unwrap());
                pb.set_prefix(title);
                pb.set_message(e.to_string());
            }
            return;
        }
    };
    const STYLE: &str = "[{prefix}] {elapsed_precise} {wide_bar} {pos}/{len} {msg}";
    pb.set_style(ProgressStyle::with_template(STYLE).unwrap());
    pb.set_prefix(title.to_string());
    let f = || -> AnyResult {
        let func = syn::PathSyn::from_curve(&target, mode)
            .ok_or("invalid target")?
            .resolution(cfg.n);
        let mut s = mh::Solver::build(mh::De::default(), func);
        if let Some(candi) = matches!(mode, syn::Mode::Closed | syn::Mode::Open)
            .then(|| cb.fetch_raw(&target, cfg.pop))
            .filter(|candi| !candi.is_empty())
        {
            s = s.pop_num(candi.len());
            let fitness = candi.iter().map(|(f, _)| *f).collect();
            let pool = candi
                .into_iter()
                .map(|(_, fb)| fb.to_norm().vec())
                .collect::<Vec<_>>();
            s = s.pool_and_fitness(mh::ndarray::arr2(&pool), fitness);
        } else {
            s = s.pop_num(cfg.pop);
        }
        let t0 = Instant::now();
        let root = file.parent().unwrap().join(title);
        if root.is_dir() {
            std::fs::remove_dir_all(&root)?;
        }
        std::fs::create_dir(&root)?;
        let mut history = Vec::with_capacity(if cfg.log { cfg.gen as usize } else { 0 });
        let mut history_fb = Vec::with_capacity(if cfg.log { cfg.gen as usize } else { 0 });
        let s = s
            .task(|ctx| ctx.gen == cfg.gen)
            .callback(|ctx| {
                if cfg.log {
                    let (f, fb) = ctx.result();
                    history_fb.push(fb);
                    history.push(f);
                }
                pb.set_position(ctx.gen);
            })
            .solve()?;
        let spent_time = Instant::now() - t0;
        if cfg.log {
            history_fb
                .into_iter()
                .enumerate()
                .try_for_each(|(i, ans)| draw_midway(i, &root, title, &target, ans, cfg.n))?;
            let path = root.join(format!("{title}_history.svg"));
            let svg = plot::SVGBackend::new(&path, (800, 600));
            plot::history(svg, history)?;
        }
        let (_, ans) = s.result();
        draw_ans(root, title, &target, ans, cfg.n)?;
        let harmonic = s.func().harmonic();
        pb.finish_with_message(format!("| spent: {spent_time:?} | harmonic: {harmonic}"));
        Ok(())
    };
    if let Err(e) = f() {
        pb.finish_with_message(format!("| error: {e}"));
    }
}

fn info(path: &Path, n: usize) -> Result<Info, SynErr> {
    let target = path
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .ok_or(SynErr::Format)
        .and_then(|s| match s {
            "ron" => {
                let fb = std::fs::read_to_string(path)
                    .map_err(|_| SynErr::Io)
                    .and_then(|s| ron::from_str::<FourBar>(&s).map_err(|_| SynErr::Ser))?;
                Some(fb.curve(n))
                    .filter(|c| c.len() > 1)
                    .ok_or(SynErr::Linkage)
            }
            "csv" | "txt" => std::fs::read_to_string(path)
                .map_err(|_| SynErr::Io)
                .and_then(|s| crate::csv::parse_csv(&s).map_err(|_| SynErr::Ser)),
            _ => Err(SynErr::Format),
        })?;
    path.file_stem()
        .and_then(std::ffi::OsStr::to_str)
        .ok_or(SynErr::Format)
        .and_then(|title| {
            title
                .rsplit('.')
                .next()
                .and_then(|s| match s {
                    "close" => Some(syn::Mode::Closed),
                    "partial" => Some(syn::Mode::Partial),
                    "open" => Some(syn::Mode::Open),
                    _ => None,
                })
                .map(|mode| {
                    let target = mode.regularize(target);
                    Info { target, title, mode }
                })
                .ok_or(SynErr::Format)
        })
}

fn draw_midway(
    i: usize,
    root: &Path,
    title: &str,
    target: &[[f64; 2]],
    ans: FourBar,
    n: usize,
) -> AnyResult {
    let curve = Some(ans.curve(n))
        .filter(|c| c.len() > 1)
        .expect("solved error");
    let curves = [("Target", target), ("Optimized", &curve)];
    {
        let path = root.join(format!("{title}_{i}_linkage.svg"));
        let svg = plot::SVGBackend::new(&path, (800, 800));
        let opt = plot::Opt::new().fb(ans).use_dot(true).title("Linkage");
        plot::plot2d(svg, &curves, opt)?;
    }
    Ok(())
}

fn draw_ans(root: PathBuf, title: &str, target: &[[f64; 2]], ans: FourBar, n: usize) -> AnyResult {
    {
        let path = root.join(format!("{title}_result.ron"));
        std::fs::write(path, ron::to_string(&ans)?)?;
    }
    let curve = Some(ans.curve(n))
        .filter(|c| c.len() > 1)
        .expect("solved error");
    let curves = [("Target", target), ("Optimized", &curve)];
    {
        let path = root.join(format!("{title}_linkage.svg"));
        let svg = plot::SVGBackend::new(&path, (800, 800));
        let opt = plot::Opt::new().fb(ans).use_dot(true).title("Linkage");
        plot::plot2d(svg, &curves, opt)?;
    }
    {
        let path = root.join(format!("{title}_result.svg"));
        let svg = plot::SVGBackend::new(&path, (800, 800));
        let opt = plot::Opt::new().use_dot(true).title("Comparison");
        plot::plot2d(svg, &curves, opt)?;
    }
    Ok(())
}
