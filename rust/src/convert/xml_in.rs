//! XML → .nxb. Streaming: uses `quick-xml`'s event reader; a record is the
//! element named by `--xml-record-tag`. Attributes become fields per
//! `--xml-attrs`.
//!
//! Stub — see plan steps `impl: xml streaming pass-1` and `impl: xml pass-2 emit`.

use super::{ImportArgs, ImportReport, InferredSchema};
use crate::error::Result;
use std::io::{Read, Write};

pub fn infer_schema<R: Read>(_reader: R, _args: &ImportArgs) -> Result<InferredSchema> {
    unimplemented!("xml_in::infer_schema")
}

pub fn emit<R: Read, W: Write>(
    _reader: R,
    _writer: W,
    _schema: &InferredSchema,
    _args: &ImportArgs,
) -> Result<ImportReport> {
    unimplemented!("xml_in::emit")
}
