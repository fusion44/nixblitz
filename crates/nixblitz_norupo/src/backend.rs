use dioxus::prelude::*;
use nixblitz_core::app_option_data::option_data::{OptionData, OptionDataChangeNotification};

#[cfg(feature = "server")]
fn get_project() -> Result<nixblitz_system::project::Project, ServerFnError> {
    use dioxus_logger::tracing::{error, info};
    use nixblitz_core::{constants::NIXBLITZ_WORK_DIR_ENV, errors::ProjectError};
    use nixblitz_system::project::Project;
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

// This is the function that interacts with the nixblitz_system.
// This one is compiled on the server only. nixblitz_system will not be available
// on the WASM client. This is the reason why we need to addredd the
// lib with its full path (nixblitz_system::apps::SupportedApps::as_string_list())
#[cfg(feature = "server")]
async fn get_supported_apps() -> Result<Vec<String>, ServerFnError> {
    use nixblitz_core::apps::SupportedApps;

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
    use error_stack::{Report, ResultExt};
    use nixblitz_core::{
        app_option_data::option_data::{GetOptionId, OptionData},
        apps::SupportedApps,
        errors::ProjectError,
    };
    use nixblitz_system::project::Project;
    use std::{env, path::PathBuf, sync::Arc};

    let mut p = get_project()?;
    p.set_selected_app(SupportedApps::from(app.as_str()).unwrap());
    let o: Result<Arc<Vec<OptionData>>, Report<ProjectError>> = p.get_app_options();
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
    use nixblitz_core::option_data::GetOptionId;
    use nixblitz_system::project::Project;
    use std::path::PathBuf;

    let mut p = get_project()?;
    p.set_selected_app(n.id().app);
    let res = p.on_option_changed(n);

    Ok(())
}
