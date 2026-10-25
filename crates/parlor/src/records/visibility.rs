#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Visibility {
    #[default]
    Hidden = 0,
    Showed = 1,
    Mucked = 2,
}
impl From<i16> for Visibility {
    fn from(v: i16) -> Self {
        match v {
            1 => Visibility::Showed,
            2 => Visibility::Mucked,
            _ => Visibility::Hidden,
        }
    }
}
