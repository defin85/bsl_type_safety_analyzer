/*!
# Core Module

Core functionality for the BSL analyzer including error handling,
configuration management, and base analyzer traits.
*/

pub mod errors;
pub mod results;

pub use errors::{AnalysisError, ErrorCollector, ErrorLevel};
pub use results::AnalysisResults;
