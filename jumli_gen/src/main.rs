#![allow(dead_code)] // TODO: Remove

use crate::sources::{RecordSource, workshop_database::WorkshopDatabase};

pub mod consts;
pub mod records;
pub mod sources;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let mut wsdb = WorkshopDatabase::new();
    wsdb.fetch().await.unwrap();
}
