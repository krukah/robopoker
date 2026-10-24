//! Emits the reels in both of the site's registers: for each demonstrated
//! game, the tree in both orientations plus the triplet and equity panels,
//! once on `Paper` (`-light`) and once on `Ink` (`-dark`). Frames stage under
//! `docs/assets/_frames/` for an external rasterizer to turn into GIF + PNG.
mod run;
use run::*;

use reel::*;

fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    Run::register(&root, &Paper, "light");
    Run::register(&root, &Ink, "dark");
}
