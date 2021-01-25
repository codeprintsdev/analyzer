use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Year {
    pub year: String,
    pub total: usize,
    pub range: Range,
}

pub type Years = Vec<Year>;
pub type Contributions = Vec<Contribution>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Timeline {
    pub years: Years,
    pub contributions: Contributions,
}

#[derive(Default, Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Range {
    pub start: String,
    pub end: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Contribution {
    pub date: String,
    pub count: usize,
    pub color: String,
    pub intensity: usize,
}
