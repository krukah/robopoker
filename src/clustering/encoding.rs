use super::layer::Layer;
use crate::cards::hole::Hole;
use crate::cards::isomorphism::Isomorphism;
use crate::cards::observation::Observation;
use crate::cards::street::Street;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::histogram::Histogram;
use crate::mccfr::bucket::Bucket;
use crate::mccfr::child::Child;
use crate::mccfr::data::Data;
use crate::mccfr::edge::Edge;
use crate::mccfr::node::Node;
use crate::mccfr::path::Path;
use crate::play::action::Action;
use crate::play::game::Game;
use crate::{Chips, Probability, Utility};
use std::collections::BTreeMap;

struct History<'tree>(Node<'tree>);

impl<'tree> History<'tree> {
    pub fn subgame(&self, child: &Child) -> Vec<Edge> {
        self.0
            .history()
            .into_iter()
            .copied()
            .chain(std::iter::once(child.edge()))
            .rev()
            .take_while(|e| e.is_choice())
            .collect()
    }
    pub fn continuations(&self, child: &Child) -> Vec<(Odds, Action)> {
        let min = child.game().to_raise();
        let max = child.game().to_shove() - 1;
        self.sizes(child)
            .into_iter()
            .map(|o| (o, Probability::from(o)))
            .map(|(o, p)| (o, p * child.game().pot() as Utility))
            .map(|(o, x)| (o, x as Chips))
            .filter(|(_, x)| min <= *x && *x <= max)
            .map(|(o, x)| (o, Action::Raise(x)))
            .collect()
    }
    pub fn options(&self, child: &Child) -> Vec<Edge> {
        let mut actions = child
            .game()
            .legal()
            .into_iter()
            .map(|a| (a, None))
            .collect::<Vec<(Action, Option<Odds>)>>();
        if let Some(raise) = actions.iter().position(|a| a.0.is_raise()) {
            actions.remove(raise);
            actions.splice(
                raise..raise,
                self.continuations(child)
                    .into_iter()
                    .map(|(o, a)| (a, Some(o)))
                    .collect::<Vec<(Action, Option<Odds>)>>(),
            );
        }
        actions
            .into_iter()
            .map(|(action, odds)| match (action, odds) {
                (a, None) => Edge::from(a),
                (_, Some(odds)) => Edge::from(odds),
            })
            .collect()
    }
    fn sizes(&self, child: &Child) -> Vec<Odds> {
        let n = self
            .0
            .history()
            .into_iter()
            .chain(std::iter::once(&child.edge()))
            .rev()
            .take_while(|e| e.is_choice())
            .filter(|e| e.is_aggro())
            .count();
        if n > crate::MAX_N_BETS {
            vec![]
        } else {
            match child.game().board().street() {
                Street::Pref => Odds::PREF_RAISES.to_vec(),
                Street::Flop => Odds::FLOP_RAISES.to_vec(),
                _ => match n {
                    0 => Odds::LATE_RAISES.to_vec(),
                    _ => Odds::LAST_RAISES.to_vec(),
                },
            }
        }
    }
}
/*





















*/
/// pot odds for a given raise size, relative to the pot
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Odds(pub Chips, pub Chips);
impl From<Odds> for Probability {
    fn from(odds: Odds) -> Self {
        odds.0 as Probability / odds.1 as Probability
    }
}

