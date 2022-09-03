use clap::Parser;
use four_bar::{
    curve, mh, plot,
    syn::{Mode, PathSyn},
    FourBar, Mechanism,
};
use indicatif::{style::ProgressStyle, MultiProgress, ProgressBar};
use std::{
    error::Error,
    fmt::Formatter,
    path::{Path, PathBuf},
    time::Instant,
};

#[derive(Parser)]
#[clap(name = "four-bar", version, author, about)]
pub struct Entry {
    /// Open file path
    files: Vec<PathBuf>,
    #[clap(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(clap::Subcommand)]
enum Cmd {
    /// Startup GUI
    Ui {
        /// Open file path
        files: Vec<PathBuf>,
    },
    /// Synthesis function without GUI
    Syn {
        /// Target file paths in "[path]/[name].[mode].[ron|csv|txt]" pattern
        files: Vec<PathBuf>,
        /// Disable parallel for all task
        #[clap(long)]
        no_parallel: bool,
        #[clap(flatten)]
        syn: Syn,
    },
}

#[derive(clap::Args, Clone)]
struct Syn {
    /// Number of the points (resolution) in curve production
    #[clap(short, long = "res", default_value_t = 90)]
    n: usize,
    /// Number of generation
    #[clap(short, long, default_value_t = 50)]
    gen: u64,
    /// Number of population
    #[clap(short, long, default_value_t = 400)]
    pop: usize,
}

impl Entry {
    pub fn parse() {
        let entry = <Self as Parser>::parse_from(wild::args());
        match entry.cmd {
            None => start_native(entry.files),
            Some(Cmd::Ui { files }) => start_native(files),
            Some(Cmd::Syn { files, no_parallel, syn }) => start_syn(files, no_parallel, syn),
        }
    }
}

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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Format => "unsupported format",
            Self::Io => "reading file error",
            Self::Ser => "serialization error",
            Self::Linkage => "invalid linkage",
        };
        f.write_str(s)
    }
}

fn start_native(files: Vec<PathBuf>) {
    use image::ImageFormat;
    let opt = {
        const ICON: &[u8] = include_bytes!("../assets/favicon.png");
        let icon = image::load_from_memory_with_format(ICON, ImageFormat::Png).unwrap();
        eframe::NativeOptions {
            icon_data: Some(eframe::IconData {
                width: icon.width(),
                height: icon.height(),
                rgba: icon.into_bytes(),
            }),
            ..Default::default()
        }
    };
    #[cfg(windows)]
    let _ = unsafe { winapi::um::wincon::FreeConsole() };
    eframe::run_native(
        "Four-bar",
        opt,
        Box::new(|ctx| crate::app::App::new(ctx, files)),
    )
}

fn start_syn(files: Vec<PathBuf>, no_parallel: bool, syn: Syn) {
    let mpb = MultiProgress::new();
    let run = |file: PathBuf| do_syn(&mpb, file, syn.clone());
    let t0 = Instant::now();
    if no_parallel {
        files.into_iter().for_each(run);
    } else {
        use mh::rayon::prelude::*;
        files.into_par_iter().for_each(run);
    }
    mpb.println(format!("Total spent: {:?}", Instant::now() - t0))
        .unwrap();
}

fn do_syn(mpb: &MultiProgress, file: PathBuf, syn: Syn) {
    let file = file.canonicalize().unwrap();
    let (target, title, mode) = match syn_info(&file, syn.n) {
        Ok(v) => v,
        Err(e) => {
            if !matches!(e, SynErr::Format) {
                let title = file.to_str().unwrap().to_string();
                mpb.println(format!("[{title}] {e}")).unwrap();
            }
            return;
        }
    };
    let pb = mpb.add(ProgressBar::new(syn.gen));
    const STYLE: &str = "[{prefix}] {elapsed_precise} {wide_bar} {pos}/{len} {msg}";
    pb.set_style(ProgressStyle::with_template(STYLE).unwrap());
    pb.set_prefix(title.to_string());
    let root = file.parent().unwrap();
    if let Err(e) = do_syn_inner(&pb, title, &target, root, mode, syn) {
        pb.finish_with_message(format!("| error: {e}"));
    }
}

fn syn_info(path: &Path, n: usize) -> Result<(Vec<[f64; 2]>, &str, Mode), SynErr> {
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
            Ok((target, title, mode))
        })
}

fn do_syn_inner(
    pb: &ProgressBar,
    title: &str,
    target: &[[f64; 2]],
    root: &Path,
    mode: Mode,
    syn: Syn,
) -> Result<(), Box<dyn Error>> {
    let Syn { n, gen, pop } = syn;
    let t0 = Instant::now();
    let s = mh::Solver::build(mh::De::default())
        .task(|ctx| ctx.gen == gen)
        .callback(|ctx| pb.set_position(ctx.gen))
        .pop_num(pop)
        .record(|ctx| ctx.best_f)
        .solve(PathSyn::from_curve(target, None, n, mode))?;
    let spent_time = Instant::now() - t0;
    let ans = s.result();
    {
        let path = root.join(format!("{title}_history.svg"));
        let svg = plot::SVGBackend::new(&path, (800, 600));
        plot::history(svg, s.report())?;
    }
    {
        let path = root.join(format!("{title}_result.ron"));
        std::fs::write(path, ron::to_string(&ans)?)?;
    }
    let [t1, t2] = ans.angle_bound().expect("solved error");
    let curve = curve::get_valid_part(&Mechanism::new(&ans).curve(t1, t2, n));
    let curves = [("Target", target), ("Optimized", &curve)];
    {
        let path = root.join(format!("{title}_linkage.svg"));
        let svg = plot::SVGBackend::new(&path, (800, 800));
        plot::curve(svg, "Linkage", &curves, ans)?;
    }
    {
        let path = root.join(format!("{title}_result.svg"));
        let svg = plot::SVGBackend::new(&path, (800, 800));
        plot::curve(svg, "Comparison", &curves, None)?;
    }
    let harmonic = s.func().harmonic();
    pb.finish_with_message(format!("| spent: {spent_time:?} | harmonic: {harmonic}"));
    Ok(())
}
