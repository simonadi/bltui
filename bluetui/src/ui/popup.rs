use std::cmp::min;

use tui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};

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
    }
}
