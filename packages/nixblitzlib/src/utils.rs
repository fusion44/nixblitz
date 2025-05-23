use std::{
    fmt::Display,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use crate::{
    apply_changes::{run_nixos_rebuild_switch_async, ProcessOutput},
    bitcoind::BitcoinDaemonService,
    blitz_api::BlitzApiService,
    blitz_webui::BlitzWebUiService,
    cln::CoreLightningService,
    lnd::LightningNetworkDaemonService,
    nix_base_config::{NixBaseConfig, NixBaseConfigsTemplates},
    project::Project,
};
use common::{
    errors::{GitError, PasswordError, ProjectError},
    system_platform::SystemPlatform,
};
use error_stack::{Report, Result, ResultExt};
use include_dir::{include_dir, Dir};
use log::{debug, error, info};
use raw_cpuid::CpuId;
use sha_crypt::{sha512_simple, Sha512Params};

// default password: "nixblitz"
pub(crate) static INITIAL_PASSWORD: &str = "$6$rounds=10000$moY2rIPxoNODYRxz$1DESwWYweHNkoB6zBxI3DUJwUfvA6UkZYskLOHQ9ulxItgg/hP5CRn2Fr4iQGO7FE16YpJAPMulrAuYJnRC9B.";

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

/// Checks the validity of a password by ensuring it matches the confirmation and is longer than 10 characters.
///
/// # Arguments
///
/// * `main` - The main password string.
/// * `confirm` - An optional confirmation password string.
///
/// # Returns
///
/// * `Ok(())` - If the password is valid.
/// * `PasswordError` - An error if the password is not valid.
///
/// # Errors
///
/// This function will return an error if:
///
/// * The confirmation password is `None`.
/// * The passwords do not match.
/// * The password is not longer than 10 characters.
pub fn check_password_validity_confirm(
    main: &str,
    confirm: &Option<String>,
) -> Result<(), PasswordError> {
    if confirm.is_none() {
        return Err(Report::new(PasswordError::MissingConfirm));
    }

    let confirm = confirm.as_ref().unwrap();

    if main != confirm {
        return Err(Report::new(PasswordError::Mismatch));
    }

    if main.len() <= 10 {
        return Err(Report::new(PasswordError::TooShort));
    }

    Ok(())
}

fn safety_checks(work_dir: &Path) -> Result<(), ProjectError> {
    if !work_dir.exists() {
        return Ok(());
    }

    if !work_dir.is_dir() {
        Err(ProjectError::CreatePathError(
            "The given path is not a directory".into(),
        ))?
    }

    let mut contents = work_dir
        .read_dir()
        .change_context(ProjectError::CreatePathError(
            "Unable to read the working directory".into(),
        ))?;
    if contents.next().is_some() {
        Err(ProjectError::CreatePathError(
            "The given path is not empty".into(),
        ))?
    }

    Ok(())
}

pub fn init_default_project(work_dir: &Path, force: Option<bool>) -> Result<(), ProjectError> {
    if !force.unwrap_or(false) {
        safety_checks(work_dir)?;
    }

    let glob = "**/*";

    let mut templ_files = vec![];
    for dir_path in BASE_TEMPLATE
        .find(glob)
        .change_context(ProjectError::GenFilesError)
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
                templ_files.push(path);
                continue;
            }

            create_file(&path, contents, force)?;
        }
    }

    render_template_files(work_dir, templ_files, force)
}

pub fn commit_config<'a, P: Into<&'a str>>(work_dir: P, message: &str) -> Result<(), GitError> {
    info!("Committing config changes with message: {}", message);
    let work_dir = work_dir.into();
    let args = &["-C", work_dir, "commit", "-a", "-m", message];
    info!("Committing with args: {:?}", args);

    let status = Command::new("git")
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .change_context(GitError::CommitError(work_dir.to_string()))?;

    if !status.success() {
        return Err(Report::new(GitError::CommitError(work_dir.to_string())).attach_printable(""));
    }

    Ok(())
}

