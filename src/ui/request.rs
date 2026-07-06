use ratatui::{
    layout::Rect,
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

    let is_editing = app.editing == Some(EditingField::Body);

    let mode_indicator = if is_editing { " [EDIT] " } else { " [VIEW] " };
    let title = format!(" Request {}", mode_indicator);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if is_editing {
        // Show cURL text with cursor
        let lines = render_multiline_with_cursor(&app.curl_text, app.cursor_pos);
        frame.render_widget(Paragraph::new(lines), inner);
    } else {
        // Show cURL with syntax coloring
        let lines = highlight_curl(&app.curl_text);
        frame.render_widget(Paragraph::new(lines), inner);
    }
}

/// Syntax-highlight a cURL command for display
fn highlight_curl(text: &str) -> Vec<Line<'static>> {
    text.lines()
        .map(|line| {
            let trimmed = line.trim_start();
            if trimmed.starts_with("curl") {
                Line::from(vec![
                    Span::styled(leading_whitespace(line), Style::default()),
                    Span::styled("curl", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::styled(trimmed.strip_prefix("curl").unwrap_or("").to_string(), Style::default().fg(Color::White)),
                ])
            } else if trimmed.starts_with("-X") {
                let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
                let flag = parts[0];
                let rest = parts.get(1).unwrap_or(&"");
                Line::from(vec![
                    Span::styled(leading_whitespace(line), Style::default()),
                    Span::styled(flag.to_string(), Style::default().fg(Color::Cyan)),
                    Span::styled(" ".to_string(), Style::default()),
                    Span::styled(rest.to_string(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                ])
            } else if trimmed.starts_with("-H") || trimmed.starts_with("-d") {
                let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
                let flag = parts[0];
                let rest = parts.get(1).unwrap_or(&"");
                Line::from(vec![
                    Span::styled(leading_whitespace(line), Style::default()),
                    Span::styled(flag.to_string(), Style::default().fg(Color::Cyan)),
                    Span::styled(" ".to_string(), Style::default()),
                    Span::styled(rest.to_string(), Style::default().fg(Color::White)),
                ])
            } else if trimmed.starts_with("'http") || trimmed.starts_with("'https") || trimmed.starts_with("http") {
                Line::from(Span::styled(line.to_string(), Style::default().fg(Color::Blue)))
            } else {
                Line::from(Span::styled(line.to_string(), Style::default().fg(Color::White)))
            }
        })
        .collect()
}

fn leading_whitespace(s: &str) -> String {
    s.chars().take_while(|c| c.is_whitespace()).collect()
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
