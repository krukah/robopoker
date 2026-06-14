//! Run mode: fixed hand count vs run-until-interrupt.

/// Chosen at `Runtime::from_args` from `--hands`/`--continuous`.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Mode {
    Fixed(usize),
    Continuous,
}
