use std::fmt::Display;

use chrono::{DateTime, Utc};
use quick_xml::SeError;
use serde::Serialize;

/// A sitemap containing a collection of URLs.
#[derive(Serialize, Clone)]
#[serde(rename = "urlset")]
pub struct Sitemap {
    #[serde(rename = "url")]
    urlset: Vec<SitemapUrl>,
}

/// A single URL entry in a sitemap.
///
/// Contains the URL and optional metadata like last modification date,
/// change frequency, and priority.
#[derive(Serialize, Default, Clone)]
pub struct SitemapUrl {
    loc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    lastmod: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    changefreq: Option<ChangeFreq>,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<f32>,
}

/// Frequency at which a URL is expected to change.
#[derive(Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ChangeFreq {
    /// The document changes every time it is accessed.
    Always,
    /// The document changes hourly.
    Hourly,
    /// The document changes daily.
    Daily,
    /// The document changes weekly.
    Weekly,
    /// The document changes monthly.
    Monthly,
    /// The document changes yearly.
    Yearly,
    /// The document never changes.
    Never,
}

impl SitemapUrl {
    /// Creates a new `SitemapUrl` with the given URL.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            loc: url.into(),
            ..Default::default()
        }
    }

    /// Convenience wrapper for format macro and new
    pub fn from_base(url_base: impl Display, path: impl Display) -> Self {
        Self {
            loc: format!("{url_base}/{path}"),
            ..Default::default()
        }
    }

    /// Sets the last modification date of the URL.
    pub fn last_modified(mut self, date_time: DateTime<Utc>) -> Self {
        self.lastmod = Some(date_time.to_rfc3339());

        self
    }

    /// Sets the last modification date of the URL to the current time.
    pub fn last_modified_now(self) -> Self {
        self.last_modified(Utc::now())
    }

    /// Sets the expected change frequency of the URL.
    pub fn change_frequency(mut self, change_frequency: ChangeFreq) -> Self {
        self.changefreq = Some(change_frequency);

        self
    }

    /// Sets the priority of the URL relative to other URLs in the sitemap.
    ///
    /// The value is clamped between 0.0 and 1.0.
    pub fn priority(mut self, priority: f32) -> Self {
        self.priority = Some(priority.clamp(0.0, 1.0));

        self
    }
}

impl Sitemap {
    /// Creates a new empty `Sitemap`.
    pub fn new() -> Self {
        Self { urlset: Vec::new() }
    }

    /// Adds a single URL to the sitemap.
    pub fn add_url(&mut self, url: SitemapUrl) {
        self.urlset.push(url);
    }

    /// Serializes the sitemap into an XML string.
    ///
    /// Returns an error if XML serialization fails.
    pub fn to_string(&self) -> Result<String, SeError> {
        Ok(format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>{}"#,
            quick_xml::se::to_string(&self)?
        ))
    }

    /// Adds multiple URLs to the sitemap from an iterator.
    pub fn add_urls(&mut self, iter: impl IntoIterator<Item = SitemapUrl>) {
        self.urlset.extend(iter);
    }
}
