#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate htp;

fuzz_target!(|data: &[u8]| {
    let mut t = Test::new(htp::tests::TestConfig());
    t.run_slice(data);
});
