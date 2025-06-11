use std::{collections::HashMap, path::Path, str::FromStr};

use alejandra::format;
use error_stack::{Report, Result, ResultExt};
use handlebars::{Handlebars, no_escape};
use nixblitz_core::option_definitions::blitz_api::{
    BlitzApiConfigOption, BlitzApiLogLevel, ConnectionType,
};
use serde::{Deserialize, Serialize};

use nixblitz_core::{
    app_config::AppConfig,
    app_option_data::{
        bool_data::BoolOptionData,
        option_data::{
            ApplicableOptionData, GetOptionId, OptionData, OptionDataChangeNotification,
            ToNixString, ToOptionId,
        },
        path_data::PathOptionData,
        string_list_data::{StringListOptionData, StringListOptionItem},
    },
    errors::{ProjectError, TemplatingError},
};

use crate::utils::{BASE_TEMPLATE, update_file};

pub const TEMPLATE_FILE_NAME: &str = "src/blitz/api.nix.templ";
pub const JSON_FILE_NAME: &str = "src/blitz/api.json";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct BlitzApiService {
    /// Whether the service is enabled or not
    pub enable: Box<BoolOptionData>,

    /// The connection type to use
    pub connection_type: Box<StringListOptionData>,

    /// Log level
    pub log_level: Box<StringListOptionData>,

    /// Whether to auto generate the env file
    pub generate_env_file: Box<BoolOptionData>,

    /// Where to write the env file
    pub env_file: Box<PathOptionData>,

    /// Where to write the password file
    pub password_file: Box<PathOptionData>,

    /// The root directory for Blitz API
    /// e.g. where the endpoint will be reachable:
    /// example: '/api' -> 'http://localhost:8080/api'
    pub root_path: Box<PathOptionData>,

    /// Whether to expose this service via nginx
    pub nginx_enable: Box<BoolOptionData>,

    /// Whether to open the filewall with the port
    pub nginx_open_firewall: Box<BoolOptionData>,

    /// Where to which path the service should be mounted to
    pub nginx_location: Box<PathOptionData>,
}

impl AppConfig for BlitzApiService {
    fn get_options(&self) -> Vec<OptionData> {
        vec![
            OptionData::Bool(self.enable.clone()),
            OptionData::StringList(Box::new(StringListOptionData::new(
                BlitzApiConfigOption::ConnectionType.to_option_id(),
                self.connection_type.value().to_string(),
                ConnectionType::to_string_array()
                    .map(|entry| StringListOptionItem::new(entry.to_string(), entry.to_string()))
                    .to_vec(),
            ))),
            OptionData::StringList(Box::new(StringListOptionData::new(
                BlitzApiConfigOption::LogLevel.to_option_id(),
                self.log_level.value().to_string(),
                BlitzApiLogLevel::to_string_array()
                    .map(|entry| StringListOptionItem::new(entry.to_string(), entry.to_string()))
                    .to_vec(),
            ))),
            OptionData::Bool(self.generate_env_file.clone()),
            OptionData::Path(self.env_file.clone()),
            OptionData::Path(self.password_file.clone()),
            OptionData::Path(self.root_path.clone()),
            OptionData::Bool(self.nginx_enable.clone()),
            OptionData::Bool(self.nginx_open_firewall.clone()),
            OptionData::Path(self.nginx_location.clone()),
        ]
    }

