use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::{App, Dialog};

pub fn draw(frame: &mut Frame, app: &App) {
    let Some(dialog) = &app.dialog else {
        return;
    };

    match dialog {
        Dialog::SaveRequest | Dialog::NewCollection | Dialog::NewEnv => {
            draw_text_input(frame, app, *dialog);
        }
        Dialog::SelectEnv => {
            draw_select_env(frame, app);
        }
        Dialog::EditEnvVars => {
            draw_edit_env_vars(frame, app);
        }
        Dialog::Help => {
            draw_help(frame);
        }
    }
}

fn draw_text_input(frame: &mut Frame, app: &App, dialog: Dialog) {
    let area = centered_rect(50, 5, frame.area());
    frame.render_widget(Clear, area);

    let title = match dialog {
        Dialog::SaveRequest => " Save Request ",
        Dialog::NewCollection => " New Collection ",
        Dialog::NewEnv => " New Environment ",
        _ => "",
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let pos = app.cursor_pos.min(app.dialog_input.len());
    let (before, after) = app.dialog_input.split_at(pos);

    let line = Line::from(vec![
        Span::styled("Name: ", Style::default().fg(Color::DarkGray)),
        Span::styled(before, Style::default().fg(Color::White)),
        Span::styled("▌", Style::default().fg(Color::Cyan)),
        Span::styled(after, Style::default().fg(Color::White)),
    ]);

    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, inner);
}

fn draw_select_env(frame: &mut Frame, app: &App) {
    let item_count = app.environments.environments.len() + 2; // none + envs + new...
    let height = (item_count as u16) + 4; // +2 for borders +2 for hint
    let area = centered_rect(40, height.min(18), frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Select Environment ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    // "none" option
    let is_selected = app.dialog_selection == 0;
    let is_active = app.environments.active.is_none();
    lines.push(env_option_line("(none)", is_selected, is_active, None));

    // Environment options
    for (i, env) in app.environments.environments.iter().enumerate() {
        let is_selected = app.dialog_selection == i + 1;
        let is_active = app.environments.active.as_ref() == Some(&env.name);
        let (r, g, b) = crate::storage::env::color_to_rgb(&env.color);
        lines.push(env_option_line(&env.name, is_selected, is_active, Some(Color::Rgb(r, g, b))));
    }

    // "new..." option
    let is_selected = app.dialog_selection == app.environments.environments.len() + 1;
    lines.push(Line::from(vec![
        Span::styled(
            if is_selected { " › " } else { "   " },
            Style::default().fg(Color::Cyan),
        ),
        Span::styled("+ new...", Style::default().fg(Color::Green)),
    ]));

    // Hint line
    lines.push(Line::from(Span::styled(
        "  c: change color",
        Style::default().fg(Color::DarkGray),
    )));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn env_option_line<'a>(name: &'a str, is_selected: bool, is_active: bool, color: Option<Color>) -> Line<'a> {
    let marker = if is_selected { " › " } else { "   " };
    let active_marker = if is_active { " ●" } else { "" };

    let env_color = color.unwrap_or(Color::White);

    let name_style = if is_selected {
        Style::default().fg(env_color).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(env_color)
    };

    Line::from(vec![
        Span::styled(marker, Style::default().fg(Color::Cyan)),
        Span::styled(name, name_style),
        Span::styled(active_marker, Style::default().fg(Color::Green)),
    ])
}

fn draw_edit_env_vars(frame: &mut Frame, app: &App) {
    let height = (app.env_edit_vars.len() as u16 + 3).min(20);
    let area = centered_rect(60, height, frame.area());
    frame.render_widget(Clear, area);

    let env_name = app.active_env_name();
    let title = format!(" Variables [{}] ", env_name);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    for (i, (key, value)) in app.env_edit_vars.iter().enumerate() {
        let is_active_row = i == app.env_edit_row;
        let display = format!("{}={}", key, value);
        let row_marker = if is_active_row { "› " } else { "  " };

        if is_active_row {
            lines.push(render_env_line_with_cursor(row_marker, &display, app.cursor_pos));
        } else if display == "=" {
            lines.push(Line::from(vec![
                Span::styled(row_marker.to_string(), Style::default().fg(Color::Cyan)),
                Span::styled("key=value".to_string(), Style::default().fg(Color::DarkGray)),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled(row_marker.to_string(), Style::default().fg(Color::Cyan)),
                Span::styled(display, Style::default().fg(Color::White)),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn render_env_line_with_cursor(marker: &str, display: &str, cursor_pos: usize) -> Line<'static> {
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

fn draw_help(frame: &mut Frame) {
    let area = centered_rect(65, 26, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<Line> = vec![
        section_header("Request"),
        help_line("Ctrl+R", "Send request"),
        help_line("Ctrl+T", "Cycle HTTP method (GET/POST/PUT/DEL/PATCH)"),
        help_line("Ctrl+U", "Edit URL"),
        help_line("Ctrl+H", "Edit headers (inline key:value)"),
        help_line("Ctrl+B", "Edit body"),
        help_line("Ctrl+P", "Edit params (inline key=value)"),
        Line::from(""),
        section_header("Collections"),
        help_line("Ctrl+S", "Save request to collection"),
        help_line("Ctrl+N", "New collection"),
        help_line("Enter", "Load request / expand collection"),
        help_line("d", "Delete request or collection"),
        Line::from(""),
        section_header("Environments"),
        help_line("Ctrl+E", "Select / create environment"),
        help_line("Ctrl+W", "Edit environment variables"),
        help_line("c", "Change env color (in selector)"),
        Line::from(""),
        section_header("Navigation"),
        help_line("Tab", "Switch panel (Sidebar/Request/Response)"),
        help_line("↑/↓/←/→", "Navigate / move cursor"),
        help_line("Esc", "Exit editing / close dialog"),
        help_line("q", "Quit"),
    ];

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn section_header(title: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!(" ─── {} ───", title),
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    ))
}

fn help_line(key: &str, desc: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("  {:>12}  ", key), Style::default().fg(Color::Cyan)),
        Span::styled(desc.to_string(), Style::default().fg(Color::White)),
    ])
}

fn centered_rect(width_percent: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((area.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_percent) / 2),
            Constraint::Percentage(width_percent),
            Constraint::Percentage((100 - width_percent) / 2),
        ])
        .split(vertical[1])[1]
}
