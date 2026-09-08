//! One row of `mechinterp`: a bucket, a name, a description.
use crate::*;
use deuce::*;
use kicker::*;
use pokerkit::*;

/// The interpretable identity of one abstraction bucket: a short name a human
/// can hold in their head, and a description that spells out the measured
/// evidence behind it.
///
/// Both strings are *derived*, never authored — [`Gloss::from`] is a pure
/// function of a [`Profile`], so re-clustering and re-running the pipeline
/// reproduces the glossary rather than orphaning hand-written prose against
/// buckets that have since moved.
///
/// Serializes as `{abs, name, description}` — the wire shape the analysis
/// frontend reads off `/topology/gloss`.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Gloss {
    abs: Abstraction,
    name: String,
    description: String,
}

impl Gloss {
    /// A kicker is worth naming when this many paired members share it.
    const MAJORITY: Probability = 0.50;
    /// A draw or an overcard read is worth naming at this frequency.
    const PLURALITY: Probability = 0.40;
    /// A bucket that missed is named by its high card from here up.
    const MISSED: Probability = 0.40;
    /// A board texture is worth naming from here up — but only if it also
    /// clears [`Gloss::SURPRISING`] against the street.
    const HALF: Probability = 0.50;
    /// Frequency points above the street's own rate before a feature counts
    /// as telling you something. Nearly every flop is two-tone; saying so is
    /// not interpretation.
    const SURPRISING: Probability = 0.15;
    /// A feature is worth listing in the description at this frequency.
    const NOTABLE: Probability = 0.05;
    /// Qualifiers allowed after the class, so a name stays a name.
    const TERMS: usize = 2;
    /// Features listed per description section.
    const LISTED: usize = 4;

    pub fn abs(&self) -> Abstraction {
        self.abs
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    /// Re-titles a gloss, for the uniqueness pass in [`Glossary`].
    pub fn retitled(self, name: String) -> Self {
        Self { name, ..self }
    }
}

// naming
impl Gloss {
    /// Preflop buckets are single hands, so they name themselves. Everywhere
    /// else the name is the canonical phrase plus the equity band that
    /// separates two buckets holding the same kind of hand.
    fn title(profile: &Profile, baseline: &Baseline) -> String {
        match profile.abs().street() {
            Street::Pref => Self::notation(profile),
            Street::Rive => {
                format!("{} · {}%", Self::phrase(profile, baseline), Self::percent(profile.census().equity()))
            }
            _ => format!(
                "{} · {}%±{}",
                Self::phrase(profile, baseline),
                Self::percent(profile.census().equity()),
                Self::percent(profile.census().spread())
            ),
        }
    }

    /// The made-hand class, then the qualifiers that survived their
    /// thresholds — capped, because a name with five clauses is a paragraph.
    fn phrase(profile: &Profile, baseline: &Baseline) -> String {
        let class = Class::from(profile);
        std::iter::once(class.label())
            .chain(
                Self::qualifiers(profile, baseline, class)
                    .into_iter()
                    .take(Self::TERMS)
                    .map(|feature| feature.label()),
            )
            .collect::<Vec<&str>>()
            .join(", ")
    }

    /// Qualifiers in the order a player would say them: what the pair holds up
    /// with, what hero has that the board does not, what it can still become,
    /// and what board it lives on.
    fn qualifiers(profile: &Profile, baseline: &Baseline, class: Class) -> Vec<Feature> {
        std::iter::empty::<Option<Feature>>()
            .chain(Some(Self::kicker(profile, class)))
            .chain(Some(Self::highcard(profile, baseline)))
            .chain(Some(Self::draw(profile, baseline)))
            .chain(Some(Self::texture(profile, baseline)))
            .flatten()
            .collect()
    }

    /// A kicker only qualifies a pair. Hung off "mixed holdings" or "one pair
    /// or better" it qualifies nothing and reads as noise.
    fn kicker(profile: &Profile, class: Class) -> Option<Feature> {
        profile
            .within(&Feature::KICKS)
            .filter(|_| class.pairs())
            .filter(|(_, p)| *p >= Self::MAJORITY)
            .map(|(feature, _)| feature)
    }

    /// What hero's unpaired cards are worth. Only says anything about buckets
    /// that missed — a bucket that paired the board is named by its pair.
    fn highcard(profile: &Profile, baseline: &Baseline) -> Option<Feature> {
        Some(profile.frequency(Feature::NoPair))
            .filter(|p| *p >= Self::MISSED)
            .and_then(|_| baseline.distinguishes(profile, &Feature::HIGHS, Self::PLURALITY))
            .map(|(feature, _)| feature)
    }

    fn draw(profile: &Profile, baseline: &Baseline) -> Option<Feature> {
        baseline
            .distinguishes(profile, &Feature::DRAWS, Self::PLURALITY)
            .map(|(feature, _)| feature)
    }

