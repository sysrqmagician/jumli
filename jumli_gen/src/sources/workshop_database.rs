use std::{collections::HashMap, env::temp_dir, fs::File, io::BufReader};

use chrono::Utc;
use git2::FetchOptions;
use serde::Deserialize;
use tracing::info;

use crate::{
    consts::LATEST_RIMWORLD_RELEASE,
    records::types::{Certainty, IngestibleData, ModIdentifier, Notice, NoticeRecord, Source},
    sources::{Diagnostics, RecordSource},
};

pub const REPOSITORY_URL: &'static str = "https://github.com/RimSort/Steam-Workshop-Database";

#[derive(Deserialize, Debug)]
pub struct Wsdb {
    database: HashMap<u64, WsdbRecord>,
}

#[derive(Deserialize, Debug)]
struct WsdbRecord {
    #[serde(rename = "packageId")]
    package_id: Option<String>,
    #[serde(rename = "gameVersions")]
    game_versions: Option<GameVersions>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum GameVersions {
    Single(String),
    Many(Vec<String>),
}

impl GameVersions {
    pub fn as_vec(&self) -> Vec<&String> {
        match self {
            Self::Single(value) => vec![value],
            Self::Many(values) => values.iter().collect(), // TODO: Unjank
        }
    }
}

pub struct WorkshopDatabase {
    records: Vec<IngestibleData>,
    errors: Vec<String>,
}

impl WorkshopDatabase {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            errors: Vec::new(),
        }
    }
}

impl RecordSource for WorkshopDatabase {
    async fn fetch(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut repo_fo = FetchOptions::new();
        repo_fo.depth(1);

        let mut repo_builder = git2::build::RepoBuilder::new();
        repo_builder.fetch_options(repo_fo);

        let mut repo_dir = temp_dir();
        repo_dir.push("jumli_wsdb");

        info!("Cloning WSDB Repo to {repo_dir:?}.");
        let _repo = repo_builder.clone(REPOSITORY_URL, &repo_dir)?;
        info!("Cloned WSDB Repo.");

        let mut db_path = repo_dir.clone();
        db_path.push("steamDB.json");

        let db: Wsdb = serde_json::from_reader(BufReader::new(File::open(db_path)?))?;

        for (steam_id, entry) in db.database.iter() {
            if entry.game_versions.is_none() {
                // Only using this source to generate outdated notices. If there isn't a version, the data has no use for us.
                continue;
            }

            if let Some(_) = entry
                .game_versions
                .as_ref()
                .expect("was checked")
                .as_vec()
                .iter()
                .find(|x| *x == &LATEST_RIMWORLD_RELEASE)
            {
                continue;
            }

            let mut identifiers = vec![ModIdentifier::WorkshopId(*steam_id)];
            if let Some(package_id) = &entry.package_id {
                identifiers.push(ModIdentifier::PackageId(package_id.clone()))
            }

            self.records.push(IngestibleData {
                identifiers,
                notices: vec![NoticeRecord {
                    certainty: Certainty::Medium,
                    date: Some(Utc::now().date_naive()),
                    notice: Notice::OutOfDate,
                    source: Source::WorkshopDatabase,
                    context_url: None,
                }],
            });
        }

        info!(
            "Completed WSDB processing, yielding {} records and {} errors.",
            self.records.len(),
            self.errors.len()
        );

        info!("Deleting WSDB Repo {repo_dir:?}.",);
        std::fs::remove_dir_all(repo_dir)?;
        Ok(())
    }

    fn get_records(&mut self) -> Option<&mut Vec<IngestibleData>> {
        if self.records.is_empty() {
            None
        } else {
            Some(&mut self.records)
        }
    }

    fn get_diagnostics(self) -> Diagnostics {
        todo!() // TODO: Need to implement if this source ends up used again
    }

    fn get_name(&self) -> &'static str {
        "Workshop Database"
    }
}
