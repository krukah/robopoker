use super::layer::Layer;
use crate::cards::hole::Hole;
use crate::cards::isomorphism::Isomorphism;
use crate::cards::observation::Observation;
use crate::cards::street::Street;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::histogram::Histogram;
use crate::mccfr::bucket::Bucket;
use crate::mccfr::data::Data;
use crate::mccfr::edge::Edge;
use crate::mccfr::node::Node;
use crate::mccfr::path::Path;
use crate::play::action::Action;
use crate::play::game::Game;
use crate::Chips;
use crate::Utility;
use std::collections::BTreeMap;

/// this is the output of the clustering module
/// it is a massive table of `Equivalence` -> `Abstraction`.
/// effectively, this is a compressed representation of the
/// full game tree, learned by kmeans
/// rooted in showdown equity at the River.
#[derive(Default)]
pub struct Encoder(BTreeMap<Isomorphism, Abstraction>);

/* learning methods
 *
 * during clustering, we're constantly inserting and updating
 * the abstraction mapping. needs to help project layers
 * hierarchically, while also
 */
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
}

/* sampling methods
 *
 * another great use case for Abstractor is to "unfold" a Tree
 * by sampling according to a given Profile. here we provide
 * methods for unraveling the Tree
 */
impl Encoder {
    /// convert gameplay types into CFR types
    /// Action -> Edge
    /// Vec<Edge> -> Path
    /// Game -> Data -> Obs -> Iso -> Abs
    /// Path -> Abs -> Bucket
    pub fn encode(&self, leaf: Game, from: Action, head: &Node) -> (Data, Edge) {
        let info = self.card_encoding(&leaf);
        let edge = self.edge_encoding(&head, &from);
        let path = self.path_encoding(&head, &edge);
        let data = Data::from((leaf, Bucket::from((path, info))));
        (data, edge)
    }
    pub fn root(&self) -> Data {
        let path = Path::from(0);
        let game = Game::root();
        let info = self.card_encoding(&game);
        let data = Data::from((game, Bucket::from((path, info))));
        data
    }
    pub fn children(&self, node: &Node) -> Vec<(Data, Edge)> {
        // cut off N-betting
        let nraises = node
            .history()
            .iter()
            .rev()
            .take_while(|e| e.is_choice())
            .filter(|e| e.is_raise())
            .count();
        Self::actions(node)
            .into_iter()
            .map(|action| (action, node.data().game().apply(action)))
            .map(|(a, g)| self.encode(g, a, node))
            .filter(|&(_, e)| !e.is_raise() || nraises < crate::MAX_N_BETS)
            .collect::<Vec<(Data, Edge)>>()
    }
    /// all actions available to the player at this node
    fn actions(node: &Node) -> Vec<Action> {
        let mut actions = node.data().game().options();
        if let Some(raise) = actions.iter().position(|a| matches!(a, Action::Raise(_))) {
            if let Some(shove) = actions.iter().position(|a| matches!(a, Action::Shove(_))) {
                if let &Action::Raise(min) = actions.get(raise).unwrap() {
                    if let &Action::Shove(max) = actions.get(shove).unwrap() {
                        actions.remove(raise);
                        actions.splice(
                            raise..raise,
                            Self::raises(node)
                                .into_iter()
                                .map(|relative| relative * node.data().game().pot() as Utility)
                                .map(|absolute| absolute as Chips)
                                .filter(|raise| min <= *raise && *raise < max)
                                .map(|absolute| Action::Raise(absolute)),
                        );
                    }
                }
            }
        }
        actions
    }
    /// discretized raise sizes, conditional on street and betting history
    fn raises(node: &Node) -> Vec<f32> {
        match node.data().game().board().street() {
            Street::Pref => vec![0.25, 0.33, 0.5, 0.66, 0.75, 1.0, 1.5, 2.0, 3.0, 4.0],
            Street::Flop => vec![0000000000000.5, 0000000.75, 1.0, 1.5, 2.0],
            _ => match node
                .history()
                .iter()
                .rev()
                .take_while(|e| e.is_choice())
                .filter(|e| e.is_raise())
                .count() // is_not_first_raise
            {
                0 => vec![00000000000000000000.5, 0000000000001.0],
                _ => vec![0000000000000000000000000000000000001.0],
            },
        }
    }

    fn edge_encoding(&self, node: &Node, action: &Action) -> Edge {
        enum Thresholds {
            QuarPot,
            HalfPot,
            FullPot,
            OverPot,
        }
        match if let Action::Raise(x) = action {
            let bets = *x as Utility;
            let wins = node.data().game().pot() as Utility;
            let odds = bets / (wins - bets);
            match odds {
                x if odds < 0.5 => Thresholds::QuarPot,
                x if odds < 1.0 => Thresholds::HalfPot,
                x if odds < 2.0 => Thresholds::FullPot,
                _ => Thresholds::OverPot,
            };
        } {
            _ => todo!(),
        }
    }
    /// i like to think of this as "positional encoding"
    /// i like to think of this as "positional encoding"
    /// later in the same round where the stakes are higher
    /// we should "learn" things i.e. when to n-bet.
    /// it also helps the recall be a bit "less imperfect"
    /// the cards we see at a Node are memoryless, but the
    /// Path represents "how we got here"
    ///
    /// we need to assert that: any Nodes in the same Infoset have the
    /// same available actions. in addition to depth, we consider
    /// whether or not we are in a Checkable or Foldable state.
    fn path_encoding(&self, node: &Node, edge: &Edge) -> Path {
        let round = node
            .history()
            .into_iter()
            .chain(std::iter::once(edge))
            .rev()
            .take_while(|e| e.is_choice());
        let depth = round.clone().count();
        let raise = round.filter(|e| e.is_raise()).count();
        Path::from((depth, raise))
    }
    /// the compressed card information for an observation
    /// this is defined up to unique Observation > Isomorphism
    /// so pocket vs public is the only distinction made. forget reveal order.
    fn card_encoding(&self, game: &Game) -> Abstraction {
        self.abstraction(&Isomorphism::from(Observation::from(game)))
    }
}

/* persistence methods
 *
 * write to disk. if you want to, on your own time,
 * you can stream this to postgres efficiently
 * with pgcopy. it's actually built from both the
 * Turn and Flop layers, with the River and Preflop being
 * straightforward to compute on the fly, for different reasons
 */

impl From<Street> for Encoder {
    fn from(street: Street) -> Self {
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
}

impl Encoder {
    /// indicates whether the abstraction table is already on disk
    pub fn done() -> bool {
        ["flop.abstraction.pgcopy", "turn.abstraction.pgcopy"]
            .iter()
            .any(|file| std::path::Path::new(file).exists())
    }

    /// pulls the entire pre-computed abstraction table
    /// into memory. ~10GB.
    pub fn load() -> Self {
        log::info!("loading encoder");
        let mut map = BTreeMap::default();
        map.extend(Self::from(Street::Flop).0);
        map.extend(Self::from(Street::Turn).0);
        Self(map)
    }

    /// persist the abstraction mapping to disk
    /// write the full abstraction lookup to disk
    /// 1. Write the PGCOPY header (15 bytes)
    /// 2. Write the flags (4 bytes)
    /// 3. Write the extension (4 bytes)
    /// 4. Write the observation and abstraction pairs
    /// 5. Write the trailer (2 bytes)
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
