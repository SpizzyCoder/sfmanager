use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub struct Popup {
    title: String,
    text: String,
    style: Option<Style>,
}

impl Popup {
    pub fn new(title: &str, text: &str, style: Option<Style>) -> Self {
        return Popup {
            title: title.to_owned(),
            text: text.to_owned(),
            style: style,
        };
    }

    pub fn render<B: Backend>(&mut self, f: &mut Frame<B>) {
        let popup_layout: Vec<Rect> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)].as_ref())
            .margin(10)
            .split(f.size());

        let text: Text;

        if self.style.is_some() {
            text = Text::styled(
                format!["{}\n\n[Press Enter or Esc]", self.text],
                self.style.clone().unwrap(),
            );
        } else {
            text = Text::from(format!["{}\n\n[Press Enter or Esc]", self.text]);
        }

        let popup_msg: Paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .title(&self.title[..])
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(Clear, popup_layout[0]);
        f.render_widget(popup_msg, popup_layout[0]);
    }
}
