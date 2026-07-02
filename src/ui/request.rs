use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, EditingField, Panel, RequestTab};

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == Panel::Request;

    let active_color = app.environments.active_env()
        .map(|e| {
            let (r, g, b) = crate::storage::env::color_to_rgb(&e.color);
            Color::Rgb(r, g, b)
        })
        .unwrap_or(Color::Cyan);

    let border_color = if is_active { active_color } else { Color::DarkGray };

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
            Constraint::Length(1), // separator
            Constraint::Length(1), // tabs
            Constraint::Length(1), // tab underline
            Constraint::Min(1),   // tab content
        ])
        .split(inner);

    draw_method_url(frame, app, chunks[0]);
    // separator line
    let sep = Paragraph::new(Line::from(Span::styled(
        "─".repeat(chunks[1].width as usize),
        Style::default().fg(Color::DarkGray),
    )));
    frame.render_widget(sep, chunks[1]);
    draw_tabs(frame, app, chunks[2]);
    // tab underline
    let underline = Paragraph::new(Line::from(Span::styled(
        "─".repeat(chunks[3].width as usize),
        Style::default().fg(Color::DarkGray),
    )));
    frame.render_widget(underline, chunks[3]);
    draw_tab_content(frame, app, chunks[4]);
}

fn draw_method_url(frame: &mut Frame, app: &App, area: Rect) {
    let method_width = 8u16;
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(method_width), Constraint::Min(1)])
        .split(area);

    let method_color = method_color(app.request.method.as_str());
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
            ("https://...", Style::default().fg(Color::DarkGray))
        } else {
            (app.request.url.as_str(), Style::default().fg(Color::White))
        };
        frame.render_widget(Paragraph::new(Span::styled(url_text, url_style)), chunks[1]);
    }
}

fn draw_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let titles = ["Headers", "Body", "Params"];
    let selected = match app.request_tab {
        RequestTab::Headers => 0,
        RequestTab::Body => 1,
        RequestTab::Params => 2,
    };

    let is_editing_tab = matches!(
        app.editing,
        Some(EditingField::Headers) | Some(EditingField::Body) | Some(EditingField::Params)
    );

    // Split area into 2 rows: tabs (1) + underline (1)
    let tab_area = Rect { height: 1, ..area };

    let width = area.width as usize;
    let tab_width = width / 3;
    let mut spans: Vec<Span> = Vec::new();

    for (i, title) in titles.iter().enumerate() {
        let is_selected = i == selected;
        let padded = format!("{:^w$}", title, w = tab_width);

        let style = if is_selected && is_editing_tab {
            Style::default().fg(Color::Black).bg(Color::White).add_modifier(Modifier::BOLD)
        } else if is_selected {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        spans.push(Span::styled(padded, style));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), tab_area);
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
        let display = format!("{}{}{}", kv.key, separator, kv.value);

        if is_active_row {
            let line = render_kv_line_with_cursor("› ", &display, app.cursor_pos);
            lines.push(line);
        } else if display == separator {
            lines.push(Line::from(Span::styled(
                format!("  key{}value", separator),
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(display, Style::default().fg(Color::White)),
            ]));
        }
    }

    if !is_editing {
        // Show hint when not editing
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  Ctrl+{} to edit", if separator == ":" { "D" } else { "P" }),
            Style::default().fg(Color::DarkGray),
        )));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

fn draw_body(frame: &mut Frame, app: &App, area: Rect) {
    let is_editing = app.editing == Some(EditingField::Body);

    if is_editing {
        let lines = render_multiline_with_cursor(&app.request.body, app.cursor_pos);
        frame.render_widget(Paragraph::new(lines), area);
    } else if app.request.body.is_empty() {
        let lines = vec![
            Line::from(Span::styled("  { ... }", Style::default().fg(Color::DarkGray))),
            Line::from(""),
            Line::from(Span::styled("  Ctrl+B to edit", Style::default().fg(Color::DarkGray))),
        ];
        frame.render_widget(Paragraph::new(lines), area);
    } else {
        let lines = highlight_json_body(&app.request.body);
        frame.render_widget(Paragraph::new(lines), area);
    }
}

fn highlight_json_body(text: &str) -> Vec<Line<'static>> {
    use syntect::easy::HighlightLines;
    use syntect::highlighting::ThemeSet;
    use syntect::parsing::SyntaxSet;

    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let syntax = ss.find_syntax_by_extension("json").unwrap_or_else(|| ss.find_syntax_plain_text());
    let theme = &ts.themes["base16-ocean.dark"];
    let mut h = HighlightLines::new(syntax, theme);

    text.lines()
        .map(|line| {
            let spans: Vec<Span<'static>> = match h.highlight_line(line, &ss) {
                Ok(ranges) => ranges
                    .into_iter()
                    .map(|(style, text)| {
                        let fg = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                        Span::styled(text.to_string(), Style::default().fg(fg))
                    })
                    .collect(),
                Err(_) => vec![Span::raw(line.to_string())],
            };
            Line::from(spans)
        })
        .collect()
}

