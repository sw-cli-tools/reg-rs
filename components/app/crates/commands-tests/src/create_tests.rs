use reg_rs_commands::utils::resolve_test_path;

#[test]
fn test_resolve_bare_name() {
    let resolved = resolve_test_path("my_test");
    let data_dir = reg_rs_discover::data_dir::data_dir();
    assert_eq!(resolved, data_dir.join("my_test.rgt").to_string_lossy());
}

#[test]
fn test_resolve_bare_name_with_tdb() {
    let resolved = resolve_test_path("my_test.tdb");
    let data_dir = reg_rs_discover::data_dir::data_dir();
    assert_eq!(resolved, data_dir.join("my_test.rgt").to_string_lossy());
}

#[test]
fn test_resolve_bare_name_with_rgt() {
    let resolved = resolve_test_path("my_test.rgt");
    let data_dir = reg_rs_discover::data_dir::data_dir();
    assert_eq!(resolved, data_dir.join("my_test.rgt").to_string_lossy());
}

#[test]
fn test_resolve_path_with_directory() {
    let resolved = resolve_test_path("/tmp/tests/foo");
    assert_eq!(resolved, "/tmp/tests/foo.rgt");
}

#[test]
fn test_resolve_path_with_directory_and_tdb() {
    let resolved = resolve_test_path("/tmp/tests/foo.tdb");
    assert_eq!(resolved, "/tmp/tests/foo.rgt");
}

#[test]
fn test_resolve_relative_path_with_directory() {
    let resolved = resolve_test_path("subdir/foo");
    assert_eq!(resolved, "subdir/foo.rgt");
}
