use pokerkit::Chips;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct HistoryDef {
    /// "P0" or "P1" — parsed by `Turn`'s Deserialize impl.
    pub turn: String,
    /// Postflop-only: board template with `*` placeholder for hole cards.
    /// e.g. "* ~ Kh 7d 2c". Preflop entries omit this.
    #[serde(default, rename = "_seen")]
    pub seen_template: Option<String>,
    /// Voluntary action history. Each string parsed by `Action::try_from`.
    pub past: Vec<String>,
    /// Override default 100bb HU stacks. Use to probe the same action
    /// sequence at a different SPR bucket.
    #[serde(default)]
    pub stacks: Option<[Chips; 2]>,
    /// SPR-bucket assertion: runner ERRORs if the constructed witness lands
    /// outside this bucket. Catches drift when editing `stacks` or `past`.
    /// Values: "committed" | "low" | "mid" | "deep".
    #[serde(default)]
    pub expected_spr: Option<String>,
    #[allow(dead_code)]
    pub desc: String,
}

impl HistoryDef {
    pub fn turn(&self) -> &str {
        &self.turn
    }

    pub fn seen_template(&self) -> Option<&String> {
        self.seen_template.as_ref()
    }

    pub fn past(&self) -> &[String] {
        &self.past
    }

    pub fn stacks(&self) -> Option<[Chips; 2]> {
        self.stacks
    }

    pub fn expected_spr(&self) -> Option<&String> {
        self.expected_spr.as_ref()
    }

    pub fn desc(&self) -> &str {
        &self.desc
    }
}
