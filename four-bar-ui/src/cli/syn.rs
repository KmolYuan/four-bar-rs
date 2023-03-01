use super::{Syn, SynCfg};
use crate::syn_cmd::*;
use four_bar::{
    cb::FbCodebook,
    csv, efd,
    mh::{self, rayon::single_thread},
    planar_syn, plot2d, FourBar,
};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    time::Instant,
};

type AnyResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
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

impl std::error::Error for SynErr {}

struct Info<'a> {
    target: Vec<[f64; 2]>,
    title: &'a str,
    mode: planar_syn::Mode,
}

pub(super) fn syn(syn: Syn) {
    let Syn { files, one_by_one, cfg, cb, refer, method_cmd } = syn;
    {
        let SynCfg { res, gen, pop, seed, log } = cfg;
        if let Some(seed) = seed {
            print!("seed={seed} ");
        }
        println!("res={res}, gen={gen}, pop={pop}, log={log}");
    }
    let cb = cb
        .map(|cb| std::env::split_paths(&cb).collect())
        .unwrap_or_default();
    let cb = load_codebook(cb).expect("Load codebook failed!");
    let refer = std::env::split_paths(&refer).collect::<Vec<_>>();
    let mpb = MultiProgress::new();
    let method_cmd = method_cmd.unwrap_or_default();
    let run: Box<dyn Fn(PathBuf) + Send + Sync> = match &method_cmd {
        SynCmd::De(s) => Box::new(|f| run(&mpb, f, &cfg, &cb, &refer, s.clone())),
        SynCmd::Fa(s) => Box::new(|f| run(&mpb, f, &cfg, &cb, &refer, s.clone())),
        SynCmd::Pso(s) => Box::new(|f| run(&mpb, f, &cfg, &cb, &refer, s.clone())),
        SynCmd::Rga(s) => Box::new(|f| run(&mpb, f, &cfg, &cb, &refer, s.clone())),
        SynCmd::Tlbo(s) => Box::new(|f| run(&mpb, f, &cfg, &cb, &refer, s.clone())),
    };
    use mh::rayon::prelude::*;
    single_thread(one_by_one, || files.into_par_iter().for_each(run));
}

fn load_codebook(cb: Vec<PathBuf>) -> AnyResult<FbCodebook> {
    if !cb.is_empty() {
        println!("Loading codebook database...");
    }
    cb.into_iter()
        .map(|path| Ok(FbCodebook::read(std::fs::File::open(path)?)?))
        .collect()
}

