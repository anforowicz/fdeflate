//! This fuzz target tests that `Decompressor::for_small_input` produces the same output
//! as `Decompressor::new`.
#![no_main]
#[macro_use]
extern crate libfuzzer_sys;

use fdeflate::Decompressor;

#[path = "../../src/decompress/tests/test_utils.rs"]
mod test_utils;
use test_utils::{decompress_by_chunks, TestDecompressionError};

fn decompress(d: Decompressor, input: &[u8]) -> Result<Vec<u8>, TestDecompressionError> {
    decompress_by_chunks(d, input, std::iter::repeat(input.len()), false)
}

fuzz_target!(|input: &[u8]| {
    let r_default = decompress(Decompressor::new(), input);
    let r_small = decompress(Decompressor::for_small_input(), input);
    match (r_default, r_small) {
        (Ok(output_default), Ok(output_small)) => assert_eq!(output_default, output_small),
        (Err(_e1), Err(_e2)) => (),
        (Ok(_), Err(e)) => panic!("Only default returned an error: {:?}", e),
        (Err(e), Ok(_)) => panic!("Only small returned an error: {:?}", e),
    }
});
