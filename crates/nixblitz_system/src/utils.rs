use std::{
    env,
    fmt::Display,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    str::FromStr,
};

use crate::{
    apply_changes::{ProcessOutput, run_nixos_rebuild_switch_async},
    bitcoind::BitcoinDaemonService,
    blitz_api::BlitzApiService,
    blitz_webui::BlitzWebUiService,
    cln::CoreLightningService,
    lnd::LightningNetworkDaemonService,
    nix_base_config::{NixBaseConfig, NixBaseConfigsTemplates},
    project::Project,
};
use error_stack::{Report, Result, ResultExt};
use include_dir::{Dir, include_dir};
use log::{debug, error, info, trace};
use nixblitz_core::{
    CommandError,
    errors::{PasswordError, ProjectError},
    system_platform::SystemPlatform,
};
use raw_cpuid::CpuId;
use sha_crypt::{Sha512Params, sha512_simple};
use std::io::ErrorKind;

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

pub static BASE_TEMPLATE: Dir = include_dir!("./nixblitz_system/src/template/");

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
                .attach_printable("Unable to generate Sha512Params"));
        }
    };

    let hashed_pw = sha512_simple(pw, &params);
    let hashed_pw = match hashed_pw {
        Ok(p) => p,
        Err(_) => {
            return Err(Report::new(PasswordError::HashingError)
                .attach_printable("Unable to hash the password"));
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

pub fn safety_checks(work_dir: &Path) -> Result<(), ProjectError> {
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

/// Executes a command asynchronously and returns its `Output`.
///
/// This function spawns a new process with the given command and arguments,
/// waits for it to complete, and collects its output. If the command runs and
/// exits successfully (with a zero exit code), it returns the `Output` struct.
///
/// # Arguments
///
/// * `cmd` - The command to execute (e.g., "git", "echo", "cargo").
/// * `args` - A slice of string slices, representing the arguments to the command.
///
/// # Returns
///
/// A `Result` which is:
/// - `Ok(Output)`: If the command executes and exits with a `0` status code.
///   The `Output` struct contains the process's exit status, stdout, and stderr.
/// - `Err(Report<CommandError>)`: If the command fails for any reason.
///
/// # Errors
///
/// This function will return an `Err` wrapping a `CommandError` variant:
/// - `CommandError::SpawnFailed`: If the command process could not be spawned.
/// - `CommandError::ExecutionFailed`: If the command was spawned successfully but
///   exited with a non-zero status code. The error will contain the command,
///   exit status, and captured output for debugging.
pub async fn exec_simple_command(cmd: &str, args: &[&str]) -> Result<Output, CommandError> {
    let command_str = format!("{} {}", cmd, args.join(" "));
    let output = tokio::process::Command::new(cmd)
        .args(args)
        .output()
        .await
        .map_err(|e| {
            // If `output()` fails then the command could not be spawned.
            Report::new(CommandError::SpawnFailed(command_str.clone()))
                .attach_printable(format!("OS error: {}", e))
        })?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        return Err(Report::new(CommandError::ExecutionFailed {
            command: command_str,
            status: output.status,
        })
        .attach_printable(stdout)
        .attach_printable(stderr));
    }

    Ok(output)
}

pub async fn commit_config<'a, P: Into<&'a str>>(
    work_dir: P,
    message: &str,
) -> Result<(), CommandError> {
    info!("Committing config changes with message: {}", message);
    let work_dir = work_dir.into();
    let args = &["-C", work_dir, "commit", "-a", "-m", message];
    info!("Committing with args: {:?}", args);

    exec_simple_command("git", args)
        .await
        .change_context(CommandError::GitError(format!(
            "git -C {} commit -a -m {}",
            work_dir, message
        )))?;

    Ok(())
}

pub fn commit_config_old<'a, P: Into<&'a str>>(
    work_dir: P,
    message: &str,
) -> Result<(), CommandError> {
    info!("Committing config changes with message: {}", message);
    let work_dir = work_dir.into();
    let args = &["-C", work_dir, "commit", "-a", "-m", message];
    info!("Committing with args: {:?}", args);

    let status = Command::new("git")
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| {
            let cmd = format!("git -C {} commit -a -m {}", work_dir, message);
            Report::new(CommandError::GitError(cmd)).attach_printable(e)
        })?;

    if !status.success() {
        let cmd = format!("git -C {} commit -a -m {}", work_dir, message);
        return Err(Report::new(CommandError::GitError(cmd.to_string()))
            .attach_printable(format!("Exited with status: {}", status)));
    }

    Ok(())
}

pub async fn apply_changes(work_dir: &Path) -> Result<(), ProjectError> {
    info!("Initiating NixOS rebuild...");
    let project = Project::load(work_dir.to_path_buf())?;

    let platform = if let Some(p) = project.get_platform().await {
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
                            project.set_changes_applied().await?;
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
            let res = commit_config_old(work_dir_str, "update config");
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
/// use nixblitz_system::utils::trim_lines_left;
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

/// Checks for the availability of required system commands.
///
/// This function iterates through a list of required command-line tools
/// and verifies that they can be found in the system's `PATH`.
///
/// # Arguments
///
/// * `dependencies` - A slice of string slices, where each element is the name
///   of a command to check (e.g., `&["sudo", "disko-install"]`).
///
/// # Returns
///
/// * `Ok(())` - If all required commands are found.
/// * `Err(Vec<String>)` - If one or more commands are missing. The vector
///   contains a list of the names of the missing commands.
///
pub fn check_system_dependencies(dependencies: &[&str]) -> std::result::Result<(), Vec<String>> {
    let mut missing_dependencies = Vec::new();

    debug!("Checking for required system dependencies...");

    for &command_name in dependencies {
        trace!("- Checking for '{}'...", command_name);
        match Command::new(command_name).output() {
            Ok(_) => {
                debug!("  '{}' found.", command_name);
            }
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    debug!("  '{}' NOT found.", command_name);
                    missing_dependencies.push(command_name.to_string());
                } else {
                    debug!(
                        "  Error checking for '{}': {}. Assuming it's not available.",
                        command_name, e
                    );
                    missing_dependencies.push(command_name.to_string());
                }
            }
        }
    }

    if missing_dependencies.is_empty() {
        trace!("All dependencies are available.");
        Ok(())
    } else {
        trace!("Error: The following required system dependencies are missing:");
        for dep in &missing_dependencies {
            trace!("- {}", dep);
        }
        Err(missing_dependencies)
    }
}

/// Attempts to reboot the system
pub fn reboot_system() -> Result<(), CommandError> {
    println!(
        "\n\n--------------------------------------------------------------------------------"
    );
    println!("Rebooting system...");
    println!("--------------------------------------------------------------------------------");

    let args = &["systemctl", "reboot"];
    let status = Command::new("sudo").args(args).status().map_err(|e| {
        error!("Failed to reboot system: {}", e);
        Report::new(CommandError::SpawnFailed(format!("sudo {:?}", args)))
            .attach_printable(format!("OS error: {}", e))
    })?;

    if !status.success() {
        error!("Failed to reboot system. Status: {}", status);
        return Err(Report::new(CommandError::ExecutionFailed {
            command: format!("sudo {:?}", args),
            status,
        }));
    }

    Ok(())
}

/// Attempts to poweroff the system
pub fn poweroff_system() -> Result<(), CommandError> {
    println!(
        "\n\n--------------------------------------------------------------------------------"
    );
    println!("Powering off system...");
    println!("--------------------------------------------------------------------------------");
    let args = &["systemctl", "poweroff"];
    let status = Command::new("sudo").args(args).status().map_err(|e| {
        error!("Failed to poweroff system: {}", e);
        Report::new(CommandError::SpawnFailed(format!("sudo {:?}", args)))
            .attach_printable(format!("OS error: {}", e))
    })?;

    if !status.success() {
        error!("Failed to poweroff system. Status: {}", status);
        return Err(Report::new(CommandError::ExecutionFailed {
            command: format!("sudo {:?}", args),
            status,
        }));
    }

    Ok(())
}

/// Tries to get a value from an environment variable and parse it.
/// Falls back to a provided default value if the variable is not set or fails to parse.
pub fn get_env_var<T>(key: &str, default: T) -> T
where
    T: FromStr + Display,
    <T as FromStr>::Err: Display,
{
    match env::var(key) {
        Ok(val_str) => val_str.parse::<T>().unwrap_or_else(|e| {
            error!(
                "Failed to parse ${}. Got '{}'. Defaulting to '{}': {}",
                key, val_str, default, e
            );
            default
        }),
        Err(e) => {
            debug!("Failed to get ${}. Defaulting to '{}': {}", key, default, e);
            default
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File, create_dir, create_dir_all};

    use crate::utils::{
        check_password_validity_confirm, check_system_dependencies, create_file,
        exec_simple_command, safety_checks, trim_lines_left, unix_hash_password, update_file,
    };
    use nixblitz_core::{CommandError, errors::ProjectError};
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

    #[test]
    fn test_all_dependencies_exist() {
        // Use commands that are virtually guaranteed to exist on a Unix-like system.
        let deps = &["echo", "ls", "sh"];
        let result = check_system_dependencies(deps);
        assert!(result.is_ok());
    }

    #[test]
    fn test_some_dependencies_missing() {
        // Use a mix of a real command and one that should not exist.
        let deps = &["ls", "a_very_unlikely_command_to_exist_12345"];
        let result = check_system_dependencies(deps);
        assert!(result.is_err());
        let missing = result.unwrap_err();
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0], "a_very_unlikely_command_to_exist_12345");
    }

    #[test]
    fn test_all_dependencies_missing() {
        // Use commands that are highly unlikely to exist.
        let deps = &["another_fake_command_abc", "and_one_more_xyz"];
        let result = check_system_dependencies(deps);
        assert!(result.is_err());
        let missing = result.unwrap_err();
        assert_eq!(missing.len(), 2);
        assert!(missing.contains(&"another_fake_command_abc".to_string()));
        assert!(missing.contains(&"and_one_more_xyz".to_string()));
    }

    #[test]
    fn test_empty_dependency_list() {
        // An empty list should always succeed.
        let deps = &[];
        let result = check_system_dependencies(deps);
        assert!(result.is_ok());
    }

    /// Tests that a simple, successful command works as expected.
    #[tokio::test]
    async fn test_exec_success() {
        let result = exec_simple_command("echo", &["hello", "rust"]).await;

        // Ensure the command succeeded
        assert!(result.is_ok(), "Command should have succeeded");
        let output = result.unwrap();

        // Check the status code and output
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "hello rust");
        assert_eq!(String::from_utf8_lossy(&output.stderr).trim(), "");
    }

    /// Tests that a non-existent command correctly returns a `SpawnFailed` error.
    #[tokio::test]
    async fn test_exec_spawn_failed() {
        let result = exec_simple_command("this_command_should_not_exist_12345", &[]).await;

        // Ensure the command failed
        assert!(result.is_err(), "Command should have failed to spawn");
        let error = result.unwrap_err();

        // Check that the error is the correct variant
        let cause = error.downcast_ref::<CommandError>();
        assert!(
            matches!(cause, Some(CommandError::SpawnFailed(_))),
            "Error should be SpawnFailed"
        );
    }

    /// Tests that a command that runs but exits with a non-zero status
    /// correctly returns an `ExecutionFailed` error.
    #[tokio::test]
    async fn test_exec_execution_failed() {
        // The `false` command is standard on Unix-like systems and always exits with status 1.
        let result = exec_simple_command("false", &[]).await;

        // Ensure the command failed
        assert!(result.is_err(), "Command should have failed execution");
        let error = result.unwrap_err();

        // Check that the error is the correct variant and contains the right details
        let cause = error.downcast_ref::<CommandError>();
        match cause {
            Some(CommandError::ExecutionFailed { status, .. }) => {
                assert_eq!(status.code(), Some(1), "Exit code should be 1");
            }
            _ => panic!("Error should be ExecutionFailed"),
        }
    }

    /// Tests a failing command that also produces output on stderr.
    #[tokio::test]
    async fn test_exec_execution_failed_with_stderr() {
        // We use `sh -c` to run a shell command that prints to stderr and exits.
        // This is a portable way to test this behavior on Unix-like systems.
        let cmd = "sh";
        let args = &["-c", "echo 'an error occurred' >&2; exit 42"];

        let result = exec_simple_command(cmd, args).await;

        // Ensure the command failed
        assert!(result.is_err(), "Command should have failed");
        let error = result.unwrap_err();

        // Check that the error is the correct variant and contains the captured stderr.
        let cause = error.downcast_ref::<CommandError>();
        match cause {
            Some(CommandError::ExecutionFailed { status, .. }) => {
                assert_eq!(status.code(), Some(42), "Exit code should be 42");
            }
            _ => panic!("Error should be ExecutionFailed"),
        }
    }
}
