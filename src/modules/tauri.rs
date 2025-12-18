use crate::{cli::TauriCommand, modules::create_temp_ferry};
use eyre::{Context, Result};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

pub fn start(
    project_dir: PathBuf,
    temp_target: PathBuf,
    final_target: PathBuf,
    command: TauriCommand,
    tauri_args: Vec<String>,
) -> Result<()> {
    let command_str = match command {
        TauriCommand::Dev => "dev",
        TauriCommand::Build => "build",
    };

    fs::create_dir_all(&temp_target)
        .with_context(|| format!("failed to create temp target dir {}", temp_target.display()))?;

    let src_tauri_dir = if project_dir.is_absolute() {
        project_dir.join("src-tauri")
    } else {
        env::current_dir()?.join(&project_dir).join("src-tauri")
    };

    let temp_target_project = create_temp_ferry(temp_target, project_dir).join("src-tauri/target");

    let mut cmd = Command::new("cargo");
    cmd.arg("tauri").arg(&command_str);
    cmd.args(&tauri_args);
    cmd.current_dir(&src_tauri_dir);

    cmd.env("CARGO_TARGET_DIR", &temp_target_project);

    let status = cmd.status().context("failed to spawn `cargo tauri`")?;
    if !status.success() {
        eyre::bail!("cargo tauri {command_str} failed with status {status}");
    }

    if command == TauriCommand::Build {
        mirror_dir(&temp_target_project, &final_target)?;
    }

    Ok(())
}

fn mirror_dir(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir_all(to)
        .with_context(|| format!("failed to create final target dir {}", to.display()))?;

    for entry in walkdir::WalkDir::new(from) {
        let entry = entry?;
        let rel = entry.path().strip_prefix(from).unwrap();
        let dest_path = to.join(rel);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&dest_path)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &dest_path)?;
        }
    }

    Ok(())
}
