//! System info collectors. Each collector lives in its own module and exposes a
//! `pub fn read() -> impl Serialize` returning a value that serializes to a uniform
//! envelope: `{ "state": "ok", ...fields }` or `{ "state": "unavailable" }`.
//!
//! Convention (so the frontend never branches on OS):
//!   #[derive(Serialize)]
//!   #[serde(tag = "state", rename_all = "lowercase")]
//!   pub enum XData { Ok { /* fields */ }, Unavailable }
//!
//! Linux is the real implementation; on other platforms a collector may return
//! `Unavailable`. A failure inside `read()` must map to `Unavailable`, never panic.
//!
//! The integration step adds one `pub mod <name>;` line here per collector and one
//! match arm in `read_tile` (lib.rs).

pub mod memory;
pub mod storage;
pub mod wifi;
pub mod internet;
pub mod volume;
pub mod printers;
pub mod weather;