    fn app_option_changed(
        &mut self,
        option: &OptionDataChangeNotification,
    ) -> Result<bool, ProjectError> {
        let id = option.id();
        if let Ok(opt) = BlitzApiConfigOption::from_str(&id.option) {
            let mut res = Ok(false);
            if opt == BlitzApiConfigOption::Enable {
                if let OptionDataChangeNotification::Bool(val) = option {
                    res = Ok(self.enable.value() != val.value);
                    self.enable.set_value(val.value);
                } else {
                    return Err(Report::new(ProjectError::ChangeOptionValueError(
                        opt.to_string(),
                    )));
                }
            } else if opt == BlitzApiConfigOption::ConnectionType {
                if let OptionDataChangeNotification::StringList(val) = option {
                    res = Ok(self.connection_type.value() != val.value);
                    self.connection_type.set_value(val.value.clone());
                } else {
                    return Err(Report::new(ProjectError::ChangeOptionValueError(
                        opt.to_string(),
                    )));
                }
            } else if opt == BlitzApiConfigOption::LogLevel {
                if let OptionDataChangeNotification::StringList(val) = option {
                    res = Ok(self.log_level.value() != val.value);
                    self.log_level.set_value(val.value.clone());
                } else {
                    return Err(Report::new(ProjectError::ChangeOptionValueError(
                        opt.to_string(),
                    )));
                }
            } else if opt == BlitzApiConfigOption::GenerateEnvFile {
                if let OptionDataChangeNotification::Bool(val) = option {
                    res = Ok(self.generate_env_file.value() != val.value);
                    self.generate_env_file.set_value(val.value);
                } else {
                    return Err(Report::new(ProjectError::ChangeOptionValueError(
                        opt.to_string(),
                    )));
                }
            } else if opt == BlitzApiConfigOption::EnvFilePath {
                if let OptionDataChangeNotification::Path(val) = option {
                    res = Ok(self.env_file.value() != val.value);
                    self.env_file.set_value(val.value.clone());
                } else {
                    return Err(Report::new(ProjectError::ChangeOptionValueError(
                        opt.to_string(),
                    )));
                }
            } else if opt == BlitzApiConfigOption::PasswordFile {
                if let OptionDataChangeNotification::Path(val) = option {
                    res = Ok(self.password_file.value() != val.value);
                    self.password_file.set_value(val.value.clone());
                } else {
                    return Err(Report::new(ProjectError::ChangeOptionValueError(
                        opt.to_string(),
                    )));
                }
            } else if opt == BlitzApiConfigOption::RootPath {
                if let OptionDataChangeNotification::Path(val) = option {
                    res = Ok(self.root_path.value() != val.value);
                    self.root_path.set_value(val.value.clone());
                } else {
                    return Err(Report::new(ProjectError::ChangeOptionValueError(
                        opt.to_string(),
                    )));
                }
            } else if opt == BlitzApiConfigOption::NginxEnable {
                if let OptionDataChangeNotification::Bool(val) = option {
                    res = Ok(self.nginx_enable.value() != val.value);
                    self.nginx_enable.set_value(val.value);
                } else {
                    return Err(Report::new(ProjectError::ChangeOptionValueError(
                        opt.to_string(),
                    )));
                }
            } else if opt == BlitzApiConfigOption::NginxOpenFirewall {
                if let OptionDataChangeNotification::Bool(val) = option {
                    res = Ok(self.nginx_open_firewall.value() != val.value);
                    self.nginx_open_firewall.set_value(val.value);
                } else {
                    return Err(Report::new(ProjectError::ChangeOptionValueError(
                        opt.to_string(),
                    )));
                }
            } else if opt == BlitzApiConfigOption::NginxLocation {
                if let OptionDataChangeNotification::Path(val) = option {
                    res = Ok(self.nginx_location.value() != val.value);
                    self.nginx_location.set_value(val.value.clone());
                } else {
                    return Err(Report::new(ProjectError::ChangeOptionValueError(
                        opt.to_string(),
                    )));
                }
            }

            return res;
        };

        Ok(false)
    }

