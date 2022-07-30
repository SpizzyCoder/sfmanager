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
    thread,
    thread::JoinHandle,
};

mod popup;
use popup::Popup;
mod panel;
use panel::Panel;

const ACTIVE_COLOR: Color = Color::LightGreen;
const INACTIVE_COLOR: Color = Color::DarkGray;

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
    operations: Vec<JoinHandle<io::Result<()>>>,
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
            operations: Vec::new(),
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
        let src_dest_paths: (PathBuf, PathBuf) = self.get_copy_move_path();
        let src_path = src_dest_paths.0;
        let dest_path = src_dest_paths.1;

        self.operations
            .push(thread::spawn(move || -> io::Result<()> {
                if src_path.is_dir() {
                    copy_recursively(&src_path, &dest_path)?;
                } else {
                    fs::copy(&src_path, &dest_path)?;
                }

                return Ok(());
            }));
    }

    pub fn move_objects(&mut self) {
        let src_dest_paths: (PathBuf, PathBuf) = self.get_copy_move_path();
        let src_path = src_dest_paths.0;
        let dest_path = src_dest_paths.1;

        self.operations
            .push(thread::spawn(move || -> io::Result<()> {
                if src_path.is_dir() {
                    copy_recursively(&src_path, &dest_path)?;
                    fs::remove_dir_all(&src_path)?;
                } else {
                    fs::copy(&src_path, &dest_path)?;
                    fs::remove_file(&src_path)?;
                }

                return Ok(());
            }));
    }

    pub fn refresh(&mut self) {
        self.left_panel.update_items();
        self.right_panel.update_items();
    }

    pub fn delete_objects(&mut self) {
        let cur_obj: PathBuf = self.get_cur_panel().get_cur_obj();

        if let Err(error) = trash::delete(&cur_obj) {
            self.popup = Some(Popup::new(
                "Error",
                &format!["Failed to delete {} [Error: {}]",cur_obj.display(),error],
                None
            ));
        }

        // self.operations
        //     .push(thread::spawn(move || -> io::Result<()> {
        //         if cur_obj.is_dir() {
        //             fs::remove_dir_all(&cur_obj)?;
        //         } else {
        //             fs::remove_file(&cur_obj)?;
        //         }

        //         return Ok(());
        //     }));
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
            Row::new(vec![
                format!["Active operations: {}", self.operations.len()],
                format!["F2 copy"],
            ]),
            Row::new(vec![format![""], format!["F3 move"]]),
            Row::new(vec![format![""], format!["F5 refresh"]]),
            Row::new(vec![format![""], format!["F12 quit"]]),
        ])
        .style(Style::default().fg(Color::White))
        .block(Block::default().title("Infos").borders(Borders::ALL))
        .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)]);

        f.render_widget(table, ui_chunks[1]);
    }

    pub fn thread_ctrl(&mut self) {
        let mut finished_indexes: Vec<usize> = Vec::new();

        for index in 0..self.operations.len() {
            if self.operations[index].is_finished() {
                finished_indexes.push(index);
            }
        }

        loop {
            if finished_indexes.len() < 1 {
                break;
            }

            let index: usize = finished_indexes.remove(0);
            finished_indexes = finished_indexes.iter().map(|x| x - 1).collect();

            let join_handle: JoinHandle<io::Result<()>> = self.operations.remove(index);
            match join_handle.join().unwrap() {
                Ok(_) => {}
                Err(error) => {
                    self.popup = Some(Popup::new(
                        "Error",
                        &error.to_string(),
                        Some(Style::default().fg(Color::Red)),
                    ));
                    return;
                }
            };
        }

        self.refresh();
    }

    // 0 -> Source path
    // 1 -> Destination path
    fn get_copy_move_path(&mut self) -> (PathBuf, PathBuf) {
        let src_path: PathBuf;
        let mut dest_path: PathBuf;

        src_path = self.get_cur_panel().get_cur_obj();

        if self.cur_panel == ActivePanel::Left {
            // Copy from left to right panel
            dest_path = self.right_panel.get_path();
        } else {
            // Copy from right to left panel
            dest_path = self.left_panel.get_path();
        }

        let file_name: &OsStr = src_path.file_name().unwrap();
        dest_path.push(file_name);

        return (src_path, dest_path);
    }

    fn get_cur_panel(&mut self) -> &mut Panel {
        if self.cur_panel == ActivePanel::Left {
            return &mut self.left_panel;
        } else {
            return &mut self.right_panel;
        }
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

    return Ok(());
}
