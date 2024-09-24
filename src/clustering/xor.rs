use crate::clustering::abstraction::NodeAbstraction;

/// A unique identifier for a pair of abstractions.
#[derive(Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Ord, Debug)]
pub struct Pair(u64);
impl From<(&NodeAbstraction, &NodeAbstraction)> for Pair {
    fn from((a, b): (&NodeAbstraction, &NodeAbstraction)) -> Self {
        Self(u64::from(*a) ^ u64::from(*b))
    }
}
impl From<Pair> for i64 {
    fn from(pair: Pair) -> Self {
        pair.0 as i64
    }
}
impl From<i64> for Pair {
    fn from(i: i64) -> Self {
        Self(i as u64)
    }
}

impl tokio_postgres::types::ToSql for Pair {
    fn to_sql(
        &self,
        ty: &tokio_postgres::types::Type,
        out: &mut bytes::BytesMut,
    ) -> Result<tokio_postgres::types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        i64::from(*self).to_sql(ty, out)
    }

    fn accepts(ty: &tokio_postgres::types::Type) -> bool {
        <i64 as tokio_postgres::types::ToSql>::accepts(ty)
    }

    fn to_sql_checked(
        &self,
        ty: &tokio_postgres::types::Type,
        out: &mut bytes::BytesMut,
    ) -> Result<tokio_postgres::types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        i64::from(*self).to_sql_checked(ty, out)
    }
}
