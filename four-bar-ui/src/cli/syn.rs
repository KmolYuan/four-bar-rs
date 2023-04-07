use super::{Syn, SynCfg};
use crate::syn_cmd::*;
use four_bar::{
    cb::FbCodebook,
    csv, efd, mh,
    plot2d::{self, IntoDrawingArea as _},
    syn2d, FourBar,
};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{
    ffi::OsStr,
    io::Write as _,
    path::{Path, PathBuf},
    time::Instant,
};

macro_rules! impl_err_from {
    ($(impl $ty:ty => $kind:ident)+) => {$(
        impl From<$ty> for SynErr {
            fn from(e: $ty) -> Self { Self::$kind(e) }
        }
    )+};
}

#[derive(Debug)]
enum SynErr {
    Format,
    Io(std::io::Error),
    Plot(plot2d::DrawingAreaErrorKind<std::io::Error>),
    CsvSer(csv::Error),
    RonSer(ron::error::SpannedError),
    RonDe(ron::error::Error),
    Linkage,
    Solver,
}

impl std::fmt::Display for SynErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Format => write!(f, "unsupported format"),
            Self::Io(e) => write!(f, "reading file error: {e}"),
            Self::Plot(e) => write!(f, "drawing plot error: {e}"),
            Self::CsvSer(e) => write!(f, "csv serialization error: {e}"),
            Self::RonSer(e) => write!(f, "ron serialization error: {e}"),
            Self::RonDe(e) => write!(f, "ron deserialization error: {e}"),
            Self::Linkage => write!(f, "invalid linkage input"),
            Self::Solver => write!(f, "solved error"),
        }
    }
}

impl std::error::Error for SynErr {}

impl_err_from! {
    impl std::io::Error => Io
    impl plot2d::DrawingAreaErrorKind<std::io::Error> => Plot
    impl csv::Error => CsvSer
    impl ron::error::SpannedError => RonSer
    impl ron::error::Error => RonDe
}

pub(super) fn syn(syn: Syn) {
    let Syn { files, one_by_one, cfg, cb, refer, method_cmd } = syn;
    println!("{cfg}");
    let cb = cb
        .map(|cb| std::env::split_paths(&cb).collect::<Vec<_>>())
        .unwrap_or_default();
    if !cb.is_empty() {
        println!("Loading codebook database...");
    }
    let cb = cb
        .into_iter()
        .map(|path| Ok(FbCodebook::read(std::fs::File::open(path)?)?))
        .collect::<Result<FbCodebook, Box<dyn std::error::Error>>>()
        .expect("Load codebook failed!");
    let mpb = MultiProgress::new();
    let method_cmd = method_cmd.unwrap_or_default();
    let run: Box<dyn Fn(PathBuf) + Send + Sync> = match &method_cmd {
        SynCmd::De(s) => Box::new(|f| run(&mpb, f, &cfg, &cb, &refer, s.clone())),
        SynCmd::Fa(s) => Box::new(|f| run(&mpb, f, &cfg, &cb, &refer, s.clone())),
        SynCmd::Pso(s) => Box::new(|f| run(&mpb, f, &cfg, &cb, &refer, s.clone())),
        SynCmd::Rga(s) => Box::new(|f| run(&mpb, f, &cfg, &cb, &refer, s.clone())),
        SynCmd::Tlbo(s) => Box::new(|f| run(&mpb, f, &cfg, &cb, &refer, s.clone())),
    };
    if one_by_one {
        files.into_iter().for_each(run);
    } else {
        use mh::rayon::prelude::*;
        files.into_par_iter().for_each(run);
    }
}

