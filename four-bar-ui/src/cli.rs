use crate::{app::App, syn_cmd::SynCmd};
use clap::Parser;
use four_bar::cb;
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
    Codebook(CbCfg),
}

#[derive(clap::Args)]
#[clap(subcommand_precedence_over_arg = true)]
struct Syn {
    /// Target file paths in "[path]/[name].[mode].[ron|csv|txt]" pattern
    #[clap(required = true)]
    files: Vec<PathBuf>,
    /// Disable parallel for running all tasks
    #[clap(long)]
    one_by_one: bool,
    /// Provide pre-generated codebook databases, support multiple paths as
    #[cfg_attr(windows, doc = "\"a.npy;b.npy\"")]
    #[cfg_attr(not(windows), doc = "\"a.npy:b.npy\"")]
    #[clap(long)]
    cb: Option<std::ffi::OsString>,
    /// Reference (competitor) path starting from file root with the same
    /// filename, support multiple paths as
    #[cfg_attr(windows, doc = "\"a.npy;b.npy\"")]
    #[cfg_attr(not(windows), doc = "\"a.npy:b.npy\"")]
    #[clap(short, long, default_value = "refer")]
    refer: std::ffi::OsString,
    #[clap(flatten)]
    cfg: SynCfg,
    #[clap(subcommand)]
    method_cmd: Option<SynCmd>,
}

#[derive(clap::Args)]
struct SynCfg {
    /// Number of the points (resolution) in curve production
    #[clap(long, default_value_t = 90)]
    res: usize,
    /// Number of generation
    #[clap(short, long, default_value_t = 50)]
    gen: usize,
    /// Number of population (the fetch number in codebook)
    #[clap(short, long, default_value_t = 200)]
    pop: usize,
    /// Fix the seed to get a determined result, default to random
    #[clap(short, long)]
    seed: Option<u64>,
    /// Plot and save the changes with log interval, default to disabled
    #[clap(long, default_value_t = 0)]
    log: usize,
}

#[derive(clap::Args)]
struct CbCfg {
    /// Output path of the codebook (in NPY format)
    file: PathBuf,
    /// Generate for open curve
    #[clap(long)]
    is_open: bool,
    /// Number of data
    #[clap(long, default_value_t = cb::Cfg::new().size)]
    size: usize,
    /// Number of the points (resolution) in curve production
    #[clap(long, default_value_t = cb::Cfg::new().res)]
    res: usize,
    /// Number of harmonics
    #[clap(long, default_value_t = cb::Cfg::new().harmonic)]
    harmonic: usize,
    /// Fix the seed to get a determined result, default to random
    #[clap(short, long)]
    seed: Option<u64>,
}

impl Entry {
    pub(super) fn parse() {
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
    #[cfg(all(windows, not(debug_assertions)))]
    unsafe {
        winapi::um::wincon::FreeConsole();
    }
    eframe::run_native("Four-bar", opt, Box::new(|ctx| App::new(ctx, files)));
}

fn codebook(cb: CbCfg) {
    let CbCfg { mut file, is_open, size, res, harmonic, seed } = cb;
    let ext = file.extension().and_then(std::ffi::OsStr::to_str);
    if !matches!(ext, Some("npy")) {
        file.set_extension("npy");
    }
    println!("Generate to: {}", file.display());
    println!("open={is_open}, size={size}, res={res}, harmonic={harmonic}");
    let cfg = cb::Cfg { is_open, size, res, harmonic, seed: seed.into() };
    let pb = indicatif::ProgressBar::new(size as u64);
    cb::Codebook::make_with(cfg, |n| pb.set_position(n as u64))
        .write(std::fs::File::create(file).unwrap())
        .unwrap();
    pb.finish_and_clear();
    println!("Done");
}
