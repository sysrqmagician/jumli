use std::error::Error;

use crate::records::types::IngestibleData;

pub mod jumli_data;
pub mod use_this_instead;
pub mod workshop_database;

pub trait RecordSource {
    /// Fetch raw data and process it for later retrieval as IngestibleData using get_records
    fn fetch(&mut self) -> impl Future<Output = Result<(), Box<dyn Error>>>;
    /// Get successfully parsed data
    fn get_records(&mut self) -> Option<&mut Vec<IngestibleData>>;
    /// Get non-fatal errors encountered while parsing individual parts of the dataset
    fn get_errors(&mut self) -> Option<&mut Vec<String>>;
}
