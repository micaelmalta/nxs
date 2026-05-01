//! Two-pass streaming sigil inference.
//!
//! Pass 1: iterate source records, keep a per-key lattice state. Pass 2 is
//! driven by the caller (json_in/csv_in/xml_in) using the frozen schema.
//!
//! Stubs only — see plan step `impl: inference lattice`.

use super::{ConflictPolicy, InferredSchema};
use crate::error::Result;

/// Per-key state maintained during pass 1.
#[derive(Debug, Default, Clone)]
pub struct KeyState {
    pub seen_int: bool,
    pub seen_float: bool,
    pub seen_bool: bool,
    pub seen_time: bool,
    pub seen_binary_hex: bool,
    pub seen_string: bool,
    pub seen_null: bool,
    pub total_records_seen_in: usize,
}

impl KeyState {
    /// Classify a raw string/JSON-value observation and merge into `self`.
    pub fn observe(&mut self, _raw: &str) {
        unimplemented!("KeyState::observe")
    }

    /// Collapse accumulated flags to a single sigil (byte) per plan priority.
    pub fn resolve_sigil(&self, _policy: ConflictPolicy) -> Result<u8> {
        unimplemented!("KeyState::resolve_sigil")
    }
}

/// Merge a per-record set of observations into the accumulator. Stub.
pub fn merge(_acc: &mut InferredSchema, _record: &[(String, String)]) {
    unimplemented!("merge")
}

/// Freeze the accumulator into a schema ready to drive `NxsWriter`. Stub.
pub fn finalize(_acc: InferredSchema, _policy: ConflictPolicy) -> Result<InferredSchema> {
    unimplemented!("finalize")
}
