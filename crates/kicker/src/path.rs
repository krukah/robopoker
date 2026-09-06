use crate::*;
use pokerkit::*;

/// A compact sequence of abstract edges packed into `W` 64-bit words.
///
/// `W` counts **words, not bits**: the word is pinned to `u64` by the backing
/// `[u64; W]`, and each word packs up to [`pokerkit::MAX_PATH_EDGES`]
/// (`⌊64/5⌋ = 12`) edges at 5 bits apiece in its low 60 bits — the historical
/// single-u64 layout — so total capacity is `W * 12` edges with no edge
/// straddling a word boundary (the top 4 bits of each word sit idle). The
/// const default `W = 1` makes bare `Path` byte-identical to the legacy
/// `Path(u64)`: existing code, DB `BIGINT` columns, and the live heads-up
/// blueprint are untouched. Subgame-carrying types pin the width via
/// [`Subgame`], which derives `W` from the compile-time player count
/// ([`pokerkit::WORDS`]): 1 at heads-up, 4 (48 edges) multiway, where a
/// single street can legally exceed 12 edges.
///
/// # Encoding
///
/// Each edge maps to a 5-bit value (values 1–31, with 0 reserved for empty).
/// Edges are stored least-significant first within each word, filling word 0
/// before word 1, so the first action occupies bits 0–4 of word 0.
///
/// # Overflow
///
/// Collecting more than `W * 12` edges **panics** — silent truncation would
/// collide distinct action sequences onto one info-set key (the failure mode
/// that motivated widening). If a legal street ever trips the assert, bump
/// [`pokerkit::WORDS`].
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Path<const W: usize = 1>([u64; W]);

impl<const W: usize> Default for Path<W> {
    fn default() -> Self {
        Self([0; W])
    }
}

impl<const W: usize> Path<W> {
    const SEPARATOR: &'static str = "/";
    /// Maximum number of edges this width can hold.
    pub const CAPACITY: usize = W * pokerkit::MAX_PATH_EDGES;
    /// Number of edges in this path.
    pub fn length(&self) -> usize {
        self.0.iter().map(|w| (68 - w.leading_zeros() as usize) / 5).sum()
    }
    /// Aggression: count of trailing aggressive edges (for bet sizing grid selection).
    /// kinda wanna deprecate, dangerous if truncated
    pub fn aggression(&self) -> usize {
        self.into_iter()
            .rev()
            .take_while(Edge::is_choice)
            .filter(Edge::is_aggro)
            .count()
    }
    /// Street derived from counting Draw edges.
    /// 0 draws = Pref, 1 = Flop, 2 = Turn, 3+ = River.
    pub fn street(&self) -> deuce::Street {
        match self.into_iter().filter(Edge::is_chance).count() {
            0 => deuce::Street::Pref,
            1 => deuce::Street::Flop,
            2 => deuce::Street::Turn,
            _ => deuce::Street::Rive,
        }
    }
    /// The raw packed words, least-significant word first.
    pub const fn words(&self) -> [u64; W] {
        self.0
    }

    /// Street-scoped accumulation: extends the path with an edge, resetting
    /// on chance. Only the current street feeds [`Self::aggression`], and a
    /// full-hand accumulation could overflow street-sized capacity multiway.
    pub fn flow(self, edge: Edge) -> Self {
        if edge.is_chance() {
            Self::default()
        } else {
            self.into_iter().chain(std::iter::once(edge)).collect()
        }
    }

    /// Stows the `i`-th edge's 5-bit payload into its word slot.
    const fn stow(mut words: [u64; W], (i, byte): (usize, u64)) -> [u64; W] {
        words[i / MAX_PATH_EDGES] |= byte << (5 * (i % MAX_PATH_EDGES));
        words
    }

    /// The lowest slot's 5-bit payload (0 when empty).
    const fn head(&self) -> u8 {
        (self.0[0] & 0x1F) as u8
    }

