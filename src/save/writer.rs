use super::derive::Derive;
use super::upload::Upload;
use crate::cards::street::Street;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::metric::Metric;
use crate::clustering::transitions::Decomp;
use crate::mccfr::encoder::Encoder;
use crate::mccfr::profile::Profile;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Seek;
use std::sync::Arc;
use tokio_postgres::binary_copy::BinaryCopyInWriter;
use tokio_postgres::Client;
use tokio_postgres::Error as E;

pub struct Writer(Arc<Client>);

impl From<Arc<Client>> for Writer {
    fn from(client: Arc<Client>) -> Self {
        Self(client)
    }
}

impl Writer {
    pub async fn save() -> Result<(), E> {
        let postgres = Self(crate::db().await);
        postgres.upload::<Metric>().await?;
        postgres.upload::<Decomp>().await?;
        postgres.upload::<Encoder>().await?;
        postgres.upload::<Profile>().await?;
        postgres.derive::<Abstraction>().await?;
        postgres.derive::<Street>().await?;
        postgres.vacuum().await?;
        Ok(())
    }

    async fn upload<U>(&self) -> Result<(), E>
    where
        U: Upload,
    {
        let ref name = U::name();
        if self.has_rows(name).await? {
            log::info!("tables data already uploaded ({})", name);
            Ok(())
        } else {
            log::info!("copying {}", name);
            self.0.batch_execute(&U::prepare()).await?;
            self.0.batch_execute(&U::nuke()).await?;
            let sink = self.0.copy_in(&U::copy()).await?;
            let writer = BinaryCopyInWriter::new(sink, U::columns());
            futures::pin_mut!(writer);
            let ref mut count = [0u8; 2];
            for ref mut reader in U::sources()
                .iter()
                .map(|s| File::open(s).expect("file not found"))
                .map(|f| BufReader::new(f))
            {
                reader.seek(std::io::SeekFrom::Start(19)).unwrap();
                while let Ok(_) = reader.read_exact(count) {
                    match u16::from_be_bytes(count.clone()) {
                        0xFFFF => break,
                        length => {
                            assert!(length == U::columns().len() as u16);
                            let row = U::read(reader);
                            let row = row.iter().map(|b| &**b).collect::<Vec<_>>();
                            writer.as_mut().write(&row).await?;
                        }
                    }
                }
            }
            writer.finish().await?;
            self.0.batch_execute(&U::indices()).await?;
            Ok(())
        }
    }

    async fn derive<D>(&self) -> Result<(), E>
    where
        D: Derive,
    {
        let ref name = D::name();
        if self.has_rows(name).await? {
            log::info!("tables data already uploaded ({})", name);
            Ok(())
        } else {
            log::info!("deriving {}", name);
            let truncate = D::prepare();
            let index = D::indexes();
            let rows = D::exhaust()
                .into_iter()
                .map(|r| r.inserts())
                .collect::<Vec<_>>();
            let ref statement = std::iter::empty()
                .chain(std::iter::once(truncate))
                .chain(std::iter::once(index))
                .chain(rows)
                .collect::<Vec<_>>()
                .join("\n;");
            self.0.batch_execute(statement).await?;
            Ok(())
        }
    }

    async fn vacuum(&self) -> Result<(), E> {
        self.0.batch_execute("VACUUM ANALYZE;").await
    }
    async fn has_rows(&self, table: &str) -> Result<bool, E> {
        if self.does_exist(table).await? {
            let ref sql = format!(
                "
                SELECT 1
                FROM   {}
                LIMIT  1;
                ",
                table
            );
            Ok(0 != self.0.query_one(sql, &[]).await?.get::<_, i64>(0))
        } else {
            Ok(false)
        }
    }
    async fn does_exist(&self, table: &str) -> Result<bool, E> {
        let ref sql = format!(
            "
            SELECT  1
            FROM    information_schema.tables
            WHERE   table_name = '{}';
            ",
            table
        );
        Ok(1 == self.0.query_one(sql, &[]).await?.get::<_, i64>(0))
    }
}
