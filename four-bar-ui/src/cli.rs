use clap::Parser;
use std::path::PathBuf;

mod syn;

#[derive(Parser)]
#[clap(name = "four-bar", version, author, about)]
pub(crate) struct Entry {
    /// Default to startup GUI then open file paths
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
    Syn(Syn),
    /// Generate codebook
    #[clap(alias = "cb")]
    Codebook(Codebook),
}

#[derive(clap::Args)]
struct Syn {
    /// Target file paths in "[path]/[name].[mode].[ron|csv|txt]" pattern
    files: Vec<PathBuf>,
    /// Disable parallel for all task
    #[clap(long)]
    no_parallel: bool,
    /// Provide pre-generated codebook databases, support join multiple paths
    /// like "cb1.npy:cb2.npy" or "cb1.npy;cb2.npy" in multi-platform
    #[clap(long)]
    cb: Option<std::ffi::OsString>,
    #[clap(flatten)]
    cfg: SynCfg,
}

#[derive(clap::Args)]
struct SynCfg {
    /// Number of the points (resolution) in curve production
    #[clap(short, long = "res", default_value_t = 90)]
    n: usize,
    /// Number of generation
    #[clap(short, long, default_value_t = 50)]
    gen: u64,
    /// Number of population
    #[clap(short, long, default_value_t = 400)]
    pop: usize,
    /// If codebook is enabled, the fetch number
    #[clap(short, default_value_t = 20)]
    k: usize,
    /// Plot and save the changes in each generation
    #[clap(long)]
    log: bool,
}

#[derive(clap::Args)]
struct Codebook {
    /// Output path of the codebook (in NPY format)
    file: PathBuf,
    /// Generate for open curve
    #[clap(long)]
    is_open: bool,
    /// Number of data
    #[clap(short, default_value_t = 102400)]
    n: usize,
    /// Number of the points (resolution) in curve production
    #[clap(long, default_value_t = 720)]
    res: usize,
    /// Number of harmonic
    #[clap(long, default_value_t = 20)]
    harmonic: usize,
}

impl Entry {
    pub(crate) fn parse() {
        let entry = <Self as Parser>::parse_from(wild::args());
        match entry.cmd {
            None => native(entry.files),
            Some(Cmd::Ui { files }) => native(files),
            Some(Cmd::Syn(syn)) => syn::syn(syn),
            Some(Cmd::Codebook(cb)) => codebook(cb),
        }
    }
}

fn native(files: Vec<PathBuf>) {
    let opt = {
        use image::ImageFormat::Png;
        const ICON: &[u8] = include_bytes!("../assets/favicon.png");
        let icon = image::load_from_memory_with_format(ICON, Png).unwrap();
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
    unsafe {
        winapi::um::wincon::FreeConsole();
    }
    use crate::app::App;
    eframe::run_native("Four-bar", opt, Box::new(|ctx| App::new(ctx, files)))
}

fn codebook(cb: Codebook) {
    let Codebook { mut file, is_open, n, res, harmonic } = cb;
    let ext = file.extension().and_then(std::ffi::OsStr::to_str);
    if !matches!(ext, Some("npy")) {
        file.set_extension("npy");
    }
    println!("Generate to: {}", file.display());
    println!("open={is_open}, n={n}, res={res}, harmonic={harmonic}");
    let pb = indicatif::ProgressBar::new(n as u64);
    four_bar::cb::Codebook::make_with(is_open, n, res, harmonic, |n| pb.set_position(n as u64))
        .write(std::fs::File::create(file).unwrap())
        .unwrap();
    pb.finish_and_clear();
    println!("Done");
}
