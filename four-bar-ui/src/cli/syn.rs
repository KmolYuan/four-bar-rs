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
    #[clap(long)]
    cb: Option<std::ffi::OsString>,
    /// Competitor path starting from file root with the same filename
    #[clap(short, long, default_value = "refer")]
    refer: PathBuf,
    #[clap(flatten)]
    cfg: SynCfg,
    #[clap(subcommand)]
    method: Option<crate::syn_cmd::SynMethod>,
}

#[derive(clap::Args)]
struct SynCfg {
    /// Font size in the plot
    #[clap(long, default_value_t = 90.)]
    font: f64,
    /// Reference number of competitor, default to eliminate
    ///
    /// Pass `--ref-num 0` to enable and leave a placeholder
    #[clap(long)]
    ref_num: Option<u8>,
    /// Linkage input angle (degrees) in the plot
    #[clap(long)]
    angle: Option<f64>,
    /// Legend position
    #[clap(long, default_value = "ur")]
    legend: plot::LegendPos,
    #[clap(flatten)]
    inner: syn_cmd::SynConfig,
}

impl std::fmt::Display for SynCfg {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        macro_rules! impl_fmt {
            ($self:ident, $($field:ident),+) => {$(
                write!(f, concat![stringify!($field), "={:?} "], $self.$field)?;
            )+};
        }
        impl_fmt!(self, res, gen, pop, seed, legend, font);
        Ok(())
    }
}

impl std::ops::Deref for SynCfg {
    type Target = syn_cmd::SynConfig;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
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
    let ext = file.extension().and_then(OsStr::to_str);
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
    let mode = match Path::new(title).extension().and_then(OsStr::to_str) {
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
    let Syn { files, each, cfg, cb, refer, method, rerun, clean } = syn;
    println!("{cfg}\n-----");
    // If codebook is provided, rerun is always enabled
    let rerun = rerun || cb.is_some();
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
    let method = method.unwrap_or_default();
    let run = |info| run(&pb, method.clone(), info, &cfg, &cb, &refer, rerun);
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

struct Solver<'a> {
    refer: &'a Path,
    info: &'a Info,
    cfg: &'a SynCfg,
    harmonic: usize,
    lnk_fb: syn_cmd::SolvedFb,
    cost_t1: Option<(f64, std::time::Duration)>,
}

impl<'a> Solver<'a> {
    fn from_runtime(
        pb: &ProgressBar,
        method: syn_cmd::SynMethod,
        info: &'a Info,
        cfg: &'a SynCfg,
        cb: &io::CbPool,
        refer: &'a Path,
        lnk_path: impl AsRef<Path>,
    ) -> Result<Self, SynErr> {
        let Info { root, target, mode, .. } = info;
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
        let (cost, harmonic, result_fb) = s.solve_verbose().map_err(|_| SynErr::Solver)?;
        let t1 = t0.elapsed();
        let path = root.join("history.svg");
        let svg = plot::SVGBackend::new(&path, (800, 600));
        plot2d::history(svg, history)?;
        match &result_fb {
            syn_cmd::SolvedFb::Fb(fb, _) => std::fs::write(lnk_path, io::ron_string(fb))?,
            syn_cmd::SolvedFb::SFb(fb, _) => std::fs::write(lnk_path, io::ron_string(fb))?,
        }
        Ok(Self {
            refer,
            info,
            cfg,
            harmonic,
            lnk_fb: result_fb,
            cost_t1: Some((cost, t1)),
        })
    }

    fn from_exist(
        pb: &ProgressBar,
        info: &'a Info,
        cfg: &'a SynCfg,
        refer: &'a Path,
        lnk_fb: syn_cmd::SolvedFb,
    ) -> Self {
        let is_open = info.mode.is_result_open();
        let harmonic = match &lnk_fb {
            syn_cmd::SolvedFb::Fb(fb, _) => {
                efd::Efd2::from_curve(fb.curve(cfg.res), is_open).harmonic()
            }
            syn_cmd::SolvedFb::SFb(fb, _) => {
                efd::Efd3::from_curve(fb.curve(cfg.res), is_open).harmonic()
            }
        };
        pb.inc(cfg.gen);
        Self { refer, info, cfg, harmonic, lnk_fb, cost_t1: None }
    }

