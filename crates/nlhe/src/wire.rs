//! Blueprint row on the binary COPY wire.
//!
//! Requires the `server` feature.
use super::*;
use kicker::*;
use pokerkit::*;
use std::pin::Pin;
use tokio_postgres::binary_copy::BinaryCopyInWriter;

/// One blueprint row in domain types, column order `(past, present, choices,
/// context, edge, weight, regret, payoff, visits)`. Every field is passed to
/// postgres as its domain type — the SQL codecs live on the types themselves
/// (`Path`/`Field`/`Edge` under kicker's `sql` feature, [`NlheSecret`] here)
/// — so no hand-rolled primitive conversions leak into the profile or sinks.
/// Local struct rather than a bare tuple because the orphan rule rejects
/// `impl daybook::Row` for a tuple type.
pub struct Wire {
    past: Subgame,
    present: NlheSecret,
    choices: Path,
    context: Field,
    edge: Edge,
    weight: Probability,
    regret: Utility,
    payoff: Utility,
    visits: u32,
}

impl From<(&NlheInfo, &NlheEdge, &mccfr::Encounter)> for Wire {
    fn from((info, edge, encounter): (&NlheInfo, &NlheEdge, &mccfr::Encounter)) -> Self {
        Self {
            past: info.subgame(),
            present: info.bucket(),
            choices: info.choices(),
            context: info.field(),
            edge: Edge::from(*edge),
            weight: encounter.weight,
            regret: encounter.regret,
            payoff: encounter.payoff,
            visits: encounter.visits,
        }
    }
}

#[rustfmt::skip]
#[async_trait::async_trait]
impl daybook::Row for Wire {
    async fn write(self, writer: Pin<&mut BinaryCopyInWriter>) {
        writer
            .write(&[
                &self.past,
                &self.present,
                &self.choices,
                &self.context,
                &self.edge,
                &self.weight,
                &self.regret,
                &self.payoff,
                &(self.visits as i32),
            ])
            .await
            .expect("write");
    }
}