    /// Shifts every word right one slot, carrying each neighbor's lowest
    /// edge into the vacated top slot (bit `5 * (MAX_PATH_EDGES - 1)`).
    fn shift(&mut self) {
        for i in 0..W {
            self.0[i] >>= 5;
            if i + 1 < W {
                self.0[i] |= (self.0[i + 1] & 0x1F) << (5 * (MAX_PATH_EDGES - 1));
            }
        }
    }
}

impl<const W: usize> serde::Serialize for Path<W> {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(&self.to_string())
    }
}
impl<'de, const W: usize> serde::Deserialize<'de> for Path<W> {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = serde::Deserialize::deserialize(d)?;
        Self::try_from(s.as_str()).map_err(serde::de::Error::custom)
    }
}

impl<const W: usize> Arbitrary for Path<W> {
    fn random() -> Self {
        let n = rand::random_range(1..=Self::CAPACITY);
        (0..n).map(|_| Edge::random()).collect()
    }
}

/// `Vec<Edge>` isomorphism
/// we (un)pack the byte representation of the edges in a Path sequence
impl<const W: usize> From<Path<W>> for Vec<Edge> {
    fn from(path: Path<W>) -> Self {
        path.into_iter().collect()
    }
}

impl<const W: usize> From<Vec<Edge>> for Path<W> {
    fn from(edges: Vec<Edge>) -> Self {
        edges.into_iter().collect()
    }
}

/// Word-array isomorphism: trivial packing and unpacking.
impl<const W: usize> From<[u64; W]> for Path<W> {
    fn from(words: [u64; W]) -> Self {
        Self(words)
    }
}

/// u64 conversions fill / read word 0. Reading a wider path into 64 bits
/// asserts the upper words are empty — a lossy narrowing must be loud, never
/// silent. At `W = 1` the assert is over an empty slice and compiles away.
impl<const W: usize> From<u64> for Path<W> {
    fn from(value: u64) -> Self {
        let mut words = [0; W];
        words[0] = value;
        Self(words)
    }
}
impl<const W: usize> From<Path<W>> for u64 {
    fn from(path: Path<W>) -> Self {
        assert!(path.0[1..].iter().all(|&w| w == 0), "lossy narrowing of Path<{W}> to u64");
        path.0[0]
    }
}
impl<const W: usize> From<Path<W>> for i64 {
    fn from(path: Path<W>) -> Self {
        u64::from(path) as i64
    }
}
impl<const W: usize> From<i64> for Path<W> {
    fn from(value: i64) -> Self {
        Self::from(value as u64)
    }
}

impl<const W: usize> TryFrom<&str> for Path<W> {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if s.is_empty() {
            return Ok(Self::default());
        }
        s.split(Self::SEPARATOR)
            .map(Edge::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map(Self::from)
    }
}

impl<const W: usize> std::fmt::Display for Path<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            (*self)
                .into_iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(Self::SEPARATOR)
        )
    }
}

impl<const W: usize> Iterator for Path<W> {
    type Item = Edge;

    fn next(&mut self) -> Option<Self::Item> {
        let head = self.head();
        (head != 0).then(|| self.shift()).map(|()| Edge::from(head))
    }
}

impl<const W: usize> DoubleEndedIterator for Path<W> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let word = (0..W).rev().find(|&w| self.0[w] != 0)?;
        let shift = ((63u32.saturating_sub(self.0[word].leading_zeros())) / 5) * 5;
        let bloop = (self.0[word] >> shift) & 0x1F;
        if bloop == 0 {
            None
        } else {
            self.0[word] &= !(0x1F << shift);
            Some(Edge::from(bloop as u8))
        }
    }
}

impl<const W: usize> std::iter::FromIterator<Edge> for Path<W> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Edge>,
    {
        iter.into_iter()
            .map(u8::from)
            .map(|byte| byte as u64)
            .enumerate()
            .inspect(|(i, _)| assert!(*i < Self::CAPACITY))
            .fold([0u64; W], Self::stow)
            .into()
    }
}

