use super::{Syn, SynCfg};
use crate::{io, syn_cmd};
use four_bar::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    borrow::Cow,
    ffi::OsStr,
    io::Write as _,
    path::{Path, PathBuf},
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

struct Info {
    root: PathBuf,
    target: io::Curve,
    target_fb: Option<io::Fb>,
    title: String,
    mode: syn::Mode,
}

pub(super) fn syn(syn: Syn) {
    let Syn { files, one_by_one, cfg, cb, refer, method } = syn;
    println!("{cfg}");
    println!("-----");
    // Load target files & create project folders
    let files = files
        .into_iter()
        .filter_map(|file| file.canonicalize().ok())
        .map(|file| {
            let mut target_fb = None;
            let target = file
                .extension()
                .and_then(OsStr::to_str)
                .ok_or(SynErr::Format)
                .and_then(|s| match s {
                    "ron" => {
                        let fb = ron::de::from_reader::<_, io::Fb>(std::fs::File::open(&file)?)?;
                        let curve = fb.curve(cfg.inner.res);
                        target_fb.replace(fb);
                        Ok(curve)
                    }
                    "csv" | "txt" => Ok(io::Curve::from_reader(std::fs::File::open(&file)?)?),
                    _ => Err(SynErr::Format),
                })?;
            match &target {
                io::Curve::P(t) => _ = efd::valid_curve(t).ok_or(SynErr::Linkage)?,
                io::Curve::S(t) => _ = efd::valid_curve(t).ok_or(SynErr::Linkage)?,
            }
            let title = file
                .file_stem()
                .ok_or(SynErr::Format)?
                .to_string_lossy()
                .into_owned();
            let mode = match Path::new(&title).extension().and_then(OsStr::to_str) {
                Some("closed") => syn::Mode::Closed,
                Some("partial") => syn::Mode::Partial,
                Some("open") => syn::Mode::Open,
                _ => Err(SynErr::Format)?,
            };
            let root = file.parent().unwrap().join(&title);
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
            Ok(Info { root, target, target_fb, title, mode })
        })
        .filter_map(|r| match r {
            Ok(r) => Some(r),
            Err(SynErr::Format) => None,
            Err(e) => {
                println!("Error: {e}");
                None
            }
        })
        .collect::<Vec<_>>();
    // Load codebook
    let cb = cb
        .map(|cb| std::env::split_paths(&cb).collect::<Vec<_>>())
        .unwrap_or_default();
    if !cb.is_empty() {
        println!("Loading codebook database...");
    }
    let cb = cb
        .into_iter()
        .map(|path| Ok(io::Cb::from_reader(std::fs::File::open(path)?)?))
        .collect::<Result<io::CbPool, Box<dyn std::error::Error>>>()
        .expect("Load codebook failed!");
    // Progress bar
    const STYLE: &str = "{eta} {wide_bar} {percent}%";
    let pb = ProgressBar::new(files.len() as u64 * cfg.gen);
    pb.set_style(ProgressStyle::with_template(STYLE).unwrap());
    // Tasks
    let method = method.unwrap_or_default();
    let run = |info| run(&pb, method.clone(), info, &cfg, &cb, &refer);
    let t0 = std::time::Instant::now();
    if one_by_one {
        files.into_iter().for_each(run);
    } else {
        use mh::rayon::prelude::*;
        files.into_par_iter().for_each(run);
    }
    pb.finish_and_clear();
    println!("-----");
    println!("Finished in {:?}", t0.elapsed());
}

fn run(
    pb: &ProgressBar,
    method: syn_cmd::SynMethod,
    info: Info,
    cfg: &SynCfg,
    cb: &io::CbPool,
    refer: &Path,
) {
    let title = &info.title;
    match try_run(pb, method, &info, cfg, cb, refer) {
        Ok(()) => pb.println(format!("Finished: {title}")),
        Err(e) => pb.println(format!("Error in {title}: {e}")),
    }
}

