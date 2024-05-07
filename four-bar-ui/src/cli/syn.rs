use crate::{io, syn_cmd::*};
use four_bar::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};

mod logger;
mod solver;

macro_rules! impl_err_from {
    ($(($ty:ty, $kind:ident)),+ $(,)?) => {$(
        impl From<$ty> for SynErr {
            fn from(e: $ty) -> Self { Self::$kind(e) }
        }
    )+};
}

#[derive(Debug)]
pub(crate) enum SynErr {
    Format,
    Io(std::io::Error),
    Plot(plot::DrawingAreaErrorKind<std::io::Error>),
    Gif(image::ImageError),
    CsvSer(csv::Error),
    RonSerde(ron::error::SpannedError),
    RonIo(ron::error::Error),
    Linkage,
}

impl std::fmt::Display for SynErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Format => write!(f, "unsupported format"),
            Self::Io(e) => write!(f, "[IO] {e}"),
            Self::Plot(e) => write!(f, "[Plot] {e}"),
            Self::Gif(e) => write!(f, "[GIF] {e}"),
            Self::CsvSer(e) => write!(f, "[CSV] {e}"),
            Self::RonSerde(e) => write!(f, "[RON-Serde] {e}"),
            Self::RonIo(e) => write!(f, "[RON-IO] {e}"),
            Self::Linkage => write!(f, "invalid linkage input"),
        }
    }
}
impl std::error::Error for SynErr {}
impl_err_from!(
    (std::io::Error, Io),
    (plot::DrawingAreaErrorKind<std::io::Error>, Plot),
    (image::ImageError, Gif),
    (csv::Error, CsvSer),
    (ron::error::SpannedError, RonSerde),
    (ron::error::Error, RonIo),
);
impl From<String> for SynErr {
    fn from(e: String) -> Self {
        Self::Io(std::io::Error::other(e))
    }
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
    /// Save GIF video of the result linkage
    ///
    /// This function compresses and optimizes the GIF file, which may take a
    /// long time
    #[clap(long)]
    video: bool,
    /// Disable parallel for running all tasks, use a single loop for
    /// benchmarking
    #[clap(long)]
    each: bool,
    /// Provide pre-generated atlas databases, support multiple paths joined by
    /// ";" (Windows) or ":" (Unix) characters
    #[clap(long)]
    atlas: Option<std::ffi::OsString>,
    /// Competitor (reference) folder path, under the same folder of the target
    /// file
    ///
    /// The reference file should be named as "[target_name].[target_mode].ron"
    #[clap(short, long, default_value = "refer")]
    refer: PathBuf,
    /// The legend position of the plot
    ///
    /// Defalut to upper right (ur), override when redrawing
    #[clap(long)]
    legend: Option<plot::LegendPos>,
    /// Disable reference comparison
    #[clap(long)]
    no_ref: bool,
    #[clap(flatten)]
    cfg: SynCfg,
    #[clap(subcommand)]
    alg: Option<SynAlg>,
}

pub(crate) struct Info<'a> {
    pub(crate) pb: ProgressBar,
    pub(crate) root: PathBuf,
    pub(crate) title: String,
    pub(crate) mode: syn::Mode,
    pub(crate) refer: Option<&'a Path>,
    pub(crate) legend: Option<plot::LegendPos>,
    pub(crate) rerun: bool,
    pub(crate) video: bool,
}

