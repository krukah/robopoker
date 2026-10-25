#[cfg(feature = "server")]
use super::*;
#[cfg(feature = "server")]
use mccfr::*;

/// A profile's rows in the blueprint table's wire format.
#[cfg(feature = "server")]
pub trait Rows {
    fn rows(&self) -> impl Iterator<Item = Wire> + '_;
}

#[cfg(feature = "server")]
impl Rows for Book<NlheInfo> {
    fn rows(&self) -> impl Iterator<Item = Wire> + '_ {
        self.encounters().iter().flat_map(|(info, edges)| {
            edges
                .iter()
                .map(move |(edge, encounter)| Wire::from((info, edge, encounter)))
        })
    }
}
