use alejandra::format;
use error_stack::{Report, Result, ResultExt};
use handlebars::{no_escape, Handlebars};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};

use crate::{errors::TemplatingError, utils::BASE_TEMPLATE};

#[derive(Debug, Serialize, Deserialize)]
pub struct NixBaseConfig {
    /// Whether to allow unfree packages from nixpkgs
    pub allow_unfree: bool,

    /// The timezone that should be used for this system
    ///
    /// [nixos.org:time.timeZone](https://search.nixos.org/options?show=time.timeZone)
    ///
    /// Default: "America/New_York"
    ///
    /// Example: "Europe/Copenhagen"
    pub time_zone: String,

    /// The default locale. It determines the language for program
    /// messages, the format for dates and times, sort order, and so on.
    /// It also determines the character set, such as UTF-8.
    ///
    /// [nixos.org:i18n.defaultLocale](https://search.nixos.org/options?show=i18n.defaultLocale)
    ///
    /// Default: "en_US.UTF-8"
    ///
    /// Example: "nl_NL.UTF-8"
    pub default_locale: String,

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
    pub hashed_password: String,

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
}

impl Default for NixBaseConfig {
    fn default() -> Self {
        Self {
            allow_unfree: false,
            time_zone: String::from("America/New_York"),
            default_locale: String::from("en_US.UTF-8"),
            username: String::from("admin"),
            ssh_password_auth: false,
            // default password: "nixblitz"
            hashed_password: "$6$rounds=10000$moY2rIPxoNODYRxz$1DESwWYweHNkoB6zBxI3DUJwUfvA6UkZYskLOHQ9ulxItgg/hP5CRn2Fr4iQGO7FE16YpJAPMulrAuYJnRC9B.".into(),
            openssh_auth_keys: vec![],
            system_packages: vec![
                String::from("bat"),
                String::from("bottom"),
                String::from("fzf"),
                String::from("git"),
                String::from("neovim"),
                String::from("ripgrep"),
                String::from("bandwhich"),
            ],
        }
    }
}

#[derive(Debug)]
pub enum NixBaseConfigsTemplates {
    /// configuration.common.nix.templ
    Common,
}

impl NixBaseConfigsTemplates {
    fn file(&self) -> &str {
        match self {
            NixBaseConfigsTemplates::Common => "src/configuration.common.nix.templ",
        }
    }
}

impl Display for NixBaseConfigsTemplates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NixBaseConfigsTemplates::Common => f.write_str(NixBaseConfigsTemplates::Common.file()),
        }
    }
}

impl NixBaseConfig {
    #![allow(clippy::too_many_arguments)]
    pub fn new(
        allow_unfree: bool,
        time_zone: String,
        default_locale: String,
        username: String,
        ssh_password_auth: bool,
        hashed_password: String,
        openssh_auth_keys: Vec<String>,
        system_packages: Vec<String>,
    ) -> Self {
        Self {
            allow_unfree,
            time_zone,
            default_locale,
            username,
            ssh_password_auth,
            hashed_password,
            openssh_auth_keys,
            system_packages,
        }
    }

