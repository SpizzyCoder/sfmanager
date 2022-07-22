use std::{
    cmp::Ordering,
    ffi::OsString,
    fs,
    fs::ReadDir,
    path::{Path, PathBuf},
};

use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

pub struct Panel {
    state: ListState,
    path: PathBuf,
    selection_history: Vec<usize>,
    items: Vec<PathBuf>,
}

impl Panel {
    pub fn new(path: &Path) -> Self {
        let mut panel: Panel = Panel {
            state: ListState::default(),
            path: path.to_path_buf(),
            selection_history: Vec::new(),
            items: Self::gen_items(path),
        };

        panel.begin();
        return panel;
    }

    pub fn get_cur_obj(&self) -> PathBuf {
        let selected_obj: usize = match self.state.selected() {
            Some(x) => x,
            None => return PathBuf::new(),
        };

        return self.items[selected_obj].clone();
    }

    pub fn get_path(&self) -> PathBuf {
        return self.path.clone();
    }

    pub fn open_dir(&mut self) {
        let selected_dir: usize = match self.state.selected() {
            Some(x) => x,
            None => return,
        };

        if self.items[selected_dir].is_dir() {
            let dir_name: OsString = self.items[selected_dir].file_name().unwrap().to_owned();
            self.path.push(dir_name);
            self.update_items();
            self.selection_history.push(selected_dir);
            self.begin();
        }
    }

    pub fn leave_dir(&mut self) {
        if self.path.pop() {
            self.update_items();
            match self.selection_history.pop() {
                Some(x) => self.state.select(Some(x)),
                None => self.begin(),
            };
        }
    }

    pub fn next(&mut self) {
        let i: Option<usize> = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    Some(i)
                } else {
                    Some(i + 1)
                }
            }
            None => None,
        };
        self.state.select(i);
    }

    pub fn previous(&mut self) {
        let i: Option<usize> = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    Some(i)
                } else {
                    Some(i - 1)
                }
            }
            None => None,
        };
        self.state.select(i);
    }

    pub fn begin(&mut self) {
        if self.items.len() < 1 {
            self.state.select(None);
            return;
        }

        self.state.select(Some(0));
    }

    pub fn end(&mut self) {
        if self.items.len() < 1 {
            self.state.select(None);
            return;
        }

        self.state.select(Some(self.items.len() - 1))
    }

    pub fn jump_to_first_matching(&mut self, search_str: &str) {
        for (i, v) in self.items.iter().enumerate() {
            let path_as_str: &str = v.file_name().unwrap().to_str().unwrap();

            if path_as_str.contains(search_str) {
                self.state.select(Some(i));
                return;
            }
        }
    }

    pub fn render<B: Backend>(&mut self, chunk: Rect, f: &mut Frame<B>, line_color: Color) {
        let mut items: Vec<ListItem> = Vec::new();

        for obj in self.items.iter() {
            let obj_color: Color = match obj.is_dir() {
                true => Color::Blue,
                false => Color::White,
            };

            items.push(
                ListItem::new(obj.file_name().unwrap().to_str().unwrap().to_string())
                    .style(Style::default().fg(obj_color).bg(Color::Black)),
            );
        }

        let items = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.path.to_str().unwrap().to_owned())
                    .border_style(Style::default().fg(line_color)),
            )
            .highlight_style(Style::default().bg(line_color).add_modifier(Modifier::BOLD));

        f.render_stateful_widget(items, chunk, &mut self.state);
    }

    pub fn update_items(&mut self) {
        self.items = Self::gen_items(&self.path);
    }

    fn gen_items(path: &Path) -> Vec<PathBuf> {
        let dir_iterator: ReadDir = match fs::read_dir(path) {
            Ok(iterator) => iterator,
            Err(_error) => {
                // TODO -> Error message
                return Vec::new();
            }
        };

        let mut dir_entries: Vec<PathBuf> = dir_iterator
            .filter_map(|x| x.ok())
            .map(|x| x.path())
            .collect();

        dir_entries.sort_by(|x, y| {
            if x.is_dir() && y.is_dir() {
                x.cmp(&y)
            } else if y.is_dir() {
                Ordering::Greater
            } else if x.is_dir() {
                Ordering::Less
            } else {
                x.cmp(&y)
            }
        });

        return dir_entries;
    }
}
