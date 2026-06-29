use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;

use crate::app::{App, EditingField, RequestTab};
use crate::http;

pub fn handle_events(
    app: &mut App,
    tx: &mpsc::Sender<Result<http::Response, String>>,
) -> std::io::Result<()> {
    if event::poll(std::time::Duration::from_millis(16))? {
        if let Event::Key(key) = event::read()? {
            if app.editing.is_some() {
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
        // Global
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => app.running = false,
        (KeyModifiers::NONE, KeyCode::Char('q')) => app.running = false,
        (KeyModifiers::NONE, KeyCode::Tab) => app.next_panel(),

        // Send request: Ctrl+R
        (KeyModifiers::CONTROL, KeyCode::Char('r')) => {
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

        // Cycle method: Ctrl+T
        (KeyModifiers::CONTROL, KeyCode::Char('t')) => {
            app.request.method = app.request.method.next();
        }

        // Request editing shortcuts
        (KeyModifiers::CONTROL, KeyCode::Char('u')) => {
            app.editing = Some(EditingField::Url);
        }
        (KeyModifiers::CONTROL, KeyCode::Char('h')) => {
            app.request_tab = RequestTab::Headers;
            app.kv_row = 0;
            app.kv_on_key = true;
            app.editing = Some(EditingField::Headers);
        }
        (KeyModifiers::CONTROL, KeyCode::Char('b')) => {
            app.request_tab = RequestTab::Body;
            app.editing = Some(EditingField::Body);
        }
        (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
            app.request_tab = RequestTab::Params;
            app.kv_row = 0;
            app.kv_on_key = true;
            app.editing = Some(EditingField::Params);
        }

        _ => {}
    }
}

fn handle_editing(app: &mut App, key: KeyEvent) {
    let field = app.editing.unwrap();

    match field {
        EditingField::Url => handle_text_edit(app, key, &mut |a| &mut a.request.url),
        EditingField::Body => handle_text_edit(app, key, &mut |a| &mut a.request.body),
        EditingField::Headers => handle_kv_edit(app, key, true),
        EditingField::Params => handle_kv_edit(app, key, false),
    }
}

fn handle_text_edit(app: &mut App, key: KeyEvent, get_field: &mut dyn FnMut(&mut App) -> &mut String) {
    match key.code {
        KeyCode::Esc | KeyCode::Enter => app.editing = None,
        KeyCode::Backspace => { get_field(app).pop(); }
        KeyCode::Char(c) => get_field(app).push(c),
        _ => {}
    }
}

fn handle_kv_edit(app: &mut App, key: KeyEvent, is_headers: bool) {
    let items = if is_headers { &app.request.headers } else { &app.request.params };
    let len = items.len();

    match key.code {
        KeyCode::Esc => app.editing = None,
        KeyCode::Tab => app.kv_on_key = !app.kv_on_key,
        KeyCode::Down | KeyCode::Enter if app.kv_row < len.saturating_sub(1) => {
            app.kv_row += 1;
            app.kv_on_key = true;
        }
        KeyCode::Up if app.kv_row > 0 => {
            app.kv_row -= 1;
            app.kv_on_key = true;
        }
        KeyCode::Backspace => {
            let items = if is_headers { &mut app.request.headers } else { &mut app.request.params };
            if app.kv_on_key {
                items[app.kv_row].key.pop();
            } else {
                items[app.kv_row].value.pop();
            }
        }
        KeyCode::Char(c) => {
            let items = if is_headers { &mut app.request.headers } else { &mut app.request.params };
            if app.kv_on_key {
                items[app.kv_row].key.push(c);
            } else {
                items[app.kv_row].value.push(c);
            }
        }
        // Add new row with Ctrl+A
        KeyCode::Enter if app.kv_row == len.saturating_sub(1) => {
            let items = if is_headers { &mut app.request.headers } else { &mut app.request.params };
            items.push(crate::http::models::KeyValue {
                key: String::new(),
                value: String::new(),
            });
            app.kv_row = items.len() - 1;
            app.kv_on_key = true;
        }
        _ => {}
    }
}
