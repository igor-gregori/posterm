pub mod dialog;
pub mod request;
pub mod response;
pub mod sidebar;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::{App, EditingField};

pub fn draw(frame: &mut Frame, app: &App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(main_layout[0]);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Sidebar
    sidebar::draw(frame, app, chunks[0]);

    // Request panel
    request::draw(frame, app, right[0]);

    // Response panel
    response::draw(frame, app, right[1]);

    // Footer (left: hints, right: env indicator)
    draw_footer(frame, app, main_layout[1]);

    // Dialog overlay (on top of everything)
    dialog::draw(frame, app);
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let hints = get_contextual_hints(app);

    let left_spans: Vec<Span> = hints
        .iter()
        .enumerate()
        .flat_map(|(i, (key, desc))| {
            let mut s = vec![
                Span::styled(
                    format!(" {} ", key),
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!(" {} ", desc), Style::default().fg(Color::DarkGray)),
            ];
            if i < hints.len() - 1 {
                s.push(Span::raw(" "));
            }
            s
        })
        .collect();

    let left = Paragraph::new(Line::from(left_spans));
    frame.render_widget(left, area);

    // Env indicator on the right
    let env_spans = get_env_indicator(app);
    let right = Paragraph::new(Line::from(env_spans)).alignment(Alignment::Right);
    frame.render_widget(right, area);
}

fn get_contextual_hints(app: &App) -> Vec<(&'static str, &'static str)> {
    if app.dialog.is_some() {
        match app.dialog.unwrap() {
            crate::app::Dialog::SelectEnv => vec![
                ("↑/↓", "Navigate"),
                ("c", "Color"),
                ("Enter", "Select"),
                ("Esc", "Cancel"),
            ],
            crate::app::Dialog::EditEnvVars => vec![
                ("↑/↓", "Navigate"),
                ("Enter", "New row"),
                ("Esc", "Save & close"),
            ],
            crate::app::Dialog::Help => vec![
                ("Esc/F1", "Close"),
            ],
            crate::app::Dialog::History => vec![
                ("↑/↓", "Navigate"),
                ("Enter", "Load"),
                ("d", "Delete"),
                ("Esc", "Close"),
            ],
            _ => vec![("Esc", "Cancel"), ("Enter", "Confirm")],
        }
    } else if let Some(field) = app.editing {
        match field {
            EditingField::Url | EditingField::Body => vec![
                ("←/→", "Cursor"),
                ("Enter", "Done"),
                ("Esc", "Done"),
                ("F1", "Help"),
            ],
            EditingField::Headers | EditingField::Params => vec![
                ("←/→", "Cursor"),
                ("↑/↓", "Rows"),
                ("Enter", "New row"),
                ("Esc", "Done"),
                ("F1", "Help"),
            ],
        }
    } else {
        vec![
            ("Ctrl+R", "Send"),
            ("Ctrl+U", "URL"),
            ("Ctrl+E", "Env"),
            ("Tab", "Panel"),
            ("F1", "Help"),
        ]
    }
}

fn get_env_indicator(app: &App) -> Vec<Span<'static>> {
    match app.environments.active_env() {
        Some(env) => {
            let (r, g, b) = crate::storage::env::color_to_rgb(&env.color);
            let env_color = Color::Rgb(r, g, b);
            vec![
                Span::styled(
                    format!(" {} ({} vars) ", env.name, env.variables.len()),
                    Style::default().fg(Color::Black).bg(env_color).add_modifier(Modifier::BOLD),
                ),
            ]
        }
        None => vec![
            Span::styled(
                " ○ none ",
                Style::default().fg(Color::DarkGray),
            ),
        ],
    }
}
