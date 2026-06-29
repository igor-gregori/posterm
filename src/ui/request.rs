use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::app::{App, Panel, RequestFocus, RequestTab};
use crate::http::models::KeyValue;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == Panel::Request;

    let block = Block::default()
        .title(" Request ")
        .borders(Borders::ALL)
        .border_style(if is_active {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        });

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // method + url
            Constraint::Length(1), // tabs
            Constraint::Min(1),   // tab content
        ])
        .split(inner);

    draw_method_url(frame, app, chunks[0]);
    draw_tabs(frame, app, chunks[1]);
    draw_tab_content(frame, app, chunks[2]);
}

fn draw_method_url(frame: &mut Frame, app: &App, area: Rect) {
    let method_width = 8u16;
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(method_width), Constraint::Min(1)])
        .split(area);

    let method_style = if app.request_focus == RequestFocus::Method && app.active_panel == Panel::Request {
        Style::default().fg(Color::Black).bg(Color::Cyan)
    } else {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    };
    let method = Paragraph::new(Span::styled(
        format!(" {} ", app.request.method.as_str()),
        method_style,
    ));
    frame.render_widget(method, chunks[0]);

    let url_style = if app.request_focus == RequestFocus::Url && app.active_panel == Panel::Request {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let url_text = if app.request.url.is_empty() {
        "https://..."
    } else {
        &app.request.url
    };
    let url = Paragraph::new(Span::styled(url_text, url_style));
    frame.render_widget(url, chunks[1]);
}

fn draw_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["Headers", "Body", "Params"];
    let selected = match app.request_tab {
        RequestTab::Headers => 0,
        RequestTab::Body => 1,
        RequestTab::Params => 2,
    };
    let tabs = Tabs::new(titles)
        .select(selected)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .divider("│");
    frame.render_widget(tabs, area);
}

fn draw_tab_content(frame: &mut Frame, app: &App, area: Rect) {
    match app.request_tab {
        RequestTab::Headers => draw_kv_editor(frame, app, area, &app.request.headers, "headers"),
        RequestTab::Body => draw_body(frame, app, area),
        RequestTab::Params => draw_kv_editor(frame, app, area, &app.request.params, "params"),
    }
}

fn draw_kv_editor(frame: &mut Frame, app: &App, area: Rect, items: &[KeyValue], _label: &str) {
    let is_tab_focused = app.active_panel == Panel::Request && app.request_focus == RequestFocus::Tab;

    let mut lines: Vec<Line> = Vec::new();
    for (i, kv) in items.iter().enumerate() {
        let is_selected = is_tab_focused && i == app.kv_row;
        let key_style = if is_selected && app.kv_on_key {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else {
            Style::default().fg(Color::Green)
        };
        let val_style = if is_selected && !app.kv_on_key {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else {
            Style::default().fg(Color::White)
        };

        let key_text = if kv.key.is_empty() { "key" } else { &kv.key };
        let val_text = if kv.value.is_empty() { "value" } else { &kv.value };

        lines.push(Line::from(vec![
            Span::styled(key_text, key_style),
            Span::styled(": ", Style::default().fg(Color::DarkGray)),
            Span::styled(val_text, val_style),
        ]));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "  (empty — press 'a' to add)",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

fn draw_body(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.active_panel == Panel::Request && app.request_focus == RequestFocus::Tab;
    let style = if is_focused {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let text = if app.request.body.is_empty() {
        "{ ... }"
    } else {
        &app.request.body
    };

    let paragraph = Paragraph::new(text).style(style);
    frame.render_widget(paragraph, area);
}
