use alejandra::format;
use error_stack::{Report, Result, ResultExt};
use handlebars::{no_escape, Handlebars};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display, path::Path, str::FromStr};
use strum::EnumCount;

use crate::{
    app_config::AppConfig,
    app_option_data::{
        bool_data::BoolOptionData,
        option_data::{
            ApplicableOptionData, GetOptionId, OptionData, OptionDataChangeNotification, OptionId,
            ToOptionId,
        },
        password_data::PasswordOptionData,
        string_list_data::{StringListOptionData, StringListOptionItem},
        text_edit_data::TextOptionData,
    },
    apps::SupportedApps,
    errors::{ProjectError, TemplatingError},
    locales::LOCALES,
    strings::INITIAL_PASSWORD,
    timezones::TIMEZONES,
    utils::{check_password_validity_confirm, unix_hash_password, update_file, BASE_TEMPLATE},
};

pub const TEMPLATE_FILE_NAME: &str = "src/configuration.common.nix.templ";
pub const JSON_FILE_NAME: &str = "src/nix_base_config.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct NixBaseConfig {
    /// Whether to allow unfree packages from nixpkgs
    pub allow_unfree: Box<BoolOptionData>,

    /// The timezone that should be used for this system
    ///
    /// [nixos.org:time.timeZone](https://search.nixos.org/options?show=time.timeZone)
    ///
    /// Default: "America/New_York"
    ///
    /// Example: "Europe/Copenhagen"
    pub time_zone: Box<StringListOptionData>,

    /// The default locale. It determines the language for program
    /// messages, the format for dates and times, sort order, and so on.
    /// It also determines the character set, such as UTF-8.
    ///
    /// [nixos.org:i18n.defaultLocale](https://search.nixos.org/options?show=i18n.defaultLocale)
    ///
    /// Default: "en_US.UTF-8"
    ///
    /// Example: "nl_NL.UTF-8"
    /// Example: "nl_NL.utf8"
    pub default_locale: Box<StringListOptionData>,

    /// The disk to use for the system
    ///
    /// Note: this will not be exposed to the user
    ///       this must be set manually via cli:
    ///       "nixblitz -w /path/to/config set base_config disko_device '/dev/vda'"
    pub disko_device: String,

    /// The login username to use. This is the user
    /// with which most of the administrative tasks are executed.
    ///
    /// [nixos.org:users.users](https://search.nixos.org/options?show=users.users)
    ///
    /// Default: "admin"
    ///
    /// Example: "nixblitz"
    pub username: String,

    /// Whether to allow SSH password authentication.
    ///
    /// [nixos.org:services.openssh.settings.PasswordAuthentication](https://search.nixos.org/options?show=services.openssh.settings.PasswordAuthentication)
    ///
    /// Default: false
    pub ssh_password_auth: bool,

    /// The initial password that will be used.
    /// Use the [`crate::utils::unix_hash_password`] utility fn to generate the hash.
    ///
    /// Default: nixblitz
    ///
    /// [nixos.org:users.users.\<name\>.hashedPassword](https://search.nixos.org/options?show=users.users.<name>.hashedPassword)
    pub hashed_password: Box<PasswordOptionData>,

    /// SSH authentication keys to allow for SSH connection attempts.
    ///
    /// The authentication keys are always valid the [username].
    ///
    /// [nixos.org:users.users.\<name\>.openssh.authorizedKeys.keys](https://search.nixos.org/options?show=users.users.<name>.openssh.authorizedKeys.keys)
    ///
    pub openssh_auth_keys: Vec<String>,

    ///  Additional packages to install from the nixos repository. Any package from the
    ///
    /// [nix pkgs](https://search.nixos.org/packages) search can be used.
    ///
    /// # Examples
    /// ```
    /// use nixblitzlib::nix_base_config::NixBaseConfig;
    ///
    /// let config = NixBaseConfig {
    ///   system_packages: vec![String::from("bat"), String::from("yazi")],
    ///   ..NixBaseConfig::default()
    /// };
    /// ```
    pub system_packages: Vec<String>,

    /// Additional ports to open.
    ///
    /// [nixos.org:networking.firewall.allowedTCPPorts](https://search.nixos.org/options?show=networking.firewall.allowedTCPPorts)
    ///
    /// # Examples
    /// ```
    /// use nixblitzlib::nix_base_config::NixBaseConfig;
    ///
    /// let config = NixBaseConfig {
    ///   ports: vec![22, 1337],
    ///   ..NixBaseConfig::default()
    /// };
    /// ```
    pub ports: Vec<usize>,

    /// Hostname of the system when started as a virtual machine
    ///
    /// [nisos.org:networking.hostName](https://search.nixos.org/options?show=networking.hostName)
    pub hostname_vm: String,

    /// Hostname of the system when started on a PI4
    ///
    /// [nisos.org:networking.hostName](https://search.nixos.org/options?show=networking.hostName)
    pub hostname_pi4: String,

    /// Hostname of the system when started on a PI5
    ///
    /// [nisos.org:networking.hostName](https://search.nixos.org/options?show=networking.hostName)
    pub hostname_pi5: String,

    /// Hostname of the system when started on an X86 machine
    ///
    /// [nisos.org:networking.hostName](https://search.nixos.org/options?show=networking.hostName)
    pub hostname_x86: String,
}

