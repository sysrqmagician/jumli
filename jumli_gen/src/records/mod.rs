use std::{collections::HashMap, error::Error};

use tracing::info;

use crate::{
    records::types::{IngestibleData, ModRecord},
    sources::RecordSource,
};

pub mod types;

pub struct DatabaseBuilder {
    raw_records: Vec<IngestibleData>,
    reported_errors: Vec<String>,
}

pub struct Database {
    pub records: Vec<ModRecord>,
    pub indices: HashMap<String, usize>,
}

impl DatabaseBuilder {
    pub fn new() -> Self {
        Self {
            raw_records: Vec::new(),
            reported_errors: Vec::new(),
        }
    }

    pub async fn ingest_from(
        &mut self,
        mut source: impl RecordSource,
    ) -> Result<(), Box<dyn Error>> {
        source.fetch().await?;

        if let Some(records) = source.get_records() {
            self.raw_records.append(records);
        }

        if let Some(errors) = source.get_errors() {
            self.reported_errors.append(errors);
        }
        Ok(())
    }

    pub async fn finalize(mut self) -> Database {
        info!(
            "Finalizing records from {} raw entries.",
            self.raw_records.len()
        );

        let mut final_records = Vec::new();
        let mut indices = HashMap::new();

        while let Some(start_record) = self.raw_records.get(0) {
            let current_index = final_records.len().saturating_sub(1);

            let mut identifiers = start_record.identifiers.clone();
            let mut notices = start_record.notices.clone();
            self.raw_records.remove(0);

            self.raw_records.retain_mut(|current| {
                for known in &identifiers {
                    if current.identifiers.contains(&known) {
                        notices.append(&mut current.notices);
                        for id in &current.identifiers {
                            if !identifiers.contains(id) {
                                identifiers.push(id.clone());
                            }
                        }
                        return false;
                    }
                }

                true
            });

            for identifier in &identifiers {
                indices.insert(identifier.to_string(), current_index);
            }

            final_records.push(ModRecord {
                notices,
                identifiers,
            });
        }

        info!(
            "Finalized database with {} unique records.",
            final_records.len()
        );

        Database {
            records: final_records,
            indices,
        }
    }
}
