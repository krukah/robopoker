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
use std::collections::BTreeMap;

/// this is the output of the clustering module
/// it is a massive table of `Equivalence` -> `Abstraction`.
/// effectively, this is a compressed representation of the
/// full game tree, learned by kmeans
/// rooted in showdown equity at the River.
#[derive(Default)]
pub struct Abstractor(BTreeMap<Isomorphism, Abstraction>);

/* learning methods
 *
 * during clustering, we're constantly inserting and updating
 * the abstraction mapping. needs to help project layers
 * hierarchically, while also
 */
impl Abstractor {
    /// only run this once.
    pub fn learn() {
        if Self::done() {
            log::info!("skipping abstraction");
        } else {
            log::info!("learning abstraction");
            Layer::outer()
                .inner() // cluster turn
                .save()
                .inner() // cluster flop
                .save();
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
        let observation = Observation::from(*outer);
        match observation.street() {
            Street::Pref => Abstraction::from(Hole::from(observation)),
            Street::Rive => Abstraction::from(observation.equity()),
            Street::Turn | Street::Flop => self
                .0
                .get(outer)
                .cloned()
                .expect("precomputed abstraction mapping for Turn/Flop"),
        }
    }
    /// at a given `Street`,
    /// 1. decompose the `Equivalence` into all of its next-street `Equivalence`s,
    /// 2. map each of them into an `Abstraction`,
    /// 3. collect the results into a `Histogram`.
    pub fn projection(&self, inner: &Isomorphism) -> Histogram {
        let inner = Observation::from(*inner); // isomorphism translation
        match inner.street() {
            Street::Turn => Histogram::from(inner),
            Street::Flop => Histogram::from(
                inner
                    .children()
                    .map(|outer| Isomorphism::from(outer)) // isomorphism translation
                    .map(|outer| self.abstraction(&outer))
                    .collect::<Vec<Abstraction>>(),
            ),
            _ => unreachable!("invalid street for projection"),
        }
    }
}

/* sampling methods
 *
 * another great use case for Abstractor is to "unfold" a Tree
 * by sampling according to a given Profile. here we provide
 * methods for unraveling the Tree
 */
impl Abstractor {
    /// abstraction methods
    pub fn chance_abstraction(&self, game: &Game) -> Abstraction {
        self.abstraction(&Isomorphism::from(Observation::from(game)))
    }
    pub fn action_abstraction(&self, _: &Vec<&Edge>) -> Path {
        // TODO
        // decide how to handle action abstraction
        Path::from(0)
    }
    /// produce the children of a Node.
    /// we may need some Trainer-level references to produce children
    pub fn children(&self, node: &Node) -> Vec<(Data, Edge)> {
        let ref past = node.history();
        let ref game = node.spot().game();
        game.children()
            .into_iter()
            .map(|(g, a)| self.convert(g, a, past))
            .collect()
    }
    /// convert gameplay types into CFR types
    /// Game -> Spot
    /// Action -> Edge
    /// Vec<Edge> -> Path
    /// wrap the (Game, Bucket) in a Data
    fn convert(&self, game: Game, action: Action, past: &Vec<&Edge>) -> (Data, Edge) {
        let edge = Edge::from(action);
        let ref mut path = past.clone();
        path.push(&edge);
        let action = self.action_abstraction(&path);
        let chance = self.chance_abstraction(&game);
        let bucket = Bucket::from((action, chance));
        let choice = Data::from((game, bucket));
        (choice, edge)
    }
}

use byteorder::ReadBytesExt;
use byteorder::WriteBytesExt;
use byteorder::BE;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

/* persistence methods
 *
 * write to disk. if you want to, on your own time,
 * you can stream this to postgres efficiently
 * with pgcopy. it's actually built from both the
 * Turn and Flop layers, with the River and Preflop being
 * straightforward to compute on the fly, for different reasons
 */

impl From<Street> for Abstractor {
    fn from(street: Street) -> Self {
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

impl Abstractor {
    /// indicates whether the abstraction table is already on disk
    pub fn done() -> bool {
        [
            "turn.abstraction.pgcopy",
            "flop.abstraction.pgcopy",
            "turn.metric.pgcopy",
            "flop.metric.pgcopy",
        ]
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
        log::info!("{:<32}{:<32}", "saving lookup", street);
        let ref mut file = File::create(format!("{}.abstraction.pgcopy", street)).expect("touch");
        file.write_all(b"PGCOPY\n\xFF\r\n\0").expect("header");
        file.write_u32::<BE>(0).expect("flags");
        file.write_u32::<BE>(0).expect("extension");
        for (obs, abs) in self.0.iter() {
            let ref obs = Observation::from(*obs); // isomorphism translation
            let obs = i64::from(*obs);
            let abs = i64::from(*abs);
            file.write_u16::<BE>(0x2).expect("field count");
            file.write_u32::<BE>(0x8).expect("8-bytes field");
            file.write_i64::<BE>(obs).expect("observation");
            file.write_u32::<BE>(0x8).expect("8-bytes field");
            file.write_i64::<BE>(abs).expect("abstraction");
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
        let save = Abstractor(
            (0..100)
                .map(|_| Observation::from(street))
                .map(|o| Isomorphism::from(o))
                .map(|o| (o, Abstraction::random()))
                .collect(),
        );
        save.save(street);
        let load = Abstractor::from(street);
        std::iter::empty()
            .chain(save.0.iter().zip(load.0.iter()))
            .chain(load.0.iter().zip(save.0.iter()))
            .all(|((s1, l1), (s2, l2))| s1 == s2 && l1 == l2);
        std::fs::remove_file(format!("{}", file)).unwrap();
    }
}
