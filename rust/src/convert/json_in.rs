//! JSON → .nxb. Streaming: uses `serde_json::Deserializer::from_reader`
//! with `StreamDeserializer` over the root array.
//!
//! Stub — see plan steps `impl: json streaming pass-1` and `impl: json pass-2 emit`.

use super::{ImportArgs, ImportReport, InferredSchema};
use crate::error::Result;
use std::io::{Read, Write};

/// Pass 1 — consume a reader, produce an `InferredSchema`. Stub.
pub fn infer_schema<R: Read>(_reader: R, _args: &ImportArgs) -> Result<InferredSchema> {
    unimplemented!("json_in::infer_schema")
}

/// Pass 2 — consume the reader again, emit .nxb bytes via NxsWriter. Stub.
pub fn emit<R: Read, W: Write>(
    _reader: R,
    _writer: W,
    _schema: &InferredSchema,
    _args: &ImportArgs,
) -> Result<ImportReport> {
    unimplemented!("json_in::emit")
}
