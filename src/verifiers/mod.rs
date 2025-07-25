/*!
# Verifiers Module

Components for verifying BSL code correctness, method calls, and type compatibility.
*/

pub mod method_verifier;

#[cfg(test)]
mod method_verifier_integration_test;

pub use method_verifier::MethodVerifier;
