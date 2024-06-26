use super::*;
use crate::app::GIF_RES;
use four_bar::plot::Style;
use plot::{full_palette::*, RGBColor};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

const HISTORY_SVG: &str = "history.svg";
const TAR_SVG: &str = "target.svg";
const TAR_FIG: &str = "target.fig.ron";
const LNK_RON: &str = "linkage.ron";
const LNK_SVG: &str = "linkage.svg";
const LNK_FIG: &str = "linkage.fig.ron";
const LNK_GIF: &str = "linkage.gif";
const EFD_CSV: &str = "target-efd.csv";
const EFD_CRUVE_CSV: &str = "target-curve-efd.csv";
const EFD_POSE_CSV: &str = "target-pose-efd.csv";
const CURVE_SVG: &str = "curve.svg";
const CURVE_FIG: &str = "curve.fig.ron";
const TARGET_COLOR: RGBColor = RED;
const SYN_COLOR: RGBColor = BLUE_900;
const ATLAS_COLOR: RGBColor = GREEN_900;
const REF_COLOR: RGBColor = ORANGE_900;

macro_rules! gif_video {
    ($info:ident, $root:ident, $fig:ident) => {
        if $info.video {
            let legend = $fig.legend;
            $fig.legend = plot::LegendPos::Hide;
            gif_video($root, &$fig, &$info.pb)?;
            $fig.legend = legend;
        }
    };
}

fn gif_video<M, const D: usize>(
    root: &Path,
    fig: &plot::FigureBase<M, [f64; D]>,
    pb: &ProgressBar,
) -> Result<(), SynErr>
where
    M: Clone + mech::CurveGen<D>,
    for<'a, 'b> plot::FigureBase<'a, 'b, M, [f64; D]>: plot::Plot,
{
    use image::{codecs::gif, DynamicImage, Frame, RgbImage};
    let mut buf = Vec::new();
    let mut w = gif::GifEncoder::new_with_speed(&mut buf, 30);
    w.set_repeat(gif::Repeat::Infinite)?;
    pb.inc_length(GIF_RES as u64);
    for curr in 0..GIF_RES {
        const SIZE: u32 = 1600;
        const BUF_SIZE: usize = (SIZE * SIZE) as usize * 3;
        let mut frame = vec![0; BUF_SIZE];
        let b = plot::BitMapBackend::with_buffer(&mut frame, (SIZE, SIZE));
        fig.plot_video(b, curr, GIF_RES)
            .map_err(|e| format!("{e}"))?;
        let image = RgbImage::from_vec(SIZE, SIZE, frame).unwrap_or_else(|| unreachable!());
        w.encode_frame(Frame::new(DynamicImage::from(image).into_rgba8()))?;
        pb.inc(1);
    }
    drop(w);
    std::fs::write(root.join(LNK_GIF), buf)?;
    Ok(())
}

