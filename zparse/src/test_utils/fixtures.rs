pub const TEST_FILES: [&str; 6] = [
    "tests/input/small.json",
    "tests/input/file.json",
    "tests/input/large.json",
    "tests/input/small.toml",
    "tests/input/file.toml",
    "tests/input/large.toml",
];

pub const INVALID_JSON_SAMPLES: [(&str, &str); 5] = [
    ("{", "Incomplete object"),
    ("[", "Incomplete array"),
    ("}", "Unexpected closing brace"),
    ("]", "Unexpected closing bracket"),
    ("invalid", "Invalid token"),
];

pub const INVALID_TOML_SAMPLES: [(&str, &str); 5] = [
    ("[invalid", "Incomplete table header"),
    ("key = ", "Missing value"),
    ("= value", "Missing key"),
    ("[table]\nkey = 42\n[table]", "Duplicate table"),
    ("[[array]]\nkey = 1\n[array]", "Invalid array table"),
];
