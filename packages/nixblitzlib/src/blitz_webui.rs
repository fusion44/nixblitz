use core::fmt;
use std::{collections::HashMap, path::Path, str::FromStr};

use alejandra::format;
use error_stack::{Report, Result, ResultExt};
use handlebars::{no_escape, Handlebars};
use serde::{Deserialize, Serialize};

use crate::{
    app_config::AppConfig,
    app_option_data::{
        bool_data::BoolOptionData,
        option_data::{
            GetOptionId, OptionData, OptionDataChangeNotification, OptionId, ToOptionId,
        },
    },
    apps::SupportedApps,
    errors::{ProjectError, TemplatingError},
    utils::{update_file, BASE_TEMPLATE},
};

pub const TEMPLATE_FILE_NAME: &str = "src/blitz/web.nix.templ";
pub const JSON_FILE_NAME: &str = "src/blitz/web.json";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct BlitzWebUiService {
    /// Whether the service is enabled or not
    pub enable: Box<BoolOptionData>,

    /// Whether to expose this service via nginx
    pub nginx_enable: Box<BoolOptionData>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BlitzWebUiConfigOption {
    Enable,
    NginxEnable,
}

impl ToOptionId for BlitzWebUiConfigOption {
    fn to_option_id(&self) -> OptionId {
        OptionId::new(SupportedApps::WebUI, self.to_string())
    }
}
impl FromStr for BlitzWebUiConfigOption {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<BlitzWebUiConfigOption, ()> {
        match s {
            "enable" => Ok(BlitzWebUiConfigOption::Enable),
            "nginx_enable" => Ok(BlitzWebUiConfigOption::NginxEnable),
            _ => Err(()),
        }
    }
}

impl fmt::Display for BlitzWebUiConfigOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let option_str = match self {
            BlitzWebUiConfigOption::Enable => "enable",
            BlitzWebUiConfigOption::NginxEnable => "nginx_enable",
        };
        write!(f, "{}", option_str)
    }
}

impl AppConfig for BlitzWebUiService {
    fn get_options(&self) -> Vec<OptionData> {
        vec![
            OptionData::Bool(self.enable.clone()),
            OptionData::Bool(self.nginx_enable.clone()),
        ]
    }

    fn app_option_changed(
        &mut self,
        option: &OptionDataChangeNotification,
    ) -> Result<bool, ProjectError> {
        let id = option.id();
        let mut res = Ok(false);
        if let Ok(opt) = BlitzWebUiConfigOption::from_str(&id.option) {
            match opt {
                BlitzWebUiConfigOption::Enable => {
                    if let OptionDataChangeNotification::Bool(val) = option {
                        res = Ok(self.enable.value() != val.value);
                        self.enable.set_value(val.value);
                    } else {
                        return Err(Report::new(ProjectError::ChangeOptionValueError(
                            opt.to_string(),
                        )));
                    }
                }
                BlitzWebUiConfigOption::NginxEnable => {
                    if let OptionDataChangeNotification::Bool(val) = option {
                        res = Ok(self.nginx_enable.value() != val.value);
                        self.nginx_enable.set_value(val.value);
                    } else {
                        return Err(Report::new(ProjectError::ChangeOptionValueError(
                            opt.to_string(),
                        )));
                    }
                }
            }

            return res;
        }

        res
    }