pub async fn apply_changes(work_dir: &Path) -> Result<(), ProjectError> {
    info!("Initiating NixOS rebuild...");
    let project = Project::load(work_dir.to_path_buf())?;

    let platform = if let Some(p) = project.get_platform() {
        p
    } else {
        return Err(Report::new(ProjectError::ApplyChangesError())
            .attach_printable("Error Unable to get platform from project."));
    };
    debug!("Got platform: {:?}", platform);

    match run_nixos_rebuild_switch_async(work_dir.to_str().unwrap().to_string(), &platform).await {
        Ok(mut receiver) => {
            while let Some(output) = receiver.recv().await {
                match output {
                    ProcessOutput::Stdout(line) => {
                        info!("OUT: {}", line);
                        println!("{}", line);
                    }
                    ProcessOutput::Stderr(line) => {
                        info!("ERR: {}", line);
                        println!("{}", line);
                    }
                    ProcessOutput::Completed(status) => {
                        info!("--- Process finished with status: {} ---", status);
                        println!("--- Process finished with status: {} ---", status);
                        if !status.success() {
                            info!("Warning: Command exited with non-zero status.");
                            Err(Report::new(ProjectError::ApplyChangesError())
                                .attach_printable("Command exited with non-zero status."))?
                        } else {
                            let mut project = Project::load(work_dir.to_path_buf())?;
                            project.set_changes_applied()?;
                        }
                    }
                    ProcessOutput::Error(err_msg) => {
                        info!("RUNTIME ERROR: {}", err_msg);
                        println!("RUNTIME ERROR: {}", err_msg);
                        break;
                    }
                }
            }
            info!("Rebuild process monitoring finished (channel closed).");
            println!("Rebuild process monitoring finished (channel closed).");
        }
        Err(report) => {
            info!("Failed to start command: {}", report);
            panic!("Failed to start command: {}", report);
        }
    }

    let work_dir_str = work_dir.to_str();
    match work_dir_str {
        Some(work_dir_str) => {
            let res = commit_config(work_dir_str, "update config");
            match res {
                Ok(()) => {
                    info!("\n✅ System config update committed successfully");
                }
                Err(e) => {
                    error!("{}", e);
                }
            };
        }
        None => {
            let message =
                "Unable to convert work_dir to string. Can't commit config changes to Git.";
            error!("{}", message);
        }
    }

    Ok(())
}

fn render_template_files(
    work_dir: &Path,
    templ_files: Vec<PathBuf>,
    force: Option<bool>,
) -> Result<(), ProjectError> {
    for path in templ_files {
        let filename = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("");
        if filename.is_empty() {
            return Err(Report::new(ProjectError::GenFilesError))
                .attach_lazy(|| format!("Unable to get filename from path: {}", filename));
        } else if filename == "cln.nix" {
            _create_cln_files(work_dir, force)?;
        } else if filename == "lnd.nix" {
            _create_lnd_files(work_dir, force)?;
        } else if filename == "bitcoind.nix" {
            _create_bitcoin_files(work_dir, force)?;
        } else if filename == "configuration.common.nix" {
            _create_nix_base_config(work_dir, force)?;
        } else if filename == "api.nix" {
            _create_blitz_api_files(work_dir, force)?;
        } else if filename == "web.nix" {
            _create_blitz_webui_files(work_dir, force)?;
        } else {
            info!("Unknown template file: {}", filename);
        }
    }

    Ok(())
}

fn _create_bitcoin_files(work_dir: &Path, force: Option<bool>) -> Result<(), ProjectError> {
    let bitcoin_cfg = BitcoinDaemonService::default();
    let rendered_json = bitcoin_cfg
        .to_json_string()
        .change_context(ProjectError::GenFilesError)?;
    let rendered_nix = bitcoin_cfg
        .render()
        .change_context(ProjectError::CreateBaseFiles(
            "Failed at rendering bitcoind config".to_string(),
        ))?;

    for (key, val) in rendered_nix.iter() {
        create_file(
            Path::new(&work_dir.join(key.replace(".templ", ""))),
            val.as_bytes(),
            force,
        )?;
    }

    create_file(
        Path::new(&work_dir.join("src/btc/bitcoind.json")),
        rendered_json.as_bytes(),
        force,
    )?;

    Ok(())
}

