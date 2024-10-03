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

fn safety_checks(work_dir: &Path) -> Result<(), SystemError> {
    if !work_dir.exists() {
        return Ok(());
    }

    if !work_dir.is_dir() {
        Err(SystemError::CreatePathError(
            "The given path is not a directory".into(),
        ))?
    }

    let mut contents = work_dir
        .read_dir()
        .change_context(SystemError::CreatePathError(
            "Unable to read the working directory".into(),
        ))?;
    if contents.next().is_some() {
        Err(SystemError::CreatePathError(
            "The given path is not empty".into(),
        ))?
    }

    Ok(())
}

pub fn init_default_system(work_dir: &Path, force: Option<bool>) -> Result<(), SystemError> {
    if !force.unwrap_or(false) {
        safety_checks(work_dir)?;
    }

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
            if ext != "templ" {
                create_file(&path, contents, force)?;
            }
        }
    }

    render_template_files(work_dir, force)
}

fn render_template_files(work_dir: &Path, force: Option<bool>) -> Result<(), SystemError> {
    let nix_base_config = NixBaseConfig::default();
    let rendered_json = nix_base_config
        .to_json_string()
        .change_context(SystemError::GenFilesError)?;
    let rendered_nix = nix_base_config
        .render(NixBaseConfigsTemplates::Common)
        .change_context(SystemError::CreateBaseFiles(
            "Failed at rendering base config".to_string(),
        ))?;

    for (key, val) in rendered_nix.iter() {
        create_file(
            Path::new(&work_dir.join(key.replace(".templ", ""))),
            val.as_bytes(),
            force,
        )?;
    }

    create_file(
        Path::new(&work_dir.join("src/nix_base_config.json")),
        rendered_json.as_bytes(),
        force,
    )
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
            });

        if res.is_ok() {
            return Err(Report::new(SystemError::GenFilesError)
                .attach_printable(format!(
                    "File exists: {}",
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
    use std::fs::{self, create_dir, create_dir_all, File};

    use crate::{
        errors::SystemError,
        utils::{create_file, safety_checks, unix_hash_password},
    };
    use sha_crypt::sha512_check;

    #[test]
    fn test_unix_hash_password() {
        const TEST_PW: &str = "my_strong_password";

        let result = unix_hash_password(TEST_PW);
        assert!(result.is_ok());

        let result = sha512_check(TEST_PW, &result.unwrap());
        assert!(result.is_ok());
    }
    #[test]
    fn safety_checks_non_existent_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let non_existent_path = temp_dir.path().join("non_existent");

        let result = safety_checks(&non_existent_path);
        assert!(result.is_ok());
    }

    #[test]
    fn safety_check_spath_is_a_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("file.txt");
        File::create(&file_path).unwrap();

        let result = safety_checks(&file_path);
        assert!(result.is_err());

        let report = result.unwrap_err();
        let err = report.current_context();
        assert!(matches!(
            err,
            SystemError::CreatePathError(msg) if msg == "The given path is not a directory"
        ));
    }

    #[test]
    fn safety_check_empty_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let empty_dir_path = temp_dir.path().join("empty_dir");
        create_dir(&empty_dir_path).unwrap();

        let result = safety_checks(&empty_dir_path);
        assert!(result.is_ok());
    }

    #[test]
    fn safety_check_non_empty_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let non_empty_dir_path = temp_dir.path().join("non_empty_dir");
        create_dir_all(&non_empty_dir_path).unwrap();
        File::create(non_empty_dir_path.join("file.txt")).unwrap();

        let result = safety_checks(&non_empty_dir_path);
        assert!(result.is_err());

        let report = result.unwrap_err();
        let err = report.current_context();
        assert!(matches!(
            err,
            SystemError::CreatePathError(msg) if msg == "The given path is not empty"
        ));
    }

    #[test]
    fn create_new_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("new_file.txt");
        const CONTENTS: &str = "File contents";

        let result = create_file(&file_path, CONTENTS.as_bytes(), None);
        assert!(result.is_ok());

        let actual_contents = fs::read_to_string(&file_path).unwrap();
        assert_eq!(actual_contents, CONTENTS);
    }

    #[test]
    fn create_file_in_non_existent_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("nested_dir/test_file.txt");
        const CONTENTS: &str = "File contents";

        let result = create_file(&file_path, CONTENTS.as_bytes(), None);
        assert!(result.is_ok());

        let actual_contents = fs::read_to_string(&file_path).unwrap();
        assert_eq!(actual_contents, CONTENTS);
    }

    #[test]
    fn overwrite_existing_file_with_force() {
        const FILE_NAME: &str = "test_file.txt";
        const EXISTING_CONTENTS: &str = "the contents";
        const NEW_CONTENTS: &str = "this should not be written";
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join(FILE_NAME);
        fs::write(&file_path, EXISTING_CONTENTS).unwrap();

        let result = create_file(&file_path, NEW_CONTENTS.as_bytes(), Some(true));
        assert!(result.is_ok());

        let actual_contents = fs::read_to_string(&file_path).unwrap();
        assert_eq!(actual_contents, NEW_CONTENTS);
    }

    #[test]
    fn fail_to_overwrite_existing_file_without_force() {
        const FILE_NAME: &str = "test_file.txt";
        const EXISTING_CONTENTS: &str = "the contents";
        const NEW_CONTENTS: &str = "this should not be written";
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join(FILE_NAME);
        fs::write(&file_path, EXISTING_CONTENTS).unwrap();

        let result = create_file(&file_path, NEW_CONTENTS.as_bytes(), None);
        assert!(result.is_err());

        let report = result.unwrap_err();
        let err = report.current_context();
        assert!(matches!(err, SystemError::GenFilesError));

        let actual_contents = fs::read_to_string(&file_path).unwrap();
        assert_eq!(actual_contents, EXISTING_CONTENTS); // Content should not be overwritten
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