impl Odds {
    pub const GRID: [Self; 10] = Self::PREF_RAISES;
    const PREF_RAISES: [Self; 10] = [
        Self(1, 4), // 0.25
        Self(1, 3), // 0.33
        Self(1, 2), // 0.50
        Self(2, 3), // 0.66
        Self(3, 4), // 0.75
        Self(1, 1), // 1.00
        Self(3, 2), // 1.50
        Self(2, 1), // 2.00
        Self(3, 1), // 3.00
        Self(4, 1), // 4.00
    ];
    const FLOP_RAISES: [Self; 5] = [
        Self(1, 2), // 0.50
        Self(3, 4), // 0.75
        Self(1, 1), // 1.00
        Self(3, 2), // 1.50
        Self(2, 1), // 2.00
    ];
    const LATE_RAISES: [Self; 2] = [
        Self(1, 2), // 0.50
        Self(1, 1), // 1.00
    ];
    const LAST_RAISES: [Self; 1] = [
        Self(1, 1), // 1.00
    ];
}
impl std::fmt::Display for Odds {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:>2}:{:>2}", self.0, self.1)
    }
}

/*





















*/
/// this is the output of the clustering module
/// it is a massive table of `Equivalence` -> `Abstraction`.
/// effectively, this is a compressed representation of the
/// full game tree, learned by kmeans
/// rooted in showdown equity at the River.
#[derive(Default)]
pub struct Encoder(BTreeMap<Isomorphism, Abstraction>);
impl Encoder {
    /// only run this once.
    pub fn learn() {
        if Self::done() {
            log::info!("skipping abstraction");
        } else {
            log::info!("learning abstraction");
            Layer::outer()
                .inner() // turn
                .inner() // flop
                .inner(); // preflop
        }
    }
    /// simple insertion.
    /// can we optimize out this clone though? maybe for key but not for value
    pub fn assign(&mut self, abs: &Abstraction, obs: &Isomorphism) {
        self.0.insert(obs.clone(), abs.clone());
    }
    /// lookup the pre-computed abstraction for the outer observation
    /// for preflop, we lookup the Hole cards, up to isomorphism
    /// for river, we compute the equity on the fly. could use MC sampling to speed up
    /// for turn and flop, we lookup the pre-computed abstraction that we woked so hard for in ::clustering
    pub fn abstraction(&self, outer: &Isomorphism) -> Abstraction {
        let observation = outer.0;
        match observation.street() {
            Street::Pref => Abstraction::from(Hole::from(observation)),
            Street::Rive => Abstraction::from(observation.equity()),
            Street::Flop | Street::Turn => self
                .0
                .get(outer)
                .cloned()
                .expect("precomputed abstraction mapping for Turn/Flop"),
        }
    }
    /// at a given `Street`,
    /// 1. decompose the `Isomorphism` into all of its next-street `Isomorphism`s,
    /// 2. map each of them into an `Abstraction`,
    /// 3. collect the results into a `Histogram`.
    pub fn projection(&self, inner: &Isomorphism) -> Histogram {
        let observation = inner.0;
        match observation.street() {
            Street::Rive => unreachable!("never project outermost abstraction layer"),
            Street::Turn => Histogram::from(observation),
            Street::Pref | Street::Flop => Histogram::from(
                observation
                    .children()
                    .map(|outer| Isomorphism::from(outer)) // isomorphism translation
                    .map(|outer| self.abstraction(&outer)) // abstraction lookup
                    .collect::<Vec<Abstraction>>(), // histogram collection
            ),
        }
    }

    // persistence methods

