use hivehq::runtime::runner::run_hive_data_stream;
use std::fs::File;
use std::io::Cursor;

/// End-to-end: load a .hive file and run it to completion.
/// This requires a valid .hive program in test_data/.
#[test]
#[ignore = "requires a compiled .hive test program"]
fn run_minimal_hive_program() {
    let file = File::open("test_data/exit_0.hive").expect("open");
    let result = run_hive_data_stream(file);
    assert!(result.is_ok());
}

#[test]
fn run_non_hive_file_errors() {
    let data = b"not a hive program";
    let result = run_hive_data_stream(Cursor::new(data));
    assert!(result.is_err());
}

#[test]
#[ignore = "requires a compiled .hive test program"]
fn run_program_with_spawn() {
    let file = File::open("test_data/spawn_child.hive").expect("open");
    let result = run_hive_data_stream(file);
    assert!(result.is_ok());
}
