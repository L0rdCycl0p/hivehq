use hivehq::error::HiveError;
use hivehq::parser::load_file::load_file_by_data;
use std::io::Cursor;

#[test]
fn load_empty_data_errors() {
    let data: &[u8] = &[];
    let result = load_file_by_data(Cursor::new(data));
    assert!(result.is_err());
}

#[test]
fn load_invalid_magic_errors() {
    let data = b"not a hive file at all";
    let result = load_file_by_data(Cursor::new(data));
    assert!(result.is_err());
}

#[test]
fn load_truncated_header_errors() {
    // A .hive file header is at least 16 bytes. Provide fewer.
    let data = b"\x00\x01\x02\x03";
    let result = load_file_by_data(Cursor::new(data));
    assert!(result.is_err());
}

/// Generate a minimal valid .hive file for round-trip tests.
/// This requires a helper that writes a valid header, section table,
/// function table, and an empty code section.
fn minimal_hive_bytes() -> Vec<u8> {
    // TODO: Replace with actual minimal .hive file bytes once the
    // format is documented. For now, this is a placeholder.
    vec![]
}

#[test]
#[ignore = "requires a valid minimal .hive file"]
fn load_minimal_hive_succeeds() {
    let data = minimal_hive_bytes();
    let loaded = load_file_by_data(Cursor::new(data)).expect("load");
    assert_eq!(loaded.header.entry_point, 0);
}