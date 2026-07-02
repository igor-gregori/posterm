use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

use crate::app::{App, Panel};
use crate::http::Response;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == Panel::Response;

    let active_color = app.environments.active_env()
        .map(|e| {
            let (r, g, b) = crate::storage::env::color_to_rgb(&e.color);
            Color::Rgb(r, g, b)
        })
        .unwrap_or(Color::Cyan);

    let block = Block::default()
        .title(" Response ")
        .borders(Borders::ALL)
        .border_style(if is_active {
            Style::default().fg(active_color)
        } else {
            Style::default().fg(Color::DarkGray)
        });

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.loading {
        let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let spinner = frames[(app.tick / 4) % frames.len()];
        let elapsed = app.loading_since.elapsed();
        let secs = elapsed.as_secs();
        let timer = if secs >= 3600 {
            format!("{}h {:02}m {:02}s", secs / 3600, (secs % 3600) / 60, secs % 60)
        } else if secs >= 60 {
            format!("{}m {:02}s", secs / 60, secs % 60)
        } else {
            format!("{}s", secs)
        };
        let loading = Paragraph::new(Line::from(vec![
            Span::styled(format!(" {} ", spinner), Style::default().fg(Color::Yellow)),
            Span::styled("Sending request... ", Style::default().fg(Color::Yellow)),
            Span::styled(timer, Style::default().fg(Color::DarkGray)),
        ]));
        frame.render_widget(loading, inner);
        return;
    }

    let Some(ref result) = app.response else {
        return;
    };

    match result {
        Err(e) => {
            let err = Paragraph::new(Span::styled(
                format!("Error: {}", e),
                Style::default().fg(Color::Red),
            ))
            .wrap(Wrap { trim: false });
            frame.render_widget(err, inner);
        }
        Ok(resp) => draw_response(frame, resp, inner, app.response_scroll),
    }
}

fn draw_response(frame: &mut Frame, resp: &Response, area: Rect, scroll: usize) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // status line
            Constraint::Length(1), // separator
            Constraint::Min(1),   // body
        ])
        .split(area);

    // Status line
    let status_color = match resp.status {
        200..=299 => Color::Green,
        300..=399 => Color::Yellow,
        400..=499 => Color::Red,
        500..=599 => Color::Magenta,
        _ => Color::White,
    };
    let duration_ms = resp.duration.as_millis();
    let status_line = Line::from(vec![
        Span::styled(
            format!("{} {}", resp.status, resp.status_text),
            Style::default().fg(status_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  {}ms", duration_ms),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            format!("  {} bytes", resp.body.len()),
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    frame.render_widget(Paragraph::new(status_line), chunks[0]);

    // Separator line
    let sep = Paragraph::new(Line::from(Span::styled(
        "─".repeat(area.width as usize),
        Style::default().fg(Color::DarkGray),
    )));
    frame.render_widget(sep, chunks[1]);

    // Body with syntax highlighting
    let body_lines = highlight_json(&resp.body);
    let body = Paragraph::new(body_lines)
        .wrap(Wrap { trim: false })
        .scroll((scroll as u16, 0));
    frame.render_widget(body, chunks[2]);
}

fn highlight_json(text: &str) -> Vec<Line<'static>> {
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let syntax = ss.find_syntax_by_extension("json").unwrap_or_else(|| ss.find_syntax_plain_text());
    let theme = &ts.themes["base16-ocean.dark"];
    let mut h = HighlightLines::new(syntax, theme);

    text.lines()
        .take(200) // cap lines for performance
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
