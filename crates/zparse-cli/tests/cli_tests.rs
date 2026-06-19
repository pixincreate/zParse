use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_parse_json() {
    let mut cmd = Command::cargo_bin("zparse").unwrap();
    cmd.arg("parse")
        .arg("--from")
        .arg("json")
        .arg("--print-output")
        .write_stdin(r#"{"name": "test", "value": 42}"#);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test"))
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_parse_csv_with_delimiter() {
    let mut cmd = Command::cargo_bin("zparse").unwrap();
    cmd.arg("parse")
        .arg("--from")
        .arg("csv")
        .arg("--csv-delimiter")
        .arg("\t")
        .arg("--print-output")
        .write_stdin("name\tage\nAlice\t30\n");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Alice"));
}

#[test]
fn test_convert_json_to_toml() {
    let mut cmd = Command::cargo_bin("zparse").unwrap();
    cmd.arg("convert")
        .arg("--from")
        .arg("json")
        .arg("--to")
        .arg("toml")
        .arg("--print-output")
        .write_stdin(r#"{"name": "test"}"#);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("name"))
        .stdout(predicate::str::contains("test"));
}

#[test]
fn test_convert_csv_to_json() {
    let mut cmd = Command::cargo_bin("zparse").unwrap();
    cmd.arg("convert")
        .arg("--from")
        .arg("csv")
        .arg("--to")
        .arg("json")
        .arg("--print-output")
        .write_stdin("name,age\nAlice,30\n");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("30"));
}

#[test]
fn test_invalid_csv_delimiter() {
    let mut cmd = Command::cargo_bin("zparse").unwrap();
    cmd.arg("parse")
        .arg("--from")
        .arg("csv")
        .arg("--csv-delimiter")
        .arg("\n")
        .write_stdin("name,age\nAlice,30\n");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("conflicts"));
}

#[test]
fn test_non_ascii_csv_delimiter() {
    let mut cmd = Command::cargo_bin("zparse").unwrap();
    cmd.arg("parse")
        .arg("--from")
        .arg("csv")
        .arg("--csv-delimiter")
        .arg("é")
        .write_stdin("name,age\nAlice,30\n");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("ASCII"));
}

#[test]
fn test_convert_yaml_to_json() {
    let mut cmd = Command::cargo_bin("zparse").unwrap();
    cmd.arg("convert")
        .arg("--from")
        .arg("yaml")
        .arg("--to")
        .arg("json")
        .arg("--print-output")
        .write_stdin("name: test\nvalue: 42");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test"))
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_convert_xml_to_json() {
    let mut cmd = Command::cargo_bin("zparse").unwrap();
    cmd.arg("convert")
        .arg("--from")
        .arg("xml")
        .arg("--to")
        .arg("json")
        .arg("--print-output")
        .write_stdin("<root><name>test</name></root>");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test"));
}

#[test]
fn test_convert_toml_to_json() {
    let mut cmd = Command::cargo_bin("zparse").unwrap();
    cmd.arg("convert")
        .arg("--from")
        .arg("toml")
        .arg("--to")
        .arg("json")
        .arg("--print-output")
        .write_stdin("name = \"test\"\nvalue = 42");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test"))
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_csv_custom_delimiter_pipe() {
    let mut cmd = Command::cargo_bin("zparse").unwrap();
    cmd.arg("convert")
        .arg("--from")
        .arg("csv")
        .arg("--to")
        .arg("json")
        .arg("--csv-delimiter")
        .arg("|")
        .arg("--print-output")
        .write_stdin("name|age|city\nAlice|30|NYC\nBob|25|LA\n");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("NYC"));
}

#[test]
fn test_jsonc_with_comments() {
    let mut cmd = Command::cargo_bin("zparse").unwrap();
    cmd.arg("convert")
        .arg("--from")
        .arg("json")
        .arg("--to")
        .arg("json")
        .arg("--json-comments")
        .arg("--print-output")
        .write_stdin(
            r#"// This is a comment
{"name": "test", // inline comment
"value": 42}"#,
        );
    cmd.assert().success();
}

#[test]
fn test_stdin_input() {
    let mut cmd = Command::cargo_bin("zparse").unwrap();
    cmd.arg("--parse")
        .arg("--from")
        .arg("json")
        .write_stdin(r#"{"test": true}"#);
    cmd.assert().success();
}
