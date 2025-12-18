use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

pub mod build;
pub mod tauri;

pub fn create_temp_ferry<P: AsRef<Path> + Debug>(temp_target: P, project_dir: P) -> PathBuf {
    let project_dir_ref = project_dir.as_ref();
    let project_dir = if project_dir_ref.is_absolute() {
        project_dir_ref
    } else {
        &std::env::current_dir().unwrap()
    };
    let project_name = project_dir.file_name().unwrap();
    temp_target.as_ref().join("build-ferry").join(project_name)
}
