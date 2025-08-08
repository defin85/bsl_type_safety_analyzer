//! Модуль конвертеров для унификации преобразования данных
//!
//! Этот модуль централизует всю логику конвертации между различными
//! представлениями типов BSL, устраняя дублирование кода.

pub mod methods;
pub mod properties;
pub mod syntax_db;

pub use methods::MethodConverter;
pub use properties::PropertyConverter;
pub use syntax_db::SyntaxDbConverter;
