/// What a scanned node is. Each variant answers to exactly one rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Type,
    Trait,
    Function,
    Field,
    Closure,
    Binding,
    Bound,
    Access,
}

impl Kind {
    /// A trait declares a type and answers to the same rule. It is its own
    /// variant because a trait, alone, can gather the types that implement it.
    pub(crate) fn declares_a_type(self) -> bool {
        matches!(self, Self::Type | Self::Trait)
    }

    pub(crate) fn rule(self) -> &'static str {
        match self {
            Self::Type | Self::Trait => super::ONE_TYPE_PER_FILE,
            Self::Function => super::FREE_FUNCTION,
            Self::Field => super::PUB_FIELD,
            Self::Closure => super::FORWARDING_CLOSURE,
            Self::Binding => super::TYPE_HINT,
            Self::Bound => super::INLINE_BOUND,
            Self::Access => super::FIELD_ACCESS,
        }
    }

    pub(crate) fn message(self, name: &str, owner: Option<&str>) -> String {
        // A path only substitutes at the reference depth the method takes: an
        // autoref receiver has none that typechecks, so `Closure` hedges.
        match self {
            Self::Type | Self::Trait => format!(
                "`{name}` shares this module body with `{}`. One struct, enum, union, or trait per file, and per inline `mod`",
                owner.unwrap_or("another type")
            ),
            Self::Function => {
                format!("`{name}` is a free function. Hang it on a type: a method, an associated function, or a trait")
            }
            Self::Field => format!(
                "`{name}` is an exposed field. Read it through an accessor and build the struct through `From`, `new`, or a builder"
            ),
            Self::Closure => format!(
                "`|{name}()` only forwards to a method. Pass the path `Type::{}` where the \
                 reference depth already matches, or reshape the chain (`.copied()` before \
                 `.filter`) so that it does",
                name.rsplit('.').next().unwrap_or("method")
            ),
            Self::Binding => format!(
                "`{name}` annotates its type on the binding. Turbofish the call so the type stays at the point of use"
            ),
            Self::Bound => format!("`{name}` carries an inline bound. Move it to a `where` clause"),
            Self::Access => format!(
                "`{name}` reads a field. Call its accessor; only the accessor's own one-line body touches the field"
            ),
        }
    }
}
