//! A library for creating data file from git repositories for use on
//! codeprints.dev
#![warn(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

/// Git helper functions
pub mod git;

mod merge;
mod parser;
mod quartiles;
mod types;

pub use crate::merge::Merger;
pub use crate::parser::Parser;
pub use crate::types::Timeline;
