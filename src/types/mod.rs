pub mod config;
pub mod mapping;

pub use config::{on_off, validate_dpi, MouseConfig, PollingRate, SpdtMode};
pub use mapping::{MappingType, MediaKey, MouseKey, ScrollDirection};
