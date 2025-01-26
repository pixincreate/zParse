# Fuzz

Fuzzing is an automated testing technique that feeds random/semi-random data into a program to find bugs, crashes, memory leaks etc.

`fuzz_targets/` contains specific test cases for fuzzing different components:

  ```rust
  - json_parser.rs: Fuzzes the JSON parser
  - toml_parser.rs: Fuzzes the TOML parser 
  - json_converter.rs: Fuzzes JSON conversion
  - toml_converter.rs: Fuzzes TOML conversion
  ```
  
## Why do we need this?

Helps find edge cases and vulnerabilities that normal testing might miss