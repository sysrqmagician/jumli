use std::{env, error::Error, fs::OpenOptions, io::BufWriter, path::PathBuf};

use maud::html;
use tracing::error;

use crate::{
    records::{DatabaseBuilder, types::ModIdentifier},
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
    //    builder.ingest_from(WorkshopDatabase::new()).await?; -- Frankly unnecessary since users of the site will already have RimSort. Just bloats the index.
    builder.ingest_from(UseThisInstead::new()).await?;
    builder.ingest_from(JumliData::new()).await?;

    let db = builder.finalize().await;

    let mods_path = out_path.join("mods");
    std::fs::create_dir(&mods_path)?;

    serde_json::to_writer(
        BufWriter::new(
            OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(mods_path.join("index.json"))?,
        ),
        &db.indices,
    )?;

    let workshop_path = out_path.join("workshop");
    let package_path = out_path.join("package");
    std::fs::create_dir(mods_path.join("mods"))?;
    std::fs::create_dir(&workshop_path)?;
    std::fs::create_dir(&package_path)?;
    for (idx, record) in db.records.iter().enumerate() {
        serde_json::to_writer(
            BufWriter::new(
                OpenOptions::new()
                    .create(true)
                    .truncate(true)
                    .write(true)
                    .open(mods_path.join(format!("{idx}.json")))?,
            ),
            &record,
        )?;

        std::fs::write(mods_path.join(format!("{idx}.html")), record.render_html())?;

        'inner: for identifier in &record.identifiers {
            match identifier {
                ModIdentifier::PackageId(id) => {
                    if id.is_empty() || !id.chars().all(|c| c != '/' && !c.is_control()) {
                        continue 'inner;
                    }
                    std::fs::create_dir(package_path.join(id))?;
                    std::fs::write(
                        package_path.join(format! {"{id}/index.html"}),
                        redirect_html(format!("/mods/{idx}.html")),
                    )?;
                }
                ModIdentifier::WorkshopId(id) => {
                    std::fs::create_dir(workshop_path.join(id.to_string()))?;
                    std::fs::write(
                        workshop_path.join(format!("{id}/index.html")),
                        redirect_html(format!("/mods/{idx}.html")),
                    )?;
                }
            }
        }
    }

    Ok(())
}

fn redirect_html(destination: String) -> String {
    html! {
        head {
            meta http-equiv="refresh" content=(format!("0; {destination}")){}
        }
        body {
            p { "You are being redirected..." }
        }
    }
    .into_string()
}
