use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, EditingField, Panel};

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

    // Compute dynamic heights
    let header_lines = app.request.headers.len().max(1);
    let param_lines = app.request.params.len().max(1);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),                    // method + url
            Constraint::Length(1),                    // separator
            Constraint::Length(1),                    // headers label
            Constraint::Length(header_lines as u16),  // headers
            Constraint::Length(1),                    // separator
            Constraint::Length(1),                    // params label
            Constraint::Length(param_lines as u16),   // params
            Constraint::Length(1),                    // separator
            Constraint::Length(1),                    // body label
            Constraint::Min(1),                      // body
        ])
        .split(inner);

    draw_method_url(frame, app, chunks[0]);
    draw_separator(frame, chunks[1]);
    draw_label(frame, "Headers", EditingField::Headers, app, chunks[2]);
    draw_kv_section(frame, app, chunks[3], &app.request.headers, EditingField::Headers, app.placeholder_header);
    draw_separator(frame, chunks[4]);
    draw_label(frame, "Params", EditingField::Params, app, chunks[5]);
    draw_kv_section(frame, app, chunks[6], &app.request.params, EditingField::Params, app.placeholder_param);
    draw_separator(frame, chunks[7]);
    draw_label(frame, "Body", EditingField::Body, app, chunks[8]);
    draw_body(frame, app, chunks[9]);
}

fn draw_separator(frame: &mut Frame, area: Rect) {
    let sep = Paragraph::new(Line::from(Span::styled(
        "─".repeat(area.width as usize),
        Style::default().fg(Color::DarkGray),
    )));
    frame.render_widget(sep, area);
}

fn draw_method_url(frame: &mut Frame, app: &App, area: Rect) {
    let method_width = 8u16;
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(method_width), Constraint::Min(1)])
        .split(area);

    let method_color = method_color(app.request.method.as_str());
    frame.render_widget(
        Paragraph::new(Span::styled(
            format!(" {} ", app.request.method.as_str()),
            Style::default().fg(method_color).add_modifier(Modifier::BOLD),
        )),
        chunks[0],
    );

    if app.editing == Some(EditingField::Url) {
        let line = render_text_with_cursor(&app.request.url, app.cursor_pos, Color::White);
        frame.render_widget(Paragraph::new(line), chunks[1]);
    } else {
        let (text, style) = if app.request.url.is_empty() {
            ("https://api.example.com/endpoint", Style::default().fg(Color::DarkGray))
        } else {
            (app.request.url.as_str(), Style::default().fg(Color::White))
        };
        frame.render_widget(Paragraph::new(Span::styled(text, style)), chunks[1]);
    }
}

fn draw_label(frame: &mut Frame, label: &str, field: EditingField, app: &App, area: Rect) {
    let is_editing = app.editing == Some(field);
    let style = if is_editing {
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    frame.render_widget(Paragraph::new(Span::styled(format!(" {}", label), style)), area);
}

fn draw_kv_section(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    items: &[crate::http::models::KeyValue],
    field: EditingField,
    placeholder: &str,
) {
    let is_editing = app.editing == Some(field);
    let sep = if field == EditingField::Headers { ": " } else { "=" };

    let mut lines: Vec<Line> = Vec::new();
    for (i, kv) in items.iter().enumerate() {
        let is_active_row = is_editing && i == app.kv_row;
        let is_empty = kv.key.is_empty() && kv.value.is_empty();
        // Invalid: has text in key but no value (separator not found)
        let is_invalid = !kv.key.is_empty() && kv.value.is_empty() && {
            let check_sep = if field == EditingField::Headers { ":" } else { "=" };
            !kv.key.contains(check_sep)
        };

        if is_active_row {
            // Build display from raw content
            let display = if is_empty {
                String::new()
            } else if kv.value.is_empty() {
                kv.key.clone()
            } else {
                format!("{}{}{}", kv.key, sep, kv.value)
            };
            lines.push(render_kv_line_with_cursor(&display, app.cursor_pos));
        } else if is_empty {
            lines.push(Line::from(Span::styled(
                format!("   {}", placeholder),
                Style::default().fg(Color::DarkGray),
            )));
        } else if is_invalid {
            // Show in red with hint
            let display = format!("   {} ⚠", kv.key);
            lines.push(Line::from(Span::styled(
                display,
                Style::default().fg(Color::Red),
            )));
        } else {
            let display = format!("   {}{}{}", kv.key, sep, kv.value);
            lines.push(Line::from(Span::styled(
                display,
                Style::default().fg(Color::White),
            )));
        }
    }

    frame.render_widget(Paragraph::new(lines), area);
}

fn draw_body(frame: &mut Frame, app: &App, area: Rect) {
    if app.editing == Some(EditingField::Body) {
        let lines = render_multiline_with_cursor(&app.request.body, app.cursor_pos);
        frame.render_widget(Paragraph::new(lines), area);
    } else if app.request.body.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                "   { \"message\": \"hello world\" }",
                Style::default().fg(Color::DarkGray),
            )),
            area,
        );
    } else {
        let lines = highlight_json_body(&app.request.body);
        frame.render_widget(Paragraph::new(lines), area);
    }
}

fn highlight_json_body(text: &str) -> Vec<Line<'static>> {
    use syntect::easy::HighlightLines;
    use syntect::highlighting::ThemeSet;
    use syntect::parsing::SyntaxSet;
    use std::sync::OnceLock;

    static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
    static THEME_SET: OnceLock<ThemeSet> = OnceLock::new();

    let ss = SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_newlines);
    let ts = THEME_SET.get_or_init(ThemeSet::load_defaults);
    let syntax = ss.find_syntax_by_extension("json").unwrap_or_else(|| ss.find_syntax_plain_text());
    let theme = &ts.themes["base16-ocean.dark"];
    let mut h = HighlightLines::new(syntax, theme);

    text.lines()
        .map(|line| {
            let spans: Vec<Span<'static>> = match h.highlight_line(line, ss) {
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

fn render_kv_line_with_cursor(display: &str, cursor_pos: usize) -> Line<'static> {
    let pos = snap_to_char_boundary(display, cursor_pos);
    let (before, after) = display.split_at(pos);

    let mut spans = vec![
        Span::styled(" › ", Style::default().fg(Color::Cyan)),
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