impl Default for NixBaseConfig {
    fn default() -> Self {
        let allow_unfree = false;
        let time_zone = "America/New_York".to_string();
        let default_locale = "en_US.utf8".to_string();
        let username = "admin".to_string();
        Self {
            allow_unfree: Box::new(BoolOptionData::new(
                NixBaseConfigOption::AllowUnfree.to_option_id(),
                allow_unfree,
            )),
            time_zone: Box::new(StringListOptionData::new(
                NixBaseConfigOption::TimeZone.to_option_id(),
                time_zone,
                TIMEZONES
                    .iter()
                    .map(|tz| StringListOptionItem::new(tz.to_string(), tz.to_string()))
                    .collect(),
            )),
            disko_device: String::from(""),
            default_locale: Box::new(StringListOptionData::new(
                NixBaseConfigOption::DefaultLocale.to_option_id(),
                default_locale,
                LOCALES
                    .iter()
                    .map(|tz| StringListOptionItem::new(tz.to_string(), tz.to_string()))
                    .collect(),
            )),
            username: username.clone(),
            ssh_password_auth: false,
            hashed_password: Box::new(PasswordOptionData::new(
                NixBaseConfigOption::InitialPassword.to_option_id(),
                INITIAL_PASSWORD.to_string(),
                true,
                10,
                false,
                INITIAL_PASSWORD.to_string(),
            )),
            openssh_auth_keys: vec![],
            system_packages: vec![
                String::from("fd"),
                String::from("bat"),
                String::from("bottom"),
                String::from("fzf"),
                String::from("neovim"),
                String::from("ripgrep"),
                String::from("bandwhich"),
                String::from("superfile"),
            ],
            ports: vec![22],
            hostname_vm: "nixblitzvm".to_string(),
            hostname_pi4: "nixblitzpi4".to_string(),
            hostname_pi5: "nixblitzpi5".to_string(),
            hostname_x86: "nixblitzx86".to_string(),
        }
    }
}

#[derive(Debug)]
pub enum NixBaseConfigsTemplates {
    Common,
}

#[derive(Debug, Clone, Copy, EnumCount, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum NixBaseConfigOption {
    AllowUnfree,
    TimeZone,
    DefaultLocale,
    DiskoDevice,
    Username,
    InitialPassword,
}

impl ToOptionId for NixBaseConfigOption {
    fn to_option_id(&self) -> OptionId {
        OptionId::new(SupportedApps::NixOS, self.to_string())
    }
}

