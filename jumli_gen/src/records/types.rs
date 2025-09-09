use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct IngestibleData {
    pub identifiers: Vec<ModIdentifier>,
    pub notices: Vec<NoticeRecord>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModRecordIndex {
    pub identifier: ModIdentifier,
    pub index: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModRecord {
    pub notices: Vec<NoticeRecord>,
    pub identifiers: Vec<ModIdentifier>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum ModIdentifier {
    PackageId(String),
    WorkshopId(u64),
}

impl ToString for ModIdentifier {
    fn to_string(&self) -> String {
        match self {
            Self::PackageId(id) => id.clone(),
            Self::WorkshopId(id) => id.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NoticeRecord {
    pub date: Option<NaiveDate>,
    pub notice: Notice,
    pub certainty: Certainty,
    pub source: Source,
    pub context_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Notice {
    BadPerformance(Option<String>),
    UseAlternative(String, Option<u64>, Option<String>),
    Bug(String),
    Unstable(Option<String>),
    OutOfDate,

    Miscellaneous(Option<String>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Certainty {
    High,
    Medium,
    Low,

    Inapplicable,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Source {
    JumliDataset(String),
    UseThisInsteadDatabase,
    WorkshopDatabase,
}
