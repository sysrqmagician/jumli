use chrono::NaiveDate;

#[derive(Debug)]
pub struct IngestibleData {
    pub identifiers: Vec<ModIdentifier>,
    pub notices: Vec<NoticeRecord>,
}

#[derive(Debug)]
pub struct ModRecordIndex {
    pub identifier: ModIdentifier,
    pub index: usize,
}

#[derive(Debug)]
pub struct ModRecord {
    pub notices: Vec<NoticeRecord>,
}

#[derive(Debug)]
pub enum ModIdentifier {
    PackageId(String),
    WorkshopId(u64),
}

#[derive(Debug)]
pub struct NoticeRecord {
    pub date: Option<NaiveDate>,
    pub notice: Notice,
    pub certainty: Certainty,
    pub source: Source,
}

#[derive(Debug)]
pub enum Notice {
    BadPerformance(Option<String>),
    UseAlternative(String, Option<u64>, Option<String>),
    Bug(String),
    Unstable(Option<String>),
    OutOfDate,

    Miscellaneous(Option<String>),
}

#[derive(Debug)]
pub enum Certainty {
    High,
    Medium,
    Low,

    Inapplicable,
}

#[derive(Debug)]
pub enum Source {
    Local,
    UseThisInsteadDatabase,
    WorkshopDatabase,
}
