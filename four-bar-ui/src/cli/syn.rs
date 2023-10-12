use crate::{io, syn_cmd};
use four_bar::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    borrow::Cow,
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
    Plot(plot::DrawingAreaErrorKind<std::io::Error>),
    CsvSer(csv::Error),
    RonSerde(ron::error::SpannedError),
    RonIo(ron::error::Error),
    Linkage,
    Solver,
}

impl std::fmt::Display for SynErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Format => write!(f, "unsupported format"),
            Self::Io(e) => write!(f, "[IO] {e}"),
            Self::Plot(e) => write!(f, "[Plot] {e}"),
            Self::CsvSer(e) => write!(f, "[CSV] {e}"),
            Self::RonSerde(e) => write!(f, "[RON-Serde] {e}"),
            Self::RonIo(e) => write!(f, "[RON-IO] {e}"),
            Self::Linkage => write!(f, "invalid linkage input"),
            Self::Solver => write!(f, "solved error"),
        }
    }
}

impl std::error::Error for SynErr {}

impl_err_from! {
    impl std::io::Error => Io
    impl plot::DrawingAreaErrorKind<std::io::Error> => Plot
    impl csv::Error => CsvSer
    impl ron::error::SpannedError => RonSerde
    impl ron::error::Error => RonIo
}

#[derive(clap::Args)]
#[clap(subcommand_precedence_over_arg = true)]
pub(super) struct Syn {
    /// Target file paths in "[path]/[name].[mode].[ron|csv|txt]" pattern
    #[clap(required = true)]
    files: Vec<PathBuf>,
    /// Force to rerun the result
    ///
    /// If the last result exists, the program will only redraw it
    #[clap(short = 'f', long, alias = "force")]
    rerun: bool,
    /// Remove the related project folders and exit
    ///
    /// This flag won't run the synthesis functions
    #[clap(long, alias = "clear")]
    clean: bool,
    /// Disable parallel for running all tasks, use a single loop for
    /// benchmarking
    #[clap(long)]
    each: bool,
    /// Provide pre-generated codebook databases, support multiple paths as
    #[cfg_attr(windows, doc = "\"a.npz;b.npz\"")]
    #[cfg_attr(not(windows), doc = "\"a.npz:b.npz\"")]
    ///
    /// If the codebook is provided, the rerun flag will be enabled
    #[clap(long)]
    cb: Option<std::ffi::OsString>,
    /// Competitor path starting from file root with the same filename
    #[clap(short, long, default_value = "refer")]
    refer: PathBuf,
    /// Disable reference comparison
    #[clap(long)]
    no_ref: bool,
    #[clap(flatten)]
    cfg: syn_cmd::SynCfg,
    #[clap(subcommand)]
    alg: Option<syn_cmd::SynAlg>,
}

struct Info {
    root: PathBuf,
    target: io::Curve,
    target_fb: Option<io::Fb>,
    title: String,
    mode: syn::Mode,
}

fn get_info(
    title: &str,
    file: &Path,
    res: usize,
    rerun: bool,
    clean: bool,
) -> Result<Info, SynErr> {
    let mut target_fb = None;
    let ext = file.extension().and_then(|p| p.to_str());
    let target = match ext.ok_or(SynErr::Format)? {
        "csv" | "txt" => io::Curve::from_reader(std::fs::File::open(file)?)?,
        "ron" => {
            let fb = ron::de::from_reader(std::fs::File::open(file)?)?;
            let curve = match &fb {
                io::Fb::Fb(fb) => io::Curve::P(fb.curve(res)),
                io::Fb::SFb(fb) => io::Curve::S(fb.curve(res)),
            };
            target_fb.replace(fb);
            curve
        }
        _ => {
            println!("Ignored: {}", file.display());
            Err(SynErr::Format)?
        }
    };
    match &target {
        io::Curve::P(t) => _ = efd::valid_curve(t).ok_or(SynErr::Linkage)?,
        io::Curve::S(t) => _ = efd::valid_curve(t).ok_or(SynErr::Linkage)?,
    }
    let mode = match Path::new(title).extension().and_then(|p| p.to_str()) {
        Some("closed") => syn::Mode::Closed,
        Some("partial") => syn::Mode::Partial,
        Some("open") => syn::Mode::Open,
        _ => Err(SynErr::Format)?,
    };
    let root = file.parent().unwrap().join(title);
    if root.is_dir() {
        if rerun {
            // Clear the root folder
            // Avoid file browser missing opening folders
            for e in std::fs::read_dir(&root)? {
                let path = e?.path();
                if path.is_dir() {
                    std::fs::remove_dir_all(path)?;
                } else {
                    std::fs::remove_file(path)?;
                }
            }
        } else if clean {
            // Just remove root folder
            std::fs::remove_dir_all(&root)?;
        }
    } else if !clean || rerun {
        std::fs::create_dir(&root)?;
    }
    let title = title.to_string();
    Ok(Info { root, target, target_fb, title, mode })
}