impl<M, const N: usize, const D: usize> PSynData<'_, M::De, syn::PathSyn<M, N, D>, D>
where
    syn::PathSyn<M, N, D>: mh::ObjFunc<Ys = mh::WithProduct<f64, M::De>>,
    M: atlas::Code<N, D>,
    M::De: mech::CurveGen<D>
        + serde::Serialize
        + serde::de::DeserializeOwned
        + Default
        + Clone
        + Sync
        + Send
        + 'static,
    efd::U<D>: efd::EfdDim<D>,
    efd::Efd<D>: Sync,
    for<'f1, 'f2> plot::FigureBase<'f1, 'f2, M::De, [f64; D]>: plot::Plot + serde::Serialize,
{
    fn solve_cli(
        self,
        cfg: &SynCfg,
        info: &Info,
        history: Arc<Mutex<Vec<f64>>>,
    ) -> Result<(), SynErr> {
        use four_bar::mech::CurveGen as _;
        let Self { s, tar_curve, tar_fb, atlas_fb } = self;
        let Info { root, title, mode, refer, .. } = info;
        let t0 = std::time::Instant::now();
        let s = s.solve();
        let t1 = t0.elapsed();
        let func = s.func();
        let harmonic = func.harmonic();
        let tar_efd = func.tar.clone();
        let (cost, fb) = s.into_err_result();
        {
            let path = root.join(HISTORY_SVG);
            let svg = plot::SVGBackend::new(&path, (800, 600));
            plot::fb::history(svg, Arc::into_inner(history).unwrap().into_inner().unwrap())?;
        }
        let refer = refer
            .map(|p| root.join("..").join(p).join(format!("{title}.ron")))
            .filter(|p| p.is_file());
        let mut log = std::fs::File::create(root.join(format!("{title}.log")))?;
        let mut log = super::logger::Logger::new(&mut log);
        log.top_title(title)?;
        write_tar_efd(root.join(EFD_CSV), &tar_efd)?;
        write_ron(root.join(LNK_RON), &fb)?;
        let curve = fb.curve(cfg.res);
        let mut fig = plot::FigureBase::new();
        if let Some(legend) = info.legend {
            fig.legend = legend;
        }
        fig.push_line("Target", &*tar_curve, Style::Circle, TARGET_COLOR);
        {
            write_ron(root.join(TAR_FIG), &fig)?;
            let path = root.join(TAR_SVG);
            let svg = plot::SVGBackend::new(&path, (1600, 1600));
            fig.plot(svg)?;
        }
        fig.set_fb_ref(&fb);
        fig.push_line("Optimized", &curve, Style::Line, SYN_COLOR);
        {
            write_ron(root.join(LNK_FIG), &fig)?;
            let path = root.join(LNK_SVG);
            let svg = plot::SVGBackend::new(&path, (1600, 1600));
            fig.plot(svg)?;
        }
        gif_video!(info, root, fig);
        if let Some(fb) = tar_fb {
            log.title("target.fb")?;
            log.log(fb)?;
        }
        if let Some((cost, fb)) = atlas_fb {
            let curve = fb.curve(cfg.res);
            log.title("atlas")?;
            log.log(Performance::cost(cost, &tar_curve, &curve).harmonic(harmonic))?;
            log.title("atlas.fb")?;
            log.log(&fb)?;
            write_ron(root.join("atlas.ron"), &fb)?;
            fig.push_line("Atlas", curve, Style::Triangle, ATLAS_COLOR);
        }
        log.title("optimized")?;
        log.log(
            Performance::cost(cost, &tar_curve, &curve)
                .time(t1)
                .harmonic(harmonic),
        )?;
        log.title("optimized.fb")?;
        log.log(&fb)?;
        if let Some(refer) = refer {
            let fb = ron::de::from_reader::<_, M::De>(std::fs::File::open(refer)?)?;
            let c = fb.curve(cfg.res);
            log.title("competitor")?;
            if matches!(mode, syn::Mode::Partial) {
                log.log(Performance::dist_err(&tar_curve, &c))?;
            } else {
                let efd = efd::Efd::from_curve_harmonic(&c, mode.is_result_open(), harmonic);
                log.log(Performance::cost(efd.err(&tar_efd), &tar_curve, &c))?;
            }
            log.title("competitor.fb")?;
            log.log(&fb)?;
            fig.push_line("Ref. [?]", c, Style::DashedLine, REF_COLOR);
        }
        fig.remove_fb();
        write_ron(root.join(CURVE_FIG), &fig)?;
        let path = root.join(CURVE_SVG);
        let svg = plot::SVGBackend::new(&path, (1600, 1600));
        fig.plot(svg)?;
        log.flush()?;
        Ok(())
    }
}