/// SQL codec, keyed on the width: a single-word `Path<1>` is the historical
/// `BIGINT` column (byte-identical to the live heads-up blueprint); wider
/// paths are `BYTEA` in little-endian word order. Encoding follows the
/// *column* type, and narrowing a populated wide path into `INT8` panics
/// loudly via the `u64` conversion.
#[cfg(feature = "sql")]
impl<const W: usize> Path<W> {
    /// The column's SQL type name for this width.
    pub const fn sql() -> &'static str {
        if W == 1 { "BIGINT" } else { "BYTEA" }
    }

    /// The column's postgres wire type for this width.
    pub fn kind() -> tokio_postgres::types::Type {
        if W == 1 {
            tokio_postgres::types::Type::INT8
        } else {
            tokio_postgres::types::Type::BYTEA
        }
    }

    /// Reassembles a path from `W` little-endian words.
    fn widen(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        (bytes.len() == 8 * W)
            .then(|| std::array::from_fn(|i| u64::from_le_bytes(bytes[8 * i..8 * (i + 1)].try_into().expect("chunk"))))
            .map(Self)
            .ok_or_else(|| format!("path BYTEA length {} != {W} words", bytes.len()).into())
    }
}

#[cfg(feature = "sql")]
impl<const W: usize> tokio_postgres::types::ToSql for Path<W> {
    fn to_sql(
        &self,
        ty: &tokio_postgres::types::Type,
        out: &mut tokio_postgres::types::private::BytesMut,
    ) -> Result<tokio_postgres::types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        if *ty == tokio_postgres::types::Type::INT8 {
            tokio_postgres::types::ToSql::to_sql(&i64::from(*self), ty, out)
        } else {
            tokio_postgres::types::ToSql::to_sql(
                &self.0.iter().flat_map(|w| w.to_le_bytes()).collect::<Vec<u8>>(),
                ty,
                out,
            )
        }
    }

    fn accepts(ty: &tokio_postgres::types::Type) -> bool {
        matches!(*ty, tokio_postgres::types::Type::INT8 | tokio_postgres::types::Type::BYTEA)
    }

    tokio_postgres::types::to_sql_checked!();
}

#[cfg(feature = "sql")]
impl<'a, const W: usize> tokio_postgres::types::FromSql<'a> for Path<W> {
    fn from_sql(
        ty: &tokio_postgres::types::Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        if *ty == tokio_postgres::types::Type::INT8 {
            <i64 as tokio_postgres::types::FromSql>::from_sql(ty, raw).map(Self::from)
        } else {
            <&[u8] as tokio_postgres::types::FromSql>::from_sql(ty, raw).and_then(Self::widen)
        }
    }