impl FromStr for NixBaseConfigOption {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<NixBaseConfigOption, ()> {
        match s {
            "allow_unfree" => Ok(NixBaseConfigOption::AllowUnfree),
            "time_zone" => Ok(NixBaseConfigOption::TimeZone),
            "default_locale" => Ok(NixBaseConfigOption::DefaultLocale),
            "disko_device" => Ok(NixBaseConfigOption::DiskoDevice),
            "username" => Ok(NixBaseConfigOption::Username),
            "initial_password" => Ok(NixBaseConfigOption::InitialPassword),
            _ => Err(()),
        }
    }
}

impl Display for NixBaseConfigOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            NixBaseConfigOption::AllowUnfree => "allow_unfree",
            NixBaseConfigOption::TimeZone => "time_zone",
            NixBaseConfigOption::DefaultLocale => "default_locale",
            NixBaseConfigOption::DiskoDevice => "disko_device",
            NixBaseConfigOption::Username => "username",
            NixBaseConfigOption::InitialPassword => "initial_password",
        };
        write!(f, "{}", s)
    }
}

const _FILES: [&str; 5] = [
    "src/configuration.common.nix.templ",
    "src/vm/configuration.nix.templ",
    "src/pi4/configuration.nix.templ",
    "src/pi5/configuration.nix.templ",
    "src/x86/configuration.nix.templ",
];

impl NixBaseConfigsTemplates {
    fn files(&self) -> [&str; 5] {
        match self {
            NixBaseConfigsTemplates::Common => _FILES,
        }
    }
}

impl Display for NixBaseConfigsTemplates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NixBaseConfigsTemplates::Common => {
                let debug_string = NixBaseConfigsTemplates::Common.files().join("\n");
                f.write_str(&debug_string)
            }
        }
    }
}

impl NixBaseConfig {
    #![allow(clippy::too_many_arguments)]
    pub fn new(
        allow_unfree: Box<BoolOptionData>,
        time_zone: Box<StringListOptionData>,
        default_locale: Box<StringListOptionData>,
        disko_device: String,
        username: String,
        ssh_password_auth: bool,
        hashed_password: Box<PasswordOptionData>,
        openssh_auth_keys: Vec<String>,
        system_packages: Vec<String>,
        ports: Vec<usize>,
        hostname_vm: String,
        hostname_pi4: String,
        hostname_pi5: String,
        hostname_x86: String,
    ) -> Self {
        Self {
            allow_unfree,
            time_zone,
            default_locale,
            disko_device,
            username: username.clone(),
            ssh_password_auth,
            hashed_password,
            openssh_auth_keys,
            system_packages,
            ports,
            hostname_vm,
            hostname_pi4,
            hostname_pi5,
            hostname_x86,
        }
    }