fn _create_blitz_webui_files(work_dir: &Path, force: Option<bool>) -> Result<(), ProjectError> {
    let blitz_webui_cfg = BlitzWebUiService::default();
    let rendered_json = blitz_webui_cfg
        .to_json_string()
        .change_context(ProjectError::GenFilesError)?;
    let rendered_nix = blitz_webui_cfg
        .render()
        .change_context(ProjectError::CreateBaseFiles(
            "Failed at rendering blitz web ui config".to_string(),
        ))?;

    for (key, val) in rendered_nix.iter() {
        create_file(
            Path::new(&work_dir.join(key.replace(".templ", ""))),
            val.as_bytes(),
            force,
        )?;
    }

    create_file(
        Path::new(&work_dir.join("src/blitz/web.json")),
        rendered_json.as_bytes(),
        force,
    )?;

    Ok(())
}

fn _create_blitz_api_files(work_dir: &Path, force: Option<bool>) -> Result<(), ProjectError> {
    let blitz_api_cfg = BlitzApiService::default();
    let rendered_json = blitz_api_cfg
        .to_json_string()
        .change_context(ProjectError::GenFilesError)?;
    let rendered_nix = blitz_api_cfg
        .render()
        .change_context(ProjectError::CreateBaseFiles(
            "Failed at rendering blitz api config".to_string(),
        ))?;

    for (key, val) in rendered_nix.iter() {
        create_file(
            Path::new(&work_dir.join(key.replace(".templ", ""))),
            val.as_bytes(),
            force,
        )?;
    }

    create_file(
        Path::new(&work_dir.join("src/blitz/api.json")),
        rendered_json.as_bytes(),
        force,
    )?;

    Ok(())
}

fn _create_cln_files(work_dir: &Path, force: Option<bool>) -> Result<(), ProjectError> {
    let cln_cfg = CoreLightningService::default();
    let rendered_json = cln_cfg
        .to_json_string()
        .change_context(ProjectError::GenFilesError)?;
    let rendered_nix = cln_cfg
        .render()
        .change_context(ProjectError::CreateBaseFiles(
            "Failed at rendering cln config".to_string(),
        ))?;

    for (key, val) in rendered_nix.iter() {
        create_file(
            Path::new(&work_dir.join(key.replace(".templ", ""))),
            val.as_bytes(),
            force,
        )?;
    }

    create_file(
        Path::new(&work_dir.join("src/btc/cln.json")),
        rendered_json.as_bytes(),
        force,
    )?;

    Ok(())
}

fn _create_lnd_files(work_dir: &Path, force: Option<bool>) -> Result<(), ProjectError> {
    let lnd_cfg = LightningNetworkDaemonService::default();
    let rendered_json = lnd_cfg
        .to_json_string()
        .change_context(ProjectError::GenFilesError)?;
    let rendered_nix = lnd_cfg
        .render()
        .change_context(ProjectError::CreateBaseFiles(
            "Failed at rendering lnd config".to_string(),
        ))?;

    for (key, val) in rendered_nix.iter() {
        create_file(
            Path::new(&work_dir.join(key.replace(".templ", ""))),
            val.as_bytes(),
            force,
        )?;
    }

    create_file(
        Path::new(&work_dir.join("src/btc/lnd.json")),
        rendered_json.as_bytes(),
        force,
    )?;

    Ok(())
}

