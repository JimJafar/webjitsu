//! The MCP server module — exposes WebJitsu's tools over stdio JSON-RPC.

pub mod server;
pub mod tools;

pub use server::run;
#[cfg(unix)]
pub use server::run_pipe;
