//! The excellent FBX library.
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

#[cfg(feature = "dom")]
pub mod dom;
pub mod low;
pub mod pull_parser;
