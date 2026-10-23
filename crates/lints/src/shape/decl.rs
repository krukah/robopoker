use super::*;

/// One type a module body exposes, with what the crowding rule needs to judge
/// it.
pub(super) struct Decl {
    item: Item,
    scope: Scope,
    allowed: bool,
}

impl Decl {
    pub(super) fn new(item: Item, scope: Scope, allowed: bool) -> Self {
        Self { item, scope, allowed }
    }

    fn item(&self) -> &Item {
        &self.item
    }

    pub(super) fn scope(&self) -> Scope {
        self.scope
    }

    pub(super) fn allowed(&self) -> bool {
        self.allowed
    }

    pub(super) fn name(&self) -> &str {
        self.item.name()
    }

    pub(super) fn line(&self) -> usize {
        self.item.line()
    }

    pub(super) fn is_a_trait(&self) -> bool {
        self.item().is(Kind::Trait)
    }

    /// A file named for a type belongs to that type, whatever else it holds.
    pub(super) fn named_for(&self, stem: &str) -> bool {
        self.snake() == stem
    }

    fn snake(&self) -> String {
        let letters = self.name().chars().collect::<Vec<char>>();
        letters
            .iter()
            .enumerate()
            .flat_map(|(index, letter)| Self::split(&letters, index, *letter))
            .collect()
    }

    fn split(letters: &[char], index: usize, letter: char) -> Vec<char> {
        let opens = letters.get(index + 1).copied().is_some_and(char::is_lowercase);
        let closes = letters
            .get(index.wrapping_sub(1))
            .is_some_and(|last| last.is_lowercase() || last.is_numeric() || opens);
        let mut out = Vec::new();
        if letter.is_uppercase() && index > 0 && closes {
            out.push('_');
        }
        out.extend(letter.to_lowercase());
        out
    }

    pub(super) fn crowded_by(&self, owner: &str) -> Item {
        self.item().crowded_by(owner)
    }
}
