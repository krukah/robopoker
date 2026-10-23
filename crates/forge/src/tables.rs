/// Table creation for the training pipeline.
pub(crate) struct Tables;

impl Tables {
    /// Ensures all training-related tables exist.
    pub async fn ensure_all(client: &tokio_postgres::Client) {
        use daybook::Ensure;
        client.ensure::<nlhe::NlheBlueprint>().await;
        client.ensure::<crate::EpochMeta>().await;
        client.ensure::<crate::Snapshot>().await;
        client.ensure::<daybook::Fingerprint>().await;
    }
}
