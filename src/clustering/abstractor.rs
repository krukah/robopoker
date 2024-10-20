use super::layer::Layer;
use crate::cards::hole::Hole;
use crate::cards::isomorphism::Isomorphism;
use crate::cards::observation::Observation;
use crate::cards::street::Street;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::histogram::Histogram;
use crate::mccfr::bucket::Bucket;
use crate::mccfr::bucket::Path;
use crate::mccfr::edge::Edge;
use crate::mccfr::node::Node;
use crate::mccfr::spot::Spot;
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
                .save()
                .inner() // cluster preflop (but really just save flop.metric)
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
    /// produce the children of a Node.
    /// we may need some Trainer-level references to produce children
    pub fn children(&self, node: &Node) -> Vec<(Spot, Edge)> {
        let ref game = node.spot().game();
        let ref past = node.history().into_iter().collect::<Vec<&Edge>>();
        game.children()
            .into_iter()
            .map(|(g, a)| (g, Edge::from(a)))
            .map(|(g, e)| self.explore(g, e, past))
            .collect()
    }
    /// extend a path with an Edge
    /// wrap the (Game, Bucket) in a Data
    fn explore(&self, game: Game, edge: Edge, history: &Vec<&Edge>) -> (Spot, Edge) {
        let mut history = history.clone();
        history.push(&edge);
        (self.data(game, history), edge)
    }
    /// generate a Bucket from Game
    /// wrap the (Game, Bucket) in a Data
    fn data(&self, game: Game, path: Vec<&Edge>) -> Spot {
        let bucket = self.bucket(&game, &path);
        Spot::from((game, bucket))
    }
    /// use the product of past actions (Path) and chance information (Abstraction)
    /// to label a given Node/Infoset under a Bucket.
    fn bucket(&self, game: &Game, path: &Vec<&Edge>) -> Bucket {
        let path = self.path_abstraction(path);
        let info = self.card_abstraction(game);
        Bucket::from((path, info))
    }
    /// abstraction methods
    pub fn card_abstraction(&self, game: &Game) -> Abstraction {
        let ref equivalence = Isomorphism::from(game); // isomorphism translation
        self.abstraction(equivalence)
    }
    pub fn path_abstraction(&self, _: &Vec<&Edge>) -> Path {
        Path::from(0)
    }
}

use byteorder::BigEndian;
use byteorder::ReadBytesExt;
use byteorder::WriteBytesExt;
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
                reader.read_u32::<BigEndian>().expect("observation length");
                let obs_i64 = reader.read_i64::<BigEndian>().expect("read observation");
                reader.read_u32::<BigEndian>().expect("abstraction length");
                let abs_i64 = reader.read_i64::<BigEndian>().expect("read abstraction");
                let observation = Isomorphism::from(obs_i64);
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
        log::info!("loading abstraction from disk");
        let mut map = BTreeMap::default();
        map.extend(Self::load_street(Street::Flop).0);
        map.extend(Self::load_street(Street::Turn).0);
        Self(map)
    }

    /// read the full abstraction lookup from disk
    /// 1. Skip PGCOPY header (15 bytes), flags (4 bytes), and header extension (4 bytes)
    /// 2. Read field count (should be 2)
    /// 3. Read observation length (4 bytes)
    /// 4. Read observation (8 bytes)
    /// 5. Read abstraction length (4 bytes)
    /// 6. Read abstraction (8 bytes)
    /// 7. Insert observation and abstraction into lookup
    /// 8. Repeat until end of file
    fn load_street(street: Street) -> Self {
        let file = File::open(format!("{}.abstraction.pgcopy", street)).expect("open file");
        let mut buffer = [0u8; 2];
        let mut lookup = BTreeMap::new();
        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::Start(19)).expect("seek past header");
        while reader.read_exact(&mut buffer).is_ok() {
            if u16::from_be_bytes(buffer) == 2 {
                reader.read_u32::<BigEndian>().expect("observation length");
                let obs_i64 = reader.read_i64::<BigEndian>().expect("read observation");
                reader.read_u32::<BigEndian>().expect("abstraction length");
                let abs_i64 = reader.read_i64::<BigEndian>().expect("read abstraction");
                let observation = Isomorphism::from(obs_i64);
                let abstraction = Abstraction::from(abs_i64);
                lookup.insert(observation, abstraction);
                continue;
            } else {
                break;
            }
        }
        log::info!("downloaded abstraction lookup {} {}", street, lookup.len());
        Self(lookup)
    }

    /// persist the abstraction mapping to disk
    /// write the full abstraction lookup to disk
    /// 1. Write the PGCOPY header (15 bytes)
    /// 2. Write the flags (4 bytes)
    /// 3. Write the extension (4 bytes)
    /// 4. Write the observation and abstraction pairs
    /// 5. Write the trailer (2 bytes)
    pub fn save(&self, street: Street) {
        log::info!("{:<32}{:<32}", "saving abstraction lookup", street);
        let ref mut file =
            File::create(format!("{}.abstraction.pgcopy", street)).expect("new file");
        file.write_all(b"PGCOPY\n\xff\r\n\0").expect("header");
        file.write_u32::<BigEndian>(0).expect("flags");
        file.write_u32::<BigEndian>(0).expect("extension");
        for (obs, abs) in self.0.iter() {
            let ref obs = Observation::from(*obs); // isomorphism translation
            let obs = i64::from(*obs);
            let abs = i64::from(*abs);
            file.write_u16::<BigEndian>(0x2).expect("field count");
            file.write_u32::<BigEndian>(0x8).expect("8-bytes field");
            file.write_i64::<BigEndian>(obs).expect("observation");
            file.write_u32::<BigEndian>(0x8).expect("8-bytes field");
            file.write_i64::<BigEndian>(abs).expect("abstraction");
        }
        file.write_u16::<BigEndian>(0xFFFF).expect("trailer");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persistence() {
        // Generate sample data on a street we don't touch
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
        // Load from disk
        let load = Abstractor::load_street(street);
        std::iter::empty()
            .chain(save.0.iter().zip(load.0.iter()))
            .chain(load.0.iter().zip(save.0.iter()))
            .all(|((s1, l1), (s2, l2))| s1 == s2 && l1 == l2);
        // Clean up
        std::fs::remove_file(format!("{}", file)).unwrap();
    }
}
