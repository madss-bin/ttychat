use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::App;
use crate::ui::centered_rect;

pub fn draw_key_info(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let center = centered_rect(area, 70, 14);
    let pubkey = app.pubkey_b64.as_deref().unwrap_or("(none)");

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  NEW IDENTITY GENERATED",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  Your public key (send this to the admin):",
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {pubkey}"),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  Ask an admin in the chat for invite:",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "  /admin invite",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  [ Press Enter or Esc to continue ]",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let para = Paragraph::new(Text::from(lines)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Span::styled(" IDENTITY ", Style::default().fg(Color::Cyan))),
    );
    frame.render_widget(para, center);
}
