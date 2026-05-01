//! .nxb → CSV. Column order defaults to schema key order; `--columns a,b,c`
//! overrides. Errors on unknown column names.
//!
//! Stub — see plan step `impl: csv export streamer`.

use super::{ExportArgs, ExportReport};
use crate::error::Result;
use std::io::{Read, Write};

pub fn run<R: Read, W: Write>(
    _reader: R,
    _writer: W,
    _args: &ExportArgs,
) -> Result<ExportReport> {
    unimplemented!("csv_out::run")
}
