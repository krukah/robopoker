use super::*;

#[test]
fn players_suffix_matches_build() {
    #[cfg(not(feature = "sixmax"))]
    assert_eq!(Signals::players_suffix(), "", "heads-up must keep historical bare table names");
    #[cfg(feature = "sixmax")]
    assert_eq!(Signals::players_suffix(), "_6max", "6-max must isolate training tables by player count");
}