fn method_color(method: &str) -> Color {
    match method {
        "GET" => Color::Green,
        "POST" => Color::Yellow,
        "PUT" => Color::Blue,
        "DELETE" => Color::Red,
        "PATCH" => Color::Magenta,
        _ => Color::White,
    }
}

/// Renders multiline text with a block cursor at the given position
fn render_multiline_with_cursor(text: &str, cursor_pos: usize) -> Vec<Line<'static>> {
    let pos = snap_to_char_boundary(text, cursor_pos);
    let before_cursor = &text[..pos];

    let full_before_lines: Vec<&str> = before_cursor.split('\n').collect();
    let cursor_line_idx = full_before_lines.len() - 1;
    let cursor_col = full_before_lines.last().unwrap_or(&"").len();

    let all_lines: Vec<&str> = text.split('\n').collect();
    let mut result: Vec<Line<'static>> = Vec::new();

    for (i, line_text) in all_lines.iter().enumerate() {
        if i == cursor_line_idx {
            let col = snap_to_char_boundary(line_text, cursor_col);
            let (before, after) = line_text.split_at(col);

            if after.is_empty() {
                result.push(Line::from(vec![
                    Span::styled(before.to_string(), Style::default().fg(Color::White)),
                    Span::styled(" ".to_string(), Style::default().fg(Color::Black).bg(Color::White)),
                ]));
            } else {
                let ch = after.chars().next().unwrap();
                let ch_len = ch.len_utf8();
                result.push(Line::from(vec![
                    Span::styled(before.to_string(), Style::default().fg(Color::White)),
                    Span::styled(after[..ch_len].to_string(), Style::default().fg(Color::Black).bg(Color::White)),
                    Span::styled(after[ch_len..].to_string(), Style::default().fg(Color::White)),
                ]));
            }
        } else {
            result.push(Line::from(Span::styled(line_text.to_string(), Style::default().fg(Color::White))));
        }
    }

    result
}

/// Renders text with a block cursor (inverted colors at cursor position)
fn render_text_with_cursor(text: &str, cursor_pos: usize, color: Color) -> Line<'static> {
    let pos = snap_to_char_boundary(text, cursor_pos);
    let (before, after) = text.split_at(pos);

    if after.is_empty() {
        Line::from(vec![
            Span::styled(before.to_string(), Style::default().fg(color)),
            Span::styled(" ".to_string(), Style::default().fg(Color::Black).bg(Color::White)),
        ])
    } else {
        let ch = after.chars().next().unwrap();
        let ch_len = ch.len_utf8();
        Line::from(vec![
            Span::styled(before.to_string(), Style::default().fg(color)),
            Span::styled(after[..ch_len].to_string(), Style::default().fg(Color::Black).bg(Color::White)),
            Span::styled(after[ch_len..].to_string(), Style::default().fg(color)),
        ])
    }
}

/// Renders a KV line with cursor in the inline display
fn render_kv_line_with_cursor(marker: &str, display: &str, cursor_pos: usize) -> Line<'static> {
    let pos = snap_to_char_boundary(display, cursor_pos);
    let (before, after) = display.split_at(pos);

    let mut spans = vec![
        Span::styled(marker.to_string(), Style::default().fg(Color::Cyan)),
    ];

    if after.is_empty() {
        spans.push(Span::styled(before.to_string(), Style::default().fg(Color::White)));
        spans.push(Span::styled(" ".to_string(), Style::default().fg(Color::Black).bg(Color::White)));
    } else {
        let ch = after.chars().next().unwrap();
        let ch_len = ch.len_utf8();
        spans.push(Span::styled(before.to_string(), Style::default().fg(Color::White)));
        spans.push(Span::styled(after[..ch_len].to_string(), Style::default().fg(Color::Black).bg(Color::White)));
        spans.push(Span::styled(after[ch_len..].to_string(), Style::default().fg(Color::White)));
    }

    Line::from(spans)
}

/// Snap a byte position to the nearest valid char boundary (rounding down)
fn snap_to_char_boundary(s: &str, pos: usize) -> usize {
    let pos = pos.min(s.len());
    if s.is_char_boundary(pos) {
        return pos;
    }
    let mut p = pos;
    while p > 0 && !s.is_char_boundary(p) {
        p -= 1;
    }
    p
}
