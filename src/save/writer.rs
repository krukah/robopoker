use super::derive::Derive;
use super::upload::Upload;
use crate::cards::street::Street;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::metric::Metric;
use crate::clustering::transitions::Decomp;
use crate::mccfr::encoder::Encoder;
use crate::mccfr::profile::Profile;
use byteorder::ReadBytesExt;
use byteorder::BE;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Seek;
use std::sync::Arc;
use tokio_postgres::binary_copy::BinaryCopyInWriter;
use tokio_postgres::types::ToSql;
use tokio_postgres::types::Type;
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
        postgres.upload::<Encoder>().await?; // Lookup ?
        postgres.upload::<Profile>().await?;
        postgres.derive::<Abstraction>().await?;
        postgres.derive::<Street>().await?;
        postgres.vacuum().await?;
        Ok(())
    }

    async fn derive<D>(&self) -> Result<(), E>
    where
        D: Derive,
    {
        let ref name = D::name();
        // if !does_exist(name).await? {
        // create
        // }
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

    async fn upload<U>(&self) -> Result<(), E>
    where
        U: Upload,
    {
        let ref name = U::name();
        // if !does_exist(name).await? {
        // create
        // }
        if self.has_rows(name).await? {
            log::info!("tables data already uploaded ({})", name);
            Ok(())
        } else {
            log::info!("copying {}", name);
            self.prepare::<U>().await?;
            self.stream::<U>().await?;
            self.index::<U>().await?;
            Ok(())
        }
    }

    async fn prepare<T>(&self) -> Result<(), E>
    where
        T: Upload,
    {
        self.0.batch_execute(&T::prepare()).await?;
        self.0.batch_execute(&T::nuke()).await?;
        Ok(())
    }

    async fn index<T>(&self) -> Result<(), E>
    where
        T: Upload,
    {
        self.0.batch_execute(&T::indices()).await?;
        Ok(())
    }

    async fn stream<T>(&self) -> Result<(), E>
    where
        T: Upload,
    {
        let sink = self.0.copy_in(&T::copy()).await?;
        let writer = BinaryCopyInWriter::new(sink, T::columns());
        futures::pin_mut!(writer);
        let ref mut count = [0u8; 2];
        for ref mut reader in T::sources()
            .iter()
            .map(|s| File::open(s).expect("file not found"))
            .map(|f| BufReader::new(f))
        {
            reader.seek(std::io::SeekFrom::Start(19)).unwrap();
            while let Ok(_) = reader.read_exact(count) {
                match u16::from_be_bytes(count.clone()) {
                    0xFFFF => break,
                    length => {
                        assert!(length == T::columns().len() as u16);
                        let row = Self::read::<T>(reader);
                        let row = row.iter().map(|b| &**b).collect::<Vec<_>>();
                        writer.as_mut().write(&row).await?;
                    }
                }
            }
        }
        writer.finish().await?;
        Ok(())
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
            Ok(!self.0.query(sql, &[]).await?.is_empty())
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
        Ok(!self.0.query(sql, &[]).await?.is_empty())
    }

    fn read<T>(reader: &mut BufReader<File>) -> Vec<Box<dyn ToSql + Sync>>
    where
        T: Upload,
    {
        T::columns()
            .iter()
            .cloned()
            .map(|ty| match ty {
                Type::FLOAT4 => {
                    assert!(reader.read_u32::<BE>().expect("length") == 4);
                    Box::new(reader.read_f32::<BE>().expect("data")) as Box<dyn ToSql + Sync>
                }
                Type::INT8 => {
                    assert!(reader.read_u32::<BE>().expect("length") == 8);
                    Box::new(reader.read_i64::<BE>().expect("data")) as Box<dyn ToSql + Sync>
                }
                _ => panic!("unsupported type: {}", ty),
            })
            .collect::<Vec<Box<dyn ToSql + Sync>>>()
    }
}
