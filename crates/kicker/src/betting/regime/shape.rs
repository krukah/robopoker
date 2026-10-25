//! The game tree's shape, as the terms that define it.

use super::*;

/// Every constant the game tree's shape depends on, as ordered `KEY=value` terms. Two runs may
/// share a `blueprint_<regime>_<version>` table iff their shapes are equal; two that differ
/// would corrupt each other's strategy, because an info-set key carries the same bytes while
/// meaning a different action once the bet-size lattice shifts.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Shape(Vec<(&'static str, String)>);

impl Shape {
    /// The terms shared by every regime — table stakes, not bet sizing.
    ///
    /// Add one whenever its change should invalidate existing blueprints:
    /// anything omitted is drift that neither the trainer's startup check nor
    /// the server's audit can see.
    pub fn common() -> Self {
        Self(vec![
            ("N", format!("{}", pokerkit::N)),
            ("STACK", format!("{}", pokerkit::STACK)),
            ("B_BLIND", format!("{}", pokerkit::B_BLIND)),
            ("S_BLIND", format!("{}", pokerkit::S_BLIND)),
            ("MAX_RAISE_REPEATS", format!("{}", pokerkit::MAX_RAISE_REPEATS)),
            ("OPENS", format!("{:?}", pokerkit::OPENS)),
        ])
    }
    /// Builds a shape from its terms, in the order they should render.
    pub fn of<I>(terms: I) -> Self
    where
        I: IntoIterator<Item = (&'static str, String)>,
    {
        Self(terms.into_iter().collect())
    }
    /// Appends another shape's terms — how a regime's own sizing joins the
    /// shared constants.
    pub fn chain(self, other: Self) -> Self {
        Self(self.0.into_iter().chain(other.0).collect())
    }
    /// The terms, in render order.
    pub fn terms(&self) -> &[(&'static str, String)] {
        &self.0
    }
    /// This shape's short stable hash.
    pub fn digest(&self) -> Digest {
        Digest::from(self)
    }
    /// Term-by-term difference against a stored shape, as `- stored` /
    /// `+ live` lines naming only what changed. Operates on terms rather than
    /// re-splitting a rendered string, so a key present on one side and absent
    /// on the other reads as one changed key, not two.
    pub fn diff(&self, live: &Self) -> String {
        self.0
            .iter()
            .map(|(k, _)| *k)
            .chain(live.0.iter().map(|(k, _)| *k))
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .filter(|key| self.value(key) != live.value(key))
            .flat_map(|key| {
                [
                    self.value(key).map(|v| format!("  - {key}={v}")),
                    live.value(key).map(|v| format!("  + {key}={v}")),
                ]
            })
            .flatten()
            .collect::<Vec<_>>()
            .join("\n")
    }
    /// One term's rendered value.
    fn value(&self, key: &str) -> Option<&str> {
        self.0.iter().find(|(k, _)| *k == key).map(|(_, v)| v.as_str())
    }
}

/// The wire format: `KEY=value;KEY=value`, in term order. Byte-compatible
/// with every fingerprint row written before [`Shape`] existed.
impl std::fmt::Display for Shape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join(";")
        )
    }
}

/// Parses a stored fingerprint back into terms. Keys are leaked to `'static`
/// — one allocation per term on a single row read at startup, which buys the
/// same type on both sides of the comparison.
impl std::str::FromStr for Shape {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split(';')
            .map(|term| {
                term.split_once('=')
                    .map(|(k, v)| (&*Box::leak(k.to_string().into_boxed_str()), v.to_string()))
                    .ok_or_else(|| anyhow::anyhow!("fingerprint term without '=': {term}"))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Shape {
        fn literal(terms: &[(&'static str, &str)]) -> Self {
            Self::of(terms.iter().map(|(k, v)| (*k, v.to_string())))
        }
    }

    /// The rendering is a wire format: a row written before [`Shape`] existed has to read back
    /// equal to the shape that produced it, or the trainer refuses to resume against a
    #[test]
    fn rendering_round_trips() {
        let live = Shape::common();
        assert_eq!(live.to_string().parse::<Shape>().expect("parses"), live);
    }

    /// Term order is part of the contract — the stored text is compared verbatim, so two shapes
    /// with the same terms in a different order are not interchangeable.
    #[test]
    fn order_is_part_of_the_format() {
        assert_eq!(Shape::literal(&[("A", "1"), ("B", "2")]).to_string(), "A=1;B=2");
        assert_ne!(Shape::literal(&[("A", "1"), ("B", "2")]), Shape::literal(&[("B", "2"), ("A", "1")]));
    }

    /// A changed term reports both sides; untouched terms stay out of it.
    #[test]
    fn diff_names_only_what_moved() {
        let diff = Shape::literal(&[("A", "1"), ("B", "2")]).diff(&Shape::literal(&[("A", "1"), ("B", "3")]));
        assert_eq!(diff, "  - B=2\n  + B=3");
    }

    /// A term added on one side is one changed key, not a deletion plus an
    /// insertion — the failure mode of diffing rendered text by splitting it.
    #[test]
    fn diff_handles_a_new_term() {
        assert_eq!(Shape::literal(&[("A", "1")]).diff(&Shape::literal(&[("A", "1"), ("N", "2")])), "  + N=2");
    }

    /// Equal shapes diff to nothing, which is what `audit` reads as no drift.
    #[test]
    fn identical_shapes_do_not_differ() {
        assert!(Shape::common().diff(&Shape::common()).is_empty());
    }

    /// A term without `=` is a corrupt row, not a silently empty shape.
    #[test]
    fn malformed_terms_are_rejected() {
        assert!("A=1;garbage".parse::<Shape>().is_err());
    }
}
