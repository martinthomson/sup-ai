#![no_main]

use sup_ai::{UsagePreference::Allowed, UsagePreferences};

libfuzzer_sys::fuzz_target!(|data: (&[u8], &[u8])| {
    let mut up = UsagePreferences::default();
    up.parse(data.0);
    _ = up.eval(data.1, Allowed);
});
