use error_stack::ResultExt;

use crate::{app::App, errors::CliError};

pub async fn start_gui(tick_rate: f64, frame_rate: f64) -> Result<(), CliError> {
    let app = App::new(tick_rate, frame_rate);
    let _ = app
        .expect("Unable to create the TUI app;")
        .run()
        .await
        .change_context(CliError::UnableToStartGui);

    Ok(())
}
