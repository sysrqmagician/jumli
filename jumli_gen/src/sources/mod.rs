use std::error::Error;

use chrono::{TimeZone, Utc};
use git2::Repository;

use crate::records::types::IngestibleData;

pub mod jumli_data;
pub mod use_this_instead;
pub mod workshop_database;

/// Diagnostics to be shown on /status.html. Note all of this information will be public.
#[derive(Default)]
pub struct Diagnostics {
    /// Properties to be displayed in a table
    properties: Option<Vec<(String, String)>>,
    /// Non-fatal errors and misc. info
    log_lines: Option<Vec<String>>,
}

impl Diagnostics {
    pub fn new() -> Self {
        Self {
            properties: None,
            log_lines: None,
        }
    }

    pub fn log(&mut self, message: impl Into<String>) {
        self.log_lines.get_or_insert_default().push(message.into());
    }

    pub fn add_property(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.properties
            .get_or_insert_default()
            .push((key.into(), value.into()));
    }

    pub fn get_properties(&self) -> Option<&Vec<(String, String)>> {
        self.properties.as_ref()
    }

    pub fn get_logs(&self) -> Option<&Vec<String>> {
        self.log_lines.as_ref()
    }

    pub fn add_git_info(&mut self, repo: &Repository) {
        let latest_commit = repo.head().and_then(|x| x.peel_to_commit());
        match latest_commit {
            Ok(latest_commit) => {
                self.add_property("git_commit", latest_commit.id().to_string());
                self.add_property(
                    "git_commit_summary",
                    latest_commit.summary().unwrap_or("Failed to retrieve."),
                );

                let timestamp = Utc
                    .timestamp_opt(
                        latest_commit.time().seconds()
                            + i64::from(latest_commit.time().offset_minutes() * 60),
                        0,
                    )
                    .latest()
                    .and_then(|x| Some(x.to_rfc3339()));

                self.add_property(
                    "git_commit_time",
                    timestamp.unwrap_or("Failed to retrieve.".into()),
                );
            }
            Err(e) => {
                self.log(format!(
                    "Failed to retrieve git commit information for diagnostics: {e}"
                ));
            }
        }
    }
}

pub trait RecordSource {
    /// Fetch raw data and process it for later retrieval as IngestibleData using get_records
    fn fetch(&mut self) -> impl Future<Output = Result<(), Box<dyn Error>>>;
    /// Get successfully parsed data
    fn get_records(&mut self) -> Option<&mut Vec<IngestibleData>>;
    /// Get info for status.html
    fn get_diagnostics(self) -> Diagnostics;
    /// Get Source name
    fn get_name(&self) -> &'static str;
}
