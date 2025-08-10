//! Simple type system for arena semantic analyzer.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimpleType {
    Number,
    String,
    Boolean,
    Null,
    Undefined,
    Object(String),
    Unknown,
}

impl SimpleType {
    pub fn literal(text: &str) -> Self {
        if text.eq_ignore_ascii_case("true") || text.eq_ignore_ascii_case("false") { Self::Boolean }
        else if text=="Null" { Self::Null }
        else if text=="Undefined" { Self::Undefined }
        else if text.chars().all(|c| c.is_ascii_digit()) { Self::Number }
        else { Self::String }
    }
}

pub fn binary_result(lhs: &SimpleType, op: &str, rhs: &SimpleType) -> SimpleType {
    match op {
        "Add" => { // условно: +
            if matches!((lhs,rhs), (SimpleType::Number, SimpleType::Number)) { SimpleType::Number } else { SimpleType::Unknown }
        }
        _ => SimpleType::Unknown
    }
}

pub fn unary_result(op: &str, inner: &SimpleType) -> SimpleType {
    match op {
        "Not" => SimpleType::Boolean,
        _ => inner.clone(),
    }
}