    pub fn render(
        &self,
        template: NixBaseConfigsTemplates,
    ) -> Result<HashMap<String, String>, TemplatingError> {
        // TODO: I'd like to return a &str key here, as it is always a 'static
        //       reference to the _FILES array. Why no workey?
        let mut handlebars = Handlebars::new();
        handlebars.register_escape_fn(no_escape);

        let mut rendered_contents = HashMap::new();
        for file_name in template.files() {
            let file = match template {
                NixBaseConfigsTemplates::Common => BASE_TEMPLATE.get_file(file_name),
            };
            let file = match file {
                Some(f) => f,
                None => {
                    return Err(
                        Report::new(TemplatingError::FileNotFound(file_name.to_string()))
                            .attach_printable(format!("File {file_name} for {template} not found")),
                    )
                }
            };

            let file =
                match file.contents_utf8() {
                    Some(f) => f,
                    None => {
                        return Err(Report::new(TemplatingError::FileNotFound(
                            file_name.to_string(),
                        ))
                        .attach_printable(format!("Unable to read file contents of {template}")))
                    }
                };

            handlebars
                .register_template_string(file_name, file)
                .attach_printable_lazy(|| format!("{handlebars:?} could not register the template"))
                .change_context(TemplatingError::Register)?;

            // TODO: de-hardcode this
            let mut data = HashMap::new();
            if file_name == "src/configuration.common.nix.templ" {
                data = HashMap::from([
                    ("allow_unfree", format!("{}", self.allow_unfree.value())),
                    ("time_zone", self.time_zone.value().into()),
                    ("default_locale", self.default_locale.value().into()),
                    ("disko_device", self.disko_device.clone()),
                    ("username", self.username.clone()),
                    ("ssh_password_auth", format!("{}", self.ssh_password_auth)),
                    (
                        "initial_password",
                        self.hashed_password.hashed_value().clone(),
                    ),
                    (
                        "openssh_auth_keys",
                        self.openssh_auth_keys
                            .iter()
                            .map(|s| format!("\"{}\"", s))
                            .collect::<Vec<_>>()
                            .join("\n"),
                    ),
                    ("system_packages", self.system_packages.join(" ")),
                    (
                        "ports",
                        self.ports
                            .iter()
                            .map(|&p| p.to_string())
                            .collect::<Vec<String>>()
                            .join(" "),
                    ),
                ]);
            } else if file_name == "src/vm/configuration.nix.templ" {
                data = HashMap::from([("hostname", self.hostname_vm.clone())]);
            } else if file_name == "src/pi4/configuration.nix.templ" {
                data = HashMap::from([("hostname", self.hostname_pi4.clone())]);
            } else if file_name == "src/pi5/configuration.nix.templ" {
                data = HashMap::from([("hostname", self.hostname_pi5.clone())]);
            } else if file_name == "src/x86/configuration.nix.templ" {
                data = HashMap::from([("hostname", self.hostname_x86.clone())]);
            } else {
                Err(
                    Report::new(TemplatingError::FileNotFound(file_name.to_owned()))
                        .attach_printable(format!(
                            "Couldn't process file {file_name} for template {template}"
                        )),
                )?
            }

            let res = handlebars
                .render(file_name, &data)
                .attach_printable(format!("Failed to render template {template}"))
                .change_context(TemplatingError::Render)?;
            let (status, text) = format::in_memory("<convert nix base>".to_string(), res);

            if let format::Status::Error(e) = status {
                Err(Report::new(TemplatingError::Format)).attach_printable_lazy(|| {
                    format!("Could not format the template file due to error: {e}")
                })?
            } else {
                rendered_contents.insert(file_name.to_string(), text);
            }
        }

        Ok(rendered_contents)
    }

    pub fn to_json_string(&self) -> Result<String, TemplatingError> {
        serde_json::to_string_pretty(self).change_context(TemplatingError::JsonRenderError)
    }

    pub fn from_json(json_data: &str) -> Result<NixBaseConfig, TemplatingError> {
        let res: NixBaseConfig =
            serde_json::from_str(json_data).change_context(TemplatingError::JsonLoadError)?;

        Ok(res)
    }
}

