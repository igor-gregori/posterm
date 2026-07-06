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
use reqwest::Client;
use tokio::sync::mpsc;

use app::App;
use storage::collections::SavedRequest;
use storage::history::{self, HistoryEntry};

#[tokio::main]
async fn main() -> io::Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut app = App::new();
    let client = Client::new();

    let (tx, mut rx) = mpsc::channel::<(SavedRequest, Result<http::Response, String>)>(1);

    loop {
        if !app.running {
            break;
        }

        terminal.draw(|frame| ui::draw(frame, &app))?;

        app.tick = app.tick.wrapping_add(1);

        // Expire status message after ~3 seconds (180 ticks at 60fps)
        if app.status_message.is_some() && app.tick.wrapping_sub(app.status_tick) > 180 {
            app.status_message = None;
        }

        // Check for completed responses
        if let Ok((saved_req, result)) = rx.try_recv() {
            app.loading = false;

            // Save to history
            let entry = HistoryEntry {
                request: saved_req,
                status: result.as_ref().ok().map(|r| r.status),
                duration_ms: result.as_ref().ok().map(|r| r.duration.as_millis() as u64),
                timestamp: history::now_timestamp(),
            };
            app.history.push(entry);
            let _ = history::save_history(&app.history);

            app.response = Some(result);
            app.response_scroll = 0;
        }

        event::handle_events(&mut app, &tx, &client)?;
    }

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}
