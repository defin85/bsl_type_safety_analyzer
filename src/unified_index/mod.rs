pub mod entity;
pub mod index;
pub mod platform_cache;
pub mod project_cache;
pub mod xml_parser;
pub mod builder;
pub mod configuration_watcher;

pub use entity::{
    BslEntity, BslEntityId, BslEntityType, BslEntityKind, BslEntitySource,
    BslApplicationMode, BslContext, BslMethod, BslParameter, BslProperty, BslEvent,
    BslInterface, BslConstraints, BslRelationships, BslLifecycle
};

pub use index::{UnifiedBslIndex, BslLanguagePreference, IncrementalUpdateResult, IndexPerformanceStats};
pub use platform_cache::PlatformDocsCache;
pub use project_cache::ProjectIndexCache;
pub use xml_parser::ConfigurationXmlParser;
pub use builder::UnifiedIndexBuilder;
pub use configuration_watcher::{ConfigurationWatcher, ChangeImpact};