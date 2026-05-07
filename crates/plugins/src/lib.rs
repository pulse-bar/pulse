pub mod error;
pub mod manifest;
pub mod plugin;
pub mod registry;
pub mod status;

pub use error::{PluginError, PluginResult};
pub use manifest::{
    AuthMethodKind, PluginCapability, PluginCategory, PluginConnectStyle, PluginManifest,
};
pub use plugin::{Plugin, PluginInstanceSummary};
pub use registry::PluginRegistry;
pub use status::{InstanceState, InstanceStatus, PluginState, PluginStatus};