    fn save(&mut self, work_dir: &Path) -> Result<(), ProjectError> {
        let rendered_json = self
            .to_json_string()
            .change_context(ProjectError::GenFilesError)?;
        let rendered_nix = self.render().change_context(ProjectError::CreateBaseFiles(
            "Failed at rendering blitz webui config".to_string(),
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
}

impl Default for BlitzWebUiService {
    fn default() -> Self {
        Self {
            enable: Box::new(BoolOptionData::new(
                BlitzWebUiConfigOption::Enable.to_option_id(),
                false,
            )),
            nginx_enable: Box::new(BoolOptionData::new(
                BlitzWebUiConfigOption::NginxEnable.to_option_id(),
                false,
            )),
        }
    }
}

impl BlitzWebUiService {
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
                .attach_printable(format!("File {TEMPLATE_FILE_NAME} not found in template")))?
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
                )))
            }
        };

        handlebars
            .register_template_string(TEMPLATE_FILE_NAME, file)
            .attach_printable_lazy(|| format!("{handlebars:?} could not register the template"))
            .change_context(TemplatingError::Register)?;

        let data: HashMap<&str, String> = HashMap::from([
            ("enable", format!("{}", self.enable.value())),
            ("nginx_enable", format!("{}", self.nginx_enable.value())),
        ]);

        let res = handlebars
            .render(TEMPLATE_FILE_NAME, &data)
            .attach_printable("Failed to render Blitz Web UI template".to_string())
            .change_context(TemplatingError::Render)?;
        let (status, text) = format::in_memory("<blitz_webui>".to_string(), res);

        if let format::Status::Error(e) = status {
            Err(Report::new(TemplatingError::Format)).attach_printable_lazy(|| {
                format!("Could not format the template file due to error: {e}")
            })?
        } else {
            rendered_contents.insert(TEMPLATE_FILE_NAME.to_string(), text);
        }

        Ok(rendered_contents)
    }

    pub(crate) fn to_json_string(&self) -> Result<String, TemplatingError> {
        serde_json::to_string(self).change_context(TemplatingError::JsonRenderError)
    }

    pub(crate) fn from_json(json_data: &str) -> Result<BlitzWebUiService, TemplatingError> {
        serde_json::from_str(json_data).change_context(TemplatingError::JsonLoadError)
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::{init_default_project, trim_lines_left};

    use std::fs;
    use tempfile::tempdir;

    use super::*;

    fn get_test_service() -> BlitzWebUiService {
        BlitzWebUiService {
            enable: Box::new(BoolOptionData::new(
                BlitzWebUiConfigOption::Enable.to_option_id(),
                true,
            )),
            nginx_enable: Box::new(BoolOptionData::new(
                BlitzWebUiConfigOption::NginxEnable.to_option_id(),
                false,
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
        let mut config = get_test_service();

        // force enable to "true"
        let _ = config
            .app_option_changed(&OptionDataChangeNotification::Bool(
                crate::app_option_data::bool_data::BoolOptionChangeData {
                    id: BlitzWebUiConfigOption::Enable.to_option_id(),
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
        let rendered_nix = config.render().unwrap();
        let expected_nix_content = rendered_nix.get(TEMPLATE_FILE_NAME).unwrap();
        let nix_content = fs::read_to_string(&nix_file_path).unwrap();
        assert_eq!(nix_content, *expected_nix_content);

        // force enable to "false"
        let _ = config
            .app_option_changed(&OptionDataChangeNotification::Bool(
                crate::app_option_data::bool_data::BoolOptionChangeData {
                    id: BlitzWebUiConfigOption::Enable.to_option_id(),
                    value: false,
                },
            ))
            .unwrap();
        let _ = config.save(work_dir);

        let json_content = fs::read_to_string(&json_file_path).unwrap();
        let expected_json_content = config.to_json_string().unwrap();
        assert_eq!(json_content, expected_json_content);

        let rendered_nix = config.render().unwrap();
        let expected_nix_content = rendered_nix.get(TEMPLATE_FILE_NAME).unwrap();
        let nix_content = fs::read_to_string(nix_file_path).unwrap();
        assert_eq!(nix_content, *expected_nix_content);
    }

    #[test]
    fn test_render() {
        let s = get_test_service();

        let result = s.render();
        if let Ok(data) = &result {
            println!("{}", data[TEMPLATE_FILE_NAME]);
            assert!(&data.contains_key(TEMPLATE_FILE_NAME));
            let data = &data[TEMPLATE_FILE_NAME];
            assert!(data.contains(&format!("enable = {};", s.enable.value())));

            let text = trim_lines_left(&format!(
                r#"
                    enable = {};
                    nginx = {{
                      enable = {};
                    }};
                "#,
                s.enable.value(),
                s.nginx_enable.value()
            ));

            let data = trim_lines_left(data);
            assert!(data.contains(&text));
        }

        assert!(result.is_ok());
    }
}
