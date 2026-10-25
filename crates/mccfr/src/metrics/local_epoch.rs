use std::cell::Cell;

thread_local! { pub(crate) static LOCAL_EPOCH: Cell<usize> = const { Cell::new(0) }; }
