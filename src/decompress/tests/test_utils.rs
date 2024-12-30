//! Testing utilities for testing `fdeflate::Decompressor`.
//!
//! These utilities are used by:
//!
//! * Unit tests (e.g. `#[test]` tests in `src/decompress.rs`)
//! * Fuzzers (e.g. `fuzz/fuzz_targets/inflate_bytewise3.rs`)

#[cfg(test)]
use crate as fdeflate;

use fdeflate::{DecompressedRead, DecompressionError, Decompressor, GenericDecompressor};

#[derive(Debug, PartialEq)]
pub enum TestDecompressionError {
    ProdError(DecompressionError),
    TestError(TestErrorKind),
}

#[derive(Debug, Eq, PartialEq)]
pub enum TestErrorKind {
    OutputTooLarge,
    TooManyIterations,
}

impl From<DecompressionError> for TestDecompressionError {
    fn from(e: DecompressionError) -> Self {
        Self::ProdError(e)
    }
}

impl From<TestErrorKind> for TestDecompressionError {
    fn from(kind: TestErrorKind) -> Self {
        Self::TestError(kind)
    }
}

/// Decompresses `input` when feeding it into a `Decompressor::read` in `chunks`.
///
/// `chunks` typically can be used to decode the whole input at once (setting `chunks` to
/// `vec![input.len]`) or byte-by-byte (setting `chunks` to `std::iter::repeat(1)`).
/// But `chunks` can also be used to replicate arbitrary chunking patterns (such as may be
/// used by some fuzzing-based repros from the `png` crate).
///
/// `early_eof` is used to the last `end_of_input` argument of `Decompressor::read` calls.
/// When `early_eof` is `false`, then `end_of_input` is `false` until the whole input is
/// consumed (and then is `Decompressor::is_done` is still false, then `Decompressor::read`
/// is called one or more times with empty input slice and `end_of_input` set to true).
/// When `early_eof` is `true` then `end_of_input` is set to `true` as soon as the slice
/// fed to `Decompressor::read` "reaches" the end of the whole input.
///
/// Unlike the `png` crate, this testing helper uses a big, fixed-size output buffer.
/// (i.e. there is no simulation of `ZlibStream.compact_out_buffer_if_needed` from the `png`
/// crate).
#[allow(dead_code)]
pub fn decompress_by_chunks(
    input: &[u8],
    chunks: impl IntoIterator<Item = usize>,
    early_eof: bool,
) -> Result<Vec<u8>, TestDecompressionError> {
    let d = Decompressor::new();
    decompress_impl(d, input, chunks, early_eof)
}

/// Decompresses `input` using the specified `LITLEN_TABLE_SIZE` and `DIST_TABLE_SIZE`.
#[allow(dead_code)]
pub fn decompress_with_table_sizes<const LITLEN_TABLE_SIZE: usize, const DIST_TABLE_SIZE: usize>(
    input: &[u8],
) -> Result<Vec<u8>, TestDecompressionError> {
    let d = GenericDecompressor::<LITLEN_TABLE_SIZE, DIST_TABLE_SIZE>::new();
    const EARLY_EOF: bool = false;
    decompress_impl(d, input, std::iter::repeat(input.len()), EARLY_EOF)
}

fn decompress_impl<const LITLEN_TABLE_SIZE: usize, const DIST_TABLE_SIZE: usize>(
    mut d: GenericDecompressor<LITLEN_TABLE_SIZE, DIST_TABLE_SIZE>,
    input: &[u8],
    chunks: impl IntoIterator<Item = usize>,
    early_eof: bool,
) -> Result<Vec<u8>, TestDecompressionError> {
    let mut chunks = chunks.into_iter();

    // `iteration_counter` helps to prevent infinite loops (which may happen with `chunks` such
    // as `std::iter::repeat(0)`).
    let mut iteration_counter = 0;

    // Ignoring checksums so that we can work with inputs generated by fuzzing.  (Fuzzing
    // typically ignores checksums to make it easier to explore the space of possible inputs.)
    d.ignore_adler32();

    let mut out_buf = vec![0; 1_000_000];
    let mut in_pos = 0;
    let mut out_pos = 0;
    while !d.is_done() {
        iteration_counter += 1;
        if iteration_counter > 5000 {
            return Err(TestErrorKind::TooManyIterations.into());
        }

        let chunk_size = chunks.next().unwrap_or(0);
        let start = in_pos;
        let end = std::cmp::min(start + chunk_size, input.len());

        let eof = if early_eof {
            end == input.len()
        } else {
            start == input.len()
        };

        let (in_consumed, out_written) =
            d.read(&input[start..end], out_buf.as_mut_slice(), out_pos, eof)?;

        in_pos += in_consumed;
        out_pos += out_written;
        if out_pos == out_buf.len() && in_consumed == 0 && !d.is_done() {
            return Err(TestErrorKind::OutputTooLarge.into());
        }
    }

    out_buf.resize(out_pos, 0xFF);
    Ok(out_buf)
}
