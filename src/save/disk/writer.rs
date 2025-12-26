use super::*;
use crate::cards::*;
use crate::clustering::*;
use crate::gameplay::*;
use crate::mccfr::*;
use crate::save::postgres::*;
use byteorder::BE;
use byteorder::ReadBytesExt;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Seek;
use std::sync::Arc;
use tokio_postgres::Client;
use tokio_postgres::Error as E;
use tokio_postgres::binary_copy::BinaryCopyInWriter;
use tokio_postgres::types::ToSql;

pub struct Writer(Arc<Client>);

impl From<Arc<Client>> for Writer {
    fn from(client: Arc<Client>) -> Self {
        Self(client)
    }
}

impl Writer {
    pub async fn publish() -> Result<(), E> {
        let postgres = Self(crate::save::db().await);
        postgres
            .0
            .batch_execute(
                r#"
            SET lock_timeout       = 0;
            SET statement_timeout  = 0;
            SET synchronous_commit = off;
        "#,
            )
            .await?;
        postgres.upload::<Metric>().await?;
        postgres.upload::<Future>().await?;
        postgres.upload::<NlheEncoder>().await?;
        postgres.upload::<NlheProfile>().await?;
        postgres.derive::<Abstraction>().await?;
        postgres.derive::<Street>().await?;
        postgres.0.batch_execute("VACUUM ANALYZE;").await?;
        Ok(())
    }

    #[allow(deprecated)]
    async fn upload<T>(&self) -> Result<(), E>
    where
        T: Schema + Disk,
    {
        let name = <T as Schema>::name();
        if self.absent(name).await? {
            log::info!("creating table ({})", name);
            self.0.batch_execute(T::creates()).await?;
            self.0.batch_execute(T::truncates()).await?;
        }
        if self.vacant(name).await? {
            log::info!("copying  table ({})", name);
            self.stream::<T>().await?;
            log::info!("indexing table ({})", name);
            self.0.batch_execute(T::indices()).await?;
            log::info!("freezing table ({})", name);
            self.0.batch_execute(T::freeze()).await?;
            Ok(())
        } else {
            log::info!("table data already uploaded ({})", name);
            Ok(())
        }
    }
    async fn derive<D>(&self) -> Result<(), E>
    where
        D: Derive,
    {
        let name = D::name();
        if self.absent(name).await? {
            log::info!("creating table ({})", name);
            self.0.batch_execute(D::creates()).await?;
        }
        if self.vacant(name).await? {
            log::info!("deriving table ({})", name);
            self.0.batch_execute(&D::derives()).await?;
            log::info!("indexing table ({})", name);
            self.0.batch_execute(D::indices()).await?;
            log::info!("freezing table ({})", name);
            self.0.batch_execute(D::freeze()).await?;
            Ok(())
        } else {
            log::info!("table data already derived ({})", name);
            Ok(())
        }
    }
    #[allow(deprecated)]
    async fn stream<T>(&self) -> Result<(), E>
    where
        T: Schema + Disk,
    {
        let sink = self.0.copy_in(T::copy()).await?;
        let writer = BinaryCopyInWriter::new(sink, T::columns());
        futures::pin_mut!(writer);
        let ref mut fields = [0u8; 2];
        for ref mut reader in T::sources()
            .iter()
            .map(File::open)
            .map(Result::unwrap)
            .map(BufReader::new)
        {
            reader.seek(std::io::SeekFrom::Start(19)).unwrap();
            while let Ok(()) = reader.read_exact(fields) {
                match u16::from_be_bytes(*fields) {
                    0xFFFF => break,
                    length => {
                        assert!(T::columns().len() == length as usize);
                        let row = (0..length).map(|_| {
                            match reader.read_u32::<BE>().expect("field size (bytes)") {
                                4 => Field::F32(reader.read_f32::<BE>().unwrap()),
                                8 => Field::I64(reader.read_i64::<BE>().unwrap()),
                                x => panic!("unsupported type: {}", x),
                            }
                        });
                        writer.as_mut().write_raw(row).await?;
                    }
                }
            }
        }
        writer.finish().await?;
        Ok(())
    }

    async fn vacant(&self, table: &str) -> Result<bool, E> {
        let ref sql = format!(
            "
            SELECT 1
            FROM   {}
            LIMIT  1;
            ",
            table
        );
        Ok(self.0.query(sql, &[]).await?.is_empty())
    }

    async fn absent(&self, table: &str) -> Result<bool, E> {
        let ref sql = format!(
            "
            SELECT  1
            FROM    information_schema.tables
            WHERE   table_name = '{}';
            ",
            table
        );
        Ok(self.0.query(sql, &[]).await?.is_empty())
    }
}

/// Polymorphic field type for binary COPY from disk files.
/// Used by Writer to handle heterogeneous column types when reading
/// from the postgres binary copy format stored on disk.
#[derive(Debug)]
enum Field {
    F32(f32),
    I64(i64),
}

impl ToSql for Field {
    fn to_sql(
        &self,
        ty: &tokio_postgres::types::Type,
        out: &mut bytes::BytesMut,
    ) -> Result<tokio_postgres::types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        match self {
            Field::F32(val) => val.to_sql(ty, out),
            Field::I64(val) => val.to_sql(ty, out),
        }
    }
    fn accepts(ty: &tokio_postgres::types::Type) -> bool {
        <f32 as ToSql>::accepts(ty) || <i64 as ToSql>::accepts(ty)
    }
    fn to_sql_checked(
        &self,
        ty: &tokio_postgres::types::Type,
        out: &mut bytes::BytesMut,
    ) -> Result<tokio_postgres::types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        match self {
            Field::F32(val) => val.to_sql_checked(ty, out),
            Field::I64(val) => val.to_sql_checked(ty, out),
        }
    }
}
