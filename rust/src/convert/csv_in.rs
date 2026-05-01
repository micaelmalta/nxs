//! CSV → .nxb. Streaming: uses the `csv` crate's row iterator; headers become
//! NXS keys unless `--csv-no-header` is set (then `col_0`, `col_1`, …).
//!
//! Stub — see plan steps `impl: csv streaming pass-1` and `impl: csv pass-2 emit`.

use super::{ImportArgs, ImportReport, InferredSchema};
use crate::error::Result;
use std::io::{Read, Write};

pub fn infer_schema<R: Read>(_reader: R, _args: &ImportArgs) -> Result<InferredSchema> {
    unimplemented!("csv_in::infer_schema")
}

pub fn emit<R: Read, W: Write>(
    _reader: R,
    _writer: W,
    _schema: &InferredSchema,
    _args: &ImportArgs,
) -> Result<ImportReport> {
    unimplemented!("csv_in::emit")
}
