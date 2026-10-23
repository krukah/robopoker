use std::collections::HashMap;

use super::*;

/// The traits a module body implements, keyed by the trait it names.
pub(super) type Implementors = HashMap<(Scope, String), Vec<String>>;

/// Every type a file exposes, grouped by the module body that holds it.
///
/// One family owns a body and the rest crowd it. A trait and the types that
/// implement it in the same body are one family: a vocabulary and the words
/// that answer it say one thing, and splitting them says it worse.
pub(super) struct Crowd {
    decls: Vec<Decl>,
    impls: Implementors,
    stem: String,
}

impl Crowd {
    pub(super) fn read(scan: Scan<'_>, stem: &str) -> Self {
        Self {
            decls: scan.decls(),
            impls: scan.impls(),
            stem: stem.to_string(),
        }
    }

    fn decls(&self) -> &[Decl] {
        &self.decls
    }

    fn impls(&self) -> &Implementors {
        &self.impls
    }

    fn stem(&self) -> &str {
        &self.stem
    }

    pub(super) fn violations(&self) -> Vec<Item> {
        self.scopes()
            .into_iter()
            .flat_map(|scope| self.crowded(scope))
            .collect()
    }

    fn scopes(&self) -> Vec<Scope> {
        let mut seen: Vec<Scope> = Vec::new();
        for scope in self.decls().iter().map(Decl::scope) {
            if !seen.contains(&scope) {
                seen.push(scope);
            }
        }
        seen
    }

    fn crowded(&self, scope: Scope) -> Vec<Item> {
        let members = self.members(scope);
        let families = self.families(scope, &members);
        let Some(owner) = self.owner(&families, &members) else {
            return Vec::new();
        };
        let head = Self::head(&families[owner], &members);
        families
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != owner)
            .flat_map(|(_, family)| family.iter().map(|index| members[*index]))
            .filter(|decl| !decl.allowed())
            .map(|decl| decl.crowded_by(head))
            .collect()
    }

    /// One declaration per name. Two same-named types in one body are
    /// `#[cfg]` alternatives — only one reaches any build, so they are one
    /// type, not a crowd.
    fn members(&self, scope: Scope) -> Vec<&Decl> {
        let mut seen: Vec<&str> = Vec::new();
        self.decls()
            .iter()
            .filter(|decl| decl.scope() == scope)
            .filter(|decl| {
                let fresh = !seen.contains(&decl.name());
                if fresh {
                    seen.push(decl.name());
                }
                fresh
            })
            .collect()
    }

    /// Each trait claims the members that implement it, in declaration order.
    /// Every member no trait claims stands alone.
    fn families(&self, scope: Scope, members: &[&Decl]) -> Vec<Vec<usize>> {
        let mut families: Vec<Vec<usize>> = Vec::new();
        let mut claimed = vec![false; members.len()];
        for index in (0..members.len()).filter(|index| members[*index].is_a_trait()) {
            claimed[index] = true;
            let mut family = vec![index];
            for name in self.implementors(scope, members[index].name()) {
                family.extend(Self::unclaimed(members, &mut claimed, name));
            }
            families.push(family);
        }
        families.extend(
            (0..members.len())
                .filter(|index| !claimed[*index])
                .map(|index| vec![index]),
        );
        families
    }

    fn unclaimed(members: &[&Decl], claimed: &mut [bool], name: &str) -> Vec<usize> {
        let found = (0..members.len())
            .filter(|index| !claimed[*index] && members[*index].name() == name)
            .collect::<Vec<usize>>();
        for index in &found {
            claimed[*index] = true;
        }
        found
    }

    fn implementors(&self, scope: Scope, name: &str) -> &[String] {
        self.impls().get(&(scope, name.to_string())).map_or(&[], Vec::as_slice)
    }

    /// The family holding the file's namesake owns the body. Failing that, the
    /// largest family; and two of a size, the one declared first.
    fn owner(&self, families: &[Vec<usize>], members: &[&Decl]) -> Option<usize> {
        let namesake = (0..families.len()).find(|index| {
            families[*index]
                .iter()
                .any(|member| members[*member].named_for(self.stem()))
        });
        namesake.or_else(|| Self::widest(families, members))
    }

    fn widest(families: &[Vec<usize>], members: &[&Decl]) -> Option<usize> {
        (0..families.len()).max_by_key(|index| {
            let opened = families[*index]
                .iter()
                .map(|member| members[*member].line())
                .min()
                .unwrap_or_default();
            (families[*index].len(), std::cmp::Reverse(opened))
        })
    }

    /// A family answers to the trait that names it, or to its only member.
    fn head<'decl>(family: &[usize], members: &[&'decl Decl]) -> &'decl str {
        family
            .iter()
            .map(|index| members[*index])
            .position(Decl::is_a_trait)
            .map_or(members[family[0]], |position| members[family[position]])
            .name()
    }
}
