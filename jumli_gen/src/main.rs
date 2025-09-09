#![allow(dead_code)] // TODO: Remove

use crate::{
    records::DatabaseBuilder,
    sources::{
        RecordSource, jumli_data::JumliData, use_this_instead::UseThisInstead,
        workshop_database::WorkshopDatabase,
    },
};

pub mod consts;
pub mod records;
pub mod sources;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let mut builder = DatabaseBuilder::new();
    builder
        .ingest_from(WorkshopDatabase::new())
        .await
        .expect("Unable to fetch WSDB Data");
    builder
        .ingest_from(UseThisInstead::new())
        .await
        .expect("Unable to fetch UTI Data");
    builder
        .ingest_from(JumliData::new())
        .await
        .expect("Unable to fetch JuMLi Data");

    let db = builder.finalize().await;
}
