#![allow(dead_code)] // TODO: Remove

use std::{
    env,
    error::Error,
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
    path::PathBuf,
};

use tracing::error;

use crate::{
    records::DatabaseBuilder,
    sources::{
        jumli_data::JumliData, use_this_instead::UseThisInstead,
        workshop_database::WorkshopDatabase,
    },
};

pub mod consts;
pub mod records;
pub mod sources;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();
    let out_path = if let Some(path) = env::args().skip(1).next() {
        PathBuf::from(path)
    } else {
        error!("Missing required argument.\nExpected: jumli_gen <out_dir>");
        return Ok(());
    };

    if !out_path.is_dir() {
        error!("Output directory {out_path:?} does not exist or is not a directory.");
        return Ok(());
    }

    let mut builder = DatabaseBuilder::new();
    builder.ingest_from(WorkshopDatabase::new()).await?;
    builder.ingest_from(UseThisInstead::new()).await?;
    builder.ingest_from(JumliData::new()).await?;

    let db = builder.finalize().await;

    let generated_path = out_path.join("generated");
    std::fs::create_dir(&generated_path)?;

    serde_json::to_writer(
        BufWriter::new(
            OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(generated_path.join("index.json"))?,
        ),
        &db.indices,
    )?;

    std::fs::create_dir(generated_path.join("mods"))?;
    for (idx, record) in db.records.iter().enumerate() {
        serde_json::to_writer(
            BufWriter::new(
                OpenOptions::new()
                    .create(true)
                    .truncate(true)
                    .write(true)
                    .open(generated_path.join(format!("mods/{idx}.json")))?,
            ),
            &record,
        )?;
    }

    Ok(())
}
