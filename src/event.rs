use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;

use crate::app::{App, Panel, RequestFocus, RequestTab};
use crate::http;

pub fn handle_events(
    app: &mut App,
    tx: &mpsc::Sender<Result<http::Response, String>>,
) -> std::io::Result<()> {
    if event::poll(std::time::Duration::from_millis(16))? {
        if let Event::Key(key) = event::read()? {
            if app.editing {
                handle_editing(app, key);
            } else {
                handle_normal(app, key, tx);
            }
        }
    }
    Ok(())
}

fn handle_normal(app: &mut App, key: KeyEvent, tx: &mpsc::Sender<Result<http::Response, String>>) {
    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => app.running = false,
        (KeyModifiers::NONE, KeyCode::Char('q')) if !app.editing => app.running = false,
        (KeyModifiers::NONE, KeyCode::Tab) => app.next_panel(),
        // Send request with Ctrl+Enter
        (KeyModifiers::CONTROL, KeyCode::Enter) => {
            if !app.loading && !app.request.url.is_empty() {
                app.loading = true;
                let request = app.request.clone();
                let tx = tx.clone();
                tokio::spawn(async move {
                    let result = http::send_request(&request).await;
                    let _ = tx.send(result).await;
                });
            }
        }
        _ => {}
    }

    if app.active_panel != Panel::Request {
        return;
    }

    match key.code {
        KeyCode::Up => match app.request_focus {
            RequestFocus::Tab => app.request_focus = RequestFocus::Url,
            RequestFocus::Url => app.request_focus = RequestFocus::Method,
            _ => {}
        },
        KeyCode::Down => match app.request_focus {
            RequestFocus::Method => app.request_focus = RequestFocus::Url,
            RequestFocus::Url => app.request_focus = RequestFocus::Tab,
            _ => {}
        },
        KeyCode::Left if app.request_focus == RequestFocus::Method => {
            app.request.method = app.request.method.prev();
        }
        KeyCode::Right if app.request_focus == RequestFocus::Method => {
            app.request.method = app.request.method.next();
        }
        KeyCode::Left if app.request_focus == RequestFocus::Tab => {
            app.request_tab = match app.request_tab {
                RequestTab::Headers => RequestTab::Params,
                RequestTab::Body => RequestTab::Headers,
                RequestTab::Params => RequestTab::Body,
            };
            app.kv_row = 0;
        }
        KeyCode::Right if app.request_focus == RequestFocus::Tab => {
            app.request_tab = app.request_tab.next();
            app.kv_row = 0;
        }
        KeyCode::Enter => {
            app.editing = true;
        }
        KeyCode::Char('j') if app.request_focus == RequestFocus::Tab => {
            let len = kv_len(app);
            if len > 0 && app.kv_row < len - 1 {
                app.kv_row += 1;
            }
        }
        KeyCode::Char('k') if app.request_focus == RequestFocus::Tab => {
            if app.kv_row > 0 {
                app.kv_row -= 1;
            }
        }
        KeyCode::Char('l')
            if app.request_focus == RequestFocus::Tab && app.request_tab != RequestTab::Body =>
        {
            app.kv_on_key = false;
        }
        KeyCode::Char('h')
            if app.request_focus == RequestFocus::Tab && app.request_tab != RequestTab::Body =>
        {
            app.kv_on_key = true;
        }
        KeyCode::Char('a')
            if app.request_focus == RequestFocus::Tab && app.request_tab != RequestTab::Body =>
        {
            let kv = crate::http::models::KeyValue {
                key: String::new(),
                value: String::new(),
            };
            match app.request_tab {
                RequestTab::Headers => app.request.headers.push(kv),
                RequestTab::Params => app.request.params.push(kv),
                _ => {}
            }
            let len = kv_len(app);
            app.kv_row = len - 1;
            app.kv_on_key = true;
            app.editing = true;
        }
        KeyCode::Char('d')
            if app.request_focus == RequestFocus::Tab && app.request_tab != RequestTab::Body =>
        {
            let len = kv_len(app);
            if len > 1 {
                match app.request_tab {
                    RequestTab::Headers => {
                        app.request.headers.remove(app.kv_row);
                    }
                    RequestTab::Params => {
                        app.request.params.remove(app.kv_row);
                    }
                    _ => {}
                }
                if app.kv_row >= kv_len(app) {
                    app.kv_row = kv_len(app).saturating_sub(1);
                }
            }
        }
        _ => {}
    }
}

fn handle_editing(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.editing = false,
        KeyCode::Enter => app.editing = false,
        KeyCode::Backspace => {
            get_editing_field(app).pop();
        }
        KeyCode::Char(c) => {
            get_editing_field(app).push(c);
        }
        _ => {}
    }
}

fn get_editing_field(app: &mut App) -> &mut String {
    match app.request_focus {
        RequestFocus::Url => &mut app.request.url,
        RequestFocus::Tab => match app.request_tab {
            RequestTab::Body => &mut app.request.body,
            RequestTab::Headers => {
                let row = app.kv_row;
                if app.kv_on_key {
                    &mut app.request.headers[row].key
                } else {
                    &mut app.request.headers[row].value
                }
            }
            RequestTab::Params => {
                let row = app.kv_row;
                if app.kv_on_key {
                    &mut app.request.params[row].key
                } else {
                    &mut app.request.params[row].value
                }
            }
        },
        RequestFocus::Method => &mut app.request.url,
    }
}

fn kv_len(app: &App) -> usize {
    match app.request_tab {
        RequestTab::Headers => app.request.headers.len(),
        RequestTab::Params => app.request.params.len(),
        RequestTab::Body => 0,
    }
}
