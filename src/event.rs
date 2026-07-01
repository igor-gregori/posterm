use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;

use crate::app::{App, Dialog, EditingField, Panel, RequestTab};
use crate::http;
use crate::http::models::RequestModel;
use crate::storage::collections::{self, Collection, SavedRequest};
use crate::storage::env::{self, Environment, interpolate};

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
        (_, KeyCode::F(1)) => {
            app.dialog = Some(Dialog::Help);
        }

        // Send request: Ctrl+R (with interpolation)
        (KeyModifiers::CONTROL, KeyCode::Char('r')) => {
            if !app.loading && !app.request.url.is_empty() {
                app.loading = true;
                let interpolated = interpolate_request(&app.request, &app.environments);
                let tx = tx.clone();
                tokio::spawn(async move {
                    let result = http::send_request(&interpolated).await;
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
            app.cursor_pos = app.request.url.len();
            app.editing = Some(EditingField::Url);
        }
        (KeyModifiers::CONTROL, KeyCode::Char('h')) => {
            app.request_tab = RequestTab::Headers;
            app.kv_row = 0;
            let kv = &app.request.headers[0];
            app.cursor_pos = format!("{}:{}", kv.key, kv.value).len();
            app.editing = Some(EditingField::Headers);
        }
        (KeyModifiers::CONTROL, KeyCode::Char('b')) => {
            app.request_tab = RequestTab::Body;
            app.cursor_pos = app.request.body.len();
            app.editing = Some(EditingField::Body);
        }
        (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
            app.request_tab = RequestTab::Params;
            app.kv_row = 0;
            let kv = &app.request.params[0];
            app.cursor_pos = format!("{}={}", kv.key, kv.value).len();
            app.editing = Some(EditingField::Params);
        }

        // Save request: Ctrl+S
        (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
            if !app.request.url.is_empty() && !app.collections.is_empty() {
                app.dialog = Some(Dialog::SaveRequest);
                app.dialog_input.clear();
                app.cursor_pos = 0;
            }
        }

        // New collection: Ctrl+N
        (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
            app.dialog = Some(Dialog::NewCollection);
            app.dialog_input.clear();
            app.cursor_pos = 0;
        }

        // Select environment: Ctrl+E
        (KeyModifiers::CONTROL, KeyCode::Char('e')) => {
            app.dialog_selection = 0;
            app.dialog = Some(Dialog::SelectEnv);
        }

        // Edit environment variables: Ctrl+W
        (KeyModifiers::CONTROL, KeyCode::Char('w')) => {
            if let Some(active_env) = app.environments.active_env() {
                app.env_edit_vars = active_env.variables.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                app.env_edit_vars.push((String::new(), String::new()));
                app.env_edit_row = 0;
                app.dialog = Some(Dialog::EditEnvVars);
            }
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

fn interpolate_request(req: &RequestModel, envs: &env::Environments) -> RequestModel {
    let active = envs.active_env();
    RequestModel {
        method: req.method,
        url: interpolate(&req.url, active),
        headers: req.headers.iter().map(|kv| crate::http::models::KeyValue {
            key: interpolate(&kv.key, active),
            value: interpolate(&kv.value, active),
        }).collect(),
        body: interpolate(&req.body, active),
        params: req.params.iter().map(|kv| crate::http::models::KeyValue {
            key: interpolate(&kv.key, active),
            value: interpolate(&kv.value, active),
        }).collect(),
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
            if app.sidebar_collection < app.collections.len() - 1 {
                app.sidebar_collection += 1;
                app.sidebar_request = None;
            }
        }
    } else {
        if app.sidebar_expanded == Some(app.sidebar_collection)
            && !app.collections[app.sidebar_collection].requests.is_empty()
        {
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
        let saved = &app.collections[app.sidebar_collection].requests[ri];
        app.request = saved.to_model();
        app.response = None;
        app.active_panel = Panel::Request;
    } else {
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
        let col = &mut app.collections[app.sidebar_collection];
        col.requests.remove(ri);
        let _ = collections::save_collection(col);
        if ri > 0 {
            app.sidebar_request = Some(ri - 1);
        } else if col.requests.is_empty() {
            app.sidebar_request = None;
        }
    } else {
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
    match app.dialog.unwrap() {
        Dialog::Help => {
            match key.code {
                KeyCode::Esc | KeyCode::F(1) => app.dialog = None,
                _ => {}
            }
        }
        Dialog::SelectEnv => handle_select_env(app, key),
        Dialog::NewEnv => handle_new_env(app, key),
        Dialog::EditEnvVars => handle_edit_env_vars(app, key),
        _ => handle_text_dialog(app, key),
    }
}

fn handle_select_env(app: &mut App, key: KeyEvent) {
    let total = app.environments.environments.len() + 2; // envs + "none" + "new..."

    match key.code {
        KeyCode::Esc => {
            app.dialog = None;
        }
        KeyCode::Up => {
            if app.dialog_selection > 0 {
                app.dialog_selection -= 1;
            }
        }
        KeyCode::Down => {
            if app.dialog_selection < total - 1 {
                app.dialog_selection += 1;
            }
        }
        KeyCode::Char('c') => {
            // Cycle color of selected environment
            let env_count = app.environments.environments.len();
            if app.dialog_selection > 0 && app.dialog_selection <= env_count {
                let env = &mut app.environments.environments[app.dialog_selection - 1];
                let colors: Vec<&str> = env::ENV_COLORS.iter().map(|(name, _, _, _)| *name).collect();
                let current_idx = colors.iter().position(|c| *c == env.color).unwrap_or(0);
                let next_idx = (current_idx + 1) % colors.len();
                env.color = colors[next_idx].to_string();
                let _ = env::save_environments(&app.environments);
            }
        }
        KeyCode::Enter => {
            let env_count = app.environments.environments.len();
            if app.dialog_selection == 0 {
                // "none" — deactivate
                app.environments.active = None;
                let _ = env::save_environments(&app.environments);
                app.dialog = None;
            } else if app.dialog_selection <= env_count {
                // Select an environment
                let name = app.environments.environments[app.dialog_selection - 1].name.clone();
                app.environments.active = Some(name);
                let _ = env::save_environments(&app.environments);
                app.dialog = None;
            } else {
                // "new..." — open new env dialog
                app.dialog_input.clear();
                app.cursor_pos = 0;
                app.dialog = Some(Dialog::NewEnv);
            }
        }
        _ => {}
    }
}

fn handle_new_env(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.dialog = None;
            app.dialog_input.clear();
        }
        KeyCode::Enter => {
            let name = app.dialog_input.trim().to_string();
            if !name.is_empty() {
                let env = Environment {
                    name: name.clone(),
                    variables: std::collections::HashMap::new(),
                    color: "green".to_string(),
                };
                app.environments.environments.push(env);
                app.environments.active = Some(name);
                let _ = env::save_environments(&app.environments);
            }
            app.dialog = None;
            app.dialog_input.clear();
        }
        KeyCode::Left => {
            if app.cursor_pos > 0 { app.cursor_pos -= 1; }
        }
        KeyCode::Right => {
            if app.cursor_pos < app.dialog_input.len() { app.cursor_pos += 1; }
        }
        KeyCode::Home => app.cursor_pos = 0,
        KeyCode::End => app.cursor_pos = app.dialog_input.len(),
        KeyCode::Backspace => {
            if app.cursor_pos > 0 {
                app.dialog_input.remove(app.cursor_pos - 1);
                app.cursor_pos -= 1;
            }
        }
        KeyCode::Delete => {
            if app.cursor_pos < app.dialog_input.len() {
                app.dialog_input.remove(app.cursor_pos);
            }
        }
        KeyCode::Char(c) => {
            app.dialog_input.insert(app.cursor_pos, c);
            app.cursor_pos += 1;
        }
        _ => {}
    }
}

fn handle_edit_env_vars(app: &mut App, key: KeyEvent) {
    let len = app.env_edit_vars.len();
    let inline = format!("{}={}", app.env_edit_vars[app.env_edit_row].0, app.env_edit_vars[app.env_edit_row].1);

    match key.code {
        KeyCode::Esc => {
            save_env_vars(app);
            app.dialog = None;
        }
        KeyCode::Up if app.env_edit_row > 0 => {
            sync_env_var_from_inline(app);
            app.env_edit_row -= 1;
            let new_inline = format!("{}={}", app.env_edit_vars[app.env_edit_row].0, app.env_edit_vars[app.env_edit_row].1);
            app.cursor_pos = new_inline.len();
        }
        KeyCode::Down if app.env_edit_row < len.saturating_sub(1) => {
            sync_env_var_from_inline(app);
            app.env_edit_row += 1;
            let new_inline = format!("{}={}", app.env_edit_vars[app.env_edit_row].0, app.env_edit_vars[app.env_edit_row].1);
            app.cursor_pos = new_inline.len();
        }
        KeyCode::Enter => {
            sync_env_var_from_inline(app);
            app.env_edit_vars.push((String::new(), String::new()));
            app.env_edit_row = app.env_edit_vars.len() - 1;
            app.cursor_pos = 0;
        }
        KeyCode::Left => {
            if app.cursor_pos > 0 { app.cursor_pos -= 1; }
        }
        KeyCode::Right => {
            if app.cursor_pos < inline.len() { app.cursor_pos += 1; }
        }
        KeyCode::Home => app.cursor_pos = 0,
        KeyCode::End => app.cursor_pos = inline.len(),
        KeyCode::Backspace => {
            if app.cursor_pos > 0 {
                let mut s = inline;
                s.remove(app.cursor_pos - 1);
                app.cursor_pos -= 1;
                write_env_inline(app, &s);
            } else if inline.is_empty() || inline == "=" {
                // Delete empty/separator-only row
                if app.env_edit_vars.len() > 1 {
                    app.env_edit_vars.remove(app.env_edit_row);
                    if app.env_edit_row > 0 { app.env_edit_row -= 1; }
                    let new_inline = format!("{}={}", app.env_edit_vars[app.env_edit_row].0, app.env_edit_vars[app.env_edit_row].1);
                    app.cursor_pos = new_inline.len();
                } else {
                    // Last row — clear it
                    app.env_edit_vars[0] = (String::new(), String::new());
                    app.cursor_pos = 0;
                }
            }
        }
        KeyCode::Delete => {
            if app.cursor_pos < inline.len() {
                let mut s = inline;
                s.remove(app.cursor_pos);
                write_env_inline(app, &s);
            }
        }
        KeyCode::Char(c) => {
            let mut s = inline;
            s.insert(app.cursor_pos, c);
            app.cursor_pos += 1;
            write_env_inline(app, &s);
        }
        _ => {}
    }
}

fn sync_env_var_from_inline(app: &mut App) {
    let row = app.env_edit_row;
    let inline = format!("{}={}", app.env_edit_vars[row].0, app.env_edit_vars[row].1);
    let (key, value) = split_env_kv(&inline);
    app.env_edit_vars[row] = (key, value);
}

fn write_env_inline(app: &mut App, inline: &str) {
    let (key, value) = split_env_kv(inline);
    app.env_edit_vars[app.env_edit_row] = (key, value);
}

fn split_env_kv(s: &str) -> (String, String) {
    if let Some(pos) = s.find('=') {
        (s[..pos].to_string(), s[pos + 1..].to_string())
    } else {
        (s.to_string(), String::new())
    }
}

fn save_env_vars(app: &mut App) {
    if let Some(ref active_name) = app.environments.active.clone() {
        if let Some(env) = app.environments.environments.iter_mut().find(|e| &e.name == active_name) {
            env.variables.clear();
            for (k, v) in &app.env_edit_vars {
                if !k.is_empty() {
                    env.variables.insert(k.clone(), v.clone());
                }
            }
            let _ = env::save_environments(&app.environments);
        }
    }
}

fn handle_text_dialog(app: &mut App, key: KeyEvent) {
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
                    _ => {}
                }
            }
            app.dialog = None;
            app.dialog_input.clear();
        }
        KeyCode::Left => {
            if app.cursor_pos > 0 { app.cursor_pos -= 1; }
        }
        KeyCode::Right => {
            if app.cursor_pos < app.dialog_input.len() { app.cursor_pos += 1; }
        }
        KeyCode::Home => app.cursor_pos = 0,
        KeyCode::End => app.cursor_pos = app.dialog_input.len(),
        KeyCode::Backspace => {
            if app.cursor_pos > 0 {
                app.dialog_input.remove(app.cursor_pos - 1);
                app.cursor_pos -= 1;
            }
        }
        KeyCode::Delete => {
            if app.cursor_pos < app.dialog_input.len() {
                app.dialog_input.remove(app.cursor_pos);
            }
        }
        KeyCode::Char(c) => {
            app.dialog_input.insert(app.cursor_pos, c);
            app.cursor_pos += 1;
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
        KeyCode::Left => {
            if app.cursor_pos > 0 {
                app.cursor_pos -= 1;
            }
        }
        KeyCode::Right => {
            let len = get_field(app).len();
            if app.cursor_pos < len {
                app.cursor_pos += 1;
            }
        }
        KeyCode::Home => app.cursor_pos = 0,
        KeyCode::End => {
            app.cursor_pos = get_field(app).len();
        }
        KeyCode::Backspace => {
            if app.cursor_pos > 0 {
                let pos = app.cursor_pos;
                get_field(app).remove(pos - 1);
                app.cursor_pos -= 1;
            }
        }
        KeyCode::Delete => {
            let pos = app.cursor_pos;
            let len = get_field(app).len();
            if pos < len {
                get_field(app).remove(pos);
            }
        }
        KeyCode::Char(c) => {
            let pos = app.cursor_pos;
            get_field(app).insert(pos, c);
            app.cursor_pos += 1;
        }
        _ => {}
    }
}