pub(super) fn loader(syn: Syn) {
    let Syn {
        files,
        each,
        cfg,
        mut atlas,
        refer,
        no_ref,
        alg,
        rerun,
        clean,
        video,
        legend,
    } = syn;
    println!("=====");
    if let Some(seed) = cfg.seed {
        print!("seed={seed} ");
    }
    println!("gen={} pop={} res={}", cfg.gen, cfg.pop, cfg.res);
    // If rerun is disabled, the atlas will be ignored
    if !rerun {
        atlas = None;
    }
    println!("rerun={rerun} clean={clean} dd={}", cfg.use_dd);
    println!("-----");
    // Reference folder path
    let refer = (!no_ref).then_some(refer.as_path());
    // Load atlas
    let atlas = atlas
        .map(|atlas| std::env::split_paths(&atlas).collect::<Vec<_>>())
        .unwrap_or_default();
    let atlas = if atlas.is_empty() {
        None
    } else {
        println!("Loading atlas database...");
        Some(
            atlas
                .into_iter()
                .map(|path| Ok(io::Atlas::from_reader(std::fs::File::open(path)?)?))
                .collect::<Result<io::AtlasPool, Box<dyn std::error::Error>>>()
                .expect("Load atlas failed"),
        )
    };
    let atlas_ref = atlas.as_ref();
    // Progress bar
    const STYLE: &str = "{eta} {wide_bar} {percent}%";
    let pb = ProgressBar::new(0);
    pb.set_style(ProgressStyle::with_template(STYLE).unwrap());
    // Load target files & create project folders
    let tasks = files
        .into_iter()
        .filter_map(|file| file.canonicalize().ok().filter(|f| f.is_file()))
        .filter_map(|file| {
            let title = file.file_stem().and_then(|p| p.to_str())?;
            // FIXME: Try block
            let info_ret = (|| {
                let ext = file.extension().and_then(|p| p.to_str());
                macro_rules! check {
                    ($c:expr) => {
                        efd::util::valid_curve($c).ok_or(SynErr::Linkage)?.into()
                    };
                    (@ $c:expr) => {{
                        let c = $c;
                        if c.len() < 3
                            || c.iter()
                                .flat_map(|(c, v)| c.iter().chain(v))
                                .any(|x| !x.is_finite())
                        {
                            return Err(SynErr::Linkage);
                        } else {
                            c.into()
                        }
                    }};
                }
                let target = match ext.ok_or(SynErr::Format)? {
                    "csv" | "txt" => {
                        match io::Curve::from_csv_reader(std::fs::File::open(&file)?)? {
                            io::Curve::P(t) => {
                                Target::fb(check!(t), None, atlas_ref.map(|a| a.as_fb()))
                            }
                            io::Curve::M(t) => Target::mfb(check!(@t), None),
                            io::Curve::S(t) => {
                                Target::sfb(check!(t), None, atlas_ref.map(|a| a.as_sfb()))
                            }
                        }
                    }
                    "ron" => match ron::de::from_reader(std::fs::File::open(&file)?)? {
                        io::Fb::P(fb) => Target::fb(check!(fb.curve(cfg.res)), Some(fb), None),
                        io::Fb::M(fb) => Target::mfb(check!(@fb.pose_zipped(cfg.res)), Some(fb)),
                        io::Fb::S(fb) => Target::sfb(check!(fb.curve(cfg.res)), Some(fb), None),
                    },
                    _ => {
                        println!("Ignored: {}", file.display());
                        Err(SynErr::Format)?
                    }
                };
                let mode = match Path::new(title).extension().and_then(|p| p.to_str()) {
                    Some("closed") => syn::Mode::Closed,
                    Some("partial") => syn::Mode::Partial,
                    Some("open") => syn::Mode::Open,
                    _ => Err(SynErr::Format)?,
                };
                let parent = file.parent().unwrap();
                let root = if cfg.use_dd {
                    parent.join(format!("{title}.dd"))
                } else {
                    parent.join(title)
                };
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
                let pb = pb.clone();
                let info = Info { root, title, mode, refer, legend, rerun, video, pb };
                Ok((info, target))
            })();
            match info_ret {
                Ok(info) => Some(info),
                Err(SynErr::Format) => None,
                Err(e) => {
                    println!("Error in {title}: {e}");
                    None
                }
            }
        })
        .collect::<Vec<_>>();
    if tasks.is_empty() {
        panic!("No valid target files!");
    }
    if clean && !rerun {
        return;
    }
    // Tasks
    let alg = alg.unwrap_or_default();
    let run = |(info, target)| solver::run(alg.clone(), info, target, &cfg);
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
