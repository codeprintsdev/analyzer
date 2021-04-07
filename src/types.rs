use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Day {
    pub commits: usize,
    pub date: NaiveDate,
}

#[derive(Debug, Default, Clone, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Year {
    pub year: String,
    pub total: usize,
    pub range: Range,
}

pub type Years = Vec<Year>;
pub type Contributions = Vec<Contribution>;

/// A timeline represents a codeprints.json file's contents
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Timeline {
    /// Years belonging to the timeline
    pub years: Years,
    /// Contributions belonging to the timeline
    pub contributions: Contributions,
}

#[derive(Default, Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Range {
    pub start: String,
    pub end: String,
}

#[derive(Default, Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Contribution {
    pub date: String,
    pub count: usize,
    pub color: String,
    pub intensity: usize,
}
