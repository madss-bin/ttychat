use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use crate::app::App;
use crate::ui::centered_rect;

pub fn draw_error(frame: &mut Frame, _app: &App, msg: &str) {
    let area = frame.area();
    let center = centered_rect(area, 60, 9);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  CONNECTION ERROR",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {msg}"),
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  [ Enter / Esc = back to connect ]",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let para = Paragraph::new(Text::from(lines))
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(Span::styled(" ERROR ", Style::default().fg(Color::Red))),
        );
    frame.render_widget(para, center);
}
