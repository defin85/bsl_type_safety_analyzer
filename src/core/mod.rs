/*!
# Core Module

Core functionality for the BSL analyzer including error handling,
configuration management, and base analyzer traits.
*/

pub mod errors;
pub mod results;
pub mod position;
pub mod fs_utils;

pub use errors::{AnalysisError, ErrorCollector, ErrorLevel};
pub use results::AnalysisResults;
pub use position::{Position, Span};
pub use fs_utils::read_bsl_file;