    /// The board texture the bucket has and the street mostly doesn't. Raw
    /// frequency would name the base rate instead — see [`Baseline`].
    fn texture(profile: &Profile, baseline: &Baseline) -> Option<Feature> {
        baseline
            .distinguishes(profile, &Feature::BOARD, Self::HALF)
            .filter(|(_, lift)| *lift >= Self::SURPRISING)
            .map(|(feature, _)| feature)
    }

    /// The 169-hand notation of a preflop bucket's only member.
    fn notation(profile: &Profile) -> String {
        profile
            .exemplars()
            .first()
            .map_or_else(|| profile.abs().to_string(), Self::shorthand)
    }

    fn shorthand(obs: &Observation) -> String {
        let cards = Vec::<Card>::from(*obs.pocket());
        let hi = cards.iter().map(Card::rank).max().expect("two hole cards");
        let lo = cards.iter().map(Card::rank).min().expect("two hole cards");
        format!("{hi}{lo}{}", Self::suffix(&cards, hi == lo))
    }

    fn suffix(cards: &[Card], paired: bool) -> &'static str {
        if paired {
            ""
        } else if cards[0].suit() == cards[1].suit() {
            "s"
        } else {
            "o"
        }
    }
}

// describing
impl Gloss {
    /// The evidence, in the order a human reads it: how big the bucket is,
    /// what equity it carries, every feature group that had something to say,
    /// then the sample those frequencies came from.
    fn gist(profile: &Profile, baseline: &Baseline) -> String {
        std::iter::empty::<String>()
            .chain([
                Self::identity(profile),
                Self::potential(profile),
                Self::distinctive(profile, baseline),
            ])
            .chain(Group::ALL.map(|group| Self::section(group, profile)))
            .chain([Self::evidence(profile)])
            .filter(|part| !part.is_empty())
            .collect::<Vec<String>>()
            .join(" ")
    }

    /// What this bucket does that its street does not — the single clause a
    /// reader needs to tell it apart from the other 255.
    fn distinctive(profile: &Profile, baseline: &Baseline) -> String {
        match baseline.distinctions(profile, Self::HALF, Self::SURPRISING) {
            ranked if ranked.is_empty() => String::new(),
            ranked => format!(
                "Distinctive: {}.",
                ranked
                    .iter()
                    .take(Self::LISTED)
                    .map(|(feature, lift)| format!("{feature} +{} pts", Self::percent(*lift)))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }

    fn identity(profile: &Profile) -> String {
        format!(
            "{} bucket {} of {} · {} isomorphism{} ({}% of the street).",
            profile.abs().street().label(),
            profile.abs().index(),
            profile.abs().street().n_abstractions(),
            Self::commas(profile.census().population()),
            Self::plural(profile.census().population()),
            Self::decimal(profile.census().share())
        )
    }

    /// River equity is the bucket definition; everywhere else it is the mean
    /// of the next-street distribution k-means clustered on, and the spread is
    /// how much of the hand's value is still undecided.
    fn potential(profile: &Profile) -> String {
        match profile.abs().street() {
            Street::Rive => format!("Equity {}% by definition.", Self::percent(profile.census().equity())),
            street => format!(
                "Equity {}% ± {} over the {} buckets it flows into.",
                Self::percent(profile.census().equity()),
                Self::percent(profile.census().spread()),
                street.next().label().to_lowercase()
            ),
        }
    }

    fn section(group: Group, profile: &Profile) -> String {
        match group.shares(profile, Self::NOTABLE) {
            ranked if ranked.is_empty() => String::new(),
            ranked => format!(
                "{}: {}.",
                group.label(),
                ranked
                    .iter()
                    .take(Self::LISTED)
                    .map(|(feature, p)| format!("{feature} {}%", Self::percent(*p)))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }

    fn evidence(profile: &Profile) -> String {
        format!(
            "Sampled {} of {}. Examples: {}.",
            profile.sampled(),
            Self::commas(profile.census().population()),
            profile
                .exemplars()
                .iter()
                .map(Self::example)
                .collect::<Vec<String>>()
                .join("; ")
        )
    }

    /// An observation as written, minus the empty board a preflop hand has.
    fn example(obs: &Observation) -> String {
        String::from(obs.to_string().trim_end().trim_end_matches('~').trim_end())
    }

    fn plural(n: usize) -> &'static str {
        if n == 1 { "" } else { "s" }
    }

    fn percent(p: Probability) -> i32 {
        (p * 100.).round() as i32
    }

    fn decimal(p: Probability) -> String {
        format!("{:.2}", p * 100.)
    }

    fn commas(n: usize) -> String {
        n.to_string()
            .as_bytes()
            .rchunks(3)
            .rev()
            .map(|chunk| String::from_utf8_lossy(chunk).to_string())
            .collect::<Vec<String>>()
            .join(",")
    }
}

impl From<(&Profile, &Baseline)> for Gloss {
    fn from((profile, baseline): (&Profile, &Baseline)) -> Self {
        Self {
            abs: profile.abs(),
            name: Self::title(profile, baseline),
            description: Self::gist(profile, baseline),
        }
    }
}

impl From<(Abstraction, String, String)> for Gloss {
    fn from((abs, name, description): (Abstraction, String, String)) -> Self {
        Self { abs, name, description }
    }
}

impl std::fmt::Display for Gloss {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:<10} {}\n{:<10} {}", self.abs.to_string(), self.name, "", self.description)
    }
}

#[cfg(feature = "server")]
impl daybook::Schema for Gloss {
    fn name() -> &'static str {
        daybook::mechinterp()
    }

