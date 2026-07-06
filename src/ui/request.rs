use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, EditingField, Panel};

/// Draw request panel and return cursor position (x, y) if editing
pub fn draw(frame: &mut Frame, app: &App, area: Rect) -> Option<(u16, u16)> {
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

    let mut cursor_pos_xy: Option<(u16, u16)> = None;

    // URL
    cursor_pos_xy = cursor_pos_xy.or(draw_method_url(frame, app, chunks[0]));
    draw_separator(frame, chunks[1]);
    draw_label(frame, "Headers", EditingField::Headers, app, chunks[2]);
    cursor_pos_xy = cursor_pos_xy.or(draw_kv_section(frame, app, chunks[3], &app.request.headers, EditingField::Headers, app.placeholder_header));
    draw_separator(frame, chunks[4]);
    draw_label(frame, "Params", EditingField::Params, app, chunks[5]);
    cursor_pos_xy = cursor_pos_xy.or(draw_kv_section(frame, app, chunks[6], &app.request.params, EditingField::Params, app.placeholder_param));
    draw_separator(frame, chunks[7]);
    draw_label(frame, "Body", EditingField::Body, app, chunks[8]);
    cursor_pos_xy = cursor_pos_xy.or(draw_body(frame, app, chunks[9]));

    cursor_pos_xy
}

fn draw_separator(frame: &mut Frame, area: Rect) {
    let sep = Paragraph::new(Line::from(Span::styled(
        "─".repeat(area.width as usize),
        Style::default().fg(Color::DarkGray),
    )));
    frame.render_widget(sep, area);
}

fn draw_method_url(frame: &mut Frame, app: &App, area: Rect) -> Option<(u16, u16)> {
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
        let available_width = chunks[1].width as usize;
        let pos = app.cursor_pos.min(app.request.url.len());

        // Horizontal scroll
        let (visible, cursor_in_visible) = scroll_text(&app.request.url, pos, available_width);
        frame.render_widget(
            Paragraph::new(Span::styled(visible, Style::default().fg(Color::White))),
            chunks[1],
        );

        Some((chunks[1].x + cursor_in_visible as u16, chunks[1].y))
    } else {
        let (text, style) = if app.request.url.is_empty() {
            ("https://api.example.com/endpoint", Style::default().fg(Color::DarkGray))
        } else {
            (app.request.url.as_str(), Style::default().fg(Color::White))
        };
        let display = truncate(text, chunks[1].width as usize);
        frame.render_widget(Paragraph::new(Span::styled(display, style)), chunks[1]);
        None
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
) -> Option<(u16, u16)> {
    let is_editing = app.editing == Some(field);
    let sep = if field == EditingField::Headers { ": " } else { "=" };
    let mut cursor_xy: Option<(u16, u16)> = None;

    let mut lines: Vec<Line> = Vec::new();
    for (i, kv) in items.iter().enumerate() {
        let is_active_row = is_editing && i == app.kv_row;
        let is_empty = kv.key.is_empty() && kv.value.is_empty();
        let is_invalid = !kv.key.is_empty() && kv.value.is_empty() && {
            let check_sep = if field == EditingField::Headers { ":" } else { "=" };
            !kv.key.contains(check_sep)
        };

        if is_active_row {
            let display = if is_empty {
                String::new()
            } else if kv.value.is_empty() {
                kv.key.clone()
            } else {
                format!("{}{}{}", kv.key, sep, kv.value)
            };

            let pos = app.cursor_pos.min(display.len());
            lines.push(Line::from(vec![
                Span::styled(" › ", Style::default().fg(Color::Cyan)),
                Span::styled(display.clone(), Style::default().fg(Color::White)),
            ]));
            // Cursor position: area.x + 3 (for " › ") + cursor pos in text
            cursor_xy = Some((area.x + 3 + pos as u16, area.y + i as u16));
        } else if is_empty {
            lines.push(Line::from(Span::styled(
                format!("   {}", placeholder),
                Style::default().fg(Color::DarkGray),
            )));
        } else if is_invalid {
            lines.push(Line::from(Span::styled(
                format!("   {} ⚠", kv.key),
                Style::default().fg(Color::Red),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                format!("   {}{}{}", kv.key, sep, kv.value),
                Style::default().fg(Color::White),
            )));
        }
    }

    frame.render_widget(Paragraph::new(lines), area);
    cursor_xy
}

fn draw_body(frame: &mut Frame, app: &App, area: Rect) -> Option<(u16, u16)> {
    if app.editing == Some(EditingField::Body) {
        let pos = app.cursor_pos.min(app.request.body.len());
        let before = &app.request.body[..pos];
        let cursor_line = before.matches('\n').count();
        let cursor_col = before.rfind('\n').map(|p| pos - p - 1).unwrap_or(pos);

        // Render body as plain text
        let lines: Vec<Line> = app.request.body
            .split('\n')
            .map(|l| Line::from(Span::styled(l.to_string(), Style::default().fg(Color::White))))
            .collect();
        frame.render_widget(Paragraph::new(lines), area);

        Some((area.x + cursor_col as u16, area.y + cursor_line as u16))
    } else if app.request.body.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                "   { \"message\": \"hello world\" }",
                Style::default().fg(Color::DarkGray),
            )),
            area,
        );
        None
    } else {
        let lines = highlight_json_body(&app.request.body);
        frame.render_widget(Paragraph::new(lines), area);
        None
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

/// Scroll text horizontally, returns (visible_text, cursor_position_in_visible)
fn scroll_text(text: &str, cursor_pos: usize, width: usize) -> (String, usize) {
    if text.len() <= width {
        return (text.to_string(), cursor_pos);
    }

    let half = width / 2;
    let start = cursor_pos.saturating_sub(half);
    let end = (start + width).min(text.len());
    let start = if end == text.len() && text.len() > width { text.len() - width } else { start };

    (text[start..end].to_string(), cursor_pos - start)
}

/// Truncate text with ellipsis if too long
fn truncate(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else if max_len > 1 {
        format!("{}…", &text[..max_len - 1])
    } else {
        text[..max_len].to_string()
    }
}