    fn accepts(ty: &tokio_postgres::types::Type) -> bool {
        matches!(*ty, tokio_postgres::types::Type::INT8 | tokio_postgres::types::Type::BYTEA)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bijective_path_empty() {
        let edges = vec![];
        let paths = Vec::<Edge>::from(Path::<1>::from(edges.clone()));
        assert_eq!(edges, paths);
    }
    #[test]
    fn bijective_str_empty() {
        let path = Path::<1>::default();
        let text = path.to_string();
        assert_eq!(text, "");
        assert_eq!(path, Path::try_from(text.as_str()).unwrap());
    }
    #[test]
    fn bijective_str_nonempty() {
        let path = (0..).map(|_| Edge::random()).take(5).collect::<Path>();
        let text = path.to_string();
        assert_eq!(path, Path::try_from(text.as_str()).unwrap());
    }

    #[test]
    fn bijective_path_edges() {
        let edges = (0..)
            .map(|_| Edge::random())
            .take(Path::<1>::CAPACITY)
            .collect::<Vec<Edge>>();
        let paths = Vec::<Edge>::from(Path::<1>::from(edges.clone()));
        assert_eq!(edges, paths);
    }

    #[test]
    fn bijective_path_collect() {
        let edges = (0..).map(|_| Edge::random()).take(5).collect::<Vec<Edge>>();
        let collected = Path::<1>::from(edges.clone()).into_iter().collect::<Vec<Edge>>();
        assert_eq!(edges, collected);
    }

    #[test]
    fn length() {
        let n = rand::random::<u64>() % (Path::<1>::CAPACITY + 1) as u64;
        let n = n as usize;
        let path = (0..).map(|_| Edge::random()).take(n).collect::<Path>();
        assert_eq!(path.length(), n);
    }

    #[test]
    fn double_ended_iterator() {
        let path = (0..).map(|_| Edge::random()).take(5).collect::<Path>();
        let forward = path;
        let reverse = path
            .into_iter()
            .rev()
            .collect::<Vec<Edge>>()
            .into_iter()
            .rev()
            .collect::<Path>();
        assert_eq!(forward, reverse);
    }

    #[test]
    fn subgame_aggression() {
        let path = [
            // this one is a late street some aggressions
            Edge::Draw,
            Edge::Raise(Odds::new(1, 2)),
            Edge::Call,
            Edge::Call,
            // new street
            Edge::Draw,
            Edge::Check,
            Edge::Check,
            Edge::Check,
            // new street
            Edge::Draw,
            Edge::Raise(Odds::new(1, 1)),
            Edge::Shove,
            Edge::Fold,
        ]
        .into_iter()
        .collect::<Path>();
        assert_eq!(path.aggression(), 2);
        let path = [
            // this one has no aggressions, new street
            Edge::Draw,
            Edge::Check,
            Edge::Check,
            Edge::Check,
        ]
        .into_iter()
        .collect::<Path>();
        assert_eq!(path.aggression(), 0);
    }

    /// Wide paths round-trip across word boundaries: forward iteration,
    /// reverse iteration, length, and Display/parse all agree beyond the
    /// single-word 12-edge capacity.
    #[test]
    fn wide_path_crosses_word_boundaries() {
        let edges = (0..).map(|_| Edge::random()).take(30).collect::<Vec<Edge>>();
        let path = edges.iter().copied().collect::<Path<4>>();
        assert_eq!(path.length(), 30);
        assert_eq!(path.into_iter().collect::<Vec<_>>(), edges);
        assert_eq!(path.into_iter().rev().collect::<Vec<_>>(), edges.iter().rev().copied().collect::<Vec<_>>());
        assert_eq!(path, Path::<4>::try_from(path.to_string().as_str()).unwrap());
        assert_eq!(path, Path::<4>::from(path.words()));
        assert_eq!(
            path.aggression(),
            edges
                .iter()
                .rev()
                .take_while(|e| e.is_choice())
                .filter(|e| e.is_aggro())
                .count()
        );
    }

    /// A word-0-only wide path narrows losslessly to u64; any populated upper
    /// word makes the narrowing panic instead of silently truncating.
    #[test]
    fn wide_path_narrowing_is_loud() {
        let narrow = (0..).map(|_| Edge::random()).take(5).collect::<Path<4>>();
        assert_eq!(u64::from(narrow), narrow.words()[0]);
        let wide = (0..).map(|_| Edge::random()).take(20).collect::<Path<4>>();
        assert!(std::panic::catch_unwind(|| u64::from(wide)).is_err(), "lossy narrowing must panic");
    }

    /// Overflow is a panic, not a silent truncation.
    #[test]
    fn overflow_panics() {
        let extra = (0..)
            .map(|_| Edge::random())
            .take(Path::<1>::CAPACITY + 1)
            .collect::<Vec<_>>();
        assert!(
            std::panic::catch_unwind(|| extra.into_iter().collect::<Path<1>>()).is_err(),
            "collecting past capacity must panic"
        );
    }
}
