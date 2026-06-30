use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;

use crate::app::{App, Dialog, EditingField, Panel, RequestTab};
use crate::http;
use crate::storage::collections::{self, Collection, SavedRequest};

pub fn handle_events(
    app: &mut App,
    tx: &mpsc::Sender<Result<http::Response, String>>,
) -> std::io::Result<()> {
    if event::poll(std::time::Duration::from_millis(16))? {
        if let Event::Key(key) = event::read()? {
            if app.dialog.is_some() {
                handle_dialog(app, key);
            } else if app.editing.is_some() {
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

        // Save request: Ctrl+S
        (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
            if !app.request.url.is_empty() && !app.collections.is_empty() {
                app.dialog = Some(Dialog::SaveRequest);
                app.dialog_input.clear();
            }
        }

        // New collection: Ctrl+N
        (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
            app.dialog = Some(Dialog::NewCollection);
            app.dialog_input.clear();
        }

        // Sidebar navigation
        (KeyModifiers::NONE, KeyCode::Up) if app.active_panel == Panel::Sidebar => {
            handle_sidebar_up(app);
        }
        (KeyModifiers::NONE, KeyCode::Down) if app.active_panel == Panel::Sidebar => {
            handle_sidebar_down(app);
        }
        (KeyModifiers::NONE, KeyCode::Enter) if app.active_panel == Panel::Sidebar => {
            handle_sidebar_enter(app);
        }
        (KeyModifiers::NONE, KeyCode::Char('d')) if app.active_panel == Panel::Sidebar => {
            handle_sidebar_delete(app);
        }

        _ => {}
    }
}

fn handle_sidebar_up(app: &mut App) {
    if let Some(ri) = app.sidebar_request {
        if ri > 0 {
            app.sidebar_request = Some(ri - 1);
        } else {
            app.sidebar_request = None;
        }
    } else if app.sidebar_collection > 0 {
        app.sidebar_collection -= 1;
        // If previous collection is expanded, go to its last request
        if app.sidebar_expanded == Some(app.sidebar_collection) {
            let len = app.collections[app.sidebar_collection].requests.len();
            if len > 0 {
                app.sidebar_request = Some(len - 1);
            }
        }
    }
}

fn handle_sidebar_down(app: &mut App) {
    if app.collections.is_empty() {
        return;
    }

    if let Some(ri) = app.sidebar_request {
        let col = &app.collections[app.sidebar_collection];
        if ri < col.requests.len() - 1 {
            app.sidebar_request = Some(ri + 1);
        } else {
            // Move to next collection
            if app.sidebar_collection < app.collections.len() - 1 {
                app.sidebar_collection += 1;
                app.sidebar_request = None;
            }
        }
    } else {
        // On a collection row
        if app.sidebar_expanded == Some(app.sidebar_collection)
            && !app.collections[app.sidebar_collection].requests.is_empty()
        {
            // Go into first request
            app.sidebar_request = Some(0);
        } else if app.sidebar_collection < app.collections.len() - 1 {
            app.sidebar_collection += 1;
        }
    }
}

fn handle_sidebar_enter(app: &mut App) {
    if app.collections.is_empty() {
        return;
    }

    if let Some(ri) = app.sidebar_request {
        // Load request into editor
        let saved = &app.collections[app.sidebar_collection].requests[ri];
        app.request = saved.to_model();
        app.response = None;
        app.active_panel = Panel::Request;
    } else {
        // Toggle expand/collapse
        if app.sidebar_expanded == Some(app.sidebar_collection) {
            app.sidebar_expanded = None;
        } else {
            app.sidebar_expanded = Some(app.sidebar_collection);
        }
    }
}

fn handle_sidebar_delete(app: &mut App) {
    if app.collections.is_empty() {
        return;
    }

    if let Some(ri) = app.sidebar_request {
        // Delete request from collection
        let col = &mut app.collections[app.sidebar_collection];
        col.requests.remove(ri);
        let _ = collections::save_collection(col);
        if ri > 0 {
            app.sidebar_request = Some(ri - 1);
        } else if col.requests.is_empty() {
            app.sidebar_request = None;
        }
    } else {
        // Delete entire collection
        let name = app.collections[app.sidebar_collection].name.clone();
        let _ = collections::delete_collection(&name);
        app.collections.remove(app.sidebar_collection);
        if app.sidebar_collection > 0 && app.sidebar_collection >= app.collections.len() {
            app.sidebar_collection = app.collections.len().saturating_sub(1);
        }
        app.sidebar_expanded = None;
    }
}

fn handle_dialog(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.dialog = None;
            app.dialog_input.clear();
        }
        KeyCode::Enter => {
            let input = app.dialog_input.trim().to_string();
            if !input.is_empty() {
                match app.dialog.unwrap() {
                    Dialog::SaveRequest => {
                        let saved = SavedRequest::from_model(&input, &app.request);
                        let col = &mut app.collections[app.sidebar_collection];
                        col.requests.push(saved);
                        let _ = collections::save_collection(col);
                    }
                    Dialog::NewCollection => {
                        let col = Collection {
                            name: input,
                            requests: Vec::new(),
                        };
                        let _ = collections::save_collection(&col);
                        app.collections.push(col);
                        app.sidebar_collection = app.collections.len() - 1;
                    }
                }
            }
            app.dialog = None;
            app.dialog_input.clear();
        }
        KeyCode::Backspace => {
            app.dialog_input.pop();
        }
        KeyCode::Char(c) => {
            app.dialog_input.push(c);
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

fn handle_text_edit(
    app: &mut App,
    key: KeyEvent,
    get_field: &mut dyn FnMut(&mut App) -> &mut String,
) {
    match key.code {
        KeyCode::Esc | KeyCode::Enter => app.editing = None,
        KeyCode::Backspace => {
            get_field(app).pop();
        }
        KeyCode::Char(c) => get_field(app).push(c),
        _ => {}
    }
}

fn handle_kv_edit(app: &mut App, key: KeyEvent, is_headers: bool) {
    let items = if is_headers {
        &app.request.headers
    } else {
        &app.request.params
    };
    let len = items.len();

    match key.code {
        KeyCode::Esc => app.editing = None,
        KeyCode::Tab => app.kv_on_key = !app.kv_on_key,
        KeyCode::Down if app.kv_row < len.saturating_sub(1) => {
            app.kv_row += 1;
            app.kv_on_key = true;
        }
        KeyCode::Up if app.kv_row > 0 => {
            app.kv_row -= 1;
            app.kv_on_key = true;
        }
        KeyCode::Backspace => {
            let items = if is_headers {
                &mut app.request.headers
            } else {
                &mut app.request.params
            };
            if app.kv_on_key {
                items[app.kv_row].key.pop();
            } else {
                items[app.kv_row].value.pop();
            }
        }
        KeyCode::Char(c) => {
            let items = if is_headers {
                &mut app.request.headers
            } else {
                &mut app.request.params
            };
            if app.kv_on_key {
                items[app.kv_row].key.push(c);
            } else {
                items[app.kv_row].value.push(c);
            }
        }
        KeyCode::Enter if app.kv_row == len.saturating_sub(1) => {
            let items = if is_headers {
                &mut app.request.headers
            } else {
                &mut app.request.params
            };
            items.push(crate::http::models::KeyValue {
                key: String::new(),
                value: String::new(),
            });
            app.kv_row = items.len() - 1;
            app.kv_on_key = true;
        }
        KeyCode::Enter => app.editing = None,
        _ => {}
    }
}
