use crate::panel::Panel;
use crate::popup::Popup;
use crate::ACTIVE_COLOR;
use crate::INACTIVE_COLOR;

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Row, Table},
    Frame,
};

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
    search_str: String,
    popup: Option<Popup>,
}

impl App {
    pub fn new() -> Self {
        let start_path: PathBuf;

        // Determine the home path
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
            search_str: String::new(),
            popup: None,
        };
    }

    pub fn is_popup(&self) -> bool {
        return self.popup.is_some();
    }

    pub fn open_dir(&mut self) {
        self.get_cur_panel().open_dir();
        self.search_str.clear();
    }

    pub fn leave_dir(&mut self) {
        self.get_cur_panel().leave_dir();
        self.search_str.clear();
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
        self.search_str.clear();
    }

    pub fn jump_to_first_matching(&mut self, ch: char) {
        self.search_str.push(ch);

        let search_str_clone: String = self.search_str.clone();
        self.get_cur_panel()
            .jump_to_first_matching(&search_str_clone);
    }

    pub fn clear_search_str(&mut self) {
        self.search_str.clear();
    }

    pub fn pop_char_from_search_str(&mut self) {
        self.search_str.pop();
    }

    pub fn open_help_popup(&mut self) {
        self.popup = Some(Popup::new(
            "Help",
            concat![
                "F1 - Show this help\n",
                "F2 - Copy\n",
                "F3 - Move\n",
                "F5 - Refresh\n",
                "F12 - Terminate sfmanager\n", // TODO -> use env
                "Arrow up - Go one entry up\n",
                "Arrow down - Go one entry down\n",
                "Home - Go to the first entry\n",
                "End - Go to the last entry\n",
                "Arrow right - Enter folder\n",
                "Enter - Enter folder\n",
                "Arrow left - Leave folder\n",
                "Backspace - Delete last char from search string\n",
                "Tab - Switch current panel\n",
                "Delete - Delete\n",
                "Esc - Clear search string\n",
            ],
            None,
        ));
    }

    pub fn close_popup(&mut self) {
        self.popup = None;
    }

    pub fn copy_objects(&mut self) {
        if let Err(error) = self.copy() {
            self.popup = Some(Popup::new(
                "Error",
                &format!["{}", error],
                Some(Style::default().fg(Color::Red)),
            ));
            return;
        }

        self.refresh();
    }

    pub fn move_objects(&mut self) {
        if let Err(error) = self.copy() {
            self.popup = Some(Popup::new(
                "Error",
                &format!["{}", error],
                Some(Style::default().fg(Color::Red)),
            ));
            return;
        }

        let src_path: PathBuf = self.get_cur_panel().get_cur_obj();

        if src_path.is_dir() {
            if let Err(error) = fs::remove_dir_all(&src_path) {
                self.popup = Some(Popup::new(
                    "Error",
                    &format!["Failed to delete {} [Error: {}]", src_path.display(), error],
                    Some(Style::default().fg(Color::Red)),
                ));
                return;
            }
        } else {
            if let Err(error) = fs::remove_file(&src_path) {
                self.popup = Some(Popup::new(
                    "Error",
                    &format!["Failed to delete {} [Error: {}]", src_path.display(), error],
                    Some(Style::default().fg(Color::Red)),
                ));
                return;
            }
        }

        self.refresh();
    }

    pub fn refresh(&mut self) {
        self.get_left_panel().update_items();
        self.get_right_panel().update_items();
    }

    pub fn delete_objects(&mut self) {
        let cur_obj: PathBuf = self.get_cur_panel().get_cur_obj();

        if cur_obj.is_dir() {
            if let Err(error) = fs::remove_dir_all(&cur_obj) {
                self.popup = Some(Popup::new(
                    "Error",
                    &format![
                        "Failed to remove {} recursively [Error: {}]",
                        cur_obj.display(),
                        error
                    ],
                    Some(Style::default().fg(Color::Red)),
                ));
                return;
            }
        } else {
            if let Err(error) = fs::remove_file(&cur_obj) {
                self.popup = Some(Popup::new(
                    "Error",
                    &format!["Failed to remove {} [Error: {}]", cur_obj.display(), error],
                    Some(Style::default().fg(Color::Red)),
                ));
                return;
            }
        }

        self.get_cur_panel().update_items();
    }

    pub fn render<B: Backend>(&mut self, f: &mut Frame<B>) {
        if self.popup.is_some() {
            self.popup.as_mut().unwrap().render(f);
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
                format!["Search string: {}", self.search_str],
                format!["F1 help"],
            ]),
            Row::new(vec![format![""], format!["F2 copy"]]),
            Row::new(vec![format![""], format!["F3 move"]]),
            Row::new(vec![format![""], format!["F5 refresh"]]),
            Row::new(vec![format![""], format!["F12 quit"]]),
        ])
        .style(Style::default().fg(Color::White))
        .block(Block::default().title("Infos").borders(Borders::ALL))
        .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)]);

        f.render_widget(table, ui_chunks[1]);
    }

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

    fn copy(&mut self) -> Result<(), String> {
        let src_path: PathBuf;
        let mut dest_path: PathBuf;

        src_path = self.get_cur_panel().get_cur_obj();

        if self.cur_panel == ActivePanel::Left {
            // Copy from left to right panel
            dest_path = self.get_right_panel().get_path();
        } else {
            // Copy from right to left panel
            dest_path = self.get_left_panel().get_path();
        }

        let file_name: &OsStr = src_path.file_name().unwrap();
        dest_path.push(file_name);

        if src_path.is_dir() {
            if let Err(error) = copy_recursively(&src_path, &dest_path) {
                return Err(format![
                    "Failed to copy {} to {} [Error: {}]",
                    src_path.display(),
                    dest_path.display(),
                    error
                ]);
            }
        } else {
            if let Err(error) = fs::copy(&src_path, &dest_path) {
                return Err(format![
                    "Failed to copy {} to {} [Error: {}]",
                    src_path.display(),
                    dest_path.display(),
                    error
                ]);
            }
        }

        return Ok(());
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
