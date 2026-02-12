use super::*;
use rbp_core::ID;
use rbp_core::Unique;

/// User represents authentication state: anonymous or authenticated.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum User {
    Anon(Lurker),
    Auth(Member),
}

impl User {
    pub fn id(&self) -> Option<ID<Member>> {
        match self {
            Self::Auth(m) => Some(m.id()),
            Self::Anon(_) => None,
        }
    }
}

impl From<Lurker> for User {
    fn from(lurker: Lurker) -> Self {
        Self::Anon(lurker)
    }
}

impl From<Member> for User {
    fn from(member: Member) -> Self {
        Self::Auth(member)
    }
}
