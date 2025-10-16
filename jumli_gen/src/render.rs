use chrono::Utc;
use maud::{PreEscaped, html};

use crate::records::{
    Database,
    types::{ModRecord, Notice, NoticeRecord},
};

pub trait RenderHtml {
    fn render_html(&self) -> String;
}

impl RenderHtml for NoticeRecord {
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
                            a class="workshop-alternative" href=(format!("https://steamcommunity.com/sharedfiles/filedetails/?id={workshop_id}")) { "Steam Workshop" }
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

impl RenderHtml for ModRecord {
    fn render_html(&self) -> String {
        html! {
            head {
                link rel="stylesheet" href="/report.css" {}
                base target="_blank" {}
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

pub fn frame_html(destination: String) -> String {
    html! {
        head {
            base target="_blank" {}
        }
        body {
            iframe src=(destination) width="100%" height="100%" frameBorder="0" { }
        }
    }
    .into_string()
}

pub fn render_diagnostics(db: &Database) -> String {
    html! {
        head {
            link rel="stylesheet" href="/index.css" {}
        }
        body {
            h1 { "JuMLi: Diagnostics" }
            nav {
                a href="/" { "Home" }
            }
            main {
                p { "JuMLi was last built around " code { (Utc::now().to_rfc3339()) } "." }
                p { "Database currently contains " (db.records.len()) " consolidated mod records."}
                @for (name, diag) in &db.named_diagnostics {
                    h3 { (name) }
                    @if let Some(props) = diag.get_properties() {
                        table class="diagnostics" {
                            @for (key, value) in props {
                               tr {
                                   th { (key) }
                                   td { (value) }
                               }
                            }
                        }
                    }
                    @if let Some(logs) = diag.get_logs() {
                        code {
                            (logs.join("\n"))
                        }
                    }
                }
            }
        }
    }
    .into_string()
}
