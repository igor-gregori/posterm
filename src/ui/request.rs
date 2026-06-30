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
        let url_line = Line::from(vec![
            Span::styled(&app.request.url, Style::default().fg(Color::White)),
            Span::styled("▌", Style::default().fg(Color::Cyan)),
        ]);
        let url = Paragraph::new(url_line);
        frame.render_widget(url, chunks[1]);
    } else {
        let (url_text, url_style) = if app.request.url.is_empty() {
            ("https://...".to_string(), Style::default().fg(Color::DarkGray))
        } else {
            (app.request.url.clone(), Style::default().fg(Color::White))
        };
        let url = Paragraph::new(Span::styled(url_text, url_style));
        frame.render_widget(url, chunks[1]);
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
        RequestTab::Headers => draw_kv_editor(frame, app, area, &app.request.headers, EditingField::Headers),
        RequestTab::Body => draw_body(frame, app, area),
        RequestTab::Params => draw_kv_editor(frame, app, area, &app.request.params, EditingField::Params),
    }
}

fn draw_kv_editor(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    items: &[crate::http::models::KeyValue],
    field: EditingField,
) {
    let is_editing = app.editing == Some(field);

    let mut lines: Vec<Line> = Vec::new();
    for (i, kv) in items.iter().enumerate() {
        let is_active_row = is_editing && i == app.kv_row;

        let key_text = if kv.key.is_empty() && is_active_row && app.kv_on_key {
            "▌".to_string()
        } else if kv.key.is_empty() {
            "key".to_string()
        } else if is_active_row && app.kv_on_key {
            format!("{}▌", kv.key)
        } else {
            kv.key.clone()
        };

        let val_text = if kv.value.is_empty() && is_active_row && !app.kv_on_key {
            "▌".to_string()
        } else if kv.value.is_empty() {
            "value".to_string()
        } else if is_active_row && !app.kv_on_key {
            format!("{}▌", kv.value)
        } else {
            kv.value.clone()
        };

        let key_style = if is_active_row && app.kv_on_key {
            Style::default().fg(Color::White).bg(Color::DarkGray)
        } else if kv.key.is_empty() {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Green)
        };

        let val_style = if is_active_row && !app.kv_on_key {
            Style::default().fg(Color::White).bg(Color::DarkGray)
        } else if kv.value.is_empty() {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::White)
        };

        let row_marker = if is_active_row { "› " } else { "  " };

        lines.push(Line::from(vec![
            Span::styled(row_marker, Style::default().fg(Color::Cyan)),
            Span::styled(key_text, key_style),
            Span::styled(" : ", Style::default().fg(Color::DarkGray)),
            Span::styled(val_text, val_style),
        ]));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

fn draw_body(frame: &mut Frame, app: &App, area: Rect) {
    let is_editing = app.editing == Some(EditingField::Body);
    let text = if app.request.body.is_empty() && !is_editing {
        "{ ... }".to_string()
    } else if is_editing {
        format!("{}▌", app.request.body)
    } else {
        app.request.body.clone()
    };

    let style = if is_editing {
        Style::default().fg(Color::White).bg(Color::DarkGray)
    } else if app.request.body.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let paragraph = Paragraph::new(Span::styled(text, style));
    frame.render_widget(paragraph, area);
}
