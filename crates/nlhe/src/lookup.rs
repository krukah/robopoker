//! Blueprint policy lookup via database queries.
//!
//! Shared query logic for resolving a player's recall into a trained
//! strategy. Used by both the analysis API and CPU player implementations.
use super::*;
use deuce::Isomorphism;
use kicker::*;
use mccfr::*;
use pokerkit::Probability;
use std::sync::OnceLock;

fn abs_sql() -> &'static str {
    static SQL: OnceLock<&str> = OnceLock::<&str>::new();
    SQL.get_or_init(|| ledger::leaked(format!("SELECT abs FROM {} WHERE obs = $1", ledger::isomorphism())))
}
fn policy_sql() -> &'static str {
    static SQL: OnceLock<&str> = OnceLock::<&str>::new();
    SQL.get_or_init(|| {
        ledger::leaked(format!(
            "SELECT edge, weight, visits, payoff FROM {} \
         WHERE past = $1 AND present = $2 AND choices = $3",
            ledger::blueprint()
        ))
    })
}

/// Looks up the trained blueprint strategy for a given recall state.
///
/// Performs two database queries:
/// 1. Maps the observation to its abstraction bucket
/// 2. Fetches the accumulated strategy weights for that information set
///
/// Returns `None` if the observation has no abstraction mapping or the
/// information set has no trained strategy.
pub async fn lookup(client: &tokio_postgres::Client, recall: &Witness) -> Option<Strategy> {
    let iso = Isomorphism::from(recall.seen());
    let abs = client
        .query_one(abs_sql(), &[&i64::from(iso)])
        .await
        .map(|row| Abstraction::from(row.get::<_, i16>(0)))
        .inspect_err(|e| tracing::warn!("obs_to_abs failed: {e}"))
        .ok()?;
    let info = NlheInfo::from((recall, abs));
    let sql = policy_sql();
    let ref history = i64::from(info.subgame());
    let ref present = i16::from(info.bucket());
    let ref choices = i64::from(info.choices());
    let rows = client
        .query(sql, &[history, present, choices])
        .await
        .inspect_err(|e| tracing::warn!("blueprint query failed: {e}"))
        .ok()?;
    match rows.len() {
        0 => {
            tracing::debug!("blueprint miss: past={history} present={present} choices={choices}");
            None
        }
        _ => Some(Strategy::from((
            info,
            rows.into_iter()
                .map(|row| Decision {
                    edge: NlheEdge::from(row.get::<_, i64>("edge") as u64),
                    mass: Probability::from(row.get::<_, f32>("weight")),
                    visits: row.get::<_, i32>("visits") as u32,
                    payoff: row.get::<_, f32>("payoff"),
                })
                .collect::<Vec<_>>(),
        ))),
    }
}
