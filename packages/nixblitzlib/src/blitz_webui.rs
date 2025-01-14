use core::fmt;
use std::{collections::HashMap, str::FromStr};

use alejandra::format;
use error_stack::{Report, Result, ResultExt};
use handlebars::{no_escape, Handlebars};
use serde::{Deserialize, Serialize};

use crate::{
    app_option_data::{
        bool_data::BoolOptionData,
        option_data::{OptionData, OptionDataChangeNotification, OptionId, ToOptionId},
    },
    apps::SupportedApps,
    errors::{ProjectError, TemplatingError},
    utils::BASE_TEMPLATE,
};

pub const TEMPLATE_FILE_NAME: &str = "src/apps/blitz_web.nix.templ";
pub const JSON_FILE_NAME: &str = "src/apps/blitz_web.json";

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

impl BlitzWebUiService {
    pub fn get_options(&self) -> Vec<OptionData> {
        vec![
            OptionData::Bool(self.enable.clone()),
            OptionData::Bool(self.nginx_enable.clone()),
        ]
    }

    pub fn app_option_changed(
        &mut self,
        id: &OptionId,
        option: &OptionDataChangeNotification,
    ) -> Result<bool, ProjectError> {
        if let Ok(opt) = BlitzWebUiConfigOption::from_str(&id.option) {
            let mut res = Ok(false);
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

        Ok(false)
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
    use crate::utils::trim_lines_left;

    use super::*;

    fn get_test_daemon() -> BlitzWebUiService {
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
    fn test_render() {
        let s = get_test_daemon();

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