    fn log(self) -> Result<(), SynErr> {
        use four_bar::fb::{CurveGen as _, Normalized as _};
        let Self {
            refer,
            info,
            cfg,
            harmonic,
            lnk_fb: result_fb,
            cost_t1,
        } = self;
        let Info { root, target, target_fb, title, mode } = info;
        let refer = cfg.ref_num.and_then(|n| {
            let path = root
                .parent()
                .unwrap()
                .join(refer)
                .join(format!("{title}.ron"));
            Some((format!("Ref. [{n}]"), std::fs::File::open(path).ok()?))
        });
        let path = root.join(format!("{title}.log"));
        let mut log = if path.is_file() {
            // Do not overwrite the exist log file
            OptionLogger(None)
        } else {
            OptionLogger(Some(std::fs::File::create(path)?))
        };
        match (target, &result_fb) {
            (io::Curve::P(target), syn_cmd::SolvedFb::Fb(fb, cb_fb)) if fb.is_valid() => {
                let curve_diff = if matches!(info.mode, syn::Mode::Partial) {
                    efd::partial_curve_diff
                } else {
                    efd::curve_diff
                };
                let efd_target =
                    efd::Efd2::from_curve_harmonic(target, info.mode.is_target_open(), harmonic);
                let curve = fb.curve(cfg.res);
                let mut fig = plot2d::Figure::new_ref(Some(fb))
                    .font(cfg.font)
                    .legend(cfg.legend)
                    .add_line("Target", target, plot::Style::Circle, plot::RED)
                    .add_line("Optimized", &curve, plot::Style::Line, plot::BLACK);
                if let Some(angle) = cfg.angle {
                    fig = fig.angle(angle.to_radians());
                }
                {
                    let path = root.join("linkage.svg");
                    let svg = plot::SVGBackend::new(&path, (1600, 1600));
                    fig.plot(svg)?;
                }
                if let Some(io::Fb::Fb(fb)) = target_fb {
                    writeln!(log, "\n[target.fb]")?;
                    log_fb(&mut log, fb)?;
                }
                if let Some((cost, fb)) = cb_fb {
                    let c = fb.curve(cfg.res);
                    let efd = efd::Efd2::from_curve_harmonic(c, mode.is_result_open(), harmonic);
                    let trans = efd.as_trans().to(efd_target.as_trans());
                    let fb = fb.clone().trans_denorm(&trans);
                    let c = fb.curve(cfg.res);
                    let err = curve_diff(target, &c);
                    writeln!(log, "\n[atlas]")?;
                    writeln!(log, "harmonic={harmonic}")?;
                    writeln!(log, "cost={cost:.04}")?;
                    writeln!(log, "error={err:.04}")?;
                    writeln!(log, "\n[atlas.fb]")?;
                    log_fb(&mut log, &fb)?;
                    std::fs::write(root.join("atlas.ron"), io::ron_string(&fb))?;
                    fig = fig.add_line("Atlas", c, plot::Style::Dot, plot::full_palette::GREEN_600);
                }
                writeln!(log, "\n[optimized]")?;
                let err = curve_diff(target, &curve);
                if let Some((cost, t1)) = cost_t1 {
                    writeln!(log, "time={t1:?}")?;
                    writeln!(log, "cost={cost:.04}")?;
                }
                writeln!(log, "error={err:.04}")?;
                writeln!(log, "harmonic={harmonic}")?;
                writeln!(log, "\n[optimized.fb]")?;
                log_fb(&mut log, fb)?;
                if let Some((name, r)) = refer {
                    let fb = ron::de::from_reader::<_, FourBar>(r)?;
                    let c = fb.curve(cfg.res);
                    let err = curve_diff(target, &c);
                    writeln!(log, "\n[competitor]")?;
                    if !matches!(mode, syn::Mode::Partial) {
                        let efd =
                            efd::Efd2::from_curve_harmonic(&c, mode.is_result_open(), harmonic);
                        let cost = efd.distance(&efd_target);
                        writeln!(log, "cost={cost:.04}")?;
                    }
                    writeln!(log, "error={err:.04}")?;
                    writeln!(log, "\n[competitor.fb]")?;
                    log_fb(&mut log, &fb)?;
                    fig = fig.add_line(name, c, plot::Style::DashedLine, plot::BLUE);
                }
                let path = root.join("curve.svg");
                let svg = plot::SVGBackend::new(&path, (1600, 1600));
                fig.remove_fb().plot(svg)?;
            }
            (io::Curve::S(target), syn_cmd::SolvedFb::SFb(fb, cb_fb)) if fb.is_valid() => {
                let curve_diff = if matches!(info.mode, syn::Mode::Partial) {
                    efd::partial_curve_diff
                } else {
                    efd::curve_diff
                };
                let efd_target =
                    efd::Efd3::from_curve_harmonic(target, info.mode.is_target_open(), harmonic);
                let curve = fb.curve(cfg.res);
                let mut fig = plot3d::Figure::new_ref(Some(fb))
                    .font(cfg.font)
                    .legend(cfg.legend)
                    .add_line("Target", target, plot::Style::Circle, plot::RED)
                    .add_line("Optimized", &curve, plot::Style::Line, plot::BLACK);
                if let Some(angle) = cfg.angle {
                    fig = fig.angle(angle.to_radians());
                }
                {
                    let path = root.join("linkage.svg");
                    let svg = plot::SVGBackend::new(&path, (1600, 1600));
                    fig.plot(svg)?;
                }
                if let Some(io::Fb::SFb(fb)) = target_fb {
                    writeln!(log, "\n[target.fb]")?;
                    log_sfb(&mut log, fb)?;
                }
                if let Some((cost, fb)) = cb_fb {
                    let c = fb.curve(cfg.res);
                    let efd = efd::Efd3::from_curve_harmonic(c, mode.is_result_open(), harmonic);
                    let trans = efd.as_trans().to(efd_target.as_trans());
                    let fb = fb.clone().trans_denorm(&trans);
                    let c = fb.curve(cfg.res);
                    let err = curve_diff(target, &c);
                    writeln!(log, "\n[atlas]")?;
                    writeln!(log, "harmonic={harmonic}")?;
                    writeln!(log, "cost={cost:.04}")?;
                    writeln!(log, "error={err:.04}")?;
                    writeln!(log, "\n[atlas.fb]")?;
                    log_sfb(&mut log, &fb)?;
                    std::fs::write(root.join("atlas.ron"), io::ron_string(&fb))?;
                    fig = fig.add_line("Atlas", c, plot::Style::Dot, plot::CYAN);
                }
                writeln!(log, "\n[optimized]")?;
                let err = curve_diff(target, &curve);
                if let Some((cost, t1)) = cost_t1 {
                    writeln!(log, "time={t1:?}")?;
                    writeln!(log, "cost={cost:.04}")?;
                }
                writeln!(log, "error={err:.04}")?;
                writeln!(log, "harmonic={harmonic}")?;
                writeln!(log, "\n[optimized.fb]")?;
                log_sfb(&mut log, fb)?;
                if let Some((name, r)) = refer {
                    let fb = ron::de::from_reader::<_, SFourBar>(r)?;
                    let c = fb.curve(cfg.res);
                    let err = curve_diff(target, &c);
                    writeln!(log, "\n[competitor]")?;
                    if !matches!(mode, syn::Mode::Partial) {
                        let efd =
                            efd::Efd3::from_curve_harmonic(&c, mode.is_result_open(), harmonic);
                        let cost = efd.distance(&efd_target);
                        writeln!(log, "cost={cost:.04}")?;
                    }
                    writeln!(log, "error={err:.04}")?;
                    writeln!(log, "\n[competitor.fb]")?;
                    log_sfb(&mut log, &fb)?;
                    fig = fig.add_line(name, c, plot::Style::DashedLine, plot::BLUE);
                }
                let path = root.join("curve.svg");
                let svg = plot::SVGBackend::new(&path, (1600, 1600));
                fig.with_fb(fb.take_sphere()).plot(svg)?;
            }
            _ => Err(SynErr::Solver)?,
        }
        log.flush()?;
        Ok(())
    }
}

fn run(
    pb: &ProgressBar,
    method: syn_cmd::SynMethod,
    info: Info,
    cfg: &SynCfg,
    cb: &io::CbPool,
    refer: &Path,
    rerun: bool,
) {
    let title = &info.title;
    let lnk_path = info.root.join("linkage.ron");
    let f = || {
        if !rerun && lnk_path.is_file() {
            match ron::de::from_reader(std::fs::File::open(&lnk_path)?) {
                Ok(fb) => {
                    // Just redraw the plots
                    let lnk_fb = match fb {
                        io::Fb::Fb(fb) => syn_cmd::SolvedFb::Fb(fb, None),
                        io::Fb::SFb(fb) => syn_cmd::SolvedFb::SFb(fb, None),
                    };
                    return Solver::from_exist(pb, &info, cfg, refer, lnk_fb).log();
                }
                Err(e) => println!("Rerun {title}: {}", SynErr::from(e)),
            }
        }
        Solver::from_runtime(pb, method, &info, cfg, cb, refer, lnk_path)?.log()
    };
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
    write!(w, "stat={}", fb.stat)?;
    Ok(())
}

fn log_sfb(mut w: impl std::io::Write, fb: &SFourBar) -> std::io::Result<()> {
    write_fields!(w, fb.unnorm, ox, oy, oz, r, p0i, p0j, a);
    write_fields!(w, fb, l1, l2, l3, l4, l5, g);
    write!(w, "stat={}", fb.stat)?;
    Ok(())
}

struct OptionLogger(Option<std::fs::File>);

impl std::io::Write for OptionLogger {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match &mut self.0 {
            Some(f) => f.write(buf),
            None => Ok(buf.len()),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match &mut self.0 {
            Some(f) => f.flush(),
            None => Ok(()),
        }
    }
}
