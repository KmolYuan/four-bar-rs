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
    let Syn { files, no_parallel, cfg, codebook } = syn;
    let mpb = MultiProgress::new();
    if !codebook.is_empty() {
        mpb.println("Loading codebook database...").unwrap();
    }
    let cb = load_codebook(codebook).expect("Load codebook failed!");
    let run = |file: PathBuf| run(&mpb, file, &cfg, &cb);
    if no_parallel {
        files.into_iter().for_each(run);
    } else {
        use mh::rayon::prelude::*;
        files.into_par_iter().for_each(run);
    }
}

fn load_codebook(cb: Vec<PathBuf>) -> AnyResult<Codebook> {
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
    let mut s = mh::Solver::build(mh::De::default());
    if let Some(candi) = matches!(mode, syn::Mode::Close | syn::Mode::Open)
        .then(|| cb.fetch_raw(&target, cfg.k))
        .filter(|candi| !candi.is_empty())
    {
        let pool = candi.iter().map(|(_, fb)| fb.vec()).collect::<Vec<_>>();
        let fitness = candi.iter().map(|(f, _)| *f).collect();
        s = s.pool_and_fitness(mh::ndarray::arr2(&pool), fitness);
        s = s.pop_num(cfg.k);
    } else {
        s = s.pop_num(cfg.pop);
    }
    let f = || -> AnyResult {
        let t0 = Instant::now();
        let func = syn::PathSyn::from_curve_gate(&target, None, mode)
            .ok_or("invalid target")?
            .resolution(cfg.n);
        let s = s
            .task(|ctx| ctx.gen == cfg.gen)
            .callback(|ctx| pb.set_position(ctx.gen))
            .record(|ctx| ctx.best_f)
            .solve(func)?;
        let spent_time = Instant::now() - t0;
        let root = file.parent().unwrap();
        {
            let path = root.join(format!("{title}_history.svg"));
            let svg = plot::SVGBackend::new(&path, (800, 600));
            plot::history(svg, s.report())?;
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
                fb.curve(n).ok_or(SynErr::Linkage)
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
                    "close" => Some(syn::Mode::Close),
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

fn draw_ans(root: &Path, title: &str, target: &[[f64; 2]], ans: FourBar, n: usize) -> AnyResult {
    {
        let path = root.join(format!("{title}_result.ron"));
        std::fs::write(path, ron::to_string(&ans)?)?;
    }
    let curve = ans.curve(n).expect("solved error");
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