impl AppConfig for NixBaseConfig {
    fn app_option_changed(
        &mut self,
        option: &OptionDataChangeNotification,
    ) -> Result<bool, ProjectError> {
        let id = option.id();
        if let Ok(opt) = NixBaseConfigOption::from_str(&id.option) {
            let mut res = Ok(false);
            if opt == NixBaseConfigOption::AllowUnfree {
                if let OptionDataChangeNotification::Bool(val) = option {
                    res = Ok(self.allow_unfree.value() != val.value);
                    self.allow_unfree.set_value(val.value);
                } else {
                    Err(Report::new(ProjectError::ChangeOptionValueError(
                        NixBaseConfigOption::AllowUnfree.to_string(),
                    )))?;
                }
            } else if opt == NixBaseConfigOption::TimeZone {
                if let OptionDataChangeNotification::StringList(val) = option {
                    res = Ok(*self.time_zone.value().to_string() != val.value);
                    self.time_zone.set_value(val.value.clone());
                } else {
                    Err(Report::new(ProjectError::ChangeOptionValueError(
                        NixBaseConfigOption::DefaultLocale.to_string(),
                    )))?;
                }
            } else if opt == NixBaseConfigOption::DefaultLocale {
                if let OptionDataChangeNotification::StringList(val) = option {
                    res = Ok(*self.default_locale.value().to_string() != val.value);
                    self.default_locale.set_value(val.value.clone());
                } else {
                    Err(Report::new(ProjectError::ChangeOptionValueError(
                        NixBaseConfigOption::DefaultLocale.to_string(),
                    )))?;
                }
            } else if opt == NixBaseConfigOption::DiskoDevice {
                if let OptionDataChangeNotification::TextEdit(val) = option {
                    self.disko_device = val.value.clone();
                    res = Ok(true);
                } else {
                    Err(Report::new(ProjectError::ChangeOptionValueError(
                        NixBaseConfigOption::DiskoDevice.to_string(),
                    )))?;
                }
            } else if opt == NixBaseConfigOption::Username {
                if let OptionDataChangeNotification::TextEdit(val) = option {
                    self.username = val.value.clone();
                    res = Ok(true);
                }
            } else if opt == NixBaseConfigOption::InitialPassword {
                if let OptionDataChangeNotification::PasswordEdit(password_opt) = option {
                    let main: String = password_opt.value.clone();
                    let confirm: Option<String> = password_opt.confirm.clone();

                    let check_result = check_password_validity_confirm(&main, &confirm);
                    if check_result.is_err() {
                        // TODO: handle invalid passwords more gracefully.
                        //       The user should be notified. For now we
                        //       expect that library users handle invalid cases
                        //       Currently there is no way to notify library
                        //       users properly.
                        return Ok(false);
                    }

                    let hashed_pw = unix_hash_password(&main).change_context(
                        ProjectError::ChangeOptionValueError("Unable to hash password".into()),
                    )?;

                    res = Ok(true);
                    self.hashed_password.set_hashed_value(hashed_pw);
                    self.hashed_password
                        .set_subtitle(self.hashed_password.hashed_value().clone());
                } else {
                    Err(Report::new(ProjectError::ChangeOptionValueError(
                        NixBaseConfigOption::InitialPassword.to_string(),
                    )))?;
                }
            } else {
                Err(
                    Report::new(ProjectError::ChangeOptionValueError(opt.to_string()))
                        .attach_printable(format!("Unknown option: {}", opt,)),
                )?
            }

            return res;
        }

        Ok(false)
    }

    fn get_options(&self) -> Vec<OptionData> {
        vec![
            OptionData::Bool(self.allow_unfree.clone()),
            OptionData::StringList(self.time_zone.clone()),
            OptionData::StringList(self.default_locale.clone()),
            OptionData::TextEdit(Box::new(TextOptionData::new(
                NixBaseConfigOption::DiskoDevice.to_option_id(),
                self.disko_device.clone(),
                1,
                false,
                self.disko_device.clone(),
            ))),
            OptionData::TextEdit(Box::new(TextOptionData::new(
                NixBaseConfigOption::Username.to_option_id(),
                self.username.clone(),
                1,
                false,
                self.username.clone(),
            ))),
            OptionData::PasswordEdit(self.hashed_password.clone()),
        ]
    }

    fn save(&mut self, work_dir: &Path) -> Result<(), ProjectError> {
        let rendered_json = self
            .to_json_string()
            .change_context(ProjectError::GenFilesError)?;
        let rendered_nix = self
            .render(NixBaseConfigsTemplates::Common)
            .change_context(ProjectError::CreateBaseFiles(
                "Failed at rendering the nix base config".to_string(),
            ))?;

        for (key, val) in rendered_nix.iter() {
            update_file(
                Path::new(&work_dir.join(key.replace(".templ", ""))),
                val.as_bytes(),
            )?;
        }

        update_file(
            Path::new(&work_dir.join(JSON_FILE_NAME)),
            rendered_json.as_bytes(),
        )?;

        Ok(())
    }