impl<M, const N: usize, const D: usize> PSynData<'_, M::De, syn::DDPathSyn<M, N, D>, D>
where
    syn::DDPathSyn<M, N, D>: mh::ObjFunc<Ys = mh::WithProduct<f64, M::De>>,
    M: atlas::Code<N, D>,
    M::De: mech::CurveGen<D>
        + serde::Serialize
        + serde::de::DeserializeOwned
        + Default
        + Clone
        + Sync
        + Send
        + 'static,
    efd::U<D>: efd::EfdDim<D>,
    efd::Efd<D>: Sync,
    for<'f1, 'f2> plot::FigureBase<'f1, 'f2, M::De, [f64; D]>: plot::Plot + serde::Serialize,
{
    fn solve_cli(
        self,
        cfg: &SynCfg,
        info: &Info,
        history: Arc<Mutex<Vec<f64>>>,
    ) -> Result<(), SynErr> {
        use four_bar::mech::CurveGen as _;
        let Self { s, tar_curve, tar_fb, atlas_fb } = self;
        let Info { root, title, refer, mode, .. } = info;
        let t0 = std::time::Instant::now();
        let s = s.solve();
        let t1 = t0.elapsed();
        let (cost, fb, func) = s.into_err_result_func();
        let tar_sig = func.tar;
        {
            let path = root.join(HISTORY_SVG);
            let svg = plot::SVGBackend::new(&path, (800, 600));
            plot::fb::history(svg, Arc::into_inner(history).unwrap().into_inner().unwrap())?;
        }
        let refer = refer
            .map(|p| root.join("..").join(p).join(format!("{title}.ron")))
            .filter(|p| p.is_file());
        let mut log = std::fs::File::create(root.join(format!("{title}.log")))?;
        let mut log = super::logger::Logger::new(&mut log);
        log.top_title(title)?;
        write_ron(root.join(LNK_RON), &fb)?;
        let curve = fb.curve(cfg.res);
        let mut fig = plot::FigureBase::new();
        if let Some(legend) = info.legend {
            fig.legend = legend;
        }
        fig.push_line("Target", &*tar_curve, Style::Circle, TARGET_COLOR);
        {
            write_ron(root.join(TAR_FIG), &fig)?;
            let path = root.join(TAR_SVG);
            let svg = plot::SVGBackend::new(&path, (1600, 1600));
            fig.plot(svg)?;
        }
        fig.set_fb_ref(&fb);
        fig.push_line("Optimized", &curve, Style::Line, SYN_COLOR);
        {
            write_ron(root.join(LNK_FIG), &fig)?;
            let path = root.join(LNK_SVG);
            let svg = plot::SVGBackend::new(&path, (1600, 1600));
            fig.plot(svg)?;
        }
        gif_video!(info, root, fig);
        if let Some(fb) = tar_fb {
            log.title("target.fb")?;
            log.log(fb)?;
        }
        if let Some((cost, fb)) = atlas_fb {
            let curve = fb.curve(cfg.res);
            log.title("atlas")?;
            log.log(Performance::cost(cost, &tar_curve, &curve))?;
            log.title("atlas.fb")?;
            log.log(&fb)?;
            write_ron(root.join("atlas.ron"), &fb)?;
            fig.push_line("Atlas", curve, Style::Triangle, ATLAS_COLOR);
        }
        log.title("optimized")?;
        log.log(Performance::cost(cost, &tar_curve, &curve).time(t1))?;
        log.title("optimized.fb")?;
        log.log(&fb)?;
        if let Some(refer) = refer {
            let fb = ron::de::from_reader::<_, M::De>(std::fs::File::open(refer)?)?;
            let c = fb.curve(cfg.res);
            log.title("competitor")?;
            if matches!(mode, syn::Mode::Partial) {
                log.log(Performance::dist_err(&tar_curve, &c))?;
            } else {
                let efd = efd::Efd::from_curve(&c, mode.is_result_open());
                log.log(Performance::cost(efd.err_sig(&tar_sig), &tar_curve, &c))?;
            }
            log.title("competitor.fb")?;
            log.log(&fb)?;
            fig.push_line("Ref. [?]", c, Style::DashedLine, REF_COLOR);
        }
        fig.remove_fb();
        write_ron(root.join(CURVE_FIG), &fig)?;
        let path = root.join(CURVE_SVG);
        let svg = plot::SVGBackend::new(&path, (1600, 1600));
        fig.plot(svg)?;
        log.flush()?;
        Ok(())
    }
}

