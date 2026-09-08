//! Every gloss for a street, with names made unique.
use crate::*;
use kicker::*;
use std::collections::HashMap;

/// The named buckets of one street.
///
/// Names come out of [`Gloss::from`] one bucket at a time, so two buckets that
/// hold the same kind of hand at the same equity can land on the same phrase.
/// The constructor is where that is resolved: any name claimed more than once
/// gets its bucket id appended, which keeps the glossary a lookup table rather
/// than a list of near-synonyms.
pub struct Glossary(Vec<Gloss>);

impl Glossary {
    pub fn glosses(&self) -> &[Gloss] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get(&self, abs: Abstraction) -> Option<&Gloss> {
        self.0.iter().find(|gloss| gloss.abs() == abs)
    }
}

impl From<Vec<Profile>> for Glossary {
    /// The street's [`Baseline`] is computed here, from the same profiles, so
    /// each bucket is named by what it does that its street does not.
    fn from(profiles: Vec<Profile>) -> Self {
        let baseline = Baseline::from(profiles.as_slice());
        Self::from(
            profiles
                .iter()
                .map(|profile| Gloss::from((profile, &baseline)))
                .collect::<Vec<Gloss>>(),
        )
    }
}

impl From<Vec<Gloss>> for Glossary {
    fn from(glosses: Vec<Gloss>) -> Self {
        let claims = glosses
            .iter()
            .fold(HashMap::<String, usize>::new(), |mut claims, gloss| {
                *claims.entry(String::from(gloss.name())).or_default() += 1;
                claims
            });
        Self(
            glosses
                .into_iter()
                .map(|gloss| match claims.get(gloss.name()).copied().unwrap_or_default() {
                    0 | 1 => gloss,
                    _ => gloss.clone().retitled(format!("{} · {}", gloss.name(), gloss.abs())),
                })
                .collect(),
        )
    }
}

impl std::fmt::Display for Glossary {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0
            .iter()
            .try_for_each(|gloss| writeln!(f, "{:<10} {}", gloss.abs().to_string(), gloss.name()))
    }
}
