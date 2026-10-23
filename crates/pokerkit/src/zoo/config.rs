/// The three boolean axes that pick out one cell of the zoo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub depth: bool,
    pub world: bool,
    pub dirac: bool,
}

impl Config {
    pub fn depth(&self) -> bool {
        self.depth
    }

    pub fn world(&self) -> bool {
        self.world
    }

    pub fn dirac(&self) -> bool {
        self.dirac
    }
}
