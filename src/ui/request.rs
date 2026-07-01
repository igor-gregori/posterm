use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::app::{App, EditingField, Panel, RequestTab};

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == Panel::Request;

    let env_color = app.environments.active_env()
        .map(|e| {
            let (r, g, b) = crate::storage::env::color_to_rgb(&e.color);
            Color::Rgb(r, g, b)
        });

    let border_color = if is_active {
        env_color.unwrap_or(Color::Cyan)
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .title(" Request ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

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

    let method_color = match app.request.method.as_str() {
        "GET" => Color::Green,
        "POST" => Color::Yellow,
        "PUT" => Color::Blue,
        "DELETE" => Color::Red,
        "PATCH" => Color::Magenta,
        _ => Color::White,
    };
    let method_style = Style::default().fg(method_color).add_modifier(Modifier::BOLD);
    let method = Paragraph::new(Span::styled(
        format!(" {} ", app.request.method.as_str()),
        method_style,
    ));
    frame.render_widget(method, chunks[0]);

    let editing_url = app.editing == Some(EditingField::Url);
    if editing_url {
        let line = render_text_with_cursor(&app.request.url, app.cursor_pos, Color::White);
        frame.render_widget(Paragraph::new(line), chunks[1]);
    } else {
        let (url_text, url_style) = if app.request.url.is_empty() {
            ("https://...".to_string(), Style::default().fg(Color::DarkGray))
        } else {
            (app.request.url.clone(), Style::default().fg(Color::White))
        };
        frame.render_widget(Paragraph::new(Span::styled(url_text, url_style)), chunks[1]);
    }
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
        RequestTab::Headers => draw_kv_editor(frame, app, area, &app.request.headers, EditingField::Headers, ":"),
        RequestTab::Body => draw_body(frame, app, area),
        RequestTab::Params => draw_kv_editor(frame, app, area, &app.request.params, EditingField::Params, "="),
    }
}

fn draw_kv_editor(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    items: &[crate::http::models::KeyValue],
    field: EditingField,
    separator: &str,
) {
    let is_editing = app.editing == Some(field);

    let mut lines: Vec<Line> = Vec::new();
    for (i, kv) in items.iter().enumerate() {
        let is_active_row = is_editing && i == app.kv_row;

        // Display as inline "key<sep>value"
        let display = format!("{}{}{}", kv.key, separator, kv.value);

        let row_marker = if is_active_row { "› " } else { "  " };

        if is_active_row {
            let line = render_kv_line_with_cursor(row_marker, &display, app.cursor_pos);
            lines.push(line);
        } else if display == separator {
            // Empty row placeholder
            lines.push(Line::from(vec![
                Span::styled(row_marker, Style::default().fg(Color::Cyan)),
                Span::styled(format!("key{}value", separator), Style::default().fg(Color::DarkGray)),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled(row_marker, Style::default().fg(Color::Cyan)),
                Span::styled(display, Style::default().fg(Color::White)),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

fn draw_body(frame: &mut Frame, app: &App, area: Rect) {
    let is_editing = app.editing == Some(EditingField::Body);

    if is_editing {
        let line = render_text_with_cursor(&app.request.body, app.cursor_pos, Color::White);
        frame.render_widget(Paragraph::new(line), area);
    } else {
        let (text, style) = if app.request.body.is_empty() {
            ("{ ... }".to_string(), Style::default().fg(Color::DarkGray))
        } else {
            (app.request.body.clone(), Style::default().fg(Color::White))
        };
        frame.render_widget(Paragraph::new(Span::styled(text, style)), area);
    }
}

/// Renders text with a block cursor (inverted colors at cursor position)
fn render_text_with_cursor(text: &str, cursor_pos: usize, color: Color) -> Line<'static> {
    let pos = cursor_pos.min(text.len());
    let (before, after) = text.split_at(pos);

    if after.is_empty() {
        // Cursor at end — show block space
        Line::from(vec![
            Span::styled(before.to_string(), Style::default().fg(color)),
            Span::styled(" ".to_string(), Style::default().fg(Color::Black).bg(Color::White)),
        ])
    } else {
        // Cursor on a character — invert it
        let cursor_char = &after[..1];
        let rest = &after[1..];
        Line::from(vec![
            Span::styled(before.to_string(), Style::default().fg(color)),
            Span::styled(cursor_char.to_string(), Style::default().fg(Color::Black).bg(Color::White)),
            Span::styled(rest.to_string(), Style::default().fg(color)),
        ])
    }
}

/// Renders a KV line with cursor in the inline display
fn render_kv_line_with_cursor(marker: &str, display: &str, cursor_pos: usize) -> Line<'static> {
    let pos = cursor_pos.min(display.len());
    let (before, after) = display.split_at(pos);

    let mut spans = vec![
        Span::styled(marker.to_string(), Style::default().fg(Color::Cyan)),
    ];

    if after.is_empty() {
        spans.push(Span::styled(before.to_string(), Style::default().fg(Color::White)));
        spans.push(Span::styled(" ".to_string(), Style::default().fg(Color::Black).bg(Color::White)));
    } else {
        let cursor_char = &after[..1];
        let rest = &after[1..];
        spans.push(Span::styled(before.to_string(), Style::default().fg(Color::White)));
        spans.push(Span::styled(cursor_char.to_string(), Style::default().fg(Color::Black).bg(Color::White)));
        spans.push(Span::styled(rest.to_string(), Style::default().fg(Color::White)));
    }

    Line::from(spans)
}
