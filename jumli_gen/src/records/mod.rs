use std::{
    collections::{HashMap, HashSet},
    error::Error,
};

use tracing::info;

use crate::{
    records::types::{IngestibleData, ModIdentifier, ModRecord},
    sources::{Diagnostics, RecordSource},
};

pub mod types;

pub struct DatabaseBuilder {
    raw_records: Vec<IngestibleData>,
    named_diagnostics: Vec<(String, Diagnostics)>,
}

pub struct Database {
    pub records: Vec<ModRecord>,
    pub indices: HashMap<String, usize>,
    pub named_diagnostics: Vec<(String, Diagnostics)>,
}

struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        UnionFind {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    fn union(&mut self, x: usize, y: usize) {
        let root_x = self.find(x);
        let root_y = self.find(y);
        if root_x == root_y {
            return;
        }

        if self.rank[root_x] < self.rank[root_y] {
            self.parent[root_x] = root_y;
        } else if self.rank[root_x] > self.rank[root_y] {
            self.parent[root_y] = root_x;
        } else {
            self.parent[root_y] = root_x;
            self.rank[root_x] += 1;
        }
    }
}

impl DatabaseBuilder {
    pub fn new() -> Self {
        Self {
            raw_records: Vec::new(),
            named_diagnostics: Vec::new(),
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

        self.named_diagnostics.push((
            format!("Source: {}", source.get_name()),
            std::mem::take(&mut source.get_diagnostics()),
        ));
        Ok(())
    }

    pub async fn finalize(mut self) -> Database {
        info!(
            "Finalizing records from {} raw entries.",
            self.raw_records.len()
        );

        let mut ident_to_records: HashMap<ModIdentifier, Vec<usize>> = HashMap::new();
        for (idx, record) in self.raw_records.iter().enumerate() {
            for identifier in &record.identifiers {
                // External sources occasionally include malformed identifiers, leading to bad consolidation
                if identifier.is_invalid() {
                    continue;
                }

                ident_to_records
                    .entry(identifier.clone())
                    .or_default()
                    .push(idx);
            }
        }

        let mut union_find = UnionFind::new(self.raw_records.len());
        for indices in ident_to_records.into_values() {
            if indices.len() < 2 {
                continue;
            }
            let first = indices[0];
            for &idx in &indices[1..] {
                union_find.union(first, idx);
            }
        }

        let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();
        for i in 0..self.raw_records.len() {
            let root = union_find.find(i);
            groups.entry(root).or_default().push(i);
        }

        let mut final_records = Vec::new();
        let mut final_indices = HashMap::new();
        for indices in groups.into_values() {
            let mut identifiers = HashSet::new();
            let mut notices = Vec::new();
            for idx in indices {
                identifiers.extend(self.raw_records[idx].identifiers.iter().cloned());
                notices.append(&mut self.raw_records[idx].notices);
            }

            final_records.push(ModRecord {
                notices,
                identifiers: identifiers.iter().cloned().collect(),
            });

            for identifier in identifiers.into_iter() {
                final_indices.insert(identifier.to_string(), final_records.len() - 1);
            }
        }

        info!(
            "Finalized database with {} unique entries.",
            final_records.len()
        );

        Database {
            records: final_records,
            indices: final_indices,
            named_diagnostics: self.named_diagnostics,
        }
    }
}
