use std::{cell::LazyCell, env::temp_dir, fs::File, io::BufReader};

use chrono::NaiveDate;
use git2::FetchOptions;
use ron::extensions::Extensions;
use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;
use tracing::info;

use crate::{
    records::types::{Certainty, IngestibleData, ModIdentifier, Notice, NoticeRecord, Source},
    sources::{Diagnostics, RecordSource},
};

pub const REPOSITORY_URL: &'static str = "https://github.com/sysrqmagician/jumli";
pub const RON_OPTIONS: LazyCell<ron::Options> =
    LazyCell::new(|| ron::Options::default().with_default_extension(Extensions::IMPLICIT_SOME));

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "Dataset")]
struct DatasetFile {
    name: String,
    description: String,
    records: Vec<DatasetFileRecord>,
}

#[derive(Serialize, Deserialize, Debug)]
struct DatasetFileRecord {
    identifiers: Vec<ModIdentifier>,
    notices: Vec<LocalNotice>,
}

#[derive(Serialize, Deserialize, Debug)]
struct LocalNotice {
    pub date: Option<NaiveDate>,
    pub notice: Notice,
    pub certainty: Certainty,
    pub context_url: Option<String>,
}

impl Into<Vec<IngestibleData>> for DatasetFile {
    fn into(self) -> Vec<IngestibleData> {
        self.records
            .into_iter()
            .map(|entry| IngestibleData {
                identifiers: entry.identifiers,
                notices: entry
                    .notices
                    .into_iter()
                    .map(|local| NoticeRecord {
                        certainty: local.certainty,
                        context_url: local.context_url,
                        date: local.date,
                        notice: local.notice,
                        source: Source::JumliDataset(self.name.clone()),
                    })
                    .collect(),
            })
            .collect()
    }
}

pub struct JumliData {
    records: Vec<IngestibleData>,
    diagnostics: Diagnostics,
}

impl JumliData {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            diagnostics: Diagnostics::new(),
        }
    }
}

impl RecordSource for JumliData {
    async fn fetch(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut repo_fo = FetchOptions::new();
        repo_fo.depth(1);

        let mut repo_builder = git2::build::RepoBuilder::new();
        repo_builder.fetch_options(repo_fo);

        let mut repo_dir = temp_dir();
        repo_dir.push("jumli_repo");

        info!("Cloning JuMLi Repo to {repo_dir:?}.");
        let repo = repo_builder.clone(REPOSITORY_URL, &repo_dir)?;
        self.diagnostics.add_git_info(&repo);
        info!("Cloned JuMLi Repo.");

        let mut records_dir = repo_dir.clone();
        records_dir.push("jumli_data/records");

        let mut handles: JoinSet<Result<Vec<IngestibleData>, String>> = JoinSet::new();

        let mut read_dir = std::fs::read_dir(&records_dir)?;
        while let Some(Ok(entry)) = read_dir.next() {
            handles.spawn(async move {
                let reader = BufReader::new(
                    File::open(entry.path())
                        .map_err(|e| format!("Unable to read dataset {:?}: {e}", entry.path()))?,
                );

                let dataset: DatasetFile = RON_OPTIONS
                    .from_reader(reader)
                    .map_err(|e| format!("Unable to parse dataset {:?}: {e}", entry.path()))?;

                Ok(dataset.into())
            });
        }

        while let Some(Ok(result)) = handles.join_next().await {
            match result {
                Ok(mut record) => self.records.append(&mut record),
                Err(error) => self.diagnostics.log(error),
            }
        }

        info!(
            "Completed JuMLi processing, yielding {} records.",
            self.records.len()
        );

        info!("Deleting JuMLi Repo {repo_dir:?}.",);

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
        self.diagnostics
    }

    fn get_name(&self) -> &'static str {
        "JuMLi Data"
    }
}
