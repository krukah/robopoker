use crate::*;
use tokio_postgres::Client;

/// Row counting for the table checks.
pub(crate) struct Rows;
impl Rows {
    pub(crate) async fn count(client: &Client, table: &str) -> usize {
        let sql = format!("SELECT COUNT(*) FROM {table}");
        Measure::measure("check.count", client.query_opt(&sql, &[]))
            .await
            .ok()
            .flatten()
            .map_or(0, |r| r.get::<_, i64>(0) as usize)
    }
}