pub(super) fn syn(syn: Syn) {
    let Syn {
        files,
        each,
        cfg,
        cb,
        refer,
        no_ref,
        alg,
        rerun,
        clean,
    } = syn;
    if let Some(seed) = cfg.seed {
        println!("seed={seed}");
    }
    println!("gen={} pop={} res={}", cfg.gen, cfg.pop, cfg.res);
    // If codebook is provided, rerun is always enabled
    let rerun = rerun || cb.is_some();
    println!("rerun={rerun} clean={clean}");
    println!("-----");
    // Load target files & create project folders
    let tasks = files
        .into_iter()
        .filter_map(|file| {
            let file = file.canonicalize().ok().filter(|f| f.is_file())?;
            let title = file.file_stem()?.to_str()?;
            match get_info(title, &file, cfg.res, rerun, clean) {
                Ok(info) => Some(info),
                Err(SynErr::Format) => None,
                Err(e) => {
                    println!("Error in {title}: {e}");
                    None
                }
            }
        })
        .collect::<Vec<_>>();
    if clean && !rerun {
        return;
    }
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
    let pb = ProgressBar::new(tasks.len() as u64 * cfg.gen);
    pb.set_style(ProgressStyle::with_template(STYLE).unwrap());
    // Tasks
    let alg = alg.unwrap_or_default();
    let refer = (!no_ref).then_some(refer.as_path());
    let run = |info| run(&pb, alg.clone(), info, &cfg, &cb, refer, rerun);
    let t0 = std::time::Instant::now();
    if each {
        tasks.into_iter().for_each(run);
    } else {
        use mh::rayon::prelude::*;
        tasks.into_par_iter().for_each(run);
    }
    pb.finish_and_clear();
    println!("-----");
    println!("Finished in {:?}", t0.elapsed());
}

const HISTORY_SVG: &str = "history.svg";
const LNK_RON: &str = "linkage.ron";
const LNK_SVG: &str = "linkage.svg";
const LNK_FIG: &str = "linkage.fig.ron";
const CURVE_SVG: &str = "curve.svg";
const CURVE_FIG: &str = "curve.fig.ron";