fn handle_kv_edit(app: &mut App, key: KeyEvent, is_headers: bool) {
    let sep = if is_headers { ":" } else { "=" };
    let items = if is_headers { &app.request.headers } else { &app.request.params };
    let len = items.len();

    // Build the inline string for current row
    let inline = {
        let kv = &items[app.kv_row];
        format!("{}{}{}", kv.key, sep, kv.value)
    };

    match key.code {
        KeyCode::Esc => {
            // Sync current inline back before exiting
            sync_kv_from_cursor(app, is_headers, sep);
            app.editing = None;
        }
        KeyCode::Up if app.kv_row > 0 => {
            sync_kv_from_cursor(app, is_headers, sep);
            app.kv_row -= 1;
            let items = if is_headers { &app.request.headers } else { &app.request.params };
            let new_inline = format!("{}{}{}", items[app.kv_row].key, sep, items[app.kv_row].value);
            app.cursor_pos = new_inline.len();
        }
        KeyCode::Down if app.kv_row < len.saturating_sub(1) => {
            sync_kv_from_cursor(app, is_headers, sep);
            app.kv_row += 1;
            let items = if is_headers { &app.request.headers } else { &app.request.params };
            let new_inline = format!("{}{}{}", items[app.kv_row].key, sep, items[app.kv_row].value);
            app.cursor_pos = new_inline.len();
        }
        KeyCode::Enter => {
            sync_kv_from_cursor(app, is_headers, sep);
            // Add new row
            let items = if is_headers { &mut app.request.headers } else { &mut app.request.params };
            items.push(crate::http::models::KeyValue {
                key: String::new(),
                value: String::new(),
            });
            app.kv_row = items.len() - 1;
            app.cursor_pos = 0;
        }
        KeyCode::Left => {
            if app.cursor_pos > 0 { app.cursor_pos -= 1; }
        }
        KeyCode::Right => {
            if app.cursor_pos < inline.len() { app.cursor_pos += 1; }
        }
        KeyCode::Home => app.cursor_pos = 0,
        KeyCode::End => app.cursor_pos = inline.len(),
        KeyCode::Backspace => {
            if app.cursor_pos > 0 {
                let mut s = inline;
                s.remove(app.cursor_pos - 1);
                app.cursor_pos -= 1;
                write_inline(app, is_headers, sep, &s);
            } else if inline.is_empty() || inline == sep {
                // Delete empty row
                let items = if is_headers { &mut app.request.headers } else { &mut app.request.params };
                if items.len() > 1 {
                    items.remove(app.kv_row);
                    if app.kv_row > 0 { app.kv_row -= 1; }
                    let items = if is_headers { &app.request.headers } else { &app.request.params };
                    let new_inline = format!("{}{}{}", items[app.kv_row].key, sep, items[app.kv_row].value);
                    app.cursor_pos = new_inline.len();
                }
            }
        }
        KeyCode::Delete => {
            if app.cursor_pos < inline.len() {
                let mut s = inline;
                s.remove(app.cursor_pos);
                write_inline(app, is_headers, sep, &s);
            }
        }
        KeyCode::Char(c) => {
            let mut s = inline;
            s.insert(app.cursor_pos, c);
            app.cursor_pos += 1;
            write_inline(app, is_headers, sep, &s);
        }
        _ => {}
    }
}

/// Sync the inline string back into the KeyValue struct by splitting on separator
fn sync_kv_from_cursor(app: &mut App, is_headers: bool, sep: &str) {
    let items = if is_headers { &mut app.request.headers } else { &mut app.request.params };
    let kv = &items[app.kv_row];
    let inline = format!("{}{}{}", kv.key, sep, kv.value);
    let (key, value) = split_kv(&inline, sep);
    items[app.kv_row].key = key;
    items[app.kv_row].value = value;
}

/// Write an inline string back into the KV struct
fn write_inline(app: &mut App, is_headers: bool, sep: &str, inline: &str) {
    let items = if is_headers { &mut app.request.headers } else { &mut app.request.params };
    let (key, value) = split_kv(inline, sep);
    items[app.kv_row].key = key;
    items[app.kv_row].value = value;
}

/// Split "key<sep>value" into (key, value). If no separator, all goes to key.
fn split_kv(s: &str, sep: &str) -> (String, String) {
    if let Some(pos) = s.find(sep) {
        (s[..pos].to_string(), s[pos + sep.len()..].to_string())
    } else {
        (s.to_string(), String::new())
    }
}
