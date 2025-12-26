use crate::autotrain::*;
use crate::clustering::*;
use crate::mccfr::*;
use crate::save::*;
use std::sync::Arc;
use tokio_postgres::Client;

/// Get a database connection, run migrations, and return the client.
pub async fn db() -> Arc<Client> {
    log::info!("connecting to database");
    let tls = tokio_postgres::tls::NoTls;
    let ref url = std::env::var("DB_URL").expect("DB_URL must be set");
    let (client, connection) = tokio_postgres::connect(url, tls)
        .await
        .expect("database connection failed");
    tokio::spawn(connection);
    client
        .execute("SET client_min_messages TO WARNING", &[])
        .await
        .expect("set client_min_messages");
    client
        .batch_execute(&Epoch::creates())
        .await
        .expect("epoch");
    client
        .batch_execute(&Metric::creates())
        .await
        .expect("metric");
    client
        .batch_execute(&Future::creates())
        .await
        .expect("transitions");
    client
        .batch_execute(&Lookup::creates())
        .await
        .expect("isomorphism");
    client
        .batch_execute(&NlheProfile::creates())
        .await
        .expect("blueprint");
    Arc::new(client)
}
