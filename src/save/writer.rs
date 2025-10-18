use super::derive::Derive;
use super::upload::Table;
use crate::cards::street::Street;
use crate::clustering::metric::Metric;
use crate::clustering::transitions::Shadow;
use crate::gameplay::abstraction::Abstraction;
use crate::mccfr::nlhe::encoder::Encoder;
use crate::mccfr::nlhe::profile::Profile;
use byteorder::ReadBytesExt;
use byteorder::BE;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Seek;
use std::sync::Arc;
use tokio_postgres::binary_copy::BinaryCopyInWriter;
use tokio_postgres::types::ToSql;
use tokio_postgres::Client;
use tokio_postgres::Error as E;

pub struct Writer(Arc<Client>);

impl From<Arc<Client>> for Writer {
    fn from(client: Arc<Client>) -> Self {
        Self(client)
    }
}

impl Writer {
    pub async fn publish() -> Result<(), E> {
        let postgres = Self(crate::db().await);
        postgres.upload::<Metric>().await?;
        postgres.upload::<Shadow>().await?;
        postgres.upload::<Encoder>().await?;
        postgres.upload::<Profile>().await?;
        postgres.derive::<Abstraction>().await?;
        postgres.derive::<Street>().await?;
        postgres.0.batch_execute("VACUUM ANALYZE;").await?;
        Ok(())
    }

    async fn upload<T>(&self) -> Result<(), E>
    where
        T: Table,
    {
        let ref name = T::name();
        if self.absent(name).await? {
            log::info!("creating table ({})", name);
            self.0.batch_execute(&T::creates()).await?;
            self.0.batch_execute(&T::truncates()).await?;
        }
        if self.vacant(name).await? {
            log::info!("copying {}", name);
            self.stream::<T>().await?;
            self.0.batch_execute(&T::indices()).await?;
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
        let ref name = D::name();
        if self.absent(name).await? {
            log::info!("creating table ({})", name);
            self.0.batch_execute(&D::creates()).await?;
        }
        if self.vacant(name).await? {
            log::info!("deriving {}", name);
            self.0.batch_execute(&D::indexes()).await?;
            self.0.batch_execute(&D::derives()).await?;
            Ok(())
        } else {
            log::info!("table data already derived  ({})", name);
            Ok(())
        }
    }

    async fn stream<T>(&self) -> Result<(), E>
    where
        T: Table,
    {
        let sink = self.0.copy_in(&T::copy()).await?;
        let writer = BinaryCopyInWriter::new(sink, T::columns());
        futures::pin_mut!(writer);
        let ref mut fields = [0u8; 2];
        for ref mut reader in T::sources()
            .iter()
            .map(|s| File::open(s).expect("file not found"))
            .map(|f| BufReader::new(f))
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

/// doing this for zero copy reasons
/// it was impossible to achieve polymorphism between column types
/// without allocating a ton since writer.as_mut().write()
/// required &[&dyn ToSql]
/// which would have required collection into a Vec<&dyn ToSql>
/// because of lifetime reasons. now, we only need an Iterator<Item = T: ToSql>
/// which is much more flexible, so we can map T::columns() to dynamically
/// iterate over table columns.
#[derive(Debug)]
enum Field {
    F32(f32),
    I64(i64),
}

impl tokio_postgres::types::ToSql for Field {
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
