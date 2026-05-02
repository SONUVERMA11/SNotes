//! S Notes Plugin API — WASM sandboxed plugins via Wasmtime

mod host;
mod api;

pub use host::PluginHost;
pub use api::PluginApi;
