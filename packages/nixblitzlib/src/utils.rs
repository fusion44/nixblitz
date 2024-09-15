use std::{
    fmt::Display,
    fs::{self, File},
    io::Write,
    path::Path,
};

use error_stack::{Report, Result, ResultExt};
use include_dir::{include_dir, Dir};

use crate::{
    errors::{PasswordError, SystemError},
    nix_base_config::{NixBaseConfig, NixBaseConfigsTemplates},
};
use sha_crypt::{sha512_simple, Sha512Params};

pub struct AutoLineString(String);

impl AutoLineString {
    pub fn new() -> Self {
        Self(String::new())
    }

    pub fn from(value: &str) -> AutoLineString {
        let mut val = Self(String::new());
        val.push_line(value);
        val
    }

    pub fn push_line(&mut self, line: &str) {
        self.0.push_str(line);
        self.0.push('\n');
    }
}

impl Display for AutoLineString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.as_str())
    }
}

impl Default for AutoLineString {
    fn default() -> Self {
        Self::new()
    }
}

pub static BASE_TEMPLATE: Dir = include_dir!("./nixblitzlib/src/template/");

/// Hashes a password using the SHA-512 algorithm.
///
/// It uses a fixed number of rounds (10,000) for the SHA-512 hashing process.
///
/// # Arguments
/// * `pw` - The password string to be hashed.
///
/// # Returns
/// * `Ok(String)` - The hashed password string if successful.
/// * `Err(PasswordError)` - An error if the password hashing process fails.
///   The error includes a descriptive message.
///
/// # Errors
/// * `PasswordError::HashingError` -  This error occurs if there's a problem generating the
///   SHA-512 parameters or if the password hashing itself fails.
pub fn unix_hash_password(pw: &str) -> Result<String, PasswordError> {
    const ROUNDS: usize = 10_000;
    let params = Sha512Params::new(ROUNDS);
    let params = match params {
        Ok(p) => p,
        Err(_) => {
            return Err(Report::new(PasswordError::HashingError)
                .attach_printable("Unable to generate Sha512Params"))
        }
    };

    let hashed_pw = sha512_simple(pw, &params);
    let hashed_pw = match hashed_pw {
        Ok(p) => p,
        Err(_) => {
            return Err(Report::new(PasswordError::HashingError)
                .attach_printable("Unable to hash the password"))
        }
    };

    Ok(hashed_pw)
}

pub fn init_default_system(work_dir: &Path, force: Option<bool>) -> Result<(), SystemError> {
    let glob = "**/*";

    for dir_path in BASE_TEMPLATE
        .find(glob)
        .change_context(SystemError::GenFilesError)
        .attach_printable_lazy(|| "Unable to get templates")?
    {
        let f = dir_path.as_file();

        if let Some(f) = f {
            let contents = f.contents();
            let path = f.path();
            let path = work_dir.join(path);

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
                        .render(NixBaseConfigsTemplates::Common)
                        .change_context(SystemError::CreateBaseFiles(
                            "Failed at rendering base config".to_string(),
                        ))?;
                    create_file(Path::new(&path_str), rendered_nix.as_bytes(), force)?;
                    create_file(
                        Path::new(&format!("{}.json", &path_str)),
                        rendered_json.as_bytes(),
                        force,
                    )?;
                }

                continue;
            }

            create_file(&path, contents, force)?;
        }
    }

    Ok(())
}

fn create_file(path: &Path, contents: &[u8], force: Option<bool>) -> Result<(), SystemError> {
    fs::create_dir_all(
        path.parent()
            .ok_or_else(|| Report::new(SystemError::GenFilesError))?,
    )
    .change_context(SystemError::GenFilesError)
    .attach_printable_lazy(|| {
        format!("Path: {}", path.to_str().unwrap_or("Unable to unwrap path"))
    })?;

    let force = force.unwrap_or(false);
    if !force {
        let res = fs::read_to_string(path)
            .change_context(SystemError::GenFilesError)
            .attach_printable_lazy(|| {
                format!(
                    "Unable to read file {} to check if it is empty",
                    path.to_str().unwrap_or_default()
                )
            })?;
        if !res.is_empty() {
            return Err(Report::new(SystemError::GenFilesError)
                .attach_printable(format!(
                    "File exists and is not empty: {}",
                    path.to_str().unwrap_or_default()
                ))
                .attach_printable("Suggestion: 'force' to force overwriting the files"));
        }
    }

    let mut file = File::create(path)
        .change_context(SystemError::GenFilesError)
        .attach_printable_lazy(|| {
            format!(
                "Unable to create file {}",
                path.to_str().unwrap_or("Unable to unwrap path")
            )
        })?;

    file.write_all(contents)
        .change_context(SystemError::GenFilesError)
        .attach_printable_lazy(|| {
            format!(
                "Unable to write file contents to {}",
                path.to_str().unwrap_or_default()
            )
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use crate::utils::{create_file, unix_hash_password};
    use sha_crypt::sha512_check;
    use tempfile::NamedTempFile;

    #[test]
    fn test_unix_hash_password() {
        const TEST_PW: &str = "my_strong_password";

        let result = unix_hash_password(TEST_PW);
        assert!(result.is_ok());

        let result = sha512_check(TEST_PW, &result.unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        let path = Path::new(&path);

        let _ = create_file(path, b"Hello Test", Some(false));

        assert!(fs::metadata(path).is_ok());
        assert_eq!(fs::read_to_string(path).unwrap(), "Hello Test");

        let res = create_file(path, b"Hello Test", Some(false));
        assert!(res.is_err());

        let res = create_file(path, b"Hello Test", None);
        assert!(res.is_err());

        let res = create_file(path, b"Hello Force Test", Some(true));
        assert!(res.is_ok());
        assert_eq!(fs::read_to_string(path).unwrap(), "Hello Force Test");
    }

    #[test]
    fn test_init_default_system_creates_files() {
        // let temp_dir = tempfile::tempdir().unwrap();
        // let work_dir = temp_dir.path();
        //
        // // TODO: fixme
        // let res = init_default_system(work_dir);
        // if let Err(ref e) = res {
        //     println!("error: {}", e);
        // }
        // assert!(res.is_ok());
    }
}
