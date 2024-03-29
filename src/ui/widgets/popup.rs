use std::cmp::min;

use crate::bluetooth::agent::BluezError;
use crossterm::event::KeyCode;
use tokio::sync::oneshot::Sender;
use tui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};

use super::{statics::blue_box, text_style};

pub trait Popup {
    fn confirm(&self);
    fn cancel(&self);
    fn handle_keypress(&mut self, keycode: KeyCode);
}

pub struct YesNoPopup {
    question: String,
    state: ListState,
    responder: Option<Sender<Result<(), BluezError>>>,
}

impl YesNoPopup {
    pub fn new(question: String, tx: Sender<Result<(), BluezError>>) -> YesNoPopup {
        YesNoPopup {
            question,
            state: ListState::default(),
            responder: Some(tx),
        }
    }

    pub fn move_selector_down(&mut self) {
        let current_index = self.state.selected();

        if let Some(index) = current_index {
            self.state.select(Some(min(index + 1, 1)));
        } else {
            self.state.select(Some(0));
        }
    }

    pub fn move_selector_up(&mut self) {
        let current_index = self.state.selected();

        if let Some(index) = current_index {
            self.state.select(Some(index.saturating_sub(1)));
        } else {
            self.state.select(Some(0));
        }
    }

    pub fn get_widget(&self) -> YesNoPopupWidget {
        YesNoPopupWidget {
            question: self.question.clone(),
            state: self.state.clone(),
        }
    }

    pub fn confirm(&mut self) {
        let current_index = self.state.selected();
        let result = if let Some(index) = current_index {
            if index == 0 {
                Ok(())
            } else {
                Err(BluezError::Rejected("rejected".to_string()))
            }
        } else {
            Err(BluezError::Canceled("canceled".to_string()))
        };

        let tx = self.responder.take().unwrap();

        tx.send(result).unwrap();
    }
}

pub struct YesNoPopupWidget {
    question: String,
    state: ListState,
}

impl Widget for YesNoPopupWidget {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        for x in area.left()..area.right() {
            for y in area.top()..area.bottom() {
                buf.get_mut(x, y).reset();
            }
        }

        let block = Some(blue_box(None));

        let question_area = match block {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(question_area);

        Paragraph::new(self.question)
            .style(text_style())
            .alignment(Alignment::Center)
            .render(chunks[0], buf);

        StatefulWidget::render(
            List::new(vec![ListItem::new("Yes"), ListItem::new("No")])
                .style(text_style())
                .highlight_style(Style::default().bg(Color::White).fg(Color::Black))
                .highlight_symbol("->"),
            chunks[1],
            buf,
            &mut self.state,
        );
    }
}

// impl Popup for YesNoPopup {}

// impl Widget for YesNoPopup {}

// pub struct PincodePopup {
//     question: String,
//     pincode: String,
// }

// pub struct PasskeyPopup {
//     question: String,
//     passkey: String,
// }
