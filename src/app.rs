use crate::ACTIVE_COLOR;
use crate::INACTIVE_COLOR;

use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, Paragraph, Row, Table, Wrap},
    Frame,
};

use crate::panel::Panel;

use std::{
    env,
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
};

#[derive(PartialEq)]
pub enum ActivePanel {
    Left,
    Right,
}

pub struct App {
    cur_panel: ActivePanel,
    left_panel: Panel,
    right_panel: Panel,
    help_popup: bool,
    error_str: String,
}

impl App {
    fn get_left_panel(&mut self) -> &mut Panel {
        return &mut self.left_panel;
    }

    fn get_right_panel(&mut self) -> &mut Panel {
        return &mut self.right_panel;
    }

    fn get_cur_panel(&mut self) -> &mut Panel {
        if self.cur_panel == ActivePanel::Left {
            return &mut self.left_panel;
        } else {
            return &mut self.right_panel;
        }
    }

    pub fn new() -> Self {
        let start_path: PathBuf;

        if cfg![windows] {
            start_path = PathBuf::from(format![
                "{}{}",
                env::var("HOMEDRIVE").unwrap(),
                env::var("HOMEPATH").unwrap()
            ]);
        } else {
            start_path = PathBuf::from(env::var("HOME").unwrap());
        }

        return App {
            cur_panel: ActivePanel::Left,
            left_panel: Panel::new(&start_path),
            right_panel: Panel::new(&start_path),
            help_popup: false,
            error_str: String::new(),
        };
    }

    pub fn is_help_popup(&self) -> bool {
        return self.help_popup;
    }

    pub fn open_dir(&mut self) {
        self.get_cur_panel().open_dir();
        self.get_cur_panel().clear_search_str();
    }

    pub fn leave_dir(&mut self) {
        self.get_cur_panel().leave_dir();
        self.get_cur_panel().clear_search_str();
    }

    pub fn next(&mut self) {
        self.get_cur_panel().next();
    }

    pub fn previous(&mut self) {
        self.get_cur_panel().previous();
    }

    pub fn begin(&mut self) {
        self.get_cur_panel().begin();
    }

    pub fn end(&mut self) {
        self.get_cur_panel().end();
    }

    pub fn switch_active_panel(&mut self) {
        if self.cur_panel == ActivePanel::Left {
            self.cur_panel = ActivePanel::Right;
        } else {
            self.cur_panel = ActivePanel::Left;
        }
    }

    pub fn jump_to_first_matching(&mut self, ch: char) {
        self.get_cur_panel().push_char_to_search_str(ch);
        self.get_cur_panel().jump_to_first_matching();
    }

    pub fn clear_search_str(&mut self) {
        self.get_cur_panel().clear_search_str();
    }

    pub fn pop_char_from_search_str(&mut self) {
        self.get_cur_panel().pop_char_from_search_str();
    }

    pub fn open_help_popup(&mut self) {
        self.help_popup = true;
    }

    pub fn close_help_popup(&mut self) {
        self.help_popup = false;
    }

    pub fn is_error_popup(&self) -> bool {
        return !self.error_str.is_empty();
    }

    pub fn close_error_popup(&mut self) {
        self.error_str.clear();
    }

    pub fn copy(&mut self) {
        let src_path: PathBuf;
        let mut dest_path: PathBuf;

        if self.cur_panel == ActivePanel::Left {
            // Copy from left to right panel
            src_path = self.get_cur_panel().get_cur_obj();
            dest_path = self.get_right_panel().get_path();
        } else {
            // Copy from right to left panel
            src_path = self.get_cur_panel().get_cur_obj();
            dest_path = self.get_left_panel().get_path();
        }

        let file_name: &OsStr = src_path.file_name().unwrap();
        dest_path.push(file_name);

        if src_path.is_dir() {
            if let Err(error) = copy_recursively(&src_path, &dest_path) {
                self.error_str = format![
                    "Failed to copy {} to {} [Error: {}]",
                    src_path.display(),
                    dest_path.display(),
                    error
                ];
                return;
            }
        } else {
            if let Err(error) = fs::copy(&src_path, &dest_path) {
                self.error_str = format![
                    "Failed to copy {} to {} [Error: {}]",
                    src_path.display(),
                    dest_path.display(),
                    error
                ];
                return;
            }
        }

        self.get_left_panel().update_items();
        self.get_right_panel().update_items();
    }

