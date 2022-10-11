use super::{Syn, SynCfg};
use four_bar::{
    codebook::Codebook,
    curve, mh, plot,
    syn::{Mode, PathSyn},
    FourBar,
};
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
    mode: Mode,
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

fn load_codebook(cb: Vec<PathBuf>) -> AnyResult<Vec<Codebook>> {
    cb.into_iter()
        .map(|path| Ok(Codebook::read(std::fs::File::open(path)?)?))
        .collect()
}

fn run(mpb: &MultiProgress, file: PathBuf, cfg: &SynCfg, cb: &[Codebook]) {
    let file = file.canonicalize().unwrap();
    let pb = mpb.add(ProgressBar::new(cfg.gen));
    let info = match info(&file, cfg.n) {
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
    pb.set_prefix(info.title.to_string());
    let root = file.parent().unwrap();
    let res = if let Some(ans) = codebook(cb, &info, cfg.pop) {
        draw_ans(root, info.title, &info.target, ans, cfg.n)
    } else {
        optimize(&pb, info, root, cfg)
    };
    if let Err(e) = res {
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
                curve::from_four_bar(fb, n).ok_or(SynErr::Linkage)
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
            let mode = title
                .rsplit('.')
                .next()
                .and_then(|s| match s {
                    "close" => Some(Mode::Close),
                    "partial" => Some(Mode::Partial),
                    "open" => Some(Mode::Open),
                    _ => None,
                })
                .ok_or(SynErr::Format)?;
            Ok(Info { target, title, mode })
        })
}

fn codebook(cb: &[Codebook], info: &Info, n: usize) -> Option<FourBar> {
    use four_bar::mh::rayon::prelude::*;
    cb.into_par_iter()
        .filter(|cb| info.mode.is_target_open() == cb.is_open())
        .flat_map(|cb| cb.fetch(&info.target, n))
        .min_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap())
        .map(|(_, fb)| fb)
}

fn optimize(pb: &ProgressBar, info: Info, root: &Path, cfg: &SynCfg) -> AnyResult {
    let Info { target, title, mode } = info;
    let t0 = Instant::now();
    let func = PathSyn::from_curve_gate(&target, None, mode)
        .ok_or("invalid target")?
        .resolution(cfg.n);
    let s = mh::Solver::build(mh::De::default())
        .task(|ctx| ctx.gen == cfg.gen)
        .callback(|ctx| pb.set_position(ctx.gen))
        .pop_num(cfg.pop)
        .record(|ctx| ctx.best_f)
        .solve(func)?;
    let spent_time = Instant::now() - t0;
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
}

fn draw_ans(root: &Path, title: &str, target: &[[f64; 2]], ans: FourBar, n: usize) -> AnyResult {
    {
        let path = root.join(format!("{title}_result.ron"));
        std::fs::write(path, ron::to_string(&ans)?)?;
    }
    let [t1, t2] = ans.angle_bound().expect("solved error");
    let curve = curve::get_valid_part(ans.curve(t1, t2, n));
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
