use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::app::ChatMessage;

pub fn user_color_simple(username: &str) -> Color {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    username.hash(&mut h);
    let hash = h.finish();
    let colors = [
        Color::Cyan, Color::Yellow, Color::Magenta,
        Color::Green, Color::Blue, Color::Red,
    ];
    colors[(hash as usize) % colors.len()]
}
pub fn render_message<'a>(msg: &'a ChatMessage, my_username: &str, max_width: usize) -> Vec<Line<'a>> {
    let is_self = msg.from == my_username;
    let is_system = msg.is_system;
    let is_admin = msg.is_admin;

    let user_color = if is_self {
        Color::LightCyan
    } else if is_system {
        Color::DarkGray
    } else if is_admin {
        Color::Red
    } else {
        user_color_simple(&msg.from)
    };

    let time_str = format!("{} ", msg.timestamp);
    let time_len = time_str.len();
    let time_span = Span::styled(time_str, Style::default().fg(Color::DarkGray));

    let user_str = if is_system {
        format!("── {} ── ", msg.from)
    } else {
        format!("{}: ", msg.from)
    };
    let user_len = user_str.len();
    let user_span = if is_system {
        Span::styled(
            user_str,
            Style::default()
                .fg(user_color)
                .add_modifier(Modifier::DIM),
        )
    } else {
        Span::styled(
            user_str,
            Style::default()
                .fg(user_color)
                .add_modifier(Modifier::BOLD),
        )
    };

    let text_color = if is_self {
        Color::LightGreen
    } else if is_system {
        Color::Gray
    } else if is_admin {
        Color::LightRed
    } else {
        Color::White
    };

    let prefix_str = if is_self { "▶ " } else { "  " };
    let prefix_len = prefix_str.len();
    let prefix_span = Span::styled(prefix_str, Style::default().fg(Color::DarkGray));

    let header_width = prefix_len + time_len + user_len;
    let available_width = max_width.saturating_sub(header_width + 1);

    if available_width == 0 {
        return vec![Line::from(vec![prefix_span, time_span, user_span])];
    }

    let mut lines = Vec::new();
    let mut current_line_text = String::new();
    let mut current_width = 0;

    for ch in msg.text.chars() {
        let ch_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        
        if current_width + ch_width > available_width {
            let line_spans = if lines.is_empty() {
                vec![
                    prefix_span.clone(),
                    time_span.clone(),
                    user_span.clone(),
                    Span::styled(current_line_text.clone(), Style::default().fg(text_color)),
                ]
            } else {
                vec![
                    Span::raw(" ".repeat(header_width)),
                    Span::styled(current_line_text.clone(), Style::default().fg(text_color)),
                ]
            };
            lines.push(Line::from(line_spans));
            current_line_text.clear();
            current_width = 0;
        }
        
        current_line_text.push(ch);
        current_width += ch_width;
    }

    if !current_line_text.is_empty() || lines.is_empty() {
        let line_spans = if lines.is_empty() {
            vec![
                prefix_span,
                time_span,
                user_span,
                Span::styled(current_line_text, Style::default().fg(text_color)),
            ]
        } else {
            vec![
                Span::raw(" ".repeat(header_width)),
                Span::styled(current_line_text, Style::default().fg(text_color)),
            ]
        };
        lines.push(Line::from(line_spans));
    }

    lines
}
