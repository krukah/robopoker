#[allow(async_fn_in_trait)]
pub trait Persistence {
    async fn save(&self);
    async fn load() -> Self
    where
        Self: Sized;
}
