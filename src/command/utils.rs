use crate::rgt;

pub fn resolve_test_path(test: &str) -> String {
    let path = std::path::Path::new(test);
    let mut resolved = if path.parent().is_some_and(|p| p != std::path::Path::new("")) {
        path.to_path_buf()
    } else {
        crate::data_dir().join(test)
    };
    if resolved
        .extension()
        .is_some_and(|ext| ext == crate::TDB_EXTENSION || ext == rgt::RGT_EXTENSION)
    {
        resolved.set_extension("");
    }
    let mut name = resolved.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".{}", rgt::RGT_EXTENSION));
    resolved.set_file_name(name);
    resolved.to_string_lossy().to_string()
}

pub fn read_test_command(test_path: &str) -> crate::error::Result<(String, String)> {
    if test_path.ends_with(&format!(".{}", rgt::RGT_EXTENSION)) {
        let spec = rgt::parse_rgt(test_path)?;
        let tdb_path = rgt::tdb_path_for_rgt(test_path);
        Ok((spec.command, tdb_path))
    } else {
        let original = crate::db::read_original_results(test_path)?;
        Ok((original.command, test_path.to_string()))
    }
}

pub fn format_test_name(test_path: &str) -> String {
    std::path::Path::new(test_path)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}
