use crate::cfr::turn::Turn;

/// Represents a game state
pub trait Game: Clone + Copy {
    fn player<T>(&self) -> T
    where
        T: Turn;
}

impl Game for crate::gameplay::game::Game {
    fn player<T>(&self) -> T
    where
        T: Turn,
    {
        todo!()
    }
}
