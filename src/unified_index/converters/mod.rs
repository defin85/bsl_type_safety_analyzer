//! Модуль конвертеров для унификации преобразования данных
//! 
//! Этот модуль централизует всю логику конвертации между различными
//! представлениями типов BSL, устраняя дублирование кода.

pub mod syntax_db;
pub mod methods;
pub mod properties;

pub use syntax_db::SyntaxDbConverter;
pub use methods::MethodConverter;
pub use properties::PropertyConverter;