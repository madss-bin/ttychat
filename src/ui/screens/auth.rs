use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::App;
use crate::ui::centered_rect;

pub fn draw_auth(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let center = centered_rect(area, 50, 7);
    let spinner_chars = ['⣾', '⣽', '⣻', '⢿', '⡿', '⣟', '⣯', '⣷'];
    let spin = spinner_chars[(app.tick_count as usize / 2) % spinner_chars.len()];
    let status = app.status_msg.as_deref().unwrap_or("Connecting…");

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {spin}  {status}"),
            Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  Press Esc to cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let para = Paragraph::new(Text::from(lines)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Span::styled(" AUTHENTICATING ", Style::default().fg(Color::Cyan))),
    );
    frame.render_widget(para, center);
}
