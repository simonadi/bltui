use tui::{buffer::Buffer, layout::Rect, style::Style, widgets::{StatefulWidget, Widget}};

use super::widgets::statics::blue_box;

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

pub struct QuestionPopup {
    question: String,
    items: Vec<QuestionPopupItem>,
    style: Style,
    highlight_style: Style,
}

impl QuestionPopup {
    pub fn new<T>(question: String, items: T) -> QuestionPopup
    where
        T: Into<Vec<QuestionPopupItem>>,
        // U: Into<Text<'a>>,
    {
        QuestionPopup {
            question,
            items: items.into(),
            style: Style::default(),
            highlight_style: Style::default(),
        }
    }

    pub fn style(mut self, style: Style) -> QuestionPopup {
        self.style = style;
        self
    }

    pub fn highlight_style(mut self, style: Style) -> QuestionPopup {
        self.highlight_style = style;
        self
    }
}

impl<'a> StatefulWidget for QuestionPopup {
    type State = QuestionPopupState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        blue_box(Some("Confirm".to_string())).render(area, buf);
    //     for x in area.left()..area.right() {
    //         for y in area.top()..area.bottom() {
    //             buf.get_mut(x, y).reset();
    //         }
    //     }

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
