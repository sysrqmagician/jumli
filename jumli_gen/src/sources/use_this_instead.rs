use std::{env::temp_dir, fs::File, io::BufReader};

use git2::FetchOptions;
use serde::Deserialize;
use tokio::task::JoinSet;
use tracing::info;

use crate::{
    records::types::{Certainty, IngestibleData, ModIdentifier, Notice, NoticeRecord, Source},
    sources::RecordSource,
};

pub const REPOSITORY_URL: &'static str = "https://github.com/emipa606/UseThisInstead";

#[derive(Deserialize)]
pub struct UtiDocument {
    #[serde(rename = "ModId")]
    pub mod_id: String,
    #[serde(rename = "ModName")]
    pub mod_name: String,
    #[serde(rename = "Author")]
    pub author: String,
    #[serde(rename = "SteamId")]
    pub steam_id: u64,
    #[serde(rename = "Versions")]
    pub versions: String,
    #[serde(rename = "ReplacementModId")]
    pub replacement_mod_id: String,
    #[serde(rename = "ReplacementName")]
    pub replacement_name: String,
    #[serde(rename = "ReplacementAuthor")]
    pub replacement_author: String,
    #[serde(rename = "ReplacementSteamId")]
    pub replacement_steam_id: u64,
    #[serde(rename = "ReplacementVersions")]
    pub replacement_versions: String,
}

pub struct UseThisInstead {
    records: Vec<IngestibleData>,
    errors: Vec<String>,
}

impl UseThisInstead {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            records: Vec::new(),
        }
    }
}

impl RecordSource for UseThisInstead {
    async fn fetch(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut repo_fo = FetchOptions::new();
        repo_fo.depth(1);

        let mut repo_builder = git2::build::RepoBuilder::new();
        repo_builder.fetch_options(repo_fo);

        let mut repo_dir = temp_dir();
        repo_dir.push("jumli_uti");

        info!("Cloning UTI Repo to {repo_dir:?}.");
        let _repo = repo_builder.clone(REPOSITORY_URL, &repo_dir)?;
        info!("Cloned UTI Repo.");

        let mut replacements_dir = repo_dir.clone();
        replacements_dir.push("Replacements");

        let mut handles: JoinSet<Result<IngestibleData, String>> = JoinSet::new();

        let mut read_dir = std::fs::read_dir(&replacements_dir)?;
        while let Some(Ok(entry)) = read_dir.next() {
            if entry.path().extension().and_then(|e| e.to_str()) != Some("xml") {
                continue;
            }

            handles.spawn(async move {
                let doc: UtiDocument =
                    quick_xml::de::from_reader(BufReader::new(File::open(entry.path()).map_err(
                        |e| format!("Failed to parse '{}': {e}", entry.path().display()),
                    )?))
                    .map_err(|e| format!("Failed to parse '{}': {e}", entry.path().display()))?;

                Ok(IngestibleData {
                    identifiers: vec![
                        ModIdentifier::WorkshopId(doc.steam_id),
                        ModIdentifier::PackageId(doc.mod_id),
                    ],
                    notices: vec![NoticeRecord {
                        notice: Notice::UseAlternative(
                            doc.replacement_mod_id,
                            Some(doc.replacement_steam_id),
                            None,
                        ),
                        date: None,
                        certainty: Certainty::Inapplicable,
                        source: Source::UseThisInsteadDatabase,
                        context_url: None,
                    }],
                })
            });
        }

        while let Some(Ok(result)) = handles.join_next().await {
            match result {
                Ok(record) => self.records.push(record),
                Err(error) => self.errors.push(error),
            }
        }

        info!(
            "Completed UTI processing, yielding {} records and {} errors.",
            self.records.len(),
            self.errors.len()
        );

        info!("Deleting UTI Repo {repo_dir:?}.",);

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

    fn get_errors(&mut self) -> Option<&mut Vec<String>> {
        if self.errors.is_empty() {
            None
        } else {
            Some(&mut self.errors)
        }
    }
}
