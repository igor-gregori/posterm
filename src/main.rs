mod app;
mod event;
mod http;
mod storage;
mod ui;

use std::io;

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::sync::mpsc;

use app::App;

#[tokio::main]
async fn main() -> io::Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut app = App::new();

    let (tx, mut rx) = mpsc::channel::<Result<http::Response, String>>(1);

    loop {
        if !app.running {
            break;
        }

        terminal.draw(|frame| ui::draw(frame, &app))?;

        // Check for completed responses
        if let Ok(result) = rx.try_recv() {
            app.loading = false;
            app.response = Some(result);
        }

        event::handle_events(&mut app, &tx)?;
    }

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}
