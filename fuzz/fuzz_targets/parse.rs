#![no_main]

use libfuzzer_sys::fuzz_target;
use sup_ai::UsagePreferences;

fuzz_target!(|data: (&[u8], &[u8])| {
    let mut up = UsagePreferences::default();
    up.parse(data.0);
    up.parse(data.1);
});
