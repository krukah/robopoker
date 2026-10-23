// Rayon workers free what other workers allocated; mimalloc does it without the system lock.
#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;
