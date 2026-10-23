use pokerkit::*;

/// Namespace for the request routines.
pub(crate) struct Request;

impl Request {
    pub(crate) fn default_stacks() -> [Chips; N] {
        [STACK; N]
    }

    pub(crate) fn default_limit() -> i64 {
        100
    }
}
