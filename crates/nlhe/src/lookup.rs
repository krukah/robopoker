//! Blueprint policy lookup via database queries — resolving a recall into a
//! trained strategy, shared by the analysis API and the CPU players.
use super::*;
use deuce::Isomorphism;
use kicker::*;
use mccfr::*;
use pokerkit::Probability;
use std::sync::OnceLock;

fn abs_sql() -> &'static str {
    static SQL: OnceLock<&str> = OnceLock::<&str>::new();
    SQL.get_or_init(|| daybook::leaked(format!("SELECT abs FROM {} WHERE obs = $1", daybook::isomorphism())))
}
fn policy_sql() -> &'static str {
    static SQL: OnceLock<&str> = OnceLock::<&str>::new();
    SQL.get_or_init(|| {
        daybook::leaked(format!(
            "SELECT edge, weight, visits, payoff FROM {} \
         WHERE past = $1 AND present = $2 AND choices = $3 AND context = $4",
            daybook::blueprint()
        ))
    })
}

/// Trained blueprint strategy for a recall state: one query to map the
/// observation to its abstraction bucket, one to fetch that info set's
/// accumulated weights. `None` if either is missing.
pub async fn lookup(client: &tokio_postgres::Client, recall: &Witness) -> Option<Strategy> {
    let iso = Isomorphism::from(recall.seen());
    let abs = client
        .query_one(abs_sql(), &[&iso])
        .await
        .map(|row| row.get::<_, Abstraction>(0))
        .inspect_err(|e| tracing::warn!("obs_to_abs failed: {e}"))
        .ok()?;
    let info = NlheInfo::from((recall, abs));
    let sql = policy_sql();
    let rows = client
        .query(sql, &[&info.subgame(), &info.bucket(), &info.choices(), &info.field()])
        .await
        .inspect_err(|e| tracing::warn!("blueprint query failed: {e}"))
        .ok()?;
    match rows.len() {
        0 => {
            tracing::debug!(
                "blueprint miss: past={} present={} choices={}",
                info.subgame(),
                info.bucket(),
                info.choices()
            );
            None
        }
        _ => Some(Strategy::from((
            info,
            rows.into_iter()
                .map(|row| Decision {
                    edge: NlheEdge::from(row.get::<_, Edge>("edge")),
                    mass: Probability::from(row.get::<_, f32>("weight")),
                    visits: row.get::<_, i32>("visits") as u32,
                    payoff: row.get::<_, f32>("payoff"),
                })
                .collect::<Vec<_>>(),
        ))),
    }
}
