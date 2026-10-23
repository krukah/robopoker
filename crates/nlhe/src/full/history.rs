use crate::*;
use deuce::*;
use kicker::*;
use neural::*;

/// Every edge of the hand so far, chance edges included.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct History(Path<4>);

impl History {
    /// Room for the longest street the grid allows; `longest_street_fits` pins it.
    pub const SLOTS: usize = 8;

    pub fn edges(&self) -> impl Iterator<Item = NlheEdge> {
        self.0.into_iter().map(NlheEdge::from)
    }

    pub fn length(&self) -> usize {
        self.0.length()
    }

    pub fn street(&self) -> Street {
        self.0.street()
    }

    /// Trailing aggressive edges on the current street, which pick its raise grid.
    pub fn aggression(&self) -> usize {
        self.0.aggression()
    }

    /// Each street's choice edges, preflop first.
    pub fn streets(&self) -> Vec<Vec<NlheEdge>> {
        self.edges().fold(vec![Vec::new()], |mut streets, edge| {
            match edge.is_chance() {
                true => streets.push(Vec::new()),
                false => streets.last_mut().expect("starts with preflop").push(edge),
            }
            streets
        })
    }
}

/// Per street, `SLOTS` one-hots over the bet grid, empty past the last edge.
impl Block for History {
    const WIDTH: usize = 4 * Self::SLOTS * NlheEdge::CHOICES;

    fn values(self) -> impl Iterator<Item = f32> {
        let streets = self.streets();
        debug_assert!(streets.iter().all(|street| street.len() <= Self::SLOTS), "{streets:?}");
        (0..4)
            .flat_map(|street| (0..Self::SLOTS).map(move |slot| (street, slot)))
            .map(|(street, slot)| {
                streets
                    .get(street)
                    .and_then(|edges| edges.get(slot))
                    .map(NlheEdge::index)
            })
            .flat_map(|index| (0..NlheEdge::CHOICES).map(move |i| f32::from(index == Some(i))))
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl FromIterator<NlheEdge> for History {
    fn from_iter<I>(edges: I) -> Self
    where
        I: IntoIterator<Item = NlheEdge>,
    {
        Self(edges.into_iter().map(Edge::from).collect())
    }
}