    fn save(&mut self, work_dir: &Path) -> Result<(), ProjectError> {
        let rendered_json = self
            .to_json_string()
            .change_context(ProjectError::GenFilesError)?;
        let rendered_nix = self.render().change_context(ProjectError::CreateBaseFiles(
            "Failed at rendering blitz api config".to_string(),
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
        self.enable.set_applied();
        self.nginx_enable.set_applied();
        self.log_level.set_applied();
        self.generate_env_file.set_applied();
        self.env_file.set_applied();
        self.password_file.set_applied();
        self.root_path.set_applied();
        self.nginx_enable.set_applied();
        self.nginx_open_firewall.set_applied();
        self.nginx_location.set_applied();
    }
}

impl Default for BlitzApiService {
    fn default() -> Self {
        Self {
            enable: Box::new(BoolOptionData::new(
                BlitzApiConfigOption::Enable.to_option_id(),
                false,
            )),
            connection_type: Box::new(StringListOptionData::new(
                BlitzApiConfigOption::ConnectionType.to_option_id(),
                ConnectionType::None.to_string(),
                ConnectionType::to_string_array()
                    .map(|entry| StringListOptionItem::new(entry.to_string(), entry.to_string()))
                    .to_vec(),
            )),
            log_level: Box::new(StringListOptionData::new(
                BlitzApiConfigOption::LogLevel.to_option_id(),
                BlitzApiLogLevel::Info.to_string(),
                BlitzApiLogLevel::to_string_array()
                    .map(|entry| StringListOptionItem::new(entry.to_string(), entry.to_string()))
                    .to_vec(),
            )),
            generate_env_file: Box::new(BoolOptionData::new(
                BlitzApiConfigOption::GenerateEnvFile.to_option_id(),
                true,
            )),
            env_file: Box::new(PathOptionData::default_from(
                BlitzApiConfigOption::EnvFilePath.to_option_id(),
                Some("/etc/blitz_api/env".to_string()),
            )),
            password_file: Box::new(PathOptionData::default_from(
                BlitzApiConfigOption::PasswordFile.to_option_id(),
                None,
            )),
            root_path: Box::new(PathOptionData::default_from(
                BlitzApiConfigOption::RootPath.to_option_id(),
                Some("/".to_string()),
            )),
            nginx_enable: Box::new(BoolOptionData::new(
                BlitzApiConfigOption::NginxEnable.to_option_id(),
                false,
            )),
            nginx_open_firewall: Box::new(BoolOptionData::new(
                BlitzApiConfigOption::NginxOpenFirewall.to_option_id(),
                false,
            )),
            nginx_location: Box::new(PathOptionData::default_from(
                BlitzApiConfigOption::NginxLocation.to_option_id(),
                Some("/api".to_string()),
            )),
        }
    }
}

impl BlitzApiService {
    pub fn render(&self) -> Result<HashMap<String, String>, TemplatingError> {
        let mut handlebars = Handlebars::new();
        handlebars.register_escape_fn(no_escape);

        let mut rendered_contents = HashMap::new();
        let file = BASE_TEMPLATE.get_file(TEMPLATE_FILE_NAME);
        let file = match file {
            Some(f) => f,
            None => {
                return Err(Report::new(TemplatingError::FileNotFound(
                    TEMPLATE_FILE_NAME.to_string(),
                ))
                .attach_printable(format!("File {TEMPLATE_FILE_NAME} not found in template")))?;
            }
        };

        let file = match file.contents_utf8() {
            Some(f) => f,
            None => {
                return Err(Report::new(TemplatingError::FileNotFound(
                    TEMPLATE_FILE_NAME.to_string(),
                ))
                .attach_printable(format!(
                    "Unable to read file contents of {TEMPLATE_FILE_NAME}"
                )));
            }
        };

        handlebars
            .register_template_string(TEMPLATE_FILE_NAME, file)
            .attach_printable_lazy(|| format!("{handlebars:?} could not register the template"))
            .change_context(TemplatingError::Register)?;

        let data: HashMap<&str, String> = HashMap::from([
            ("enable", self.enable.value().to_string()),
            ("connection_type", self.connection_type.to_nix_string(true)),
            ("log_level", self.log_level.to_nix_string(true)),
            (
                "generate_env_file",
                self.generate_env_file.value().to_string(),
            ),
            ("env_file", self.env_file.to_nix_string(true)),
            ("password_file", self.password_file.to_nix_string(true)),
            ("root_path", self.root_path.to_nix_string(true)),
            ("nginx_enable", format!("{}", self.nginx_enable.value())),
            (
                "nginx_open_firewall",
                format!("{}", self.nginx_open_firewall.value()),
            ),
            ("nginx_location", self.nginx_location.to_nix_string(true)),
        ]);

        let res = handlebars
            .render(TEMPLATE_FILE_NAME, &data)
            .attach_printable("Failed to render Blitz API template".to_string())
            .change_context(TemplatingError::Render)?;
        let (status, text) = format::in_memory("<blitz_api>".to_string(), res);

        if let format::Status::Error(e) = status {
            Err(Report::new(TemplatingError::Format))
                .attach_printable_lazy(|| text)
                .attach_printable_lazy(|| {
                    format!("Could not format the template file due to error: {e}")
                })?
        } else {
            rendered_contents.insert(TEMPLATE_FILE_NAME.to_string(), text);
        }

        Ok(rendered_contents)
    }

    pub(crate) fn to_json_string(&self) -> Result<String, TemplatingError> {
        serde_json::to_string_pretty(self).change_context(TemplatingError::JsonRenderError)
    }

    pub(crate) fn from_json(json_data: &str) -> Result<BlitzApiService, TemplatingError> {
        serde_json::from_str(json_data).change_context(TemplatingError::JsonLoadError)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use tempfile::tempdir;

    use crate::utils::{init_default_project, trim_lines_left};
    use nixblitz_core::{
        app_option_data::option_data::ToNixString,
        option_definitions::blitz_api::{BlitzApiLogLevel, ConnectionType},
    };

    use super::*;

    fn get_test_service() -> BlitzApiService {
        BlitzApiService {
            enable: Box::new(BoolOptionData::new(
                BlitzApiConfigOption::Enable.to_option_id(),
                true,
            )),
            connection_type: Box::new(StringListOptionData::new(
                BlitzApiConfigOption::ConnectionType.to_option_id(),
                ConnectionType::ClnJrpc.to_string(),
                ConnectionType::to_string_array()
                    .map(|entry| StringListOptionItem::new(entry.to_string(), entry.to_string()))
                    .to_vec(),
            )),
            log_level: Box::new(StringListOptionData::new(
                BlitzApiConfigOption::LogLevel.to_option_id(),
                BlitzApiLogLevel::Info.to_string(),
                BlitzApiLogLevel::to_string_array()
                    .map(|entry| StringListOptionItem::new(entry.to_string(), entry.to_string()))
                    .to_vec(),
            )),
            generate_env_file: Box::new(BoolOptionData::new(
                BlitzApiConfigOption::GenerateEnvFile.to_option_id(),
                true,
            )),
            env_file: Box::new(PathOptionData::default_from(
                BlitzApiConfigOption::EnvFilePath.to_option_id(),
                Some("/etc/blitz_api/env".to_string()),
            )),
            password_file: Box::new(PathOptionData::default_from(
                BlitzApiConfigOption::PasswordFile.to_option_id(),
                Some("/etc/blitz_api/password".to_string()),
            )),
            root_path: Box::new(PathOptionData::default_from(
                BlitzApiConfigOption::RootPath.to_option_id(),
                Some("/api".to_string()),
            )),
            nginx_enable: Box::new(BoolOptionData::new(
                BlitzApiConfigOption::NginxEnable.to_option_id(),
                false,
            )),
            nginx_open_firewall: Box::new(BoolOptionData::new(
                BlitzApiConfigOption::NginxOpenFirewall.to_option_id(),
                false,
            )),
            nginx_location: Box::new(PathOptionData::default_from(
                BlitzApiConfigOption::NginxLocation.to_option_id(),
                Some("/".to_string()),
            )),
        }
    }

    #[test]
    fn test_save_function() {
        // Note: maybe test every field? Right now we just check if
        //       enable is set to true or false respectively

        // Create a temporary directory
        let temp_dir = tempdir().unwrap();
        let work_dir = temp_dir.path();

        let _ = init_default_project(work_dir, Some(false));

        let mut service = get_test_service();
        // force enable to "true"
        let _ = service
            .app_option_changed(&OptionDataChangeNotification::Bool(
                nixblitz_core::app_option_data::bool_data::BoolOptionChangeData {
                    id: BlitzApiConfigOption::Enable.to_option_id(),
                    value: true,
                },
            ))
            .unwrap();

        // Call the save function
        let result = service.save(work_dir);

        // Assert that the save function returns Ok
        assert!(result.is_ok());

        let json_file_path = work_dir.join(JSON_FILE_NAME);
        // Check that the JSON file contains the expected content
        let json_content = fs::read_to_string(&json_file_path).unwrap();
        let expected_json_content = service.to_json_string().unwrap();
        assert_eq!(json_content, expected_json_content);

        // Check that the Nix file contains the expected content
        let nix_file_path = work_dir.join(TEMPLATE_FILE_NAME.replace(".templ", ""));
        let rendered_nix = service.render().unwrap();
        let expected_nix_content = rendered_nix.get(TEMPLATE_FILE_NAME).unwrap();
        let nix_content = fs::read_to_string(&nix_file_path).unwrap();
        assert_eq!(nix_content, *expected_nix_content);

        // force enable to "false"
        let _ = service
            .app_option_changed(&OptionDataChangeNotification::Bool(
                nixblitz_core::app_option_data::bool_data::BoolOptionChangeData {
                    id: BlitzApiConfigOption::Enable.to_option_id(),
                    value: false,
                },
            ))
            .unwrap();
        let _ = service.save(work_dir);

        let json_content = fs::read_to_string(&json_file_path).unwrap();
        let expected_json_content = service.to_json_string().unwrap();
        assert_eq!(json_content, expected_json_content);

        let rendered_nix = service.render().unwrap();
        let expected_nix_content = rendered_nix.get(TEMPLATE_FILE_NAME).unwrap();
        let nix_content = fs::read_to_string(nix_file_path).unwrap();
        assert_eq!(nix_content, *expected_nix_content);
    }

    #[test]
    fn test_render() {
        let s = get_test_service();

        let result = s.render();
        if let Ok(data) = &result {
            assert!(&data.contains_key(TEMPLATE_FILE_NAME));
            let data = &data[TEMPLATE_FILE_NAME];
            println!("{}", data);
            assert!(data.contains(&format!("enable = {};", s.enable.value())));
            let text = &format!(
                "ln.connectionType = \"{}\";",
                s.connection_type.to_nix_string(false)
            );
            assert!(data.contains(text));
            assert!(data.contains(&format!(
                "logLevel = \"{}\";",
                s.log_level.to_nix_string(false)
            )));
            assert!(data.contains(&format!(
                "generateDotEnvFile = {};",
                s.generate_env_file.to_nix_string(false)
            )));
            assert!(data.contains(&format!(
                "dotEnvFile = \"{}\";",
                s.env_file.to_nix_string(false)
            )));
            assert!(data.contains(&format!(
                "passwordFile = \"{}\";",
                s.password_file.to_nix_string(false)
            )));
            assert!(data.contains(&format!(
                "rootPath = \"{}\";",
                s.root_path.to_nix_string(false)
            )));
            let data = trim_lines_left(data);
            println!("{}", data);
            let text = trim_lines_left(&format!(
                r#"
                nginx = {{
                  enable = {};
                  openFirewall = {};
                  location = "{}";
                }};"#,
                s.nginx_enable.value(),
                s.nginx_open_firewall.value(),
                s.nginx_location.to_nix_string(false),
            ));
            println!("{}", text);
            assert!(data.contains(&text));
        } else if let Err(e) = result {
            let msg = e.to_string();
            panic!("{msg}");
        }
    }
}
