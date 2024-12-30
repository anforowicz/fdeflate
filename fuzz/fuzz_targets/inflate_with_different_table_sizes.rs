//! This fuzz target tests that feeding bytes into the decompressor one at a time always produces
//! valid output.

#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate miniz_oxide;

#[path = "../../src/decompress/tests/test_utils.rs"]
mod test_utils;
use test_utils::decompress_with_table_sizes;

fuzz_target!(|input: &[u8]| {
    let r_default = decompress_with_table_sizes::<4096, 512>(input);
    let r_small = decompress_with_table_sizes::<512, 128>(input);
    match (r_default, r_small) {
        (Ok(output_whole), Ok(output_bytewise)) => assert_eq!(output_whole, output_bytewise),
        (Err(_e1), Err(_e2)) => (),
        (Ok(_), Err(e)) => panic!("Only default returned an error: {:?}", e),
        (Err(e), Ok(_)) => panic!("Only small returned an error: {:?}", e),
    }
});
