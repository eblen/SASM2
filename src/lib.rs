// Top-level public modules
pub mod assemble;
pub mod config;
pub mod disassemble;

// Internal modules used by assemble and config
mod data;
mod output;
mod syntax;
mod zpm;

// Value returned to user
pub use output::Code;

// Simplify the interface for users
pub use assemble::assemble;
pub use config::Config;
pub use disassemble::disassemble;
