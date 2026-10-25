use crate::*;

pub const RULES: &[&str] = &[
    "long-comment",
    "long-doc-comment",
    "hidden-doc-comment",
    "doc-code-ratio",
    "type-comment",
    "ticket-reference",
    "commented-code",
    "todo-comment",
    "banner-comment",
    complexity::RULE,
    shape::ONE_TYPE_PER_FILE,
    ONE_FILE_DIRECTORY,
    WIRING_FILE,
    ENTRY_FILE,
    shape::FREE_FUNCTION,
    shape::PUB_FIELD,
    shape::FORWARDING_CLOSURE,
    shape::TYPE_HINT,
    shape::INLINE_BOUND,
    shape::FIELD_ACCESS,
];

/// A rule that fires only when `--enable` or `--only` names it, so CI never
/// fails on one. Every other rule gates every file a PR touches.
pub const OPT_IN_RULES: &[&str] = &[
    shape::FORWARDING_CLOSURE,
    shape::TYPE_HINT,
    shape::INLINE_BOUND,
    shape::FIELD_ACCESS,
];

pub const MAX_COMMENT_LINES: usize = 2;
pub const MAX_DOC_COMMENT_LINES: usize = 5;

pub const MAX_DOC_CODE_RATIO: f64 = 1.0 / 3.0;
pub const MIN_RATIO_CODE_LINES: usize = 32;