    fn creates() -> &'static str {
        static SQL: std::sync::OnceLock<&str> = std::sync::OnceLock::<&str>::new();
        SQL.get_or_init(|| {
            daybook::leaked(format!(
                "CREATE TABLE IF NOT EXISTS {} (
                abs         SMALLINT PRIMARY KEY,
                name        TEXT NOT NULL,
                description TEXT NOT NULL
            );",
                daybook::mechinterp()
            ))
        })
    }

    fn indices() -> &'static str {
        ""
    }

    fn truncates() -> &'static str {
        static SQL: std::sync::OnceLock<&str> = std::sync::OnceLock::<&str>::new();
        SQL.get_or_init(|| daybook::leaked(format!("TRUNCATE TABLE {};", daybook::mechinterp())))
    }

    fn copy() -> &'static str {
        unimplemented!("Gloss is upserted, not bulk-copied")
    }

    fn freeze() -> &'static str {
        unimplemented!("Gloss is rewritten on every run")
    }

    fn columns() -> &'static [tokio_postgres::types::Type] {
        unimplemented!("Gloss is upserted, not bulk-copied")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The wire contract the analysis frontend reads off `/topology/gloss`.
    /// `abs` goes over as the display form, not the packed integer, so a
    /// payload stays readable and the client parses it back to the same
    /// bucket.
    #[test]
    fn wire_shape_is_abs_name_description() {
        assert_eq!(
            serde_json::to_string(&Gloss::from((
                Abstraction::from((Street::Flop, 9)),
                String::from("top pair, weak kicker"),
                String::from("Flop bucket 9 of 256."),
            )))
            .expect("serialize"),
            r#"{"abs":"F::09","name":"top pair, weak kicker","description":"Flop bucket 9 of 256."}"#
        );
    }

    #[test]
    fn wire_round_trips() {
        let gloss = Gloss::from((
            Abstraction::from((Street::Turn, 195)),
            String::from("no pair, high board"),
            String::from("Turn bucket 195 of 256."),
        ));
        let back =
            serde_json::from_str::<Gloss>(&serde_json::to_string(&gloss).expect("serialize")).expect("deserialize");
        assert_eq!(gloss.abs(), back.abs());
        assert_eq!(gloss.name(), back.name());
        assert_eq!(gloss.description(), back.description());
    }

    fn profile(index: usize, hands: &[&str]) -> Profile {
        Profile::from((
            Census::from((Abstraction::from((Street::Flop, index)), hands.len(), 0.5, 0.62, 0.11)),
            hands
                .iter()
                .map(|obs| Observation::try_from(*obs).expect("parses"))
                .collect::<Vec<Observation>>(),
        ))
    }

    /// Both buckets sit on high boards, so the texture carries no lift and no
    /// texture term is earned — which leaves the class and its kicker.
    fn street() -> Vec<Profile> {
        vec![
            profile(0, &["As2h ~ Ad7c9s", "Ks2h ~ Kd7c9s", "Qs3h ~ Qd7c9s", "As4h ~ Ad7c8s"]),
            profile(1, &["5s4h ~ Ad7c9s", "6s3h ~ Kd7c9s", "5s2h ~ Qd7c9s", "6s4h ~ Ad7c8s"]),
        ]
    }

    fn name(index: usize) -> String {
        let street = street();
        String::from(Gloss::from((&street[index], &Baseline::from(street.as_slice()))).name())
    }

    #[test]
    fn a_decisive_class_leads_the_name_and_takes_its_kicker() {
        assert_eq!("top pair, weak kicker · 62%±11", name(0));
    }

    #[test]
    fn the_description_names_what_is_distinctive() {
        let street = street();
        let gloss = Gloss::from((&street[0], &Baseline::from(street.as_slice())));
        assert!(gloss.description().contains("Distinctive: top pair +50 pts"), "{}", gloss.description());
    }

    #[test]
    fn a_bucket_that_missed_says_so() {
        assert_eq!("no pair · 62%±11", name(1));
    }
}
