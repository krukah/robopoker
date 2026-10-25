//! `.phh` file → [`Record`].
//!
//! PHH is a TOML document, so the table parse is delegated; the work here is turning the
//! `actions` array of terse command strings into typed [`Step`]s.

use crate::*;
use anyhow::Context;
use anyhow::anyhow;
use anyhow::bail;
use deuce::*;

/// Namespace for the parse routines.
pub struct Parse;

impl Parse {
    /// Reads and parses one `.phh` file.
    pub fn load(path: impl AsRef<std::path::Path>) -> anyhow::Result<Record> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        Parse::parse(&text, &path.display().to_string()).with_context(|| format!("parsing {}", path.display()))
    }

    /// Reads and parses every `.phh` file under `root`, recursively.
    ///
    /// Results come back sorted by path so runs are reproducible.
    pub fn load_dir(root: impl AsRef<std::path::Path>) -> anyhow::Result<Vec<Record>> {
        let mut paths = Vec::new();
        Parse::collect(root.as_ref(), &mut paths)?;
        paths.sort();
        paths.iter().map(Parse::load).collect()
    }

    fn collect(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) -> anyhow::Result<()> {
        for entry in std::fs::read_dir(dir).with_context(|| format!("reading dir {}", dir.display()))? {
            let path = entry?.path();
            if path.is_dir() {
                Parse::collect(&path, out)?;
            } else if path.extension().is_some_and(|e| e == "phh") {
                out.push(path);
            }
        }
        Ok(())
    }

    /// Parses PHH source text. `source` is a label used in error messages.
    pub fn parse(text: &str, source: &str) -> anyhow::Result<Record> {
        let table = text.parse::<toml::Table>()?;
        let variant = Parse::string(&table, "variant")?;
        let starting_stacks = Parse::amounts(&table, "starting_stacks")?;
        let n = starting_stacks.len();
        let actions = table
            .get("actions")
            .ok_or_else(|| anyhow!("missing `actions`"))?
            .as_array()
            .ok_or_else(|| anyhow!("`actions` is not an array"))?
            .iter()
            .map(|v| v.as_str().ok_or_else(|| anyhow!("non-string action {v:?}")))
            .collect::<anyhow::Result<Vec<_>>>()?
            .into_iter()
            .map(|s| Parse::step(s, n))
            .collect::<anyhow::Result<Vec<_>>>()?;
        Ok(Record::new(
            variant,
            Parse::amounts(&table, "antes").unwrap_or_else(|_| vec![0; n]),
            Parse::amounts(&table, "blinds_or_straddles").unwrap_or_else(|_| vec![0; n]),
            table.get("min_bet").and_then(toml::Value::as_integer).unwrap_or(0) as Amount,
            starting_stacks,
            Parse::halves(&table, "finishing_stacks").unwrap_or_default(),
            Parse::strings(&table, "players").unwrap_or_default(),
            actions,
            table.get("hand").and_then(toml::Value::as_integer).map(|h| h as u64),
            source.to_string(),
        ))
    }

    /// Parses one action string. `n` bounds the seat index. A bare `sm` is a
    /// muck: the player forfeits without revealing.
    fn step(s: &str, n: usize) -> anyhow::Result<Step> {
        let f = s.split_whitespace().collect::<Vec<_>>();
        match f.as_slice() {
            ["d", "dh", who, cards] => Ok(Step::Deal(Parse::seat(who, n)?, Parse::hole(cards)?)),
            ["d", "db", cards] => Ok(Step::Board(Card::parse(cards).map_err(|e| anyhow!("{e}: {cards}"))?)),
            [who, "f"] => Ok(Step::Fold(Parse::seat(who, n)?)),
            [who, "cc"] => Ok(Step::CheckCall(Parse::seat(who, n)?)),
            [who, "cbr", amount] => Ok(Step::BetTo(Parse::seat(who, n)?, amount.parse::<Amount>()?)),
            [who, "sm", cards] => Ok(Step::Show(Parse::seat(who, n)?, Parse::hole(cards)?)),
            [who, "sm"] => Ok(Step::Muck(Parse::seat(who, n)?)),
            _ => bail!("unrecognized action {s:?}"),
        }
    }

    /// `p3` → seat index 2. PHH seats are 1-indexed.
    fn seat(who: &str, n: usize) -> anyhow::Result<usize> {
        let index = who
            .strip_prefix('p')
            .ok_or_else(|| anyhow!("expected p<i>, got {who:?}"))?
            .parse::<usize>()?
            .checked_sub(1)
            .ok_or_else(|| anyhow!("seat index is 1-based, got {who:?}"))?;
        if index >= n {
            bail!("seat {who} out of range for {n} players");
        }
        Ok(index)
    }

    fn hole(cards: &str) -> anyhow::Result<Hole> {
        Hole::try_from(cards).map_err(|e| anyhow!("{e}: {cards}"))
    }

    fn string(table: &toml::Table, key: &str) -> anyhow::Result<String> {
        table
            .get(key)
            .and_then(toml::Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| anyhow!("missing or non-string `{key}`"))
    }

    fn strings(table: &toml::Table, key: &str) -> anyhow::Result<Vec<String>> {
        table
            .get(key)
            .and_then(toml::Value::as_array)
            .ok_or_else(|| anyhow!("missing or non-array `{key}`"))?
            .iter()
            .map(|v| {
                v.as_str()
                    .map(str::to_string)
                    .ok_or_else(|| anyhow!("non-string in `{key}`"))
            })
            .collect()
    }

    /// Settled stacks, doubled. PHH writes a half chip as a float (`10112.5`) when
    /// an odd pot splits two ways, so integers and floats both have to land here.
    fn halves(table: &toml::Table, key: &str) -> anyhow::Result<Vec<Halves>> {
        table
            .get(key)
            .and_then(toml::Value::as_array)
            .ok_or_else(|| anyhow!("missing or non-array `{key}`"))?
            .iter()
            .map(|v| {
                v.as_integer()
                    .map(|i| i * 2)
                    .or_else(|| v.as_float().map(|f| (f * 2.0).round() as Halves))
                    .ok_or_else(|| anyhow!("non-numeric in `{key}`"))
            })
            .collect()
    }

    fn amounts(table: &toml::Table, key: &str) -> anyhow::Result<Vec<Amount>> {
        table
            .get(key)
            .and_then(toml::Value::as_array)
            .ok_or_else(|| anyhow!("missing or non-array `{key}`"))?
            .iter()
            .map(|v| {
                v.as_integer()
                    .map(|i| i as Amount)
                    .or_else(|| v.as_float().map(|f| f as Amount))
                    .ok_or_else(|| anyhow!("non-numeric in `{key}`"))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r"
variant = 'NT'
ante_trimming_status = true
antes = [0, 0, 0, 0, 0, 0]
blinds_or_straddles = [50, 100, 0, 0, 0, 0]
min_bet = 100
starting_stacks = [10000, 10000, 10000, 10000, 10000, 10000]
actions = ['d dh p1 TcQc', 'd dh p2 8s4c', 'd dh p3 9c3d', 'd dh p4 Ah4h', 'd dh p5 Th5s', 'd dh p6 6c7s', 'p3 f', 'p4 cbr 210', 'p5 f', 'p6 f', 'p1 cc', 'p2 f', 'd db 7d5h9d', 'p1 cc', 'p4 cc', 'd db 7c', 'p1 cc', 'p4 cc', 'd db Qh', 'p1 cbr 230', 'p4 f']
hand = 0
players = ['MrBlue', 'MrBlonde', 'MrWhite', 'MrPink', 'MrBrown', 'Pluribus']
finishing_stacks = [10310, 9900, 10000, 9790, 10000, 10000]
";

    #[test]
    fn parses_a_pluribus_hand() {
        let record = Parse::parse(SAMPLE, "sample").unwrap();
        assert_eq!(record.variant(), "NT");
        assert_eq!(record.n(), 6);
        assert_eq!(record.hand(), Some(0));
        assert_eq!(record.blinds(), [50, 100, 0, 0, 0, 0]);
        assert_eq!(record.seat_of("Pluribus"), Some(5));
        assert_eq!(record.board().len(), 5);
        assert_eq!(record.holes().iter().filter(|h| h.is_some()).count(), 6);
    }

    #[test]
    fn typed_steps_round_trip() {
        let record = Parse::parse(SAMPLE, "sample").unwrap();
        assert_eq!(record.actions()[0], Step::Deal(0, Hole::try_from("TcQc").unwrap()));
        assert_eq!(record.actions()[6], Step::Fold(2));
        assert_eq!(record.actions()[7], Step::BetTo(3, 210));
        assert_eq!(record.actions()[10], Step::CheckCall(0));
    }

    #[test]
    fn seats_are_one_indexed_and_bounded() {
        assert_eq!(Parse::seat("p1", 6).unwrap(), 0);
        assert_eq!(Parse::seat("p6", 6).unwrap(), 5);
        assert!(Parse::seat("p7", 6).is_err());
        assert!(Parse::seat("x1", 6).is_err());
    }
}
