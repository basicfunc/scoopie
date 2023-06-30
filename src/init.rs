use argh::FromArgs;
use dirs::{data_dir, home_dir};
use std::{
    fmt::Display,
    fs::{DirBuilder, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    vec,
};
use toml::Value;

#[derive(FromArgs, PartialEq, Debug)]
/// Initialize Scoopie, useful while installing Scoopie itself
#[argh(subcommand, name = "init")]
pub struct InitCommand {
    #[argh(positional)]
    path: Option<PathBuf>,
}

#[derive(Debug)]
pub enum InitError {
    HomeDirUnavailable,
    DataDirUnavailable,
    ScoopieDirAlreadyexists(PathBuf),
    FailedToMkdir(PathBuf),
    FailedToTouch(PathBuf),
    TOMLParse,
    ConfigWrite,
    UnableToSetEnvVar,
    AbsoultePathResolve,
}

const DEFAULT_TOML: &'static str = r#"
[repos]
main = "https://github.com/ScoopInstaller/Main"
"#;

#[derive(Debug)]
pub struct ScoopieDirStats {
    home: PathBuf,
    data: PathBuf,
    config: PathBuf,
}

impl Display for ScoopieDirStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "🎊 Congrats! Scoopie initialized.\nLocated at: {}\nData at: {}\nConfig at: {}",
            self.home.display(),
            self.data.display(),
            self.config.display()
        )
    }
}

impl InitCommand {
    pub fn from(config: InitCommand) -> Result<ScoopieDirStats, InitError> {
        let home_dir = home_dir().ok_or(InitError::HomeDirUnavailable)?;

        let scoopie_path = match config.path {
            Some(x) => get_absolute_path(&x)?,
            None => home_dir.clone(),
        }
        .join("scoopie");

        if scoopie_path.exists() {
            return Err(InitError::ScoopieDirAlreadyexists(scoopie_path));
        }

        let directories = vec![
            scoopie_path.clone(),
            scoopie_path.join("apps"),
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
            let toml: Value = toml::from_str(DEFAULT_TOML).map_err(|_| InitError::TOMLParse)?;

            let toml = toml::to_string_pretty(&toml).map_err(|_| InitError::TOMLParse)?;

            write_toml(&scoopie_config, toml.as_bytes())?;
        }

        let data_dir = data_dir().ok_or(InitError::DataDirUnavailable)?;
        let scoopie_data_dir = data_dir.join("scoopie");

        if !scoopie_data_dir.exists() {
            let data_dirs = vec![scoopie_data_dir.clone(), scoopie_data_dir.join("repos")];

            data_dirs
                .iter()
                .try_for_each(|path| create_directory(path))?;

            let repo = scoopie_data_dir.join("repos.json");
            File::create(&repo).map_err(|_| InitError::FailedToTouch(repo))?;
        }

        set_environment_variable("SCOOPIE_HOME", &scoopie_path.display().to_string())?;

        Ok(ScoopieDirStats {
            home: scoopie_path,
            data: scoopie_data_dir,
            config: scoopie_config,
        })
    }
}

fn get_absolute_path(path: &PathBuf) -> Result<PathBuf, InitError> {
    let absolute_path = path
        .canonicalize()
        .map_err(|_| InitError::AbsoultePathResolve)?;
    let absolute_path_str = absolute_path.to_string_lossy().to_string();

    // Remove the `\\?\` prefix from the absolute path string
    let path_without_prefix = if absolute_path_str.starts_with("\\\\?\\") {
        absolute_path_str[4..].to_string()
    } else {
        absolute_path_str
    };

    Ok(PathBuf::from(path_without_prefix))
}

fn set_environment_variable(name: &str, value: &str) -> Result<(), InitError> {
    Command::new("cmd")
        .args(&["/C", "setx", name, value])
        .output()
        .map_err(|_| InitError::UnableToSetEnvVar)
        .and_then(|output| {
            if output.status.success() {
                Ok(())
            } else {
                Err(InitError::UnableToSetEnvVar)
            }
        })
}

fn create_directory(path: &Path) -> Result<(), InitError> {
    DirBuilder::new()
        .recursive(true)
        .create(path)
        .map_err(|_| InitError::FailedToMkdir(path.to_path_buf()))
}

fn write_toml(path: &Path, data: &[u8]) -> Result<(), InitError> {
    File::create(path)
        .map_err(|_| InitError::FailedToTouch(path.to_path_buf()))
        .and_then(|mut file| file.write_all(data).map_err(|_| InitError::ConfigWrite))
}
