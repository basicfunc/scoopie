use argh::FromArgs;
use dirs::home_dir;
use std::{
    fmt::Display,
    fs::{DirBuilder, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};
use toml::Value;

use crate::error::{InitError, ScoopieError};

pub type InitResult = Result<ScoopieDirStats, ScoopieError>;

#[derive(FromArgs, PartialEq, Debug)]
/// Initialize Scoopie, useful while installing Scoopie itself
#[argh(subcommand, name = "init")]
pub struct InitCommand {
    #[argh(positional)]
    path: Option<PathBuf>,
}

const DEFAULT_TOML: &'static str = r#"
[repos]
main = "https://github.com/ScoopInstaller/Main"
"#;

#[derive(Debug)]
pub struct ScoopieDirStats {
    home: PathBuf,
    config: PathBuf,
}

impl Display for ScoopieDirStats {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "🎊 Congrats! Scoopie initialized.\nLocated at: {}\nConfig at: {}",
            self.home.display(),
            self.config.display()
        )
    }
}

impl InitCommand {
    pub fn from(config: InitCommand) -> InitResult {
        let home_dir = home_dir().ok_or(ScoopieError::HomeDirUnavailable)?;

        let scoopie_path = match config.path {
            Some(x) => get_absolute_path(&x)?,
            None => home_dir.clone(),
        }
        .join("scoopie");

        if scoopie_path.exists() {
            return Err(ScoopieError::DirAlreadyExists(scoopie_path));
        }

        let directories = vec![
            scoopie_path.clone(),
            scoopie_path.join("apps"),
            scoopie_path.join("buckets"),
            scoopie_path.join("cache"),
            scoopie_path.join("persists"),
            scoopie_path.join("shims"),
        ];

        directories
            .iter()
            .try_for_each(|path| create_directory(path))?;

        let config_dir = home_dir.join(".config");

        if !config_dir.exists() {
            create_directory(&config_dir)?;
        }

        let scoopie_config = config_dir.join("scoopie.toml");

        if !scoopie_config.exists() {
            let toml: Value = toml::from_str(DEFAULT_TOML)
                .map_err(|_| ScoopieError::Init(InitError::TOMLParse))?;

            let toml = toml::to_string_pretty(&toml)
                .map_err(|_| ScoopieError::Init(InitError::TOMLParse))?;

            write_toml(&scoopie_config, toml.as_bytes())?;
        }

        set_environment_variable("SCOOPIE_HOME", &scoopie_path.display().to_string())?;

        Ok(ScoopieDirStats {
            home: scoopie_path,
            config: scoopie_config,
        })
    }
}

fn get_absolute_path(path: &PathBuf) -> Result<PathBuf, ScoopieError> {
    let absolute_path = path
        .canonicalize()
        .map_err(|_| ScoopieError::AbsoultePathResolve)?;
    let absolute_path_str = absolute_path.to_string_lossy().to_string();

    // Remove the `\\?\` prefix from the absolute path string
    let path_without_prefix = if absolute_path_str.starts_with("\\\\?\\") {
        absolute_path_str[4..].to_string()
    } else {
        absolute_path_str
    };

    Ok(PathBuf::from(path_without_prefix))
}

fn set_environment_variable(name: &str, value: &str) -> Result<(), ScoopieError> {
    Command::new("cmd")
        .args(&["/C", "setx", name, value])
        .output()
        .map_err(|_| ScoopieError::Init(InitError::UnableToSetEnvVar))
        .and_then(|output| {
            if output.status.success() {
                Ok(())
            } else {
                Err(ScoopieError::Init(InitError::UnableToSetEnvVar))
            }
        })
}

fn create_directory(path: &Path) -> Result<(), ScoopieError> {
    DirBuilder::new()
        .recursive(true)
        .create(path)
        .map_err(|_| ScoopieError::FailedToMkdir(path.to_path_buf()))
}

fn write_toml(path: &Path, data: &[u8]) -> Result<(), ScoopieError> {
    File::create(path)
        .map_err(|_| ScoopieError::FailedToTouch(path.to_path_buf()))
        .and_then(|mut file| {
            file.write_all(data)
                .map_err(|_| ScoopieError::Init(InitError::ConfigWrite))
        })
}
