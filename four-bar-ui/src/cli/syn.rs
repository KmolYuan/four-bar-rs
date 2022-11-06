use super::{Syn, SynCfg};
use crate::syn_method::SynMethod;
use four_bar::{cb::Codebook, mh, plot, syn, FourBar};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    time::Instant,
};

type AnyResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

enum SynErr {
    // Unsupported format
    Format,
    // Reading file error
    Io(std::io::Error),
    // Serialization error
    CsvSer(csv::Error),
    // Serialization error
    RonSer(ron::error::SpannedError),
    // Invalid linkage
    Linkage,
}

impl std::fmt::Display for SynErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Format => write!(f, "unsupported format"),
            Self::Io(e) => write!(f, "reading file error: {e}"),
            Self::CsvSer(e) => write!(f, "csv serialization error: {e}"),
            Self::RonSer(e) => write!(f, "ron serialization error: {e}"),
            Self::Linkage => write!(f, "invalid linkage"),
        }
    }
}

struct Info<'a> {
    target: Vec<[f64; 2]>,
    title: &'a str,
    mode: syn::Mode,
}

pub(super) fn syn(syn: Syn) {
    let Syn { files, method, no_parallel, cfg, cb } = syn;
    {
        let SynCfg { res, gen, pop, log } = cfg;
        println!("method={method:?}, res={res}, gen={gen}, pop={pop}, log={log}");
    }
    let cb = cb
        .map(|cb| std::env::split_paths(&cb).collect())
        .unwrap_or_default();
    let cb = load_codebook(cb).expect("Load codebook failed!");
    let mpb = MultiProgress::new();
    let run: Box<dyn Fn(PathBuf) + Send + Sync> = match method {
        SynMethod::De => Box::new(|file| run(&mpb, file, &cfg, &cb, mh::De::new())),
        SynMethod::Fa => Box::new(|file| run(&mpb, file, &cfg, &cb, mh::Fa::new())),
        SynMethod::Pso => Box::new(|file| run(&mpb, file, &cfg, &cb, mh::Pso::new())),
        SynMethod::Rga => Box::new(|file| run(&mpb, file, &cfg, &cb, mh::Rga::new())),
        SynMethod::Tlbo => Box::new(|file| run(&mpb, file, &cfg, &cb, mh::Tlbo::new())),
    };
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

fn run<S>(mpb: &MultiProgress, file: PathBuf, cfg: &SynCfg, cb: &Codebook, setting: S)
where
    S: mh::Setting,
{
    let pb = mpb.add(ProgressBar::new(cfg.gen as u64));
    let file = match file.canonicalize() {
        Ok(path) => path,
        Err(e) => {
            const STYLE: &str = "{msg}";
            pb.set_style(ProgressStyle::with_template(STYLE).unwrap());
            pb.finish_with_message(e.to_string());
            return;
        }
    };
    let Info { target, title, mode } = match info(&file, cfg.res) {
        Ok(info) => info,
        Err(e) => {
            if !matches!(e, SynErr::Format) {
                const STYLE: &str = "[{prefix}] {msg}";
                pb.set_style(ProgressStyle::with_template(STYLE).unwrap());
                pb.set_prefix(file.to_str().unwrap().to_string());
                pb.finish_with_message(e.to_string());
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
            .res(cfg.res);
        let mut s = mh::Solver::build(setting, func);
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
        let use_log = cfg.log > 0;
        let mut history = Vec::with_capacity(if use_log { cfg.gen as usize } else { 0 });
        let mut history_fb = Vec::with_capacity(if use_log { cfg.gen as usize } else { 0 });
        let s = s
            .task(|ctx| ctx.gen == cfg.gen as u64)
            .callback(|ctx| {
                if use_log && ctx.gen % cfg.log as u64 == 0 {
                    let (f, fb) = ctx.result();
                    history_fb.push(fb);
                    history.push(f);
                }
                pb.set_position(ctx.gen);
            })
            .solve()?;
        let spent_time = t0.elapsed();
        if use_log {
            history_fb
                .into_iter()
                .enumerate()
                .try_for_each(|(i, ans)| draw_midway(i, &root, title, &target, ans, cfg.res))?;
            let path = root.join(format!("{title}_history.svg"));
            let svg = plot::SVGBackend::new(&path, (800, 600));
            plot::history(svg, history)?;
        }
        let (_, ans) = s.result();
        draw_ans(root, title, &target, ans, cfg.res)?;
        let harmonic = s.func().harmonic();
        pb.finish_with_message(format!("| spent: {spent_time:?} | harmonic: {harmonic}"));
        Ok(())
    };
    if let Err(e) = f() {
        pb.finish_with_message(format!("| error: {e}"));
    }
}

fn info(path: &Path, res: usize) -> Result<Info, SynErr> {
    let target = path
        .extension()
        .and_then(OsStr::to_str)
        .ok_or(SynErr::Format)
        .and_then(|s| match s {
            "ron" => {
                let fb = std::fs::read_to_string(path)
                    .map_err(SynErr::Io)
                    .and_then(|s| ron::from_str::<FourBar>(&s).map_err(SynErr::RonSer))?;
                Some(fb.curve(res))
                    .filter(|c| c.len() > 1)
                    .ok_or(SynErr::Linkage)
            }
            "csv" | "txt" => std::fs::read_to_string(path)
                .map_err(SynErr::Io)
                .and_then(|s| crate::csv::parse_csv(&s).map_err(SynErr::CsvSer)),
            _ => Err(SynErr::Format),
        })?;
    let f = || {
        let title = path.file_stem().and_then(OsStr::to_str)?;
        let mode = Path::new(title)
            .extension()
            .and_then(OsStr::to_str)
            .and_then(|mode| match mode {
                "close" => Some(syn::Mode::Closed),
                "partial" => Some(syn::Mode::Partial),
                "open" => Some(syn::Mode::Open),
                _ => None,
            })?;
        Some(Info { target: mode.regularize(target), title, mode })
    };
    f().ok_or(SynErr::Format)
}

fn draw_midway(
    i: usize,
    root: &Path,
    title: &str,
    target: &[[f64; 2]],
    ans: FourBar,
    res: usize,
) -> AnyResult {
    let curve = Some(ans.curve(res))
        .filter(|c| c.len() > 1)
        .expect("solved error");
    let curves = [("Target", target), ("Optimized", &curve)];
    {
        let path = root.join(format!("{title}_{i}_linkage.svg"));
        let svg = plot::SVGBackend::new(&path, (800, 800));
        let opt = plot::Opt::new().fb(ans).use_dot(true);
        plot::plot2d(svg, &curves, opt)?;
    }
    Ok(())
}

fn draw_ans(
    root: PathBuf,
    title: &str,
    target: &[[f64; 2]],
    ans: FourBar,
    res: usize,
) -> AnyResult {
    {
        let path = root.join(format!("{title}_result.ron"));
        std::fs::write(path, ron::to_string(&ans)?)?;
    }
    let curve = Some(ans.curve(res))
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