fn run<S>(
    mpb: &MultiProgress,
    file: PathBuf,
    cfg: &SynCfg,
    cb: &FbCodebook,
    refer: &Path,
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
    let mut target_fb = None;
    let mut f = || -> Result<_, SynErr> {
        let target = file
            .extension()
            .and_then(OsStr::to_str)
            .ok_or(SynErr::Format)
            .and_then(|s| match s {
                "ron" => {
                    let fb = ron::from_str::<FourBar>(&std::fs::read_to_string(&file)?)?;
                    let curve = fb.curve(cfg.res);
                    target_fb.replace(fb);
                    Ok(curve)
                }
                "csv" | "txt" => Ok(csv::parse_csv(&std::fs::read_to_string(&file)?)?),
                _ => Err(SynErr::Format),
            })?;
        let title = file
            .file_stem()
            .and_then(OsStr::to_str)
            .ok_or(SynErr::Format)?;
        let mode = match Path::new(title).extension().and_then(OsStr::to_str) {
            Some("closed") => Ok(syn2d::Mode::Closed),
            Some("partial") => Ok(syn2d::Mode::Partial),
            Some("open") => Ok(syn2d::Mode::Open),
            _ => Err(SynErr::Format),
        }?;
        Ok((mode.regularize(target), title, mode))
    };
    let (target, title, mode) = match f() {
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
    let f = || -> Result<(), SynErr> {
        let func = syn2d::PlanarSyn::from_curve(&target, mode)
            .ok_or(SynErr::Linkage)?
            .res(cfg.res);
        let root = file.parent().unwrap().join(title);
        if root.is_dir() {
            // Avoid file browser missing opening folders
            for e in std::fs::read_dir(&root)? {
                let path = e?.path();
                if path.is_dir() {
                    std::fs::remove_dir_all(path)?;
                } else {
                    std::fs::remove_file(path)?;
                }
            }
        } else {
            std::fs::create_dir(&root)?;
        }
        let t0 = Instant::now();
        let use_log = cfg.log > 0;
        let mut history = Vec::with_capacity(if use_log { cfg.gen } else { 0 });
        let mut s = mh::Solver::build(setting, func)
            .seed(cfg.seed)
            .task(|ctx| ctx.gen == cfg.gen as u64)
            .callback(|ctx| {
                if use_log && ctx.gen % cfg.log as u64 == 0 {
                    let (_, ans) = ctx.result();
                    let curve = ans.curve(cfg.res);
                    let path = root.join(format!("{title}_{}_linkage.svg", ctx.gen));
                    let svg = plot2d::SVGBackend::new(&path, (800, 800));
                    let curves = [("Target", target.as_slice()), ("Optimized", &curve)];
                    let mut opt = plot2d::Opt::from(&ans).dot(true).font(cfg.font);
                    if let Some(angle) = cfg.angle {
                        opt = opt.angle(angle.to_radians());
                    }
                    plot2d::plot(svg, curves, opt).unwrap();
                }
                history.push(ctx.best_f);
                pb.set_position(ctx.gen);
            });
        let mut cb_fb = None;
        if let Some(candi) = matches!(mode, syn2d::Mode::Closed | syn2d::Mode::Open)
            .then(|| cb.fetch_raw(&target, cfg.pop))
            .filter(|candi| !candi.is_empty())
        {
            cb_fb.replace(candi[0].clone());
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
        let s = s.solve().unwrap();
        let (cost, ans) = s.result();
        if !ans.is_valid() {
            return Err(SynErr::Solver);
        }
        let t1 = t0.elapsed();
        {
            let path = root.join(format!("{title}_history.svg"));
            let svg = plot2d::SVGBackend::new(&path, (800, 600));
            plot2d::history(svg, history)?;
        }
        let path = root.join(format!("{title}_result.ron"));
        std::fs::write(path, ron::to_string(&ans)?)?;
        let h = s.func().harmonic();
        let curve = ans.curve(cfg.res);
        let efd_target = efd::Efd2::from_curve_harmonic(&target, h).unwrap();
        let curve_diff = if matches!(mode, syn2d::Mode::Partial) {
            efd::partial_curve_diff
        } else {
            efd::curve_diff
        };
        let err = curve_diff(&target, &mode.regularize(&curve));
        let mut curves = vec![("Target", target), ("Optimized", curve)];
        let path = root.join(format!("{title}_result.svg"));
        let svg = plot2d::SVGBackend::new(&path, (1600, 800));
        let (root_l, root_r) = svg.into_drawing_area().split_horizontally(800);
        let mut opt = plot2d::Opt::from(&ans)
            .dot(true)
            .axis(false)
            .font(cfg.font)
            .scale_bar(true);
        if let Some(angle) = cfg.angle {
            opt = opt.angle(angle.to_radians());
        }
        plot2d::plot(root_l, curves.iter().map(|(s, c)| (*s, c.as_slice())), opt)?;
        let mut log = std::fs::File::create(root.join(format!("{title}.log")))?;
        writeln!(log, "[{title}]")?;
        if let Some(fb) = target_fb {
            writeln!(log, "\n[target.fb]")?;
            log_fb(&mut log, &fb)?;
        }
        if let Some((cost, fb)) = cb_fb {
            let c = fb.curve(cfg.res);
            let efd = efd::Efd2::from_curve_harmonic(mode.regularize(c), h).unwrap();
            let trans = efd.as_trans().to(efd_target.as_trans());
            let fb = FourBar::from(fb).transform(&trans);
            let c = fb.curve(cfg.res);
            let err = curve_diff(&curves[0].1, &mode.regularize(&c));
            writeln!(log, "\n[catalog]")?;
            writeln!(log, "harmonic={}", cb.harmonic())?;
            writeln!(log, "error={err}")?;
            writeln!(log, "cost={cost}")?;
            writeln!(log, "\n[catalog.fb]")?;
            log_fb(&mut log, &fb)?;
            curves.push(("Catalog", c));
        }
        writeln!(log, "\n[optimized]")?;
        writeln!(log, "time={t1:?}")?;
        writeln!(log, "harmonic={h}")?;
        writeln!(log, "error={err}")?;
        writeln!(log, "cost={cost}")?;
        writeln!(log, "\n[optimized.fb]")?;
        log_fb(&mut log, &ans)?;
        let refer = root
            .parent()
            .unwrap()
            .join(refer)
            .join(format!("{title}.ron"));
        if let Ok(s) = std::fs::read_to_string(refer) {
            let fb = ron::from_str::<FourBar>(&s)?;
            let c = fb.curve(cfg.res);
            let err = curve_diff(&curves[0].1, &mode.regularize(&c));
            writeln!(log, "\n[competitor]")?;
            writeln!(log, "error={err}")?;
            if !matches!(mode, syn2d::Mode::Partial) {
                let efd = efd::Efd2::from_curve_harmonic(mode.regularize(&c), h).unwrap();
                let cost = efd.l1_norm(&efd_target);
                writeln!(log, "cost={cost}")?;
            }
            writeln!(log, "\n[competitor.fb]")?;
            log_fb(&mut log, &fb)?;
            curves.push(("Competitor", c));
        }
        let opt = plot2d::Opt::new().dot(true).font(cfg.font);
        plot2d::plot(root_r, curves.iter().map(|(s, c)| (*s, c.as_slice())), opt)?;
        log.flush()?;
        pb.finish();
        Ok(())
    };
    if let Err(e) = f() {
        pb.finish_with_message(format!("| error: {e}"));
    }
}

fn log_fb(mut w: impl std::io::Write, fb: &FourBar) -> std::io::Result<()> {
    macro_rules! impl_fmt {
        ($($field:ident),+) => {$(
            writeln!(w, concat![stringify!($field), "={}"], fb.$field())?;
        )+};
    }
    impl_fmt!(p0x, p0y, a, l0, l1, l2, l3, l4, g, inv);
    Ok(())
}
