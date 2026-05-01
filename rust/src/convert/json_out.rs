//! .nxb → JSON. Walks the tail-index one record at a time; supports
//! `--pretty`, `--ndjson`, and `--binary base64|hex|skip`.
//!
//! Stub — see plan step `impl: json export streamer`.

use super::{ExportArgs, ExportReport};
use crate::error::Result;
use std::io::{Read, Write};

pub fn run<R: Read, W: Write>(
    _reader: R,
    _writer: W,
    _args: &ExportArgs,
) -> Result<ExportReport> {
    unimplemented!("json_out::run")
}
