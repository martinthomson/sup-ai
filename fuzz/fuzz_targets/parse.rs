#![no_main]

use sup_ai::UsagePreferences;

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let mut up = UsagePreferences::default();
    up.parse(data);
});