    pub fn done() -> bool {
        ["flop.abstraction.pgcopy", "turn.abstraction.pgcopy"]
            .iter()
            .any(|file| std::path::Path::new(file).exists())
    }
    pub fn load() -> Self {
        log::info!("loading encoder");
        let mut map = BTreeMap::default();
        map.extend(Self::from(Street::Flop).0);
        map.extend(Self::from(Street::Turn).0);
        Self(map)
    }
    pub fn from(street: Street) -> Self {
        use byteorder::ReadBytesExt;
        use byteorder::BE;
        use std::fs::File;
        use std::io::BufReader;
        use std::io::Read;
        use std::io::Seek;
        use std::io::SeekFrom;
        let file = File::open(format!("{}.abstraction.pgcopy", street)).expect("open file");
        let mut buffer = [0u8; 2];
        let mut lookup = BTreeMap::new();
        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::Start(19)).expect("seek past header");
        while reader.read_exact(&mut buffer).is_ok() {
            if u16::from_be_bytes(buffer) == 2 {
                reader.read_u32::<BE>().expect("observation length");
                let iso_i64 = reader.read_i64::<BE>().expect("read observation");
                reader.read_u32::<BE>().expect("abstraction length");
                let abs_i64 = reader.read_i64::<BE>().expect("read abstraction");
                let observation = Isomorphism::from(iso_i64);
                let abstraction = Abstraction::from(abs_i64);
                lookup.insert(observation, abstraction);
                continue;
            } else {
                break;
            }
        }
        Self(lookup)
    }
    pub fn save(&self, street: Street) {
        log::info!("{:<32}{:<32}", "saving encoding", street);
        use byteorder::WriteBytesExt;
        use byteorder::BE;
        use std::fs::File;
        use std::io::Write;
        let ref mut file = File::create(format!("{}.abstraction.pgcopy", street)).expect("touch");
        file.write_all(b"PGCOPY\n\xFF\r\n\0").expect("header");
        file.write_u32::<BE>(0).expect("flags");
        file.write_u32::<BE>(0).expect("extension");
        for (Isomorphism(obs), abs) in self.0.iter() {
            const N_FIELDS: u16 = 2;
            file.write_u16::<BE>(N_FIELDS).unwrap();
            file.write_u32::<BE>(size_of::<i64>() as u32).unwrap();
            file.write_i64::<BE>(i64::from(*obs)).unwrap();
            file.write_u32::<BE>(size_of::<i64>() as u32).unwrap();
            file.write_i64::<BE>(i64::from(*abs)).unwrap();
        }
        file.write_u16::<BE>(0xFFFF).expect("trailer");
    }
}

/*





















*/
#[derive(Default)]
pub struct Sampler(Encoder);
impl Sampler {
    pub fn load() -> Self {
        Self(Encoder::load())
    }
    pub fn children(&self, node: &Node) -> Vec<(Data, Edge)> {
        node.data()
            .game()
            .children()
            .iter()
            .map(|child| self.sample(node, child))
            .collect()
    }
    pub fn root(&self) -> Data {
        let game = Game::root();
        let present = self.recall(&game);
        let history = Path::default();
        let futures = Path::default(); // technically wrong but doesn't matter
        let infoset = Bucket::from((history, present, futures));
        let data = Data::from((game, infoset));
        data
    }
    fn sample(&self, node: &Node, child: &Child) -> (Data, Edge) {
        let game = child.game().clone();
        let present = self.recall(&game);
        let history = Path::from(History(node.clone()).subgame(child));
        let futures = Path::from(History(node.clone()).options(child));
        let infoset = Bucket::from((history, present, futures));
        let data = Data::from((game, infoset));
        let edge = child.edge();
        (data, edge)
    }
    fn recall(&self, game: &Game) -> Abstraction {
        self.0
            .abstraction(&Isomorphism::from(Observation::from(game)))
    }
}

/*





















*/
#[cfg(test)]
mod tests {
    use super::*;

    /// Generate sample data on a street we don't touch
    /// Load from disk
    /// Clean up
    #[test]
    fn persistence() {
        let street = Street::Rive;
        let file = format!("{}.abstraction.pgcopy", street);
        let save = Encoder(
            (0..100)
                .map(|_| Observation::from(street))
                .map(|o| Isomorphism::from(o))
                .map(|o| (o, Abstraction::random()))
                .collect(),
        );
        save.save(street);
        let load = Encoder::from(street);
        std::iter::empty()
            .chain(save.0.iter().zip(load.0.iter()))
            .chain(load.0.iter().zip(save.0.iter()))
            .all(|((s1, l1), (s2, l2))| s1 == s2 && l1 == l2);
        std::fs::remove_file(format!("{}", file)).unwrap();
    }
}
