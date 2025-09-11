use std::{env, error::Error, fs::OpenOptions, io::BufWriter, path::PathBuf};

use tracing::{error, info};

use crate::{
    records::{DatabaseBuilder, types::ModIdentifier},
    render::{RenderHtml, redirect_html, render_diagnostics},
    sources::{jumli_data::JumliData, use_this_instead::UseThisInstead},
};

pub mod consts;
pub mod records;
pub mod render;
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

    let static_path = if let Some(path) = env::args().skip(2).next() {
        Some(PathBuf::from(path))
    } else {
        None
    };

    if !out_path.is_dir() {
        error!("Output directory {out_path:?} does not exist or is not a directory.");
        return Ok(());
    }

    if let Some(static_path) = static_path {
        info!("Copying static assets.");
        copy_dir(&static_path, &out_path)?;
    }

    let mut builder = DatabaseBuilder::new();
    //    builder.ingest_from(WorkshopDatabase::new()).await?; -- Bloats the index and appears to be out-of-date often
    builder.ingest_from(UseThisInstead::new()).await?;
    builder.ingest_from(JumliData::new()).await?;

    let db = builder.finalize().await;

    info!("Rendering diagnostics.");
    std::fs::write(out_path.join("diagnostics.html"), render_diagnostics(&db))?;

    let mods_path = out_path.join("mods");
    std::fs::create_dir(&mods_path)?;

    info!("Saving index.");
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

    info!("Rendering reports.");
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

fn copy_dir(from: &PathBuf, to: &PathBuf) -> Result<(), std::io::Error> {
    let read_dir = std::fs::read_dir(from)?;

    for entry in read_dir {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            std::fs::create_dir(to.join(entry.file_name()))?;
            copy_dir(&entry.path(), &to.join(entry.file_name()))?;
            continue;
        }

        std::fs::copy(entry.path(), to.join(entry.file_name()))?;
    }

    Ok(())
}
