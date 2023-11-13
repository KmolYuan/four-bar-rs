use four_bar::atlas;

#[derive(clap::Args)]
pub(super) struct AtlasCfg {
    /// Output path of the atlas (in NPZ format)
    file: std::path::PathBuf,
    /// Generate for open curve
    #[clap(long)]
    is_open: bool,
    /// Generate for spherical linkage
    #[clap(long)]
    sphere: bool,
    /// Number of data
    #[clap(long, default_value_t = atlas::Cfg::new().size)]
    size: usize,
    /// Number of the points (resolution) in curve production
    #[clap(long, default_value_t = atlas::Cfg::new().res)]
    res: usize,
    /// Number of harmonics
    #[clap(long, default_value_t = atlas::Cfg::new().harmonic)]
    harmonic: usize,
    /// Fix the seed to get a determined result, default to random
    #[clap(short, long)]
    seed: Option<u64>,
}

pub(super) fn atlas(atlas: AtlasCfg) {
    let AtlasCfg {
        mut file,
        is_open,
        sphere,
        size,
        res,
        harmonic,
        seed,
    } = atlas;
    let ext = file.extension().and_then(std::ffi::OsStr::to_str);
    if !matches!(ext, Some("npz")) {
        file.set_extension("npz");
    }
    println!("Generate to: {}", file.display());
    println!("open={is_open}, size={size}, res={res}, harmonic={harmonic}");
    let t0 = std::time::Instant::now();
    let cfg = atlas::Cfg { is_open, size, res, harmonic, seed: seed.into() };
    let pb = indicatif::ProgressBar::new(size as u64);
    let fs = std::fs::File::create(file).unwrap();
    let callback = |n| pb.set_position(n as u64);
    if sphere {
        atlas::SFbAtlas::make_with(cfg, callback).write(fs).unwrap();
    } else {
        atlas::FbAtlas::make_with(cfg, callback).write(fs).unwrap();
    }
    let t0 = t0.elapsed();
    pb.finish_and_clear();
    println!("Time spent: {t0:?}");
    println!("Done");
}
