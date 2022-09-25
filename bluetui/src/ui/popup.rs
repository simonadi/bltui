use std::cmp::min;

use crossterm::event::KeyCode;
use tui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};

trait Popup {
    fn validate(&self);
    fn handle_keypress(&mut self, keycode: KeyCode);
}

enum QuestionPopupType {
    YesNo,
    TextInput,
}

#[derive(Debug, Clone, Default)]
pub struct QuestionPopupState {
    selected: usize,
}

impl QuestionPopupState {
    pub fn selected(&self) -> usize {
        self.selected
    }
}

#[derive(Debug, Clone)]
pub struct QuestionPopupItem {
    content: String,
    // style: Style,
}

impl QuestionPopupItem {
    pub fn unstyled(content: String) -> QuestionPopupItem {
        QuestionPopupItem {
            content,
            // style: Style::default(),
        }
    }

    // pub fn styled(content: String, style: Style) -> QuestionPopupItem
    // {
    //     QuestionPopupItem {
    //         content: content,
    //         style,
    //     }
    // }
}

#[derive(Clone)]
pub struct QuestionPopup<'a> {
    question: String,
    options: Vec<ListItem<'a>>,
    state: ListState,
    block: Option<Block<'a>>,
}

impl<'a> QuestionPopup<'a> {
    pub fn new<T>(question: String, options: T) -> QuestionPopup<'a>
    where
        T: Into<Vec<ListItem<'a>>>,
        // U: Into<Text<'a>>,
    {
        QuestionPopup {
            question,
            options: options.into(),
            state: ListState::default(),
            block: None,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> QuestionPopup<'a> {
        self.block = Some(block);
        self
    }

    pub fn move_selector_down(&mut self) {
        let current_index = self.state.selected();

        if let Some(index) = current_index {
            self.state
                .select(Some(min(index + 1, self.options.len() - 1)));
        } else if !self.options.is_empty() {
            self.state.select(Some(0));
        }
    }

    pub fn move_selector_up(&mut self) {
        let current_index = self.state.selected();

        if let Some(index) = current_index {
            self.state.select(Some(index.saturating_sub(1)));
        } else if !self.options.is_empty() {
            self.state.select(Some(0));
        }
    }
}

// impl<'a> StatefulWidget for QuestionPopup<'a> {
impl<'a> Widget for QuestionPopup<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        for x in area.left()..area.right() {
            for y in area.top()..area.bottom() {
                buf.get_mut(x, y).reset();
            }
        }

        let question_area = match self.block.take() {
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
            .alignment(Alignment::Center)
            .render(chunks[0], buf);

        StatefulWidget::render(
            List::new(self.options)
                .highlight_style(Style::default().bg(Color::White).fg(Color::Black))
                .highlight_symbol("->"),
            chunks[1],
            buf,
            &mut self.state,
        );

        //     buf.set_style(area, self.style);

        //     // let question_width = self.question.chars().map(|chara| StyledGrapheme { symbol: &String::from(chara), style: self.style });
        //     let question_width = self.question.len();
        //     let answers_width = self
        //         .items
        //         .clone()
        //         .into_iter()
        //         .map(|item| item.content.len())
        //         .sum::<usize>();

        //     // info!("width : {}", question_width);

        //     let center_x = area.left() + (area.right() - area.left()) / 2;
        //     let start_question = center_x - (question_width as u16 / 2);
        //     let answers_offset =
        //         ((area.right() - area.left()) - answers_width as u16) / (self.items.len() + 1) as u16;

        //     buf.set_stringn(
        //         start_question,
        //         area.top(),
        //         self.question,
        //         question_width,
        //         Style::default(),
        //     );

        //     let mut cursor = area.left() + answers_offset;
        //     let selected = state.selected();

        //     for (i, answer) in self.items.iter().enumerate() {
        //         buf.set_stringn(
        //             cursor,
        //             area.bottom() - 2,
        //             &answer.content,
        //             100,
        //             if i == selected {
        //                 self.highlight_style
        //             } else {
        //                 self.style
        //             },
        //         );

        //         cursor += answer.content.len() as u16 + answers_offset;
        //     }
    }
}
