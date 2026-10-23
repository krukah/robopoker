use daybook::*;
use std::sync::OnceLock;

/// Prepared statements for the sink.
pub(crate) struct Upsert;
impl Upsert {
    pub(crate) fn upsert_sql() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| {
            daybook::Pool::leaked(format!(
                "INSERT INTO {} (past, present, choices, context, edge, weight, regret, payoff, visits) \
             VALUES         ($1,   $2,      $3,       $4,      $5,   $6,     $7,     $8,     $9) \
             ON CONFLICT (past, present, choices, context, edge) \
             DO UPDATE SET \
                 weight = EXCLUDED.weight, \
                 regret = EXCLUDED.regret, \
                 payoff = EXCLUDED.payoff, \
                 visits = EXCLUDED.visits",
                blueprint()
            ))
        })
    }

    pub(crate) fn advance_sql() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| {
            daybook::Pool::leaked(format!("UPDATE {} SET value = value + 1 WHERE key = 'current'", epoch()))
        })
    }
}
