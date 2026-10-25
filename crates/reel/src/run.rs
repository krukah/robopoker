use super::*;

/// Namespace for the main routines.
pub(crate) struct Run;

impl Run {
    /// One game's full asset suite: both tree orientations (with animated
    /// flipbooks), the triplet, and the equity curve.
    pub(crate) fn suite<G, P>(root: &std::path::Path, paint: &P, show: &Ensemble<G, P>, names: [&str; 4])
    where
        G: Atlas,
        P: Palette,
    {
        Run::stage(root, paint, names[0], &show.film(Orient::Landscape), true);
        Run::stage(root, paint, names[1], &show.film(Orient::Portrait), true);
        Run::stage(root, paint, names[2], &show.trio(), false);
        Run::stage(root, paint, names[3], &show.curve(), false);
    }

    /// Stages one panel: per-frame SVGs plus the representative still, and the
    /// animated flipbook for tree reels (`animate`).
    pub(crate) fn stage<P>(root: &std::path::Path, paint: &P, name: &str, panel: &dyn Panel, animate: bool)
    where
        P: Palette,
    {
        let (w, h) = panel.size();
        let frames = panel.frames();
        let assets = root.join("docs/assets/components");
        let staged = root.join("docs/assets/_frames").join(name);
        std::fs::remove_dir_all(&staged).ok();
        std::fs::create_dir_all(&staged).expect("stage dir");
        std::fs::create_dir_all(&assets).expect("assets dir");
        frames.iter().enumerate().for_each(|(i, frame)| {
            std::fs::write(staged.join(format!("{i:04}.svg")), frame.document(w, h, &paint.bg())).expect("frame");
        });
        let still = frames[frames.len() / 3].document(w, h, &paint.bg());
        std::fs::write(staged.join("still.svg"), &still).expect("still");
        let cover = if animate { Frame::book(&frames, w, h, &paint.bg()) } else { still };
        std::fs::write(assets.join(format!("{name}.svg")), cover).expect("cover");
        println!("{name}: {} frames @ {FPS}fps, {w:.0}x{h:.0}", frames.len());
    }
}
