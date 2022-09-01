use clap::Parser;
use four_bar::{
    curve, mh, plot,
    syn::{Mode, PathSyn},
    FourBar, Mechanism,
};
use std::path::PathBuf;

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
    /// Synthesis without GUI
    Syn {
        /// Target file paths
        files: Vec<PathBuf>,
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
    /// Mode
    #[clap(short, long, value_enum, default_value_t = ModeOpt::Close)]
    mode: ModeOpt,
}

#[derive(clap::ValueEnum, Clone)]
enum ModeOpt {
    Close,
    Partial,
    Open,
}

impl From<ModeOpt> for Mode {
    fn from(m: ModeOpt) -> Self {
        match m {
            ModeOpt::Close => Self::Close,
            ModeOpt::Partial => Self::Partial,
            ModeOpt::Open => Self::Open,
        }
    }
}

impl Entry {
    pub fn parse() {
        let entry = <Self as Parser>::parse();
        match entry.cmd {
            None => native(entry.files),
            Some(Cmd::Ui { files }) => native(files),
            Some(Cmd::Syn { files, syn }) => syn_cli(files, syn),
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

fn syn_cli(files: Vec<PathBuf>, syn: Syn) {
    for file in files {
        println!("============");
        println!("File: \"{}\"", file.display());
        if let Err(e) = syn_cli_inner(file, syn.clone()) {
            println!("Error: \"{e}\"");
        }
    }
}

fn syn_cli_inner(file: PathBuf, syn: Syn) -> Result<(), Box<dyn std::error::Error>> {
    let target = if file.ends_with(".ron") {
        let fb = ron::from_str::<FourBar>(&std::fs::read_to_string(&file)?)?;
        curve::from_four_bar(fb, syn.n).ok_or("invalid linkage")?
    } else if file.ends_with(".csv") {
        crate::csv::parse_csv::<[f64; 2]>(&std::fs::read_to_string(&file)?)?
    } else {
        return Err("unsupported format".into());
    };
    let title = file
        .file_stem()
        .ok_or("no filename")?
        .to_str()
        .ok_or("invalid path encoding")?
        .to_string();
    let Syn { n, gen, pop, mode } = syn;
    let target = target.as_slice();
    let t0 = std::time::Instant::now();
    let pb = indicatif::ProgressBar::new(gen);
    let s = mh::Solver::build(mh::De::default())
        .task(|ctx| ctx.gen == gen)
        .callback(|ctx| pb.set_position(ctx.gen))
        .pop_num(pop)
        .record(|ctx| ctx.best_f)
        .solve(PathSyn::from_curve(target, None, n, mode.into()))?;
    pb.finish();
    println!("Finish at: {:?}", std::time::Instant::now() - t0);
    let his_filename = format!("{title}_history.svg");
    let svg = plot::SVGBackend::new(&his_filename, (800, 600));
    plot::history(svg, s.report())?;
    let ans = s.result();
    std::fs::write(format!("{title}_result.ron"), ron::to_string(&ans)?)?;
    let [t1, t2] = ans.angle_bound().expect("solved error");
    let curve = curve::get_valid_part(&Mechanism::new(&ans).curve(t1, t2, n));
    println!("harmonic: {}", s.func().harmonic());
    println!("seed: {}", s.seed());
    let filename = format!("{title}_linkage.svg");
    let curves = [("Target", target), ("Optimized", &curve)];
    let svg = plot::SVGBackend::new(&filename, (800, 800));
    plot::curve(svg, "Linkage", &curves, ans)?;
    let filename = format!("{title}_result.svg");
    let svg = plot::SVGBackend::new(&filename, (800, 800));
    plot::curve(svg, "Comparison", &curves, None)?;
    Ok(())
}
