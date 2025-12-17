use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::{Parser, ValueEnum};
use eyre::{Context, ContextCompat, Result};
use serde::{Deserialize, Serialize};

use crate::modules::tauri;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TauriCommand {
    Dev,
    Build,
}

#[derive(Debug, Parser)]
#[command(version)]
pub enum Cli {
    /// Build using a fast temp target directory and mirror artifacts back
    Build {
        /// Path to the project to build (where Cargo.toml lives)
        #[arg(long, default_value = ".")]
        project_dir: PathBuf,

        /// Temp/fast target dir (e.g. NVMe)
        #[arg(long)]
        temp_target: Option<PathBuf>,

        /// Final HDD target dir to mirror artifacts to
        #[arg(long)]
        final_target: Option<PathBuf>,

        /// Build profile: debug or release
        #[arg(long, default_value = "debug")]
        profile: String,

        /// Extra args to pass to cargo after `--`
        #[arg(last = true)]
        cargo_args: Vec<String>,
    },

    /// Run Tauri CLI (dev/build) using a fast temp target dir
    Tauri {
        /// Path to the Tauri project (where src-tauri lives)
        #[arg(long, default_value = ".")]
        project_dir: PathBuf,

        /// Temp/fast target dir for src-tauri (e.g. NVMe)
        #[arg(long)]
        temp_target: Option<PathBuf>,

        /// Final HDD target dir to mirror src-tauri artifacts to
        #[arg(long)]
        final_target: Option<PathBuf>,

        /// Which Tauri command to run: dev or build
        #[arg(long, value_enum, default_value_t = TauriCommand::Dev)]
        command: TauriCommand,

        /// Extra args to pass after `tauri dev/build --`
        #[arg(last = true)]
        tauri_args: Vec<String>,
    },
}

impl Cli {
    pub fn has_targets(&self, config: &Config) -> bool {
        if config.temp_target.is_some() && config.final_target.is_some() {
            return true;
        }
        match self {
            Cli::Build {
                temp_target,
                final_target,
                ..
            }
            | Cli::Tauri {
                temp_target,
                final_target,
                ..
            } => temp_target.is_some() && final_target.is_some(),
        }
    }
}

pub fn start() -> Result<()> {
    let cli = Cli::parse();

    let config = match &cli {
        Cli::Tauri { project_dir, .. } | Cli::Build { project_dir, .. } => {
            resolve_config(project_dir)
        }
    };

    if !cli.has_targets(&config) {
        eyre::bail!(
            "temp-target and final-target must be specified either via CLI arguments or config files"
        );
    }

    match cli {
        Cli::Build {
            project_dir,
            temp_target,
            final_target,
            profile,
            cargo_args,
        } => {

            // build::start(project_dir, temp_target, final_target, profile, cargo_args);
        }

        Cli::Tauri {
            project_dir,
            temp_target,
            final_target,
            command,
            tauri_args,
        } => {
            tauri::start(
                project_dir,
                temp_target.or(config.temp_target).unwrap(),
                final_target.or(config.final_target).unwrap(),
                command,
                tauri_args,
            )?;
        }
    }

    Ok(())
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
    /// Default temp target directory (NVMe)
    pub temp_target: Option<PathBuf>,

    /// Default final target directory
    pub final_target: Option<PathBuf>,
}

pub fn resolve_config(project_dir: &Path) -> Config {
    let global_config = load_global_config().unwrap_or_default();

    let project_config = load_project_config(project_dir).unwrap_or_default();

    let config = Config {
        temp_target: project_config.temp_target.or(global_config.temp_target),

        final_target: project_config.final_target.or(global_config.final_target),
    };

    config
}

fn load_global_config() -> Result<Config> {
    let config_path = dirs::config_dir()
        .context("could not find config directory")?
        .join("build-ferry")
        .join("config.toml");

    let content = fs::read_to_string(&config_path)?;
    toml::from_str(&content).context("failed to parse global config")
}

fn load_project_config(project_dir: &Path) -> Result<Config> {
    let config_path = project_dir.join(".build-ferry.toml");
    let content = fs::read_to_string(&config_path)?;
    toml::from_str(&content).context("failed to parse project config")
}