fn from_runtime(
    pb: &ProgressBar,
    alg: syn_cmd::SynAlg,
    info: &Info,
    cfg: &syn_cmd::SynCfg,
    cb: &io::CbPool,
    refer: Option<&Path>,
) -> Result<(), SynErr> {
    use four_bar::fb::{CurveGen as _, Normalized as _};
    use plot::full_palette::*;
    let Info { root, target, target_fb, title, mode } = info;
    let mode = *mode;
    let mut history = Vec::with_capacity(cfg.gen as usize);
    let t0 = std::time::Instant::now();
    let s = {
        let target = match &target {
            io::Curve::P(t) => syn_cmd::Target::P(Cow::Borrowed(t), Cow::Borrowed(cb.as_fb())),
            io::Curve::S(t) => syn_cmd::Target::S(Cow::Borrowed(t), Cow::Borrowed(cb.as_sfb())),
        };
        let cfg = syn_cmd::SynCfg { mode, ..cfg.clone() };
        let stop = || false;
        syn_cmd::Solver::new(alg, target, cfg, stop, |best_f, _| {
            history.push(best_f);
            pb.inc(1);
        })
    };
    let (cost, harmonic, lnk_fb) = s.solve_verbose().map_err(|_| SynErr::Solver)?;
    let t1 = t0.elapsed();
    {
        let path = root.join(HISTORY_SVG);
        let svg = plot::SVGBackend::new(&path, (800, 600));
        plot2d::history(svg, history)?;
    }
    let lnk_path = root.join(LNK_RON);
    match &lnk_fb {
        syn_cmd::SolvedFb::Fb(fb, _) => write_ron(lnk_path, fb)?,
        syn_cmd::SolvedFb::SFb(fb, _) => write_ron(lnk_path, fb)?,
    }
    // Log results
    let refer = refer
        .map(|p| root.join("..").join(p).join(format!("{title}.ron")))
        .filter(|p| p.is_file());
    let mut log = std::fs::File::create(root.join(format!("{title}.log")))?;
    writeln!(log, "title={title}")?;
    match (target, &lnk_fb) {
        (io::Curve::P(target), syn_cmd::SolvedFb::Fb(fb, cb_fb)) if fb.is_valid() => {
            let curve_diff = if matches!(mode, syn::Mode::Partial) {
                efd::partial_curve_diff
            } else {
                efd::curve_diff
            };
            let efd_target =
                efd::Efd2::from_curve_harmonic(target, mode.is_target_open(), harmonic);
            let curve = fb.curve(cfg.res);
            let mut fig = plot2d::Figure::new_ref(Some(fb))
                .add_line("Target", target, plot::Style::Circle, RED)
                .add_line("Optimized", &curve, plot::Style::Line, BLUE_900);
            {
                write_ron(root.join(LNK_FIG), &fig)?;
                let path = root.join(LNK_SVG);
                let svg = plot::SVGBackend::new(&path, (1600, 1600));
                fig.plot(svg)?;
            }
            if let Some(io::Fb::Fb(fb)) = target_fb {
                writeln!(log, "[target.fb]")?;
                log_fb(&mut log, fb)?;
            }
            if let Some((cost, fb)) = cb_fb {
                let c = fb.curve(cfg.res);
                let efd = efd::Efd2::from_curve_harmonic(c, mode.is_result_open(), harmonic);
                let trans = efd.as_trans().to(efd_target.as_trans());
                let fb = fb.clone().trans_denorm(&trans);
                let c = fb.curve(cfg.res.min(30));
                let err = curve_diff(target, &c);
                writeln!(log, "\n[atlas]")?;
                writeln!(log, "harmonic={harmonic}")?;
                writeln!(log, "cost={cost:.04}")?;
                writeln!(log, "error={err:.04}")?;
                writeln!(log, "\n[atlas.fb]")?;
                log_fb(&mut log, &fb)?;
                write_ron(root.join("atlas.ron"), &fb)?;
                fig.push_line("Atlas", c, plot::Style::Triangle, GREEN_900);
            }
            writeln!(log, "\n[optimized]")?;
            let err = curve_diff(target, &curve);
            writeln!(log, "time={t1:.02?}")?;
            writeln!(log, "cost={cost:.04}")?;
            writeln!(log, "error={err:.04}")?;
            writeln!(log, "harmonic={harmonic}")?;
            writeln!(log, "\n[optimized.fb]")?;
            log_fb(&mut log, fb)?;
            if let Some(refer) = refer {
                let fb = ron::de::from_reader::<_, FourBar>(std::fs::File::open(refer)?)?;
                let c = fb.curve(cfg.res);
                let err = curve_diff(target, &c);
                writeln!(log, "\n[competitor]")?;
                if !matches!(mode, syn::Mode::Partial) {
                    let efd = efd::Efd2::from_curve_harmonic(&c, mode.is_result_open(), harmonic);
                    let cost = efd.distance(&efd_target);
                    writeln!(log, "cost={cost:.04}")?;
                }
                writeln!(log, "error={err:.04}")?;
                writeln!(log, "\n[competitor.fb]")?;
                log_fb(&mut log, &fb)?;
                fig.push_line("Ref. [?]", c, plot::Style::DashedLine, ORANGE_900);
            }
            fig.fb = None;
            write_ron(root.join(CURVE_FIG), &fig)?;
            let path = root.join(CURVE_SVG);
            let svg = plot::SVGBackend::new(&path, (1600, 1600));
            fig.plot(svg)?;
        }
        (io::Curve::S(target), syn_cmd::SolvedFb::SFb(fb, cb_fb)) if fb.is_valid() => {
            let curve_diff = if matches!(mode, syn::Mode::Partial) {
                efd::partial_curve_diff
            } else {
                efd::curve_diff
            };
            let efd_target =
                efd::Efd3::from_curve_harmonic(target, mode.is_target_open(), harmonic);
            let curve = fb.curve(cfg.res);
            let mut fig = plot3d::Figure::new_ref(Some(fb))
                .add_line("Target", target, plot::Style::Circle, RED)
                .add_line("Optimized", &curve, plot::Style::Line, BLUE_900);
            {
                write_ron(root.join(LNK_FIG), &fig)?;
                let path = root.join(LNK_SVG);
                let svg = plot::SVGBackend::new(&path, (1600, 1600));
                fig.plot(svg)?;
            }
            if let Some(io::Fb::SFb(fb)) = target_fb {
                writeln!(log, "[target.fb]")?;
                log_sfb(&mut log, fb)?;
            }
            if let Some((cost, fb)) = cb_fb {
                let c = fb.curve(cfg.res);
                let efd = efd::Efd3::from_curve_harmonic(c, mode.is_result_open(), harmonic);
                let trans = efd.as_trans().to(efd_target.as_trans());
                let fb = fb.clone().trans_denorm(&trans);
                let c = fb.curve(cfg.res.min(30));
                let err = curve_diff(target, &c);
                writeln!(log, "\n[atlas]")?;
                writeln!(log, "harmonic={harmonic}")?;
                writeln!(log, "cost={cost:.04}")?;
                writeln!(log, "error={err:.04}")?;
                writeln!(log, "\n[atlas.fb]")?;
                log_sfb(&mut log, &fb)?;
                write_ron(root.join("atlas.ron"), &fb)?;
                fig.push_line("Atlas", c, plot::Style::Triangle, GREEN_900);
            }
            writeln!(log, "\n[optimized]")?;
            let err = curve_diff(target, &curve);
            writeln!(log, "time={t1:.02?}")?;
            writeln!(log, "cost={cost:.04}")?;
            writeln!(log, "error={err:.04}")?;
            writeln!(log, "harmonic={harmonic}")?;
            writeln!(log, "\n[optimized.fb]")?;
            log_sfb(&mut log, fb)?;
            if let Some(refer) = refer {
                let fb = ron::de::from_reader::<_, SFourBar>(std::fs::File::open(refer)?)?;
                let c = fb.curve(cfg.res);
                let err = curve_diff(target, &c);
                writeln!(log, "\n[competitor]")?;
                if !matches!(mode, syn::Mode::Partial) {
                    let efd = efd::Efd3::from_curve_harmonic(&c, mode.is_result_open(), harmonic);
                    let cost = efd.distance(&efd_target);
                    writeln!(log, "cost={cost:.04}")?;
                }
                writeln!(log, "error={err:.04}")?;
                writeln!(log, "\n[competitor.fb]")?;
                log_sfb(&mut log, &fb)?;
                fig.push_line("Ref. [?]", c, plot::Style::DashedLine, ORANGE_900);
            }
            fig.fb = Some(Cow::Owned(fb.take_sphere()));
            write_ron(root.join(CURVE_FIG), &fig)?;
            let path = root.join(CURVE_SVG);
            let svg = plot::SVGBackend::new(&path, (1600, 1600));
            fig.plot(svg)?;
        }
        _ => unreachable!(),
    }
    log.flush()?;
    Ok(())
}

