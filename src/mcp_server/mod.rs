/// <module>
///   <name>mcp_server</name>
///   <purpose>Model Context Protocol server implementation for BSL Type Safety Analyzer</purpose>
///   <description>
///     Provides MCP interface for LLMs to query BSL type system,
///     validate code, and get method information.
///   </description>
/// </module>

mod analyzer;
mod tools;
mod types;

pub use analyzer::BslTypeAnalyzer;
pub use tools::{FindTypeResult, MethodInfo, TypeCompatibilityResult, ValidationResult};
pub use types::{BslLanguagePreference, McpError, McpResult};