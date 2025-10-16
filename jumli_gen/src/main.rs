use std::{env, error::Error, fs::OpenOptions, io::BufWriter, path::PathBuf};

use mapmysite::{ChangeFreq, Sitemap, SitemapUrl};
use tracing::{error, info};

use crate::{
    records::{DatabaseBuilder, types::ModIdentifier},
    render::{RenderHtml, frame_html, render_diagnostics},
    sources::{jumli_data::JumliData, use_this_instead::UseThisInstead},
};

pub mod consts;
pub mod records;
pub mod render;
pub mod sources;

pub const SUBDIR_WORKSHOP_REDIRECT: &'static str = "workshop";
pub const SUBDIR_PACKAGEID_REDIRECT: &'static str = "package";
pub const SUBDIR_MOD_REPORTS: &'static str = "mods";
pub const PATH_DIAGNOSTICS_REPORT: &'static str = "diagnostics.html";
pub const SITEMAP_URL_BASE: &'static str = "https://jumli.sysrqmagician.dev";

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

    let mut builder = DatabaseBuilder::new();
    //    builder.ingest_from(WorkshopDatabase::new()).await?; -- Bloats the index and appears to be out-of-date often
    builder.ingest_from(UseThisInstead::new()).await?;
    builder.ingest_from(JumliData::new()).await?;

    let db = builder.finalize().await;

    let mut sitemap = Sitemap::new();
    sitemap.add_url(
        SitemapUrl::from_base(SITEMAP_URL_BASE, "")
            .change_frequency(ChangeFreq::Daily)
            .last_modified_now(),
    );

    if let Some(static_path) = static_path {
        info!("Copying static assets.");
        copy_static(&mut sitemap, &static_path, &out_path)?;
    }

    info!("Rendering diagnostics.");
    std::fs::write(
        out_path.join(PATH_DIAGNOSTICS_REPORT),
        render_diagnostics(&db),
    )?;
    sitemap.add_url(
        SitemapUrl::from_base(SITEMAP_URL_BASE, PATH_DIAGNOSTICS_REPORT)
            .change_frequency(ChangeFreq::Daily)
            .last_modified_now()
            .priority(1.0),
    );
    let mods_path = out_path.join(SUBDIR_MOD_REPORTS);
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
    let workshop_path = out_path.join(SUBDIR_WORKSHOP_REDIRECT);
    let package_path = out_path.join(SUBDIR_PACKAGEID_REDIRECT);
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
                        frame_html(format!("/{SUBDIR_MOD_REPORTS}/{idx}.html")),
                    )?;

                    sitemap.add_url(
                        SitemapUrl::from_base(
                            SITEMAP_URL_BASE,
                            format!("{SUBDIR_PACKAGEID_REDIRECT}/{id}"),
                        )
                        .change_frequency(ChangeFreq::Daily)
                        .last_modified_now()
                        .priority(0.5),
                    );
                }

                ModIdentifier::WorkshopId(id) => {
                    std::fs::create_dir(workshop_path.join(id.to_string()))?;
                    std::fs::write(
                        workshop_path.join(format!("{id}/index.html")),
                        frame_html(format!("/{SUBDIR_MOD_REPORTS}/{idx}.html")),
                    )?;

                    sitemap.add_url(
                        SitemapUrl::from_base(
                            SITEMAP_URL_BASE,
                            format!("{SUBDIR_WORKSHOP_REDIRECT}/{id}"),
                        )
                        .change_frequency(ChangeFreq::Daily)
                        .last_modified_now()
                        .priority(0.5),
                    );
                }
            }
        }
    }
    std::fs::write(out_path.join("sitemap.xml"), sitemap.to_string()?)?;

    Ok(())
}

fn copy_static(sitemap: &mut Sitemap, from: &PathBuf, to: &PathBuf) -> Result<(), Box<dyn Error>> {
    fn recurse(
        sitemap: &mut Sitemap,
        root: &PathBuf,
        from: &PathBuf,
        to: &PathBuf,
    ) -> Result<(), Box<dyn Error>> {
        let read_dir = std::fs::read_dir(from)?;
        for entry in read_dir {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                std::fs::create_dir(to.join(entry.file_name()))?;
                recurse(sitemap, &root, &entry.path(), &to.join(entry.file_name()))?;
                continue;
            }

            std::fs::copy(entry.path(), to.join(entry.file_name()))?;
            if entry.path().extension().and_then(|x| x.to_str()) == Some("html") {
                sitemap.add_url(
                    SitemapUrl::from_base(
                        SITEMAP_URL_BASE,
                        entry.path().strip_prefix(root)?.display(),
                    )
                    .change_frequency(ChangeFreq::Daily)
                    .last_modified_now()
                    .priority(1.0),
                );
            }
        }
        Ok(())
    }
    recurse(sitemap, from, from, to)?;

    Ok(())
}
