use clap::Parser;
use std::path::PathBuf;

mod syn;

#[derive(Parser)]
#[clap(name = "four-bar", version, author, about)]
pub(crate) struct Entry {
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
        /// Provide a pre-generated codebook database
        #[clap(long, alias = "codebook", action = clap::ArgAction::Append)]
        cb: Vec<PathBuf>,
    },
    /// Generate codebook
    Codebook(Codebook),
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

#[derive(clap::Args)]
struct Codebook {
    /// Output path of the codebook (in NPY format)
    file: PathBuf,
    /// Generate for open curve
    #[clap(long)]
    open: bool,
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
            Some(Cmd::Syn { files, no_parallel, syn, cb }) => syn::syn(files, no_parallel, syn, cb),
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
    eframe::run_native(
        "Four-bar",
        opt,
        Box::new(|ctx| crate::app::App::new(ctx, files)),
    )
}

fn codebook(cb: Codebook) {
    let Codebook { mut file, open, n, res, harmonic } = cb;
    let ext = file.extension().and_then(std::ffi::OsStr::to_str);
    if !matches!(ext, Some("npy")) {
        file.set_extension("npy");
    }
    println!("Generate to: {}", file.display());
    println!("open={open}, n={n}, res={res}, harmonic={harmonic}");
    let pb = indicatif::ProgressBar::new(n as u64);
    four_bar::codebook::Codebook::make_with(open, n, res, harmonic, |n| pb.set_position(n as u64))
        .write(std::fs::File::create(file).unwrap())
        .unwrap();
    pb.finish_and_clear();
    println!("Done");
}
