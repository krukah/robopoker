use pokerkit::Position;

/// A seat index as a database column: the `seat`/`dealer` SMALLINT columns'
/// domain type. `Position` itself is deliberately a bare `usize` alias for
/// game logic (array indexing, modular rotation), so this newtype exists to
/// carry the SQL codec — the boundary writes `Chair::from(seat)` and reads
/// `row.get::<_, Chair>(..)` instead of casting through `i16`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chair(Position);

impl From<Position> for Chair {
    fn from(position: Position) -> Self {
        Self(position)
    }
}

impl From<Chair> for Position {
    fn from(chair: Chair) -> Self {
        chair.0
    }
}

#[cfg(feature = "server")]
pokerkit::codec!(Chair as i16, |c: &Chair| c.0 as i16, |v| Chair(v as Position));