fn from_exist(info: &Info) -> Result<(), SynErr> {
    let root = &info.root;
    macro_rules! impl_plot {
        ($fig:ty) => {{
            let path = root.join(LNK_FIG);
            let fig = ron::de::from_reader::<_, $fig>(std::fs::File::open(path)?)?;
            let path = root.join(LNK_SVG);
            let svg = plot::SVGBackend::new(&path, (1600, 1600));
            fig.plot(svg)?;
            let path = root.join(CURVE_FIG);
            let fig = ron::de::from_reader::<_, $fig>(std::fs::File::open(path)?)?;
            let path = root.join(CURVE_SVG);
            let svg = plot::SVGBackend::new(&path, (1600, 1600));
            fig.plot(svg)?;
            Ok(())
        }};
    }
    match info.target {
        io::Curve::P(_) => impl_plot!(plot2d::Figure),
        io::Curve::S(_) => impl_plot!(plot3d::Figure),
    }
}

fn run(
    pb: &ProgressBar,
    alg: syn_cmd::SynAlg,
    info: Info,
    cfg: &syn_cmd::SynCfg,
    cb: &io::CbPool,
    refer: Option<&Path>,
    rerun: bool,
) {
    let f = || {
        let root = &info.root;
        if !rerun && root.join(LNK_FIG).is_file() && root.join(CURVE_FIG).is_file() {
            pb.inc(cfg.gen);
            from_exist(&info)
        } else {
            from_runtime(pb, alg, &info, cfg, cb, refer)
        }
    };
    let title = &info.title;
    match f() {
        Ok(()) => pb.println(format!("Finished: {title}")),
        Err(e) => pb.println(format!("Error in {title}: {e}")),
    }
}

macro_rules! write_fields {
    ($w: ident, $obj: expr $(, $fields: ident)+ $(,)?) => {
        $(writeln!($w, concat![stringify!($fields), "={:.04}"], $obj.$fields)?;)+
    };
}

fn log_fb(mut w: impl std::io::Write, fb: &FourBar) -> std::io::Result<()> {
    write_fields!(w, fb.unnorm, p0x, p0y, a);
    write_fields!(w, fb, l1);
    write_fields!(w, fb.unnorm, l2);
    write_fields!(w, fb, l3, l4, l5, g);
    writeln!(w, "stat={}", fb.stat)?;
    Ok(())
}

fn log_sfb(mut w: impl std::io::Write, fb: &SFourBar) -> std::io::Result<()> {
    write_fields!(w, fb.unnorm, ox, oy, oz, r, p0i, p0j, a);
    write_fields!(w, fb, l1, l2, l3, l4, l5, g);
    writeln!(w, "stat={}", fb.stat)?;
    Ok(())
}

fn write_ron<S>(path: impl AsRef<Path>, s: &S) -> Result<(), SynErr>
where
    S: serde::Serialize,
{
    ron::ser::to_writer_pretty(std::fs::File::create(path)?, s, Default::default())?;
    Ok(())
}
