use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::App;
use crate::ui::centered_rect;

pub fn draw_enroll(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let center = centered_rect(area, 60, 12);

    let status = app.status_msg.as_deref().unwrap_or("");
    let code = &app.enroll_form.invite_code;
    let cursor_visible = (app.tick_count / 6).wrapping_rem(2) == 0;
    let field_display = format!(
        "{}{}",
        code,
        if cursor_visible { "█" } else { " " }
    );

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(status, Style::default().fg(Color::Red))),
        Line::from(""),
        Line::from(Span::styled(
            "  Enter your invite code from the admin:",
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  ▶ {field_display}"),
            Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  Ask admin for invite",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  [ Enter = submit  |  Esc = back ]",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let para = Paragraph::new(Text::from(lines)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Span::styled(" ENROLL ", Style::default().fg(Color::Yellow))),
    );
    frame.render_widget(para, center);
}

pub fn handle_enroll_key(app: &mut App, key: crossterm::event::KeyEvent) {
    use crossterm::event::KeyCode;
    match key.code {
        KeyCode::Esc => { app.screen = crate::app::Screen::Connect; }
        KeyCode::Enter => {
            if !app.enroll_form.invite_code.trim().is_empty() {
                let code = app.enroll_form.invite_code.trim().to_string();
                app.enroll_form.invite_code.clear();
                app.start_connection(Some(code));
            }
        }
        KeyCode::Backspace => { app.enroll_form.invite_code.pop(); }
        KeyCode::Char(c) => { app.enroll_form.invite_code.push(c); }
        _ => {}
    }
}
