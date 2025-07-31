use std::{
    io, panic,
    path::Path,
    sync::{Arc, Mutex},
};

use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use error_stack::{Result, ResultExt};
use iocraft::prelude::*;
use log::info;
use tokio_tungstenite::connect_async;

use crate::{errors::CliError, tui_components::utils::load_or_create_project};

pub async fn connect_installer_engine() -> Result<(), CliError> {
    let url = "ws://127.0.0.1:3000/ws";
    let (ws_stream, _) = connect_async(url)
        .await
        .change_context(CliError::InstallerEngineUnreachable)?;

    // let (write, read) = ws_stream.split();

    Ok(())
}

pub async fn start_install_app(work_dir: &Path) -> Result<(), CliError> {
    let project = match load_or_create_project(work_dir, true).await? {
        Some(p) => p,
        None => return Ok(()),
    };

    let project = Arc::new(Mutex::new(project));

    connect_installer_engine().await?;
    info!("Connected to installer engine");

    fn restore_terminal() {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, Show);
    }

    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        restore_terminal();
        original_hook(panic_info);
    }));

    enable_raw_mode().change_context(CliError::UnableToStartTui)?;
    execute!(io::stdout(), EnterAlternateScreen, Hide)
        .change_context(CliError::UnableToStartTui)?;

    let result = tokio::task::spawn(async move {
        let _ = element! {
            ContextProvider(value: Context::owned(project)) {
                InstallerTuiApp
            }
        }
        .render_loop()
        .await;
    })
    .await;

    restore_terminal();

    if let Err(e) = result {
        eprintln!("Render loop panicked: {:?}", e);
        return Err(CliError::GenericError("Render loop failed".to_string()).into());
    }

    Ok(())
}

#[component]
fn InstallerTuiApp(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut show_help = hooks.use_state(|| false);
    let mut should_exit = hooks.use_state(|| false);

    hooks.use_terminal_events({
        move |event| match event {
            TerminalEvent::Key(KeyEvent { code, kind, .. }) if kind != KeyEventKind::Release => {
                match code {
                    KeyCode::Char('q') => should_exit.set(true),
                    KeyCode::Char('?') => {
                        if !show_help.get() {
                            show_help.set(true)
                        } else {
                            show_help.set(false)
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    });

    if should_exit.get() {
        system.exit();
    }

    element! {
        View(
        ) {
            Text(content: "NixBlitz Installer".to_string())
        }
    }
}