    pub fn refresh(&mut self) {
        self.get_left_panel().update_items();
        self.get_right_panel().update_items();
    }

    pub fn delete(&mut self) {
        let cur_obj: PathBuf = self.get_cur_panel().get_cur_obj();

        if cur_obj.is_dir() {
            if let Err(error) = fs::remove_dir_all(&cur_obj) {
                self.error_str = format![
                    "Failed to remove {} recursively [Error: {}]",
                    cur_obj.display(),
                    error
                ];
                return;
            }
        } else {
            if let Err(error) = fs::remove_file(&cur_obj) {
                self.error_str =
                    format!["Failed to remove {} [Error: {}]", cur_obj.display(), error];
                return;
            }
        }

        self.get_cur_panel().update_items();
    }

    pub fn render<B: Backend>(&mut self, f: &mut Frame<B>) {
        if !self.error_str.is_empty() {
            let error_layout: Vec<Rect> = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(100)].as_ref())
                .margin(15)
                .split(f.size());

            let text = vec![
                Spans::from(Span::styled(
                    &self.error_str,
                    Style::default().fg(Color::Red),
                )),
                Spans::from(Span::raw("Press ENTER or ESC")),
            ];

            let error_msg: Paragraph = Paragraph::new(text)
                .block(Block::default().title("Error").borders(Borders::ALL))
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });

            f.render_widget(Clear, error_layout[0]);
            f.render_widget(error_msg, error_layout[0]);
            return;
        }

        if self.help_popup {
            let table: Table = Table::new(vec![
                Row::new(vec!["F1", "Show this help"]),
                Row::new(vec!["F12", "Terminate sfmanager"]),
                Row::new(vec!["Arrow down", "Go one entry down"]),
                Row::new(vec!["Arrow up", "Go one entry up"]),
                Row::new(vec!["Home", "Go to the first entry"]),
                Row::new(vec!["End", "Go to the last entry"]),
                Row::new(vec!["Arrow right", "Go into folder"]),
                Row::new(vec!["Enter", "Go into folder"]),
                Row::new(vec!["Arrow left", "Go out of folder"]),
                Row::new(vec!["Backspace", "Delete last char from search string"]),
                Row::new(vec!["Tab", "Switch current panel"]),
                Row::new(vec!["Esc", "Close this help or clear search string"]),
            ])
            .style(Style::default().fg(Color::White))
            .block(Block::default().title("Help").borders(Borders::ALL))
            .widths(&[Constraint::Percentage(8), Constraint::Percentage(92)]);

            f.render_widget(Clear, f.size());
            f.render_widget(table, f.size());
            return;
        }

        let ui_chunks: Vec<Rect> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(85), Constraint::Percentage(15)].as_ref())
            .split(f.size());

        let panel_chunks: Vec<Rect> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(ui_chunks[0]);

        let mut color: Color = match self.cur_panel {
            ActivePanel::Left => ACTIVE_COLOR,
            ActivePanel::Right => INACTIVE_COLOR,
        };

        self.left_panel.render(panel_chunks[0], f, color);

        color = match self.cur_panel {
            ActivePanel::Right => ACTIVE_COLOR,
            ActivePanel::Left => INACTIVE_COLOR,
        };

        self.right_panel.render(panel_chunks[1], f, color);

        let table: Table = Table::new(vec![
            Row::new(vec![
                format![
                    "Search string: {}",
                    self.get_cur_panel().get_search_string()
                ],
                format!["F1 help"],
            ]),
            Row::new(vec![format![""], format!["F2 copy"]]),
            Row::new(vec![format![""], format!["F5 refresh"]]),
            Row::new(vec![format![""], format!["F12 quit"]]),
        ])
        .style(Style::default().fg(Color::White))
        .block(Block::default().title("Infos").borders(Borders::ALL))
        .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)]);

        f.render_widget(table, ui_chunks[1]);
    }
}

fn copy_recursively(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let filetype = entry.file_type()?;
        if filetype.is_dir() {
            copy_recursively(entry.path(), destination.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), destination.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