fn _create_nix_base_config(work_dir: &Path, force: Option<bool>) -> Result<(), ProjectError> {
    let mut nix_base_config = NixBaseConfig::default();
    nix_base_config
        .platform
        .set_value(get_system_platform().as_short_str().into());

    let rendered_json = nix_base_config
        .to_json_string()
        .change_context(ProjectError::GenFilesError)?;
    let rendered_nix = nix_base_config
        .render(NixBaseConfigsTemplates::Common)
        .change_context(ProjectError::CreateBaseFiles(
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
    )?;

    Ok(())
}

pub fn create_file(path: &Path, contents: &[u8], force: Option<bool>) -> Result<(), ProjectError> {
    fs::create_dir_all(
        path.parent()
            .ok_or_else(|| Report::new(ProjectError::GenFilesError))?,
    )
    .change_context(ProjectError::GenFilesError)
    .attach_printable_lazy(|| {
        format!("Path: {}", path.to_str().unwrap_or("Unable to unwrap path"))
    })?;

    let force = force.unwrap_or(false);
    if !force {
        let res = fs::read_to_string(path)
            .change_context(ProjectError::GenFilesError)
            .attach_printable_lazy(|| {
                format!(
                    "Unable to read file {} to check if it is empty",
                    path.to_str().unwrap_or_default()
                )
            });

        if res.is_ok() {
            return Err(Report::new(ProjectError::GenFilesError)
                .attach_printable(format!(
                    "File exists: {}",
                    path.to_str().unwrap_or_default()
                ))
                .attach_printable("Suggestion: 'force' to force overwriting the files"));
        }
    }

    let mut file = File::create(path)
        .change_context(ProjectError::GenFilesError)
        .attach_printable_lazy(|| {
            format!(
                "Unable to create file {}",
                path.to_str().unwrap_or("Unable to unwrap path")
            )
        })?;

    file.write_all(contents)
        .change_context(ProjectError::GenFilesError)
        .attach_printable_lazy(|| {
            format!(
                "Unable to write file contents to {}",
                path.to_str().unwrap_or_default()
            )
        })?;

    Ok(())
}

/// Updates the contents of an existing file.
///
/// # Arguments
///
/// * `path` - A reference to the path of the file to update.
/// * `contents` - The new contents to write to the file.
///
/// # Returns
///
/// * `Ok(())` - If the file is successfully updated.
/// * `Err(ProjectError)` - An error if the file does not exist or cannot be updated.
///
/// # Errors
///
/// This function will return an error if:
///
/// * The file does not exist.
/// * The file cannot be opened for writing.
/// * The file cannot be written to.
pub fn update_file(path: &Path, contents: &[u8]) -> Result<(), ProjectError> {
    if !path.exists() {
        return Err(Report::new(ProjectError::FileNotFound(
            path.to_str().unwrap_or("Unable to unwrap path").to_string(),
        )));
    }

    let mut file = File::create(path)
        .change_context(ProjectError::GenFilesError)
        .attach_printable_lazy(|| {
            format!(
                "Unable to open file {} for updating",
                path.to_str().unwrap_or("Unable to unwrap path")
            )
        })?;

    file.write_all(contents)
        .change_context(ProjectError::GenFilesError)
        .attach_printable_lazy(|| {
            format!(
                "Unable to write updated contents to {}",
                path.to_str().unwrap_or_default()
            )
        })?;

    Ok(())
}

/// Loads the contents of a JSON file.
///
/// # Arguments
///
/// * `file_path` - A reference to the path of the file to load.
///
/// # Returns
///
/// * `Ok(String)` - The contents of the file as a String.
/// * `Err(Report)` - An error occurred while loading the file.
///
/// # Errors
///
/// This function will return an error if:
///
/// * The file does not exist.
/// * The file cannot be opened.
/// * The file cannot be read.
pub fn load_json_file(file_path: &Path) -> Result<String, ProjectError> {
    if !file_path.exists() {
        return Err(Report::new(ProjectError::FileNotFound(
            file_path
                .to_str()
                .unwrap_or("Unable to unwrap path")
                .to_string(),
        )));
    }

    let mut file = File::open(file_path).change_context(ProjectError::FileOpenError(
        file_path
            .to_str()
            .unwrap_or("Uable to unwrap path")
            .to_string(),
    ))?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .change_context(ProjectError::FileReadError(
            file_path
                .to_str()
                .unwrap_or("Unable to unwrap path")
                .to_string(),
        ))?;

    Ok(contents)
}

/// Trims leading whitespace from each line in the input string. Blank lines
/// will be conserved.
///
/// # Arguments
///
/// * `input` - A string slice that holds the text to be processed.
///
/// # Returns
///
/// * A `String` with leading whitespace removed from each line.
///
/// # Example
///
/// ```
/// use nixblitzlib::utils::trim_lines_left;
///
/// let input = "\n  line 1\n  line 2\n\n";
/// let result = trim_lines_left(input);
/// assert_eq!(result, "\nline 1\nline 2\n\n");
/// ```
pub fn trim_lines_left(input: &str) -> String {
    let mut result = input
        .lines()
        .map(|line| line.trim_start())
        .collect::<Vec<_>>()
        .join("\n");

    if input.ends_with('\n') {
        result.push('\n');
    }

    result
}

/// Detects the system architecture and virtualization status (for x86_64) AT RUNTIME.
///
/// Uses compile-time checks (`cfg!`) *only* to include the appropriate architecture-specific
/// runtime detection logic. The actual check for virtualization on x86_64 happens
/// by executing the CPUID instruction when the function is called.
///
/// # Returns
///
/// A `SystemPlatform` enum variant indicating the platform detected at runtime.
pub fn get_system_platform() -> SystemPlatform {
    // TODO: add support for other architectures
    if cfg!(target_arch = "x86_64") {
        let cpuid = CpuId::new();
        match cpuid.get_feature_info() {
            Some(finfo) if finfo.has_hypervisor() => SystemPlatform::X86_64Vm,
            _ => SystemPlatform::X86_64BareMetal,
        }
    } else if cfg!(target_arch = "aarch64") {
        SystemPlatform::Arm64
    } else {
        SystemPlatform::Unsupported
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{self, create_dir, create_dir_all, File};

    use crate::utils::{
        check_password_validity_confirm, create_file, safety_checks, trim_lines_left,
        unix_hash_password, update_file,
    };
    use common::errors::ProjectError;
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
    fn test_check_password_sanity_confirm() {
        let main_password = "strong_password";
        let confirm_password = Some("strong_password".to_string());

        // Test matching passwords
        let result = check_password_validity_confirm(main_password, &confirm_password);
        assert!(result.is_ok());

        // Test non-matching passwords
        let non_matching_confirm = Some("different_password".to_string());
        let result = check_password_validity_confirm(main_password, &non_matching_confirm);
        assert!(result.is_err());

        // Test short password
        let short_password = "short";
        let result = check_password_validity_confirm(short_password, &confirm_password);
        assert!(result.is_err());

        // Test None confirm password
        let result = check_password_validity_confirm(main_password, &None);
        assert!(result.is_err());
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
            ProjectError::CreatePathError(msg) if msg == "The given path is not a directory"
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
            ProjectError::CreatePathError(msg) if msg == "The given path is not empty"
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
    fn test_update_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("update_test.txt");
        const INITIAL_CONTENTS: &str = "Initial contents";
        const UPDATED_CONTENTS: &str = "Updated contents";

        // Create the file with initial contents
        fs::write(&file_path, INITIAL_CONTENTS).unwrap();

        // Update the file with new contents
        let result = update_file(&file_path, UPDATED_CONTENTS.as_bytes());
        assert!(result.is_ok());

        // Verify the file contents have been updated
        let actual_contents = fs::read_to_string(&file_path).unwrap();
        assert_eq!(actual_contents, UPDATED_CONTENTS);
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
        assert!(matches!(err, ProjectError::GenFilesError));

        let actual_contents = fs::read_to_string(&file_path).unwrap();
        assert_eq!(actual_contents, EXISTING_CONTENTS); // Content should not be overwritten
    }

    #[test]
    fn test_init_default_project_creates_files() {
        // let temp_dir = tempfile::tempdir().unwrap();
        // let work_dir = temp_dir.path();
        //
        // // TODO: fixme
        // let res = init_default_project(work_dir);
        // if let Err(ref e) = res {
        //     println!("error: {}", e);
        // }
        // assert!(res.is_ok());
    }

    #[test]
    fn test_trim_lines_left() {
        let input = r#"

        line 1
            line 2
        line 3

        "#;
        let expected_output = "\n\nline 1\nline 2\nline 3\n\n";
        let result = trim_lines_left(input);
        assert_eq!(result, expected_output);

        let expected_output = "\nline 1 \nline 2\nline 3\n";
        let result = trim_lines_left("\nline 1 \nline 2\nline 3\n     ");
        assert_eq!(result, expected_output);

        let expected_output = "line 1    \nline 2 \nline 3  ";
        let result = trim_lines_left("         line 1    \nline 2 \n    line 3  ");
        assert_eq!(result, expected_output);
    }
}
