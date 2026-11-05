//! System services shared by several modules.

pub mod file_index;
pub mod icons;
pub mod module_host;
pub mod paths;
pub mod programs;
#[cfg(windows)]
pub mod shell;
pub mod text;
