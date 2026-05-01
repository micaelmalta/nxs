//! .nxb → human/JSON report. Reuses the existing `decoder::decode` for the
//! structural walk; formatting is rendered as text (default) or JSON
//! (--json), with the JSON shape frozen by inspect_json_schema in the spec.
//!
//! Stub — see plan steps `impl: inspect text renderer` and `impl: inspect json renderer`.

use super::{InspectArgs, InspectReport};
use crate::error::Result;
use std::io::Write;

pub fn render_text<W: Write>(_writer: W, _args: &InspectArgs) -> Result<InspectReport> {
    unimplemented!("inspect::render_text")
}

pub fn render_json<W: Write>(_writer: W, _args: &InspectArgs) -> Result<InspectReport> {
    unimplemented!("inspect::render_json")
}
