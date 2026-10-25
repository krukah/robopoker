/// Extension trait for f32 to enable method chaining.
pub(crate) trait ThenF32 {
    fn then(self, f: impl FnOnce(f32) -> f32) -> f32;
}
