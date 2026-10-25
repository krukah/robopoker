//! Provenance markers for [`Trail`](crate::Trail)(crate::Trail).
/// Where a [`Trail`](crate::Trail) came from. A closed vocabulary of three markers —
/// they carry no data and exist only to keep structurally identical
/// descent lists from being spliced into each other.
pub trait Provenance {}

/// A full play-through from the game root to "now". Marks [`Replay`](crate::Replay).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Fresh;

/// Context that predates the solver, fixed at construction. Marks
/// [`Prefix`](crate::Prefix), and deliberately has no `push`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Fixed;

/// A story growing during a biased rollout. Marks [`Story`](crate::Story), the only
/// provenance that can `push`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Grown;

impl Provenance for Fresh {}

impl Provenance for Fixed {}

impl Provenance for Grown {}
