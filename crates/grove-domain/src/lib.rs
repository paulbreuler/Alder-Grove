//! Grove Domain — pure types, port traits, and business rules.
//!
//! Zero framework dependencies. Both grove-api and grove-tauri depend on this crate.

pub mod acp;
pub mod agent;
pub mod collaborative_document;
pub mod common;
pub mod error;
pub mod event;
pub mod gate;
pub mod guardrail;
pub mod journey;
pub mod note;
pub mod persona;
pub mod ports;
pub mod repository;
pub mod session;
pub mod snapshot;
pub mod specification;
pub mod step;
pub mod step_specification;
pub mod task;
pub mod workspace;
