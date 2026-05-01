//! Converter suite — JSON/CSV/XML ⇄ .nxb.
//!
//! Spec: ../../context/data/2026-04-30-converter-suite-spec.yaml
//! Plan: ../../context/plans/2026-04-30-converter-suite.md
//!
//! Module map:
//!   infer     — two-pass streaming sigil inference shared by all import sources
//!   json_in   — JSON → NxsWriter
//!   csv_in    — CSV  → NxsWriter
//!   xml_in    — XML  → NxsWriter
//!   json_out  — .nxb → JSON (streaming, optional ndjson/pretty)
//!   csv_out   — .nxb → CSV
//!   inspect   — .nxb → human/JSON report
//!
//! All public entry points below are stubs returning `Unimplemented` until the
//! implementation steps in the plan are executed in TDD order.

use crate::error::Result;

pub mod csv_in;
pub mod csv_out;
pub mod infer;
pub mod inspect;
pub mod json_in;
pub mod json_out;
pub mod xml_in;

/// Common options parsed once by each binary's `main()`.
#[derive(Debug, Default, Clone)]
pub struct CommonOpts {
    pub input_path: Option<std::path::PathBuf>, // None → stdin
    pub output_path: Option<std::path::PathBuf>, // None → stdout
    pub verify_roundtrip: bool,
}

/// Thin enum representing the user's `--on-conflict` choice.
#[derive(Debug, Clone, Copy, Default)]
pub enum ConflictPolicy {
    #[default]
    Error,
    CoerceString,
    FirstWins,
}

/// Return type of the inference pass: each key's chosen sigil plus whether it
/// is optional (absent in ≥1 record).
#[derive(Debug, Default)]
pub struct InferredSchema {
    pub keys: Vec<InferredKey>,
}

#[derive(Debug)]
pub struct InferredKey {
    pub name: String,
    pub sigil: u8,
    pub optional: bool,
    pub list_of: Option<u8>,
}

/// Top-level driver for nxs-import (dispatched on `--from`). Stub.
pub fn run_import(_args: &ImportArgs) -> Result<ImportReport> {
    unimplemented!("run_import — see plan step `impl: nxs-import JSON dispatch in convert::run_import`")
}

/// Top-level driver for nxs-export (dispatched on `--to`). Stub.
pub fn run_export(_args: &ExportArgs) -> Result<ExportReport> {
    unimplemented!("run_export — see plan step `impl: nxs-export JSON dispatch`")
}

/// Top-level driver for nxs-inspect. Stub.
pub fn run_inspect(_args: &InspectArgs) -> Result<InspectReport> {
    unimplemented!("run_inspect — see plan step `impl: nxs-inspect CLI`")
}

#[derive(Debug, Default)]
pub struct ImportArgs {
    pub common: CommonOpts,
    pub from: ImportFormat,
    pub schema_hint: Option<std::path::PathBuf>,
    pub conflict: ConflictPolicy,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum ImportFormat {
    #[default]
    Json,
    Csv,
    Xml,
}

#[derive(Debug, Default)]
pub struct ImportReport {
    pub records_written: usize,
    pub output_bytes: usize,
}

#[derive(Debug, Default)]
pub struct ExportArgs {
    pub common: CommonOpts,
    pub to: ExportFormat,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum ExportFormat {
    #[default]
    Json,
    Csv,
}

#[derive(Debug, Default)]
pub struct ExportReport {
    pub records_read: usize,
    pub output_bytes: usize,
}

#[derive(Debug, Default)]
pub struct InspectArgs {
    pub common: CommonOpts,
    pub json_output: bool,
    pub records_to_show: Option<usize>, // None → all
    pub verify_hash: bool,
}

#[derive(Debug, Default)]
pub struct InspectReport {
    pub dict_hash_ok: Option<bool>,
    pub record_count: usize,
}
