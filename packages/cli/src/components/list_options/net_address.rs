use std::{net::IpAddr, str::FromStr};

use error_stack::{Report, Result, ResultExt};
use nixblitzlib::{
    app_option_data::{
        net_address_data::{NetAddressOptionChangeData, NetAddressOptionData},
        option_data::{GetOptionId, OptionDataChangeNotification},
    },
    strings::OPTION_TITLES,
};
use ratatui::{layout::Rect, Frame};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::Action,
    app_contexts::{RenderContext, UpdateContext},
    components::Component,
    errors::CliError,
};

use super::{
    base_option::{draw_item, OptionListItem},
    text_popup::TextInputPopup,
};

#[derive(Debug)]
pub struct NetAddressOptionComponent<'a> {
    data: NetAddressOptionData,
    title: &'a str,
    subtitle: String,
    selected: bool,
    editing: bool,
    action_tx: Option<UnboundedSender<Action>>,
    popup: Option<Box<TextInputPopup<'a>>>,
}

impl<'a> NetAddressOptionComponent<'a> {
    pub fn new(data: &NetAddressOptionData, selected: bool) -> Result<Self, CliError> {
        let title = OPTION_TITLES
            .get(data.id())
            .ok_or(CliError::OptionTitleRetrievalError(data.id().to_string()))?;

        let mut i = Self {
            data: data.clone(),
            title,
            subtitle: "".into(),
            selected,
            editing: false,
            action_tx: None,
            popup: None,
        };
        i.update_subtitle();

        Ok(i)
    }

    fn reset_popup(&mut self) {
        self.popup = None;
    }

    fn build_popup(&mut self) -> Result<(), CliError> {
        // TODO: implement a popup
        let val = if let Some(v) = self.data.value() {
            v.to_string()
        } else {
            "".to_string()
        };

        let mut pop = TextInputPopup::new(self.title, vec![val], 1)?;
        if let Some(h) = &self.action_tx {
            pop.register_action_handler(h.clone())?;
        }
        self.popup = Some(Box::new(pop));

        Ok(())
    }

    fn update_subtitle(&mut self) {
        if let Some(val) = self.data.value() {
            self.subtitle = val.to_string();
        } else {
            self.subtitle = "".to_string();
        }
    }

    pub fn set_data(&mut self, data: &NetAddressOptionData) {
        self.data = data.clone();
    }
}

impl<'a> OptionListItem for NetAddressOptionComponent<'a> {
    fn selected(&self) -> bool {
        self.selected
    }

    fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    fn is_dirty(&self) -> bool {
        self.data.dirty()
    }

    fn on_edit(&mut self) -> std::result::Result<(), Report<CliError>> {
        if !self.editing {
            self.editing = !self.editing;
            self.build_popup()?;
            if let Some(tx) = &self.action_tx {
                let _ = tx.send(Action::PushModal(true));
            }
        }

        Ok(())
    }
}

impl<'a> Component for NetAddressOptionComponent<'a> {
    fn update(&mut self, ctx: &UpdateContext) -> Result<Option<Action>, CliError> {
        if ctx.action == Action::Esc && self.editing {
            if let Some(ref mut p) = self.popup {
                p.update(ctx)?;
            }
        } else if ctx.action == Action::PopModal(true) && self.editing {
            self.editing = false;
            if let Some(ref mut p) = self.popup {
                let val = p.get_result()[0].clone();
                let net_addr = if val.is_empty() {
                    None
                } else {
                    match IpAddr::from_str(&val) {
                        Ok(res) => Some(res),
                        Err(e) => Err(CliError::StringParseError(e.to_string()))
                            .attach_printable_lazy(|| {
                                format!("Unable to parse IP address from String: {}", val)
                            })?,
                    }
                };

                self.data.set_value(net_addr);
                self.update_subtitle();

                if let Some(tx) = &self.action_tx {
                    tx.send(Action::AppTabOptionChangeProposal(
                        OptionDataChangeNotification::NetAddress(NetAddressOptionChangeData::new(
                            self.data.id().clone(),
                            self.data.value(),
                        )),
                    ))
                    .change_context(CliError::Unknown)?
                }
            }

            self.update_subtitle();
            self.reset_popup();
        } else if ctx.action == Action::PopModal(false) && self.editing {
            self.editing = false;
            self.reset_popup();
        }

        Ok(None)
    }

    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<(), CliError> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn handle_key_event(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> Result<Option<Action>, CliError> {
        if !self.editing {
            return Ok(None);
        }

        if let Some(ref mut p) = self.popup {
            return p.handle_key_event(key);
        }

        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect, ctx: &RenderContext) -> Result<(), CliError> {
        draw_item(
            self.selected,
            self.title,
            &self.subtitle,
            self.data.dirty(),
            frame,
            area,
        )
        .change_context(CliError::UnableToDrawComponent)
        .attach_printable_lazy(|| format!("Drawing list item titled {}", self.title))?;

        if let Some(ref mut p) = self.popup {
            p.draw(frame, area, ctx)?;
        }

        Ok(())
    }
}
