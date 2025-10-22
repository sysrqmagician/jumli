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

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Eq, Hash)]
pub enum ModIdentifier {
    PackageId(String),
    WorkshopId(u64),
}

impl ModIdentifier {
    pub fn is_invalid(&self) -> bool {
        match self {
            Self::PackageId(id) => id.trim().is_empty(),
            Self::WorkshopId(id) => *id == 0,
        }
    }
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
    pub historical: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Notice {
    BadPerformance(Option<String>),
    UseAlternative(String, Option<u64>, Option<String>),
    Bug(String),
    Unstable(Option<String>),
    OutOfDate,

    Miscellaneous(String),
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

impl ToString for Source {
    fn to_string(&self) -> String {
        match self {
            Source::JumliDataset(name) => format!("JuMLi Dataset: {name}"),
            Source::UseThisInsteadDatabase => "Use This Instead Database".into(),
            Source::WorkshopDatabase => "Steam Workshop Database".into(),
        }
    }
}
