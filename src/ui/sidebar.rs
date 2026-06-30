use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, Panel};

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == Panel::Sidebar;

    let block = Block::default()
        .title(" Collections ")
        .borders(Borders::ALL)
        .border_style(if is_active {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        });

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.collections.is_empty() {
        let hint = Paragraph::new(Span::styled(
            " (empty)",
            Style::default().fg(Color::DarkGray),
        ));
        frame.render_widget(hint, inner);
        return;
    }

    let mut lines: Vec<Line> = Vec::new();

    for (ci, col) in app.collections.iter().enumerate() {
        let is_selected_col = ci == app.sidebar_collection && app.sidebar_request.is_none();
        let is_expanded = app.sidebar_expanded == Some(ci);

        let icon = if is_expanded { "▼" } else { "▶" };
        let col_style = if is_selected_col && is_active {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        lines.push(Line::from(vec![
            Span::styled(
                if is_selected_col && is_active { "›" } else { " " },
                Style::default().fg(Color::Cyan),
            ),
            Span::styled(format!("{} ", icon), Style::default().fg(Color::DarkGray)),
            Span::styled(&col.name, col_style),
            Span::styled(
                format!(" ({})", col.requests.len()),
                Style::default().fg(Color::DarkGray),
            ),
        ]));

        if is_expanded {
            for (ri, req) in col.requests.iter().enumerate() {
                let is_selected_req = is_active
                    && ci == app.sidebar_collection
                    && app.sidebar_request == Some(ri);

                let method_color = match req.method.as_str() {
                    "GET" => Color::Green,
                    "POST" => Color::Yellow,
                    "PUT" => Color::Blue,
                    "DELETE" => Color::Red,
                    "PATCH" => Color::Magenta,
                    _ => Color::White,
                };

                let name_style = if is_selected_req {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                lines.push(Line::from(vec![
                    Span::styled(
                        if is_selected_req { " ›" } else { "  " },
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(
                        format!(" {} ", req.method),
                        Style::default().fg(method_color),
                    ),
                    Span::styled(&req.name, name_style),
                ]));
            }
        }
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}
