use chrono::NaiveDate;
use maud::{PreEscaped, html};
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

impl ModRecord {
    pub fn render_html(&self) -> String {
        html! {
            head {
                link rel="stylesheet" href="/report.css" {}
            }
            body {
                h4 { "Identifiers" }
                ul {
                    @for identifier in &self.identifiers {
                        li { (identifier.to_string()) }
                    }
                }
                h4 { "Notices" }
                div.notices {
                    @for notice in &self.notices {
                        (PreEscaped(notice.render_html()))
                    }
                }
            }
        }
        .into_string()
    }
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

impl NoticeRecord {
    fn render_html(&self) -> String {
        html! {
            div.notice {
                @match &self.notice {
                    Notice::BadPerformance(reason) => {
                        strong { "Bad Performance"}
                        @if let Some(reason) = reason {
                            p { (reason) }
                        } @else {
                            p { "No reason provided." }
                        }
                    },
                    Notice::UseAlternative(alternative_name, workshop_id, reason) => {
                        strong { (format!("Better Alternative Available: {alternative_name}")) }
                        @if let Some(workshop_id) = workshop_id {
                            a href=(format!("https://steamcommunity.com/sharedfiles/filedetails/?id={workshop_id}")) { "Steam Workshop" }
                        }
                        @if let Some(reason) = reason {
                            p { (reason) }
                        } @else {
                            p { "No reason provided." }
                        }
                    },
                    Notice::Bug(description) => {
                        strong { "Current Bug" }
                        p { (description) }
                    },
                    Notice::Unstable(description) => {
                        strong { "Unstable" }
                        @if let Some(description) = description {
                            p { (description) }
                        } @else {
                            p { "No description provided." }
                        }
                    }
                    Notice::OutOfDate => {
                        strong { "Out Of Date"}
                        p { "This mod is not tagged as being compatible with the latest RimWorld version. If you use it anyway, it will likely lead to game-breaking bugs."}
                    }
                    Notice::Miscellaneous(body) => {
                        strong { "Note" }
                        p { (body) }
                    }
                }
                @if let Some(context_url) = &self.context_url {
                    a.context href=(context_url) { "Click here for additional context" }
                }
                p.source { (self.source.to_string()) @if let Some(date) = self.date { " (" (date) ")" } }
            }
        }
        .into_string()
    }
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
