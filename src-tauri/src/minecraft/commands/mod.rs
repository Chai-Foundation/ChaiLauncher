pub mod versions;
pub mod java;
pub mod instances;
pub mod external;
pub mod settings;

// Re-export all commands for backwards compatibility
pub use versions::*;
pub use java::*;
pub use instances::*;
pub use external::*;
pub use settings::*;

// Re-export types that may be used by other modules
pub use external::ExternalInstance;