impl MSynData<'_, syn::MOFit, syn::MFbSyn> {
    fn solve_cli(
        self,
        cfg: &SynCfg,
        info: &Info,
        history: Arc<Mutex<Vec<f64>>>,
    ) -> Result<(), SynErr> {
        let Self { s, tar_p, tar_v, tar_fb } = self;
        let Info { root, title, mode, refer, .. } = info;
        let t0 = std::time::Instant::now();
        let s = s.solve();
        let t1 = t0.elapsed();
        let func = s.func();
        let harmonic = func.harmonic();
        let tar_efd = func.tar.clone();
        let (cost, fb) = s.into_err_result();
        {
            let path = root.join(HISTORY_SVG);
            let svg = plot::SVGBackend::new(&path, (800, 600));
            plot::fb::history(svg, Arc::into_inner(history).unwrap().into_inner().unwrap())?;
        }
        let refer = refer
            .map(|p| root.join("..").join(p).join(format!("{title}.ron")))
            .filter(|p| p.is_file());
        let mut log = std::fs::File::create(root.join(format!("{title}.log")))?;
        let mut log = super::logger::Logger::new(&mut log);
        log.top_title(title)?;
        write_tar_efd(root.join(EFD_CRUVE_CSV), tar_efd.as_curve())?;
        write_tar_efd(root.join(EFD_POSE_CSV), tar_efd.as_pose())?;
        write_ron(root.join(LNK_RON), &fb)?;
        let length = tar_efd.as_curve().as_geo().scale();
        let (curve_p, curve_q) = fb.ext_curve(length, cfg.res);
        let mut fig = plot::mfb::Figure::new();
        if let Some(legend) = info.legend {
            fig.legend = legend;
        }
        let tar_q = efd::posed::guide_from_curve(&tar_p, tar_v, length);
        fig.push_series("Target", (&tar_p, &tar_q), Style::Line, TARGET_COLOR, false);
        {
            let t = efd::get_norm_t(&tar_p, true);
            let curve_p = tar_efd.as_curve().recon_by(&t);
            let pose = tar_efd.as_pose().recon_by(&t);
            let curve_q = tar_efd.as_curve().as_geo().transform(pose);
            fig.push_series(
                "Target Recon.",
                (curve_p, curve_q),
                Style::DashedLine,
                SYN_COLOR,
                false,
            );
            write_ron(root.join(TAR_FIG), &fig)?;
            let path = root.join(TAR_SVG);
            let svg = plot::SVGBackend::new(&path, (1600, 1600));
            fig.plot(svg)?;
            fig.lines.pop();
        }
        fig.set_fb_ref(fb.as_fb());
        fig.push_series(
            "Optimized",
            (&curve_p, &curve_q),
            Style::DashedLine,
            SYN_COLOR,
            true,
        );
        {
            write_ron(root.join(LNK_FIG), &fig)?;
            let path = root.join(LNK_SVG);
            let svg = plot::SVGBackend::new(&path, (1600, 1600));
            fig.plot(svg)?;
        }
        gif_video!(info, root, fig);
        if let Some(fb) = tar_fb {
            log.title("target.fb")?;
            log.log(fb)?;
        }
        log.title("optimized")?;
        log.log(
            Performance::cost_m(cost, &tar_p, &tar_q, &curve_p, &curve_q)
                .time(t1)
                .harmonic(harmonic),
        )?;
        log.title("optimized.fb")?;
        log.log(&fb)?;
        if let Some(refer) = refer {
            let fb = ron::de::from_reader::<_, MFourBar>(std::fs::File::open(refer)?)?;
            let (c, v) = fb.pose(cfg.res);
            let c_q = efd::posed::guide_from_curve(&c, &v, length);
            log.title("competitor")?;
            if matches!(mode, syn::Mode::Partial) {
                log.log(Performance::dist_err_m(&tar_p, &tar_q, &c, &c_q))?;
            } else {
                let efd =
                    efd::PosedEfd::from_uvec_harmonic(&c, &v, mode.is_result_open(), harmonic);
                let cost = efd.err(&tar_efd);
                log.log(Performance::cost_m(cost, &tar_p, &tar_q, &c, &c_q))?;
            }
            log.title("competitor.fb")?;
            log.log(&fb)?;
            fig.push_series(
                "Ref. [?]",
                (c, c_q),
                Style::DashDottedLine,
                ATLAS_COLOR,
                true,
            );
        }
        fig.remove_fb();
        write_ron(root.join(CURVE_FIG), &fig)?;
        let path = root.join(CURVE_SVG);
        let svg = plot::SVGBackend::new(&path, (1600, 1600));
        fig.plot(svg)?;
        log.flush()?;
        Ok(())
    }
}

