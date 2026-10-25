//! Emits the reels: for each demonstrated game, the tree in both
//! orientations plus the triplet and equity panels — one generic suite,
//! instantiated per game. Frames stage under `docs/assets/_frames/` for an
//! external rasterizer to turn into GIF + PNG.
mod run;
use run::*;

use reel::*;

fn main() {
    let paint = Hex;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    Run::suite(
        &root,
        &paint,
        &Ensemble::<nlhe::NlheGame, Hex>::new(&paint),
        ["tree_v1_connected", "tree_v1_transposed", "triplet", "equity"],
    );
    #[cfg(feature = "kuhn")]
    Run::suite(
        &root,
        &paint,
        &Ensemble::<kuhn::KuhnGame, Hex>::new(&paint),
        ["tree_kuhn", "kuhn_tree_portrait", "kuhn_triplet", "kuhn_equity"],
    );
}
