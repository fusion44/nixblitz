use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    str::FromStr,
};

use error_stack::{Report, Result, ResultExt};

use crate::{
    errors::SystemError,
    nix_base_config::{self, NixBaseConfig},
    utils::BASE_TEMPLATE,
};

/// Represents a system config that is stored at the [System::path].
#[derive(Default, Debug)]
pub struct System {
    /// Whether to create an initial system if the given directory is empty
    init: bool,

    /// The path we operate in
    path: PathBuf,
}

impl System {
    pub fn new(path: PathBuf, init: bool) -> Self {
        Self { init, path }
    }

    /// Initializes the system based on the configuration found in the given path.
    ///
    /// This function checks if the specified path exists and is a directory. If the directory is empty,
    /// it will initialize a new system if init was set to true. Otherwise, it returns an error.
    ///
    /// # Errors
    ///
    /// * [SystemError::ParseError] - If the path is invalid, doesn't exist, or is not a directory.
    /// * [SystemError::NoSystemFound] - If the directory is empty and init is false.
    pub fn init(self) -> Result<(), SystemError> {
        if self.path.exists() {
            let items = self.path.read_dir();
            let mut items = match items {
                Ok(items) => items,
                Err(e) => {
                    return Err(Report::new(SystemError::ParseError)
                        .attach_printable(format!("Error reading items in the directory: {}", e)))
                }
            };

            let item = items.next();
            if item.is_none() {
                if self.init {
                    // Path exists, but no files are in it
                    self.init_default_system()?;
                    return Ok(());
                } else {
                    return Err(Report::new(SystemError::ParseError).attach_printable(
                        "Given folder is empty, use --init to initialize a new system",
                    ));
                }
            }

            return Ok(());
        }

        if self.init {
            // Path doesn't exist, create it
            self.init_default_system()?;
            return Ok(());
        }

        Err(Report::new(SystemError::ParseError)
            .attach_printable("Given folder is empty, use --init to initialize a new system"))
    }

    fn init_default_system(self) -> Result<(), SystemError> {
        // get all files
        let glob = "**/*";

        for dir_path in BASE_TEMPLATE.find(glob).unwrap() {
            let f = dir_path.as_file();

            if let Some(f) = f {
                let contents = f.contents();
                let path = f.path();
                let path = self.path.join(path);

                let ext = path
                    .extension()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default();
                if ext == "templ" {
                    let path_str = path.to_str().unwrap_or_default().replace(".templ", "");
                    if path_str.contains("configuration.common.nix") {
                        let nix_base_config = NixBaseConfig::default();
                        let rendered_json = nix_base_config
                            .to_json_string()
                            .change_context(SystemError::GenFilesError)?;
                        let rendered_nix = nix_base_config
                            .render(nix_base_config::NixBaseConfigsTemplates::Common)
                            .change_context(SystemError::CreateBaseFiles(
                                "Failed at rendering base config".to_string(),
                            ))?;
                        Self::create_file(Path::new(&path_str), rendered_nix.as_bytes())?;
                        Self::create_file(
                            Path::new(&format!("{}.json", &path_str)),
                            rendered_json.as_bytes(),
                        )?;
                    }

                    continue;
                }

                Self::create_file(&path, contents)?;
            }
        }
        Ok(())
    }

    fn create_file(path: &Path, contents: &[u8]) -> Result<(), SystemError> {
        fs::create_dir_all(
            path.parent()
                .ok_or_else(|| Report::new(SystemError::GenFilesError))?,
        )
        .change_context(SystemError::GenFilesError)
        .attach_printable_lazy(|| {
            format!("Path: {}", path.to_str().unwrap_or("Unable to unwrap path"))
        })?;
        let mut file = File::create(path)
            .change_context(SystemError::GenFilesError)
            .attach_printable_lazy(|| {
                format!(
                    "Unable to create file {}",
                    path.to_str().unwrap_or("Unable to unwrap path")
                )
            })?;
        file.write_all(contents)
            .change_context(SystemError::GenFilesError)?;
        Ok(())
    }
}