impl MSynData<'_, f64, syn::MFbDDSyn> {
    fn solve_cli(
        self,
        cfg: &SynCfg,
        info: &Info,
        history: Arc<Mutex<Vec<f64>>>,
    ) -> Result<(), SynErr> {
        let Self { s, tar_p, tar_v, tar_fb } = self;
        let Info { root, title, refer, mode, .. } = info;
        let t0 = std::time::Instant::now();
        let s = s.solve();
        let t1 = t0.elapsed();
        let (cost, fb, func) = s.into_err_result_func();
        let tar_sig = func.tar;
        {
            let path = root.join(HISTORY_SVG);
            let svg = plot::SVGBackend::new(&path, (800, 600));
            plot::fb::history(svg, Arc::into_inner(history).unwrap().into_inner().unwrap())?;
        }
        let refer = refer
            .map(|p| root.join("..").join(p).join(format!("{title}.ron")))
            .filter(|p| p.is_file());
        let mut log = std::fs::File::create(root.join(format!("{title}.log")))?;
        let mut log = super::logger::Logger::new(&mut log);
        log.top_title(title)?;
        write_ron(root.join(LNK_RON), &fb)?;
        let length = tar_sig.as_geo().scale();
        let (curve_p, curve_q) = fb.ext_curve(length, cfg.res);
        let mut fig = plot::mfb::Figure::new();
        if let Some(legend) = info.legend {
            fig.legend = legend;
        }
        let tar_q = efd::posed::guide_from_curve(&tar_p, tar_v, length);
        fig.push_series("Target", (&tar_p, &tar_q), Style::Line, TARGET_COLOR, false);
        {
            let efd =
                efd::PosedEfd::from_series_harmonic(&curve_p, &curve_q, mode.is_target_open(), 1);
            let curve_p = efd.as_curve().recon_by(tar_sig.as_t());
            let pose = efd.as_pose().recon_by(tar_sig.as_t());
            let curve_q = efd.as_curve().as_geo().transform(pose);
            fig.push_series(
                "DD Recon.",
                (curve_p, curve_q),
                Style::DashedLine,
                SYN_COLOR,
                false,
            );
            write_ron(root.join(TAR_FIG), &fig)?;
            let path = root.join(TAR_SVG);
            let svg = plot::SVGBackend::new(&path, (1600, 1600));
            fig.plot(svg)?;
            fig.lines.pop();
        }
        fig.set_fb_ref(fb.as_fb());
        fig.push_series(
            "Optimized",
            (&curve_p, &curve_q),
            Style::DashedLine,
            SYN_COLOR,
            true,
        );
        {
            write_ron(root.join(LNK_FIG), &fig)?;
            let path = root.join(LNK_SVG);
            let svg = plot::SVGBackend::new(&path, (1600, 1600));
            fig.plot(svg)?;
        }
        gif_video!(info, root, fig);
        if let Some(fb) = tar_fb {
            log.title("target.fb")?;
            log.log(fb)?;
        }
        log.title("optimized")?;
        log.log(Performance::cost_m(cost, &tar_p, &tar_q, &curve_p, &curve_q).time(t1))?;
        log.title("optimized.fb")?;
        log.log(&fb)?;
        if let Some(refer) = refer {
            let fb = ron::de::from_reader::<_, MFourBar>(std::fs::File::open(refer)?)?;
            let (c, v) = fb.pose(cfg.res);
            let c_q = efd::posed::guide_from_curve(&c, &v, length);
            log.title("competitor")?;
            if matches!(mode, syn::Mode::Partial) {
                log.log(Performance::dist_err_m(&tar_p, &tar_q, &c, &c_q))?;
            } else {
                let efd = efd::PosedEfd::from_uvec(&c, &v, mode.is_result_open());
                let cost = efd.err_sig(&tar_sig);
                log.log(Performance::cost_m(cost, &tar_p, &tar_q, &c, &c_q))?;
            }
            log.title("competitor.fb")?;
            log.log(&fb)?;
            fig.push_series(
                "Ref. [?]",
                (c, c_q),
                Style::DashDottedLine,
                ATLAS_COLOR,
                true,
            );
        }
        fig.remove_fb();
        write_ron(root.join(CURVE_FIG), &fig)?;
        let path = root.join(CURVE_SVG);
        let svg = plot::SVGBackend::new(&path, (1600, 1600));
        fig.plot(svg)?;
        log.flush()?;
        Ok(())
    }
}

