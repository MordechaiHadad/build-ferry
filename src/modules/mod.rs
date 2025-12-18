use std::path::{Path, PathBuf};

pub mod build;
pub mod tauri;

pub fn create_temp_ferry<P: AsRef<Path>>(temp_target: P, project_dir: P) -> PathBuf {
    let project_name = project_dir.as_ref().file_name().unwrap();
    temp_target.as_ref().join("build-ferry").join(project_name)
}