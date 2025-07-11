use crossterm::event::{KeyEvent, MouseEvent};
use error_stack::Result;
use ratatui::{Frame, layout::Rect};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::Action,
    app_contexts::{RenderContext, UpdateContext},
    config::Config,
    errors::CliError,
    tui::Event,
};

pub mod app_list;
pub mod app_options;
pub mod default_theme;
pub mod list_options;
pub mod menu;
pub mod password_input;
pub mod theme;
pub mod title;

/// `Component` is a trait that represents a visual and interactive element of the user interface.
///
/// Implementors of this trait can be registered with the main application loop and will be able to
/// receive events, update state, and be rendered on the screen.
pub trait Component {
    /// Register an action handler that can send actions for processing if necessary.
    ///
    /// # Arguments
    ///
    /// * `tx` - An unbounded sender that can send actions.
    ///
    /// # Returns
    ///
    /// * `Result<(), CliError>` - An Ok result or an error.
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<(), CliError> {
        let _ = tx; // to appease clippy
        Ok(())
    }
    /// Register a configuration handler that provides configuration settings if necessary.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration settings.
    ///
    /// # Returns
    ///
    /// * `Result<(), TempGenericError>` - An Ok result or an error.
    fn register_config_handler(&mut self, config: Config) -> Result<(), CliError> {
        let _ = config; // to appease clippy
        Ok(())
    }
    /// Initialize the component with a specified area if necessary.
    ///
    /// # Arguments
    ///
    /// * `area` - Rectangular area to initialize the component within.
    ///
    /// # Returns
    ///
    /// * `Result<(), TempGenericError>` - An Ok result or an error.
    fn init(&mut self, area: Rect) -> Result<(), CliError> {
        let _ = area; // to appease clippy
        Ok(())
    }
    /// Handle incoming events and produce actions if necessary.
    ///
    /// # Arguments
    ///
    /// * `event` - An optional event to be processed.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Action>>` - An action to be processed or none.
    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>, CliError> {
        let action = match event {
            Some(Event::Key(key_event)) => self.handle_key_event(key_event)?,
            Some(Event::Mouse(mouse_event)) => self.handle_mouse_event(mouse_event)?,
            _ => None,
        };
        Ok(action)
    }
    /// Handle key events and produce actions if necessary.
    ///
    /// # Arguments
    ///
    /// * `key` - A key event to be processed.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Action>>` - An action to be processed or none.
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>, CliError> {
        let _ = key; // to appease clippy
        Ok(None)
    }
    /// Handle mouse events and produce actions if necessary.
    ///
    /// # Arguments
    ///
    /// * `mouse` - A mouse event to be processed.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Action>>` - An action to be processed or none.
    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Result<Option<Action>, CliError> {
        let _ = mouse; // to appease clippy
        Ok(None)
    }
    /// Update the state of the component based on a received action. (REQUIRED)
    ///
    /// # Arguments
    ///
    /// * `action` - An action that may modify the state of the component.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Action>>` - An action to be processed or none.
    fn update(&mut self, ctx: &UpdateContext) -> Result<Option<Action>, CliError> {
        // to appease clippy
        let _ = ctx;
        Ok(None)
    }
    /// Render the component on the screen. (REQUIRED)
    ///
    /// # Arguments
    ///
    /// * `area` - The area in which the component should be drawn.
    ///
    /// # Returns
    ///
    /// * `Result<(), TempGenericError>` - An Ok result or an error.
    fn draw(&mut self, frame: &mut Frame, area: Rect, ctx: &RenderContext) -> Result<(), CliError>;
}
