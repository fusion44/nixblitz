use common::app_option_data::option_data::{
    GetOptionId, OptionData, OptionDataChangeNotification, OptionId,
};
use dioxus::prelude::*;

#[cfg(feature = "server")]
fn get_project() -> Result<nixblitzlib::project::Project, ServerFnError> {
    use common::{constants::NIXBLITZ_WORK_DIR_ENV, errors::ProjectError};
    use dioxus_logger::tracing::{error, info};
    use nixblitzlib::project::Project;
    use std::{env, path::PathBuf};

    let work_dir = env::var(NIXBLITZ_WORK_DIR_ENV)?;
    let mut p = Project::load(PathBuf::from(work_dir.clone()));

    match p {
        Ok(p) => Ok(p),
        Err(e) => {
            info!("Loaded project from {}", work_dir);
            return Err(ServerFnError::ServerError(e.to_string()));
        }
    }
}

// This function is used to get the list of supported apps from the server.
// It is compiled into the WASM client
#[server]
pub(crate) async fn get_supported_apps_wrapper() -> Result<Vec<String>, ServerFnError> {
    get_supported_apps().await
}

// This is the function that interacts with the nixblitzlib.
// This one is compiled on the server only. nixblitzlib will not be available
// on the WASM client. This is the reason why we need to addredd the
// lib with its full path (nixblitzlib::apps::SupportedApps::as_string_list())
#[cfg(feature = "server")]
async fn get_supported_apps() -> Result<Vec<String>, ServerFnError> {
    use common::apps::SupportedApps;

    Ok(SupportedApps::as_string_list()
        .iter()
        .map(|s| s.to_string())
        .collect())
}

#[server]
pub(crate) async fn get_app_options_wrapper(app: String) -> Result<Vec<OptionData>, ServerFnError> {
    get_app_options(app).await
}

#[cfg(feature = "server")]
async fn get_app_options(app: String) -> Result<Vec<OptionData>, ServerFnError> {
    use common::{
        app_option_data::option_data::{GetOptionId, OptionData},
        apps::SupportedApps,
        errors::ProjectError,
    };
    use error_stack::{Report, ResultExt};
    use nixblitzlib::project::Project;
    use std::{env, path::PathBuf, rc::Rc};

    let mut p = get_project()?;
    p.set_selected_app(SupportedApps::from(app.as_str()).unwrap());
    let o: Result<Rc<Vec<OptionData>>, Report<ProjectError>> = p.get_app_options();
    match o {
        Ok(o) => Ok(o.to_vec()),
        Err(e) => Err(ServerFnError::ServerError(e.to_string())),
    }
}

#[server]
pub(crate) async fn set_app_option_wrapper(
    n: OptionDataChangeNotification,
) -> Result<(), ServerFnError> {
    set_app_options(n).await
}

#[cfg(feature = "server")]
pub async fn set_app_options(n: OptionDataChangeNotification) -> Result<(), ServerFnError> {
    use nixblitzlib::project::Project;
    use std::path::PathBuf;

    let mut p = get_project()?;
    p.set_selected_app(n.id().app);
    let res = p.on_option_changed(n);

    Ok(())
}
