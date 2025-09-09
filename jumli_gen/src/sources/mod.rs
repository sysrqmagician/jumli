use std::error::Error;

use crate::records::IngestibleData;

pub mod local;
pub mod use_this_instead;
pub mod workshop_database;

pub trait RecordSource {
    /// Fetch raw data and process it for later retrieval as IngestibleData using get_records
    fn fetch(&mut self) -> impl Future<Output = Result<(), Box<dyn Error>>>;
    /// Get successfully parsed data
    fn get_records(&self) -> Option<&Vec<IngestibleData>>;
    /// Get non-fatal errors encountered while parsing individual parts of the dataset
    fn get_errors(&self) -> Option<&Vec<String>>;
}
