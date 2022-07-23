use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    style::Color,
    Terminal,
};

mod app;
use app::App;
mod panel;
mod popup;

const ACTIVE_COLOR: Color = Color::LightGreen;
const INACTIVE_COLOR: Color = Color::DarkGray;

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
        terminal.draw(|f| app.render(f))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::F(1) => app.open_help_popup(),
                KeyCode::F(2) => app.copy_objects(),
                KeyCode::F(3) => app.move_objects(),
                KeyCode::F(5) => app.refresh(),
                KeyCode::F(12) => return Ok(()),
                KeyCode::Up => app.previous(),
                KeyCode::Down => app.next(),
                KeyCode::Home => app.begin(),
                KeyCode::End => app.end(),
                KeyCode::Right => app.open_dir(),
                KeyCode::Enter => {
                    if app.is_popup() {
                        app.close_popup();
                    } else {
                        app.open_dir();
                    }
                }
                KeyCode::Left => app.leave_dir(),
                KeyCode::Backspace => app.pop_char_from_search_str(),
                KeyCode::Tab => app.switch_active_panel(),
                KeyCode::Delete => app.delete_objects(),
                KeyCode::Char(x @ ' '..='~') => app.jump_to_first_matching(x),
                KeyCode::Esc => {
                    if app.is_popup() {
                        app.close_popup();
                    } else {
                        app.clear_search_str();
                    }
                }
                _ => {}
            }
        }
    }
}
