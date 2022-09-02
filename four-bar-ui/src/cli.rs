use clap::Parser;
use four_bar::{
    curve, mh, plot,
    syn::{Mode, PathSyn},
    FourBar, Mechanism,
};
use indicatif::{style::ProgressStyle, MultiProgress, ProgressBar};
use std::{error::Error, path::PathBuf, time::Instant};

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
        let entry = <Self as Parser>::parse();
        match entry.cmd {
            None => native(entry.files),
            Some(Cmd::Ui { files }) => native(files),
            Some(Cmd::Syn { files, no_parallel, syn }) => syn_cli(files, no_parallel, syn),
        }
    }
}

fn native(files: Vec<PathBuf>) {
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

fn syn_cli(files: Vec<PathBuf>, no_parallel: bool, syn: Syn) {
    let mpb = MultiProgress::new();
    let run = |file: PathBuf| {
        let pb = mpb.add(ProgressBar::new(syn.gen));
        const STYLE: &str = "[{prefix}] {elapsed_precise} {wide_bar} {pos}/{len} | {msg}";
        pb.set_style(ProgressStyle::with_template(STYLE).unwrap());
        if let Err(e) = syn_cli_inner(&pb, file, syn.clone()) {
            pb.finish_with_message(format!("Error: \"{e}\""));
        }
    };
    if no_parallel {
        files.into_iter().for_each(run);
    } else {
        use mh::rayon::prelude::*;
        files.into_par_iter().for_each(run);
    }
}

fn syn_cli_inner(pb: &ProgressBar, file: PathBuf, syn: Syn) -> Result<(), Box<dyn Error>> {
    let target = match file.extension().and_then(std::ffi::OsStr::to_str) {
        Some("ron") => {
            let fb = ron::from_str::<FourBar>(&std::fs::read_to_string(&file)?)?;
            curve::from_four_bar(fb, syn.n).ok_or("invalid linkage")?
        }
        Some("csv" | "txt") => crate::csv::parse_csv::<[f64; 2]>(&std::fs::read_to_string(&file)?)?,
        _ => return Err("unsupported format".into()),
    };
    let (title, mode) = match file.file_stem().and_then(std::ffi::OsStr::to_str) {
        Some(title) => {
            let mode = match title.rsplit('.').next().ok_or("unsupported mode")? {
                "close" => Mode::Close,
                "partial" => Mode::Partial,
                "open" => Mode::Open,
                _ => return Err("unsupported mode".into()),
            };
            (title, mode)
        }
        _ => return Err("no filename".into()),
    };
    pb.set_prefix(title.to_string());
    let Syn { n, gen, pop } = syn;
    let target = target.as_slice();
    let t0 = Instant::now();
    let s = mh::Solver::build(mh::De::default())
        .task(|ctx| ctx.gen == gen)
        .callback(|ctx| pb.set_position(ctx.gen))
        .pop_num(pop)
        .record(|ctx| ctx.best_f)
        .solve(PathSyn::from_curve(target, None, n, mode))?;
    pb.finish();
    let spent_time = Instant::now() - t0;
    let his_filename = format!("{title}_history.svg");
    let svg = plot::SVGBackend::new(&his_filename, (800, 600));
    plot::history(svg, s.report())?;
    let ans = s.result();
    std::fs::write(format!("{title}_result.ron"), ron::to_string(&ans)?)?;
    let [t1, t2] = ans.angle_bound().expect("solved error");
    let curve = curve::get_valid_part(&Mechanism::new(&ans).curve(t1, t2, n));
    let filename = format!("{title}_linkage.svg");
    let curves = [("Target", target), ("Optimized", &curve)];
    let svg = plot::SVGBackend::new(&filename, (800, 800));
    plot::curve(svg, "Linkage", &curves, ans)?;
    let filename = format!("{title}_result.svg");
    let svg = plot::SVGBackend::new(&filename, (800, 800));
    plot::curve(svg, "Comparison", &curves, None)?;
    let harmonic = s.func().harmonic();
    pb.set_message(format!("spent: {spent_time:?} | harmonic: {harmonic}"));
    Ok(())
}
