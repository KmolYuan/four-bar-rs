use crate::{app::App, syn_cmd};
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
    /// Force to rerun the result
    #[clap(long)]
    rerun: bool,
    /// Remove related project folders
    #[clap(long, alias = "clear")]
    clean: bool,
    /// Disable parallel for running all tasks
    #[clap(long)]
    one_by_one: bool,
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
    #[clap(long, default_value_t = 45.)]
    font: f64,
    /// Reference number
    #[clap(long)]
    ref_num: Option<std::num::NonZeroU8>,
    /// Linkage input angle (degrees) in the plot
    #[clap(long)]
    angle: Option<f64>,
    /// Legend position
    #[clap(long, default_value = "ll")]
    legend_pos: four_bar::plot2d::LegendPos,
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
        impl_fmt!(self, res, gen, pop, seed, font);
        Ok(())
    }
}

impl std::ops::Deref for SynCfg {
    type Target = syn_cmd::SynConfig;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(clap::Args)]
struct CbCfg {
    /// Output path of the codebook (in NPZ format)
    file: PathBuf,
    /// Generate for open curve
    #[clap(long)]
    is_open: bool,
    /// Generate for spherical linkage
    #[clap(long)]
    sphere: bool,
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
    eframe::run_native("Four-bar", opt, Box::new(|ctx| App::new(ctx, files))).unwrap();
}

fn codebook(cb: CbCfg) {
    let CbCfg {
        mut file,
        is_open,
        sphere,
        size,
        res,
        harmonic,
        seed,
    } = cb;
    let ext = file.extension().and_then(std::ffi::OsStr::to_str);
    if !matches!(ext, Some("npz")) {
        file.set_extension("npz");
    }
    println!("Generate to: {}", file.display());
    println!("open={is_open}, size={size}, res={res}, harmonic={harmonic}");
    let t0 = std::time::Instant::now();
    let cfg = cb::Cfg { is_open, size, res, harmonic, seed: seed.into() };
    let pb = indicatif::ProgressBar::new(size as u64);
    let fs = std::fs::File::create(file).unwrap();
    let callback = |n| pb.set_position(n as u64);
    if sphere {
        cb::SFbCodebook::make_with(cfg, callback).write(fs).unwrap();
    } else {
        cb::FbCodebook::make_with(cfg, callback).write(fs).unwrap();
    }
    let t0 = t0.elapsed();
    pb.finish_and_clear();
    println!("Time spent: {t0:?}");
    println!("Done");
}
