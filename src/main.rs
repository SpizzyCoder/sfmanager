use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    cmp::Ordering,
    env,
    error::Error,
    ffi::OsString,
    fs,
    fs::ReadDir,
    io,
    path::{Path, PathBuf},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame, Terminal,
};

const ACTIVE_COLOR: Color = Color::LightGreen;
const INACTIVE_COLOR: Color = Color::DarkGray;

#[derive(PartialEq)]
enum ActivePanel {
    Left,
    Right,
}

struct Panel {
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
            items: Self::get_items(path),
        };

        panel.begin();
        return panel;
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

    fn update_items(&mut self) {
        self.items = Self::get_items(&self.path);
    }

    fn get_items(path: &Path) -> Vec<PathBuf> {
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

struct App {
    cur_panel: ActivePanel,
    left_panel: Panel,
    right_panel: Panel,
    search_str: String,
}

impl App {
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
            search_str: String::new(),
        };
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

        let search_str: String = self.search_str.clone();
        self.get_cur_panel().jump_to_first_matching(&search_str);
    }

    pub fn clear_search_str(&mut self) {
        self.search_str.clear();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    return Ok(());
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::F(12) => return Ok(()),
                KeyCode::Down => app.next(),
                KeyCode::Up => app.previous(),
                KeyCode::Home => app.begin(),
                KeyCode::End => app.end(),
                KeyCode::Right => app.open_dir(),
                KeyCode::Enter => app.open_dir(),
                KeyCode::Left => app.leave_dir(),
                KeyCode::Backspace => app.leave_dir(),
                KeyCode::Tab => app.switch_active_panel(),
                KeyCode::Char(x @ ' '..='>' | x @ '@'..='~') => app.jump_to_first_matching(x), // Everything except '?'
                KeyCode::Char('?') => println!["Help"], // TODO -> Help popup
                KeyCode::Esc => app.clear_search_str(),
                _ => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let panel_chunks: Vec<Rect> = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    let left_title: String = app.left_panel.path.to_str().unwrap().to_owned();
    let right_title: String = app.right_panel.path.to_str().unwrap().to_owned();

    let mut color: Color = match app.cur_panel {
        ActivePanel::Left => ACTIVE_COLOR,
        ActivePanel::Right => INACTIVE_COLOR,
    };

    render_panel(
        &mut app.left_panel,
        panel_chunks[0],
        f,
        color,
        &left_title,
    );

    color = match app.cur_panel {
        ActivePanel::Right => ACTIVE_COLOR,
        ActivePanel::Left => INACTIVE_COLOR,
    };

    render_panel(
        &mut app.right_panel,
        panel_chunks[1],
        f,
        color,
        &right_title,
    );
}

fn render_panel<B: Backend>(
    panel: &mut Panel,
    chunk: Rect,
    f: &mut Frame<B>,
    line_color: Color,
    title: &str,
) {
    let mut items: Vec<ListItem> = Vec::new();

    for obj in panel.items.iter() {
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
                .title(title)
                .border_style(Style::default().fg(line_color)),
        )
        .highlight_style(Style::default().bg(line_color).add_modifier(Modifier::BOLD));

    f.render_stateful_widget(items, chunk, &mut panel.state);
}