fn run<S>(
    mpb: &MultiProgress,
    file: PathBuf,
    cfg: &SynCfg,
    cb: &FbCodebook,
    refer: &[PathBuf],
    setting: S,
) where
    S: mh::Setting + Send,
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
        let func = planar_syn::PlanarSyn::from_curve(&target, mode)
            .ok_or("invalid target")?
            .res(cfg.res);
        let root = file.parent().unwrap().join(title);
        if root.is_dir() {
            std::fs::remove_dir_all(&root)?;
        }
        std::fs::create_dir(&root)?;
        let use_log = cfg.log > 0;
        let mut history = Vec::with_capacity(if use_log { cfg.gen } else { 0 });
        let mut s = mh::Solver::build(setting, func)
            .seed(cfg.seed)
            .task(|ctx| ctx.gen == cfg.gen as u64)
            .callback(|ctx| {
                if use_log && ctx.gen % cfg.log as u64 == 0 {
                    let (_, ans) = ctx.result();
                    let _ = draw_midway(ctx.gen, &root, title, &target, ans, cfg.res);
                }
                history.push(ctx.best_f);
                pb.set_position(ctx.gen);
            });
        if let Some(candi) = matches!(mode, planar_syn::Mode::Closed | planar_syn::Mode::Open)
            .then(|| cb.fetch_raw(&target, cfg.pop))
            .filter(|candi| !candi.is_empty())
        {
            s = s.pop_num(candi.len());
            let fitness = candi.iter().map(|(f, _)| *f).collect();
            let pool = candi
                .into_iter()
                .map(|(_, fb)| fb.as_array())
                .collect::<Vec<_>>();
            s = s.pool_and_fitness(mh::ndarray::arr2(&pool), fitness);
        } else {
            s = s.pop_num(cfg.pop);
        }
        let t0 = Instant::now();
        let s = s.solve_single_thread(false)?;
        let t1 = t0.elapsed();
        {
            let path = root.join(format!("{title}_history.svg"));
            let svg = plot2d::SVGBackend::new(&path, (800, 600));
            plot2d::history(svg, history)?;
        }
        let (err, ans) = s.result();
        {
            let path = root.join(format!("{title}_result.ron"));
            std::fs::write(path, ron::to_string(&ans)?)?;
        }
        let curve = Some(ans.curve(cfg.res))
            .filter(|c| c.len() > 1)
            .ok_or(format!("solved error: {:?}", &ans))?;
        let h = s.func().harmonic();
        let efd_target = efd::Efd2::from_curve_harmonic(&target, h).unwrap();
        let err = match mode {
            planar_syn::Mode::Partial => {
                let efd = efd::Efd2::from_curve_harmonic(mode.regularize(&curve), h).unwrap();
                efd_target.l1_norm(&efd)
            }
            _ => err,
        };
        let legend = format!("Synthesized ({err:.04})");
        let mut curves = vec![("Target", target.as_slice()), (&legend, &curve)];
        {
            let path = root.join(format!("{title}_linkage.svg"));
            let svg = plot2d::SVGBackend::new(&path, (800, 800));
            let opt = plot2d::Opt::new().fb(ans).use_dot(true);
            plot2d::plot(svg, curves.clone(), opt)?;
        }
        let refer = refer
            .iter()
            .map(|f| file.parent().unwrap().join(f).join(format!("{title}.ron")))
            .filter(|f| f.is_file())
            .map(std::fs::read_to_string)
            .map(|s| -> Result<_, SynErr> {
                let s = s.map_err(SynErr::Io)?;
                let fb = ron::from_str::<FourBar>(&s).map_err(SynErr::RonSer)?;
                let c = fb.curve(cfg.res);
                let efd = efd::Efd2::from_curve_harmonic(mode.regularize(&c), h).unwrap();
                let err = efd_target.l1_norm(&efd);
                Ok((format!("Competitor ({err:.04})"), c))
            })
            .collect::<Result<Vec<_>, _>>()?;
        curves.extend(refer.iter().map(|(s, c)| (s.as_str(), c.as_slice())));
        {
            let path = root.join(format!("{title}_result.svg"));
            let svg = plot2d::SVGBackend::new(&path, (800, 800));
            let title = format!("Harmonic: {h} | Time: {t1:?}");
            let opt = plot2d::Opt::new().use_dot(true).title(&title);
            plot2d::plot(svg, curves, opt)?;
        }
        pb.finish();
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
                .and_then(|s| csv::parse_csv(&s).map_err(SynErr::CsvSer)),
            _ => Err(SynErr::Format),
        })?;
    let f = || {
        let title = path.file_stem().and_then(OsStr::to_str)?;
        let mode = Path::new(title)
            .extension()
            .and_then(OsStr::to_str)
            .and_then(|mode| match mode {
                "closed" => Some(planar_syn::Mode::Closed),
                "partial" => Some(planar_syn::Mode::Partial),
                "open" => Some(planar_syn::Mode::Open),
                _ => None,
            })?;
        Some(Info { target: mode.regularize(target), title, mode })
    };
    f().ok_or(SynErr::Format)
}

fn draw_midway(
    i: u64,
    root: &Path,
    title: &str,
    target: &[[f64; 2]],
    ans: FourBar,
    res: usize,
) -> AnyResult {
    let curve = Some(ans.curve(res))
        .filter(|c| c.len() > 1)
        .ok_or(format!("solved error: {:?}", &ans))?;
    let curves = [("Target", target), ("Synthesized", &curve)];
    {
        let path = root.join(format!("{title}_{i}_linkage.svg"));
        let svg = plot2d::SVGBackend::new(&path, (800, 800));
        let opt = plot2d::Opt::new().fb(ans).use_dot(true);
        plot2d::plot(svg, curves, opt)?;
    }
    Ok(())
}
