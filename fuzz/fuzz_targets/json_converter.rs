#![no_main]
use libfuzzer_sys::fuzz_target;
use zparse::{parser::JsonParser, Converter};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok(mut parser) = JsonParser::new(s) {
            if let Ok(value) = parser.parse() {
                let _ = Converter::json_to_toml(value);
            }
        }
    }
});