fn try_run(
    pb: &ProgressBar,
    method: syn_cmd::SynMethod,
    info: &Info,
    cfg: &SynCfg,
    cb: &io::CbPool,
    refer: &Path,
) -> Result<(), SynErr> {
    let Info { root, target, target_fb, title, mode } = info;
    let mut history = Vec::with_capacity(cfg.gen as usize);
    let t0 = std::time::Instant::now();
    let s = {
        let mut cfg = cfg.inner.clone();
        cfg.mode = *mode;
        let target = match &target {
            io::Curve::P(t) => syn_cmd::Target::P(Cow::Borrowed(t), Cow::Borrowed(cb.as_fb())),
            io::Curve::S(t) => syn_cmd::Target::S(Cow::Borrowed(t), Cow::Borrowed(cb.as_sfb())),
        };
        syn_cmd::Solver::new(method, target, cfg, |best_f, _| {
            history.push(best_f);
            pb.inc(1);
        })
    };
    let (cost, h, result_fb) = s.solve_verbose().unwrap();
    let t1 = t0.elapsed();
    {
        let path = root.join(format!("{title}_history.svg"));
        let svg = plot2d::SVGBackend::new(&path, (800, 600));
        plot2d::history(svg, history)?;
    }
    use plot2d::IntoDrawingArea as _;
    macro_rules! impl_log {
        ($fb:ident, $cb_fb:ident, $target:ident, $log_fb:ident, $fb_enum:ident, $fb_ty:ident, $efd:ident, $plot:ident) => {{
            if !$fb.is_valid() {
                return Err(SynErr::Solver);
            }
            let path = root.join(format!("{title}_result.ron"));
            std::fs::write(path, ron::to_string(&$fb)?)?;
            let efd_target = efd::$efd::from_curve_harmonic(&$target, mode.is_target_open(), h);
            let curve = $fb.curve(cfg.res);
            let curve_diff = if matches!(mode, syn::Mode::Partial) {
                efd::partial_curve_diff
            } else {
                efd::curve_diff
            };
            let err = curve_diff(&$target, &curve);
            let target_str = cfg
                .ref_num
                .map(|n| format!("Target, Ref. [{n}]"))
                .unwrap_or("Target".to_string());
            let path = root.join(format!("{title}_result.svg"));
            let svg = $plot::SVGBackend::new(&path, (1600, 800));
            let (root_l, root_r) = svg.into_drawing_area().split_horizontally(800);
            let mut fig = $plot::Figure::from(&$fb)
                .axis(false)
                .font(cfg.font)
                .legend(cfg.legend_pos);
            if let Some(angle) = cfg.angle {
                fig = fig.angle(angle.to_radians());
            }
            fig = fig
                .add_line(target_str, $target, $plot::Style::Circle, $plot::RED)
                .add_line("Optimized", &curve, $plot::Style::Triangle, $plot::BLACK);
            fig.plot(root_r)?;
            let mut log = std::fs::File::create(root.join(format!("{title}.log")))?;
            writeln!(log, "[{title}]")?;
            if let Some(io::Fb::$fb_enum(fb)) = target_fb {
                writeln!(log, "\n[target.fb]")?;
                $log_fb(&mut log, &fb)?;
            }
            if let Some((cost, fb)) = $cb_fb {
                let c = fb.curve(cfg.res);
                let efd = efd::$efd::from_curve_harmonic(c, mode.is_result_open(), h);
                let trans = efd.as_trans().to(efd_target.as_trans());
                let fb = fb.trans_denorm(&trans);
                let c = fb.curve(cfg.res);
                let err = curve_diff(&$target, &c);
                writeln!(log, "\n[atlas]")?;
                writeln!(log, "harmonic={h}")?;
                writeln!(log, "error={err}")?;
                writeln!(log, "cost={cost}")?;
                writeln!(log, "\n[atlas.fb]")?;
                $log_fb(&mut log, &fb)?;
                let path = root.join(format!("{title}_atlas.ron"));
                std::fs::write(path, ron::to_string(&fb)?)?;
                fig = fig.add_line("Atlas", c, $plot::Style::Cross, $plot::BLUE);
            }
            writeln!(log, "\n[optimized]")?;
            writeln!(log, "time={t1:?}")?;
            writeln!(log, "harmonic={h}")?;
            writeln!(log, "error={err}")?;
            writeln!(log, "cost={cost}")?;
            writeln!(log, "\n[optimized.fb]")?;
            $log_fb(&mut log, &$fb)?;
            let refer = root
                .parent()
                .unwrap()
                .join(refer)
                .join(format!("{title}.ron"));
            if let Ok(r) = std::fs::File::open(refer) {
                let fb = ron::de::from_reader::<_, $fb_ty>(r)?;
                let c = fb.curve(cfg.res);
                let err = curve_diff(&$target, &c);
                writeln!(log, "\n[competitor]")?;
                writeln!(log, "error={err}")?;
                if !matches!(mode, syn::Mode::Partial) {
                    let efd = efd::$efd::from_curve_harmonic(&c, mode.is_result_open(), h);
                    let cost = efd.l2_norm(&efd_target);
                    writeln!(log, "cost={cost}")?;
                }
                writeln!(log, "\n[competitor.fb]")?;
                $log_fb(&mut log, &fb)?;
                let competitor_str = cfg
                    .ref_num
                    .map(|n| format!("Ref. [{n}]"))
                    .unwrap_or("Competitor".to_string());
                fig = fig.add_line(competitor_str, c, $plot::Style::Square, $plot::BLUE);
            }
            fig.remove_fb().axis(true).plot(root_l)?;
            log.flush()?;
            Ok(())
        }};
    }
    match (result_fb, target) {
        (syn_cmd::CbFb::Fb(fb, cb_fb), io::Curve::P(target)) => {
            impl_log!(fb, cb_fb, target, log_fb, Fb, FourBar, Efd2, plot2d)
        }
        (syn_cmd::CbFb::SFb(fb, cb_fb), io::Curve::S(target)) => {
            impl_log!(fb, cb_fb, target, log_sfb, SFb, SFourBar, Efd3, plot3d)
        }
        _ => unreachable!(),
    }
}

macro_rules! impl_fmt {
    ($w:ident, $fb:ident, $($field:ident),+) => {{
        $(writeln!($w, concat![stringify!($field), "={}"], $fb.$field())?;)+
        Ok(())
    }};
}

fn log_fb(mut w: impl std::io::Write, fb: &FourBar) -> std::io::Result<()> {
    impl_fmt!(w, fb, p0x, p0y, a, l1, l2, l3, l4, l5, g, inv)
}

fn log_sfb(mut w: impl std::io::Write, fb: &SFourBar) -> std::io::Result<()> {
    impl_fmt!(w, fb, ox, oy, oz, r, p0i, p0j, a, l1, l2, l3, l4, l5, g, inv)
}