pub(crate) fn run(alg: SynAlg, info: Info, target: Target, cfg: &SynCfg) {
    let root = &info.root;
    let ret = if !info.rerun && root.join(LNK_FIG).is_file() && root.join(CURVE_FIG).is_file() {
        from_exist(&target, &info)
    } else {
        info.pb.inc_length(cfg.gen);
        from_runtime(alg, target, cfg, &info)
    };
    match ret {
        Ok(()) => info.pb.println(format!("Finished: {}", info.title)),
        Err(e) => info.pb.println(format!("Error in {}: {e}", info.title)),
    }
}

fn from_runtime(alg: SynAlg, target: Target, cfg: &SynCfg, info: &Info) -> Result<(), SynErr> {
    let history = Arc::new(Mutex::new(Vec::with_capacity(cfg.gen as usize)));
    let s = {
        let pb = info.pb.clone();
        let history = history.clone();
        let cfg = SynCfg { mode: info.mode, ..cfg.clone() };
        let stop = || false;
        Solver::new(alg, target, cfg, stop, move |best_f, _| {
            history.lock().unwrap().push(best_f);
            pb.inc(1);
        })
    };
    match s {
        Solver::Fb(s) => s.solve_cli(cfg, info, history),
        Solver::MFb(s) => s.solve_cli(cfg, info, history),
        Solver::SFb(s) => s.solve_cli(cfg, info, history),
        Solver::DDFb(s) => s.solve_cli(cfg, info, history),
        Solver::DDSFb(s) => s.solve_cli(cfg, info, history),
        Solver::DDMFb(s) => s.solve_cli(cfg, info, history),
    }
}