    fn set_applied(&mut self) {
        self.allow_unfree.set_applied();
        self.time_zone.set_applied();
        self.default_locale.set_applied();
        self.hashed_password.set_applied();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{init_default_project, unix_hash_password};

    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_save_function() {
        // Note: maybe test every field? Right now we just check if
        //       enable is set to true or false respectively
        // Create a temporary directory
        let temp_dir = tempdir().unwrap();
        let work_dir = temp_dir.path();

        let _ = init_default_project(work_dir, Some(false));
        let mut config = NixBaseConfig::default();

        // force enable to "true"
        let _ = config
            .app_option_changed(&OptionDataChangeNotification::Bool(
                crate::app_option_data::bool_data::BoolOptionChangeData {
                    id: NixBaseConfigOption::AllowUnfree.to_option_id(),
                    value: true,
                },
            ))
            .unwrap();

        // Call the save function
        let result = config.save(work_dir);

        // Assert that the save function returns Ok
        assert!(result.is_ok());

        let json_file_path = work_dir.join(JSON_FILE_NAME);
        // Check that the JSON file contains the expected content
        let json_content = fs::read_to_string(&json_file_path).unwrap();
        let expected_json_content = config.to_json_string().unwrap();
        assert_eq!(json_content, expected_json_content);

        // Check that the Nix file contains the expected content
        let nix_file_path = work_dir.join(TEMPLATE_FILE_NAME.replace(".templ", ""));
        let rendered_nix = config.render(NixBaseConfigsTemplates::Common).unwrap();
        let expected_nix_content = rendered_nix.get(TEMPLATE_FILE_NAME).unwrap();
        let nix_content = fs::read_to_string(&nix_file_path).unwrap();
        assert_eq!(nix_content, *expected_nix_content);

        // force enable to "false"
        let _ = config
            .app_option_changed(&OptionDataChangeNotification::Bool(
                crate::app_option_data::bool_data::BoolOptionChangeData {
                    id: NixBaseConfigOption::AllowUnfree.to_option_id(),
                    value: false,
                },
            ))
            .unwrap();
        let _ = config.save(work_dir);

        let json_content = fs::read_to_string(&json_file_path).unwrap();
        let expected_json_content = config.to_json_string().unwrap();
        assert_eq!(json_content, expected_json_content);

        let rendered_nix = config.render(NixBaseConfigsTemplates::Common).unwrap();
        let expected_nix_content = rendered_nix.get(TEMPLATE_FILE_NAME).unwrap();
        let nix_content = fs::read_to_string(nix_file_path).unwrap();
        assert_eq!(nix_content, *expected_nix_content);
    }

    #[test]
    fn test_default_config() {
        let config = NixBaseConfig::default();

        assert!(!config.allow_unfree.value());
        assert_eq!(config.time_zone.value(), "America/New_York");
        assert_eq!(config.default_locale.value(), "en_US.utf8");
        assert_eq!(config.username, "admin");
        assert!(!config.ssh_password_auth);
        assert_eq!(config.openssh_auth_keys.len(), 0);
    }

    #[test]
    fn test_render_valid_input() {
        let pw = unix_hash_password("testPW").unwrap();
        let templates = NixBaseConfigsTemplates::Common.files();

        let config = NixBaseConfig::new(
            Box::new(BoolOptionData::new(
                NixBaseConfigOption::AllowUnfree.to_option_id(),
                true,
            )),
            Box::new(StringListOptionData::new(
                NixBaseConfigOption::TimeZone.to_option_id(),
                "Europe/London".to_string(),
                TIMEZONES
                    .iter()
                    .map(|tz| StringListOptionItem::new(tz.to_string(), tz.to_string()))
                    .collect(),
            )),
            Box::new(StringListOptionData::new(
                NixBaseConfigOption::DefaultLocale.to_option_id(),
                "de_DE.utf8".to_string(),
                LOCALES
                    .iter()
                    .map(|tz| StringListOptionItem::new(tz.to_string(), tz.to_string()))
                    .collect(),
            )),
            "".to_string(),
            "myUserName".to_string(),
            true,
            Box::new(PasswordOptionData::new(
                NixBaseConfigOption::InitialPassword.to_option_id(),
                pw.to_string(),
                true,
                10,
                false,
                pw.to_string(),
            )),
            vec![String::from("123"), String::from("234")],
            vec![String::from("bat"), String::from("yazi")],
            vec![22, 1337],
            "nixblitzvm".to_string(),
            "nixblitzpi4".to_string(),
            "nixblitzpi5".to_string(),
            "nixblitzx86".to_string(),
        );

        let result = config.render(NixBaseConfigsTemplates::Common);
        assert!(result.is_ok());

        let texts = result.unwrap();
        #[allow(clippy::unnecessary_to_owned)]
        let res_base = texts.get(&templates.first().unwrap().to_string());
        assert!(res_base.is_some());
        let res_base = res_base.unwrap();
        assert!(res_base.contains(&format!(
            "nixpkgs.config.allowUnfree = {};",
            config.allow_unfree.value()
        )));
        assert!(res_base.contains(&format!(
            "time.timeZone = \"{}\";",
            config.time_zone.value()
        )));
        for key in config.openssh_auth_keys {
            assert!(res_base.contains(&format!("\"{}\"", key)));
        }
        assert!(res_base.contains(&format!(
            "i18n.defaultLocale = \"{}\";",
            config.default_locale.value()
        )));
        assert!(res_base.contains(&format!(
            "PasswordAuthentication = {};",
            config.ssh_password_auth
        )));
        assert!(res_base.contains(&format!(
            "hashedPassword = \"{}\";",
            &config.hashed_password.hashed_value()
        )));
        for pkg in config.system_packages {
            assert!(res_base.contains(&pkg.to_string()));
        }
        for port in config.ports {
            assert!(res_base.contains(&format!("{}", port)));
        }

        #[allow(clippy::unnecessary_to_owned)]
        let res_vm = texts.get(&templates.get(1).unwrap().to_string());
        assert!(res_vm.is_some());
        let res_vm = res_vm.unwrap();
        assert!(res_vm.contains(&format!(
            "networking.hostName = \"{}\";",
            config.hostname_vm
        )));

        #[allow(clippy::unnecessary_to_owned)]
        let res_pi4 = texts.get(&templates.get(2).unwrap().to_string());
        assert!(res_pi4.is_some());
        let res_pi = res_pi4.unwrap();
        assert!(res_pi.contains(&format!(
            "networking.hostName = \"{}\";",
            config.hostname_pi4
        )));

        #[allow(clippy::unnecessary_to_owned)]
        let res_pi5 = texts.get(&templates.get(3).unwrap().to_string());
        assert!(res_pi5.is_some());
        let res_pi = res_pi5.unwrap();
        assert!(res_pi.contains(&format!(
            "networking.hostName = \"{}\";",
            config.hostname_pi5
        )));

        #[allow(clippy::unnecessary_to_owned)]
        let res_x86 = texts.get(&templates.get(4).unwrap().to_string());
        assert!(res_x86.is_some());
        let res_x86 = res_x86.unwrap();
        println!("{res_x86}");
        assert!(res_x86.contains(&format!(
            "networking.hostName = \"{}\";",
            config.hostname_x86
        )));
    }

    #[test]
    fn test_nix_base_config_option_from_str_and_to_string() {
        let options = [
            NixBaseConfigOption::AllowUnfree,
            NixBaseConfigOption::TimeZone,
            NixBaseConfigOption::DefaultLocale,
            NixBaseConfigOption::Username,
            NixBaseConfigOption::InitialPassword,
        ];

        for &option in &options {
            let option_str = option.to_string();
            let parsed_option = NixBaseConfigOption::from_str(&option_str).unwrap();
            assert_eq!(option, parsed_option, "Failed for option: {:?}", option);
        }
    }
}
