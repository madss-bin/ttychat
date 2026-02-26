use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
    Frame,
};
use crate::app::App;
use crate::ui::centered_rect;
use crate::ui::assets::ASCII_LOGO;

pub fn draw_splash(frame: &mut Frame, _app: &App) {
    let area = frame.area();

    let center = centered_rect(area, 64, 10);

    let logo_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
    
    let logo_text = Text::raw(ASCII_LOGO);
    let logo_para = Paragraph::new(logo_text)
        .style(logo_style)
        .alignment(Alignment::Left);

    let footer = vec![
        Line::from(""),
        Line::from(Span::styled(
            "v0.1.0 • [ Press any key to continue ]",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let layout = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Fill(1),
        ])
        .split(center);

    frame.render_widget(logo_para, layout[0]);
    frame.render_widget(Paragraph::new(footer).alignment(Alignment::Center), layout[1]);
}

