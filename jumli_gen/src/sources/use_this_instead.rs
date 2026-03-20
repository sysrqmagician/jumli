use std::{env::temp_dir, fs::File, io::Read};

use flate2::read::GzDecoder;
use git2::FetchOptions;
use serde::Deserialize;
use tracing::info;

use crate::{
    records::types::{Certainty, IngestibleData, ModIdentifier, Notice, NoticeRecord, Source},
    sources::{Diagnostics, RecordSource},
};

pub const REPOSITORY_URL: &str = "https://github.com/emipa606/UseThisInstead";

#[derive(Deserialize)]
pub struct UtiData {
    pub rules: Vec<UtiReplacement>,
    pub version: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UtiReplacement {
    #[serde(deserialize_with = "are_you_kidding_me::string")]
    pub old_package_id: String,
    #[serde(deserialize_with = "are_you_kidding_me::u64")]
    pub old_workshop_id: u64,
    #[serde(deserialize_with = "are_you_kidding_me::string")]
    pub new_package_id: String,
    #[serde(deserialize_with = "are_you_kidding_me::string")]
    pub new_name: String,
    #[serde(deserialize_with = "are_you_kidding_me::u64")]
    pub new_workshop_id: u64,
}

// I shit you not, the UTI replacements json is at least partially hand-written
// json and completely unvalidated.
//
// Someone clearly opened it on Windows since it contains CRLF.
// There are records where the package id is null or an integer (someone pasted the workshop id of the mod there).
// There are also records where the workshop id is an actual integer instead of a string as with
// most records.
mod are_you_kidding_me {
    use serde::{Deserialize, Deserializer};

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum U64OrString {
        Number(u64),
        String(String),
    }

    pub fn u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
    {
        if let Ok(deserialized) = U64OrString::deserialize(deserializer) {
            match deserialized {
                U64OrString::Number(n) => Ok(n),
                U64OrString::String(s) => s.parse().map_err(serde::de::Error::custom),
            }
        } else {
            Ok(0)
        }
    }

    pub fn string<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        if let Ok(deserialized) = U64OrString::deserialize(deserializer) {
            match deserialized {
                U64OrString::Number(n) => Ok(n.to_string()),
                U64OrString::String(s) => Ok(s),
            }
        } else {
            Ok(String::new())
        }
    }
}

pub struct UseThisInstead {
    records: Vec<IngestibleData>,
    diagnostics: Diagnostics,
}

impl UseThisInstead {
    pub fn new() -> Self {
        Self {
            diagnostics: Diagnostics::new(),
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
        let repo = repo_builder.clone(REPOSITORY_URL, &repo_dir)?;
        self.diagnostics.add_git_info(&repo);
        info!("Cloned UTI Repo.");

        let replacements_file_gz = repo_dir.join("replacements.json.gz");
        let mut gz_decoder = GzDecoder::new(
            File::open(&replacements_file_gz)
                .map_err(|e| format!("Failed to open {replacements_file_gz:#?}: {e}"))?,
        );
        let mut replacements_file_plain = String::new();
        gz_decoder
            .read_to_string(&mut replacements_file_plain)
            .map_err(|e| format!("Failed to decode {replacements_file_gz:#?}: {e}"))?;

        let uti_data: UtiData = serde_json::from_str(
            replacements_file_plain
                .strip_prefix("\u{feff}")
                .unwrap_or(&replacements_file_plain),
        )
        .map_err(|e| format!("Failed to deserialize replacements file: {e}"))?;

        for replacement in uti_data.rules {
            let mut identifiers = vec![ModIdentifier::WorkshopId(replacement.old_workshop_id)];
            // Some of our lovely modders do not think unique package names are important
            if replacement.old_package_id != replacement.new_package_id {
                identifiers.push(ModIdentifier::PackageId(replacement.old_package_id));
            }

            self.records.push(IngestibleData {
                identifiers,
                notices: vec![NoticeRecord {
                    notice: Notice::UseAlternative(
                        replacement.new_name,
                        Some(replacement.new_workshop_id),
                        None,
                    ),
                    date: None,
                    certainty: Certainty::Inapplicable,
                    source: Source::UseThisInsteadDatabase,
                    context_url: None,
                    historical: false,
                }],
            });
        }

        info!(
            "Completed UTI processing, yielding {} records.",
            self.records.len(),
        );
        self.diagnostics
            .add_property("raw_records_count", self.records.len().to_string());
        self.diagnostics
            .add_property("uti_version", uti_data.version);

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

    fn get_diagnostics(self) -> Diagnostics {
        self.diagnostics
    }

    fn get_name(&self) -> &'static str {
        "Use This Instead"
    }
}