fn from_exist(target: &Target, info: &Info) -> Result<(), SynErr> {
    const PATH_LIST: [(&str, &str); 3] = [
        (TAR_FIG, TAR_SVG),
        (LNK_FIG, LNK_SVG),
        (CURVE_FIG, CURVE_SVG),
    ];
    let root = &info.root;
    macro_rules! plot {
        ($ty:ty) => {{
            for (path, svg_path) in PATH_LIST {
                let mut fig =
                    ron::de::from_reader::<_, $ty>(std::fs::File::open(root.join(path))?)?;
                if let Some(legend) = info.legend {
                    fig.legend = legend;
                }
                fig.plot(plot::SVGBackend::new(&root.join(svg_path), (1600, 1600)))?;
                if path == LNK_FIG {
                    gif_video!(info, root, fig);
                }
            }
            Ok(())
        }};
    }
    match target {
        // HINT: `fb::Figure` and `mfb::Figure` are the same type
        Target::Fb { .. } | Target::MFb { .. } => plot!(plot::fb::Figure),
        Target::SFb { .. } => plot!(plot::sfb::Figure),
    }
}

#[derive(serde::Serialize)]
struct Performance {
    #[serde(rename = "dist-err")]
    dist_err: f64,
    cost: Option<f64>,
    #[serde(serialize_with = "ser_time")]
    time: Option<std::time::Duration>,
    harmonic: Option<usize>,
}

fn ser_time<S>(time: &Option<std::time::Duration>, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match time {
        Some(time) => s.serialize_str(&format!("{:.3?}", time)),
        None => s.serialize_none(),
    }
}

impl Performance {
    fn dist_err<const D: usize>(tar: impl efd::Curve<D>, cur: impl efd::Curve<D>) -> Self {
        let dist_err = efd::util::dist_err(tar, cur);
        Self { dist_err, cost: None, time: None, harmonic: None }
    }

    fn cost<const D: usize>(cost: f64, tar: impl efd::Curve<D>, cur: impl efd::Curve<D>) -> Self {
        let dist_err = efd::util::dist_err(tar, cur);
        Self {
            cost: Some(cost),
            dist_err,
            time: None,
            harmonic: None,
        }
    }

    fn dist_err_m<const D: usize>(
        tar_p: impl efd::Curve<D>,
        tar_q: impl efd::Curve<D>,
        cur_p: impl efd::Curve<D>,
        cur_q: impl efd::Curve<D>,
    ) -> Self {
        let dist_err = efd::util::dist_err(tar_p, cur_p) + efd::util::dist_err(tar_q, cur_q);
        Self { dist_err, cost: None, time: None, harmonic: None }
    }

    fn cost_m<const D: usize>(
        cost: f64,
        tar_p: impl efd::Curve<D>,
        tar_q: impl efd::Curve<D>,
        cur_p: impl efd::Curve<D>,
        cur_q: impl efd::Curve<D>,
    ) -> Self {
        let dist_err = efd::util::dist_err(tar_p, cur_p) + efd::util::dist_err(tar_q, cur_q);
        Self {
            cost: Some(cost),
            dist_err,
            time: None,
            harmonic: None,
        }
    }

    fn time(self, time: std::time::Duration) -> Self {
        Self { time: Some(time), ..self }
    }

    fn harmonic(self, harmonic: usize) -> Self {
        Self { harmonic: Some(harmonic), ..self }
    }
}

fn write_ron<S>(path: impl AsRef<Path>, s: &S) -> Result<(), SynErr>
where
    S: serde::Serialize,
{
    std::fs::write(path, ron::ser::to_string_pretty(s, Default::default())?)?;
    Ok(())
}

fn write_tar_efd<const D: usize>(path: impl AsRef<Path>, efd: &efd::Efd<D>) -> Result<(), SynErr>
where
    efd::U<D>: efd::EfdDim<D>,
{
    use std::io::Write as _;
    let mut w = std::fs::File::create(path)?;
    for m in efd.coeffs_iter() {
        for (i, c) in m.iter().enumerate() {
            if i == m.len() - 1 {
                write!(w, "{c:.4}")?;
            } else {
                write!(w, "{c:.4},")?;
            }
        }
        writeln!(w)?;
    }
    w.flush()?;
    Ok(())
}