    pub fn render(&self, template: NixBaseConfigsTemplates) -> Result<String, TemplatingError> {
        let mut handlebars = Handlebars::new();
        handlebars.register_escape_fn(no_escape);

        let file = match template {
            NixBaseConfigsTemplates::Common => BASE_TEMPLATE.get_file(template.file()),
        };

        let file = match file {
            Some(f) => f,
            None => {
                return Err(Report::new(TemplatingError::FileNotFound)
                    .attach_printable(format!("File for {template} not found")))
            }
        };

        let file = match file.contents_utf8() {
            Some(f) => f,
            None => {
                return Err(Report::new(TemplatingError::FileNotFound)
                    .attach_printable(format!("Unable to read file contents of {template}")))
            }
        };

        handlebars
            .register_template_string("nix_base_cfg", file)
            .attach_printable_lazy(|| format!("{handlebars:?} could not register the template"))
            .change_context(TemplatingError::Register)?;

        let data = HashMap::from([
            ("allow_unfree", format!("{}", self.allow_unfree)),
            ("time_zone", self.time_zone.clone()),
            ("default_locale", self.default_locale.clone()),
            ("username", self.username.clone()),
            ("ssh_password_auth", format!("{}", self.ssh_password_auth)),
            ("initial_password", self.hashed_password.clone()),
            (
                "openssh_auth_keys",
                self.openssh_auth_keys
                    .iter()
                    .map(|s| format!("\"{}\"", s))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            ("system_packages", self.system_packages.join(" ")),
        ]);

        let res = handlebars
            .render("nix_base_cfg", &data)
            .attach_printable(format!("Failed to render template {template}"))
            .change_context(TemplatingError::Render)?;

        let (status, text) = format::in_memory("<convert nix base>".to_string(), res);
        match status {
            format::Status::Error(e) => Err(Report::new(TemplatingError::Format))
                .attach_printable_lazy(|| {
                    format!("Could not format the template file due to error: {e}")
                }),
            _ => Ok(text),
        }
    }

    pub(crate) fn to_json_string(&self) -> Result<String, TemplatingError> {
        serde_json::to_string(self).change_context(TemplatingError::JsonRenderError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::unix_hash_password;

    #[test]
    fn test_default_config() {
        let config = NixBaseConfig::default();

        assert!(!config.allow_unfree);
        assert_eq!(config.time_zone, "America/New_York");
        assert_eq!(config.default_locale, "en_US.UTF-8");
        assert_eq!(config.username, "admin");
        assert!(!config.ssh_password_auth);
        assert_eq!(config.openssh_auth_keys.len(), 0);
        assert_eq!(config.system_packages.len(), 7);
    }

    #[test]
    fn test_render_valid_input() {
        let pw = unix_hash_password("testPW").unwrap();
        let valid_input_res: String = String::from(
            "{pkgs, ...}: {
  nixpkgs.config.allowUnfree = true;
  time.timeZone = \"Europe/London\";
  i18n.defaultLocale = \"de_DE.UTF-8\";

  console = {
    font = \"Lat2-Terminus16\";
    useXkbConfig = true; # use xkb.options in tty.
  };

  users = {
    defaultUserShell = pkgs.nushell;
    users.myUserName = {
      isNormalUser = true;
      extraGroups = [\"wheel\"]; # Enable ‘sudo’ for the user.
      hashedPassword = \"testPWhere\";
      openssh.authorizedKeys.keys = [
        \"123\"
        \"234\"
      ];
    };
  };

  environment.systemPackages = with pkgs; [
    bat
    yazi
  ];

  services = {
    openssh = {
      enable = true;
      ports = [22];
      settings = {
        PasswordAuthentication = true;
        AllowUsers = [\"myUserName\"];
        UseDns = true;
        X11Forwarding = false;
        PermitRootLogin = \"prohibit-password\";
      };
    };

    redis.servers.\"\".enable = true;
  };

  networking.firewall.allowedTCPPorts = [22];
}
",
        );
        let valid_input_res = valid_input_res.replace("testPWhere", &pw);

        let config = NixBaseConfig {
            allow_unfree: true,
            time_zone: "Europe/London".into(),
            openssh_auth_keys: vec![String::from("123"), String::from("234")],
            default_locale: String::from("de_DE.UTF-8"),
            username: String::from("myUserName"),
            ssh_password_auth: true,
            hashed_password: pw,
            system_packages: vec![String::from("bat"), String::from("yazi")],
        };

        let result = config.render(NixBaseConfigsTemplates::Common);
        assert!(result.is_ok());

        let text = result.unwrap();
        assert_eq!(text.trim(), valid_input_res.trim());
    }
}
