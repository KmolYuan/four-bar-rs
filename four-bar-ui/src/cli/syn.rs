use super::Syn;
use four_bar::{
    codebook::Codebook,
    curve, mh, plot,
    syn::{Mode, PathSyn},
    FourBar, Mechanism,
};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use instant::Instant;
use std::path::{Path, PathBuf};

type AnyResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

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
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            Self::Format => "unsupported format",
            Self::Io => "reading file error",
            Self::Ser => "serialization error",
            Self::Linkage => "invalid linkage",
        };
        f.write_str(s)
    }
}

struct Info<'a> {
    target: Vec<[f64; 2]>,
    title: &'a str,
    mode: Mode,
}

pub(super) fn syn(files: Vec<PathBuf>, no_parallel: bool, syn: Syn, cb: Vec<PathBuf>) {
    let mpb = MultiProgress::new();
    if !cb.is_empty() {
        mpb.println("Loading codebook database...").unwrap();
    }
    let cb = load_codebook(cb).expect("Load codebook failed!");
    let run = |file: PathBuf| run(&mpb, file, syn.clone(), &cb);
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

fn load_codebook(cb: Vec<PathBuf>) -> AnyResult<Vec<Codebook>> {
    let mut v = Vec::with_capacity(cb.len());
    for path in cb {
        v.push(Codebook::read(std::fs::File::open(path)?)?);
    }
    Ok(v)
}

fn run(mpb: &MultiProgress, file: PathBuf, syn: Syn, cb: &[Codebook]) {
    let file = file.canonicalize().unwrap();
    let info = match info(&file, syn.n) {
        Ok(info) => info,
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
    pb.set_prefix(info.title.to_string());
    let root = file.parent().unwrap();
    // Codebook synthesis
    let res = if let Some((_, ans)) = cb
        .iter()
        .filter(|cb| cb.is_open() == info.mode.is_open())
        .map(|cb| cb.fetch_1st(&info.target))
        .min_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap())
    {
        draw_ans(root, info.title, &info.target, ans, syn.n)
    } else {
        // Optimization synthesis
        inner(&pb, info, root, syn)
    };
    if let Err(e) = res {
        pb.finish_with_message(format!("| error: {e}"));
    }
}

fn info(path: &Path, n: usize) -> Result<Info<'_>, SynErr> {
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
            Ok(Info { target, title, mode })
        })
}

fn inner(pb: &ProgressBar, info: Info, root: &Path, syn: Syn) -> AnyResult {
    let Info { target, title, mode } = info;
    let Syn { n, gen, pop } = syn;
    let target = target.as_slice();
    let t0 = Instant::now();
    let s = mh::Solver::build(mh::De::default())
        .task(|ctx| ctx.gen == gen)
        .callback(|ctx| pb.set_position(ctx.gen))
        .pop_num(pop)
        .record(|ctx| ctx.best_f)
        .solve(PathSyn::from_curve(target, None, n, mode))?;
    let spent_time = Instant::now() - t0;
    {
        let path = root.join(format!("{title}_history.svg"));
        let svg = plot::SVGBackend::new(&path, (800, 600));
        plot::history(svg, s.report())?;
    }
    let ans = s.result();
    draw_ans(root, title, target, ans, n)?;
    let harmonic = s.func().harmonic();
    pb.finish_with_message(format!("| spent: {spent_time:?} | harmonic: {harmonic}"));
    Ok(())
}

fn draw_ans(root: &Path, title: &str, target: &[[f64; 2]], ans: FourBar, n: usize) -> AnyResult {
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
    Ok(())
}
