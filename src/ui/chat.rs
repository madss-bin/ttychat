use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Clear, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
    Frame,
};

use crate::app::App;
use crate::widgets::messages::{render_message, user_color_simple};

pub fn draw_chat(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(area);

    draw_title_bar(frame, app, rows[0]);
    draw_main_content(frame, app, rows[1]);
    draw_input_bar(frame, app, rows[2]);
    draw_hints_bar(frame, app, rows[3]);

    if app.chat.show_help {
        draw_help_overlay(frame, area, app);
    }
}

fn draw_title_bar(frame: &mut Frame, app: &App, area: Rect) {
    let server = &app.chat.server;
    let username = &app.chat.username;
    let users = app.chat.user_count;

    let time = chrono::Local::now().format("%H:%M:%S").to_string();

    let title_text = Line::from(vec![
        Span::styled(" TTYCHAT ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
        Span::styled(server.as_str(), Style::default().fg(Color::Gray)),
        Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
        Span::styled("you: ", Style::default().fg(Color::DarkGray)),
        Span::styled(username.as_str(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("USERS: {users}"), Style::default().fg(Color::Gray)),
        Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
        Span::styled(time, Style::default().fg(Color::Cyan)),
    ]);

    let title = Paragraph::new(title_text);
    frame.render_widget(title, area);
}

fn draw_main_content(frame: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(20),
        ])
        .split(area);

    draw_messages(frame, app, cols[0]);
    draw_user_list(frame, app, cols[1]);
}

fn draw_messages(frame: &mut Frame, app: &App, area: Rect) {
    let msgs = &app.chat.messages;
    let username = &app.chat.username;

    let mut all_lines = Vec::new();
    let inner_width = area.width.saturating_sub(2) as usize;
    for msg in msgs {
        all_lines.extend(render_message(msg, username, inner_width));
    }
    
    let total_rows = all_lines.len();
    let visible_height = area.height.saturating_sub(2) as usize;

    let max_scroll = total_rows.saturating_sub(visible_height);
    let scroll = app.chat.scroll_offset.min(max_scroll);
    let scroll_from_top = max_scroll.saturating_sub(scroll);

    let start_idx = scroll_from_top;
    let end_idx = (start_idx + visible_height).min(total_rows);
    
    let visible_items: Vec<ListItem> = all_lines.into_iter()
        .skip(start_idx)
        .take(end_idx - start_idx)
        .map(ListItem::new)
        .collect();

    let list = List::new(visible_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(Span::styled(
                    " MESSAGES ",
                    Style::default().fg(Color::Gray),
                ))
                .title_alignment(Alignment::Left),
        );

    frame.render_widget(list, area);

    if total_rows > visible_height {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"))
            .track_symbol(Some("│"))
            .thumb_symbol("█")
            .style(Style::default().fg(Color::Gray));

        let mut scrollbar_state = ScrollbarState::new(max_scroll)
            .position(scroll_from_top);

        frame.render_stateful_widget(
            scrollbar,
            area.inner(ratatui::layout::Margin { horizontal: 0, vertical: 1 }),
            &mut scrollbar_state,
        );
    }

    if scroll > 0 {
        let indicator = Paragraph::new(Line::from(vec![
            Span::styled(
                format!(" ↑ {} lines above ", scroll),
                Style::default()
                    .fg(Color::Yellow)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
        ]))
        .alignment(Alignment::Center);

        let indicator_area = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: 1,
        };
        frame.render_widget(indicator, indicator_area);
    }
}

fn draw_user_list(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.chat.focus_users;

    let items: Vec<ListItem> = if app.chat.online_users.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            " (empty)",
            Style::default().fg(Color::Rgb(40, 50, 40)),
        )))]
    } else {
        app.chat
            .online_users
            .iter()
            .map(|u| {
                let is_self = u == &app.chat.username;
                let color = if is_self { Color::Cyan } else { user_color_simple(u) };
                let prefix = if is_self { "▶ " } else { "  " };
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{prefix}● "),
                        Style::default().fg(color).add_modifier(Modifier::DIM),
                    ),
                    Span::styled(
                        u.as_str(),
                        Style::default()
                            .fg(color)
                            .add_modifier(if is_self { Modifier::BOLD } else { Modifier::empty() }),
                    ),
                ]))
            })
            .collect()
    };

    let border_color = if focused { Color::Cyan } else { Color::DarkGray };
    let user_count = app.chat.online_users.len();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(Style::default().fg(border_color))
                .title(Span::styled(
                    format!(" ONLINE ({user_count}) "),
                    Style::default().fg(if focused { Color::Cyan } else { Color::Gray }),
                )),
        );

    frame.render_widget(list, area);
}

fn draw_input_bar(frame: &mut Frame, app: &App, area: Rect) {
    let input = &app.chat.input;
    let cursor_pos = app.chat.input_cursor;
    let cursor_visible = (app.tick_count / 6).wrapping_rem(2) == 0;

    let mut spans: Vec<Span> = Vec::new();
    spans.push(Span::styled(" ", Style::default()));

    let chars: Vec<char> = input.chars().collect();
    let char_strings: Vec<String> = chars.iter().map(|c| c.to_string()).collect();
    for (i, ch_str) in char_strings.iter().enumerate() {
        if i == cursor_pos {
            spans.push(Span::styled(
                if cursor_visible { "█" } else { ch_str.as_str() },
                Style::default().fg(Color::Cyan),
            ));
            if cursor_visible {
                spans.push(Span::styled(
                    ch_str.clone(),
                    Style::default().fg(Color::Black).bg(Color::Cyan),
                ));
            }
        } else {
            spans.push(Span::styled(
                ch_str.clone(),
                Style::default().fg(Color::White),
            ));
        }
    }
    let char_count = chars.len();
    if cursor_pos >= char_count {
        spans.push(Span::styled(
            if cursor_visible { "█" } else { " " },
            Style::default().fg(Color::Cyan),
        ));
    }

    let muted_indicator = if app.notifications_muted { "  muted" } else { "" };
    let placeholder = if input.is_empty() {
        vec![Span::styled(
            format!(" Type a message… (/mute · /unmute · /admin <action>){muted_indicator}"),
            Style::default().fg(Color::DarkGray),
        )]
    } else {
        spans
    };

    let para = Paragraph::new(Line::from(placeholder))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(Style::default().fg(Color::Gray))
                .title(Span::styled(
                    " INPUT ",
                    Style::default().fg(Color::Gray),
                ))
                .title_alignment(Alignment::Left),
        );

    frame.render_widget(para, area);
}

fn draw_hints_bar(frame: &mut Frame, _app: &App, area: Rect) {
    let line = Line::from(vec![
        hint_key("Enter"), hint_sep("send  "),
        hint_key("↑↓"), hint_sep("/"),
        hint_key("PgUp/Dn"), hint_sep("scroll  "),
        hint_key("Home/End"), hint_sep("top/bot  "),
        hint_key("Tab"), hint_sep("focus  "),
        hint_key("F1"), hint_sep("help  "),
        hint_key("Ctrl-C"), hint_sep("quit"),
    ]);

    let bar = Paragraph::new(line);
    frame.render_widget(bar, area);
}

fn hint_key(s: &'static str) -> Span<'static> {
    Span::styled(format!(" {s}"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD))
}
fn hint_sep(s: &'static str) -> Span<'static> {
    Span::styled(s, Style::default().fg(Color::DarkGray))
}

fn draw_help_overlay(frame: &mut Frame, area: Rect, _app: &App) {
    let overlay = Rect {
        x: area.width.saturating_sub(48) / 2,
        y: area.height.saturating_sub(26) / 2,
        width: 48.min(area.width),
        height: 26.min(area.height),
    };

    frame.render_widget(Clear, overlay);

    let lines = vec![
        Line::from(Span::styled("  KEYBINDS", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        keybind_line("Enter",    "Send message"),
        keybind_line("↑",       "Scroll up 1 line"),
        keybind_line("↓",       "Scroll down 1 line"),
        keybind_line("PgUp",    "Scroll up 10 lines"),
        keybind_line("PgDn",    "Scroll down 10 lines"),
        keybind_line("Home",    "Jump to oldest"),
        keybind_line("End",     "Jump to latest"),
        keybind_line("Tab",     "Focus input/userlist"),
        keybind_line("Ctrl-U",  "Clear input"),
        keybind_line("← / →",  "Move cursor"),
        keybind_line("F1",      "Toggle this help"),
        keybind_line("Ctrl-C",  "Quit"),
        Line::from(""),
        Line::from(Span::styled("  COMMANDS", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        keybind_line("/mute",        "Mute notifications"),
        keybind_line("/unmute",      "Unmute notifications"),
        keybind_line("/admin invite","Request invite code"),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled("  [ Any key = close ]", Style::default().fg(Color::DarkGray))),
    ];

    let help = Paragraph::new(Text::from(lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(Span::styled(
                    " HELP ",
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                )),
        );
    frame.render_widget(help, overlay);
}

fn keybind_line(key: &'static str, desc: &'static str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("  {:14}", key),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::styled(desc, Style::default().fg(Color::Gray)),
    ])
}
pub fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) -> bool {
    use crossterm::event::{KeyCode, KeyModifiers};

    if key.code == KeyCode::F(1) {
        app.chat.show_help = !app.chat.show_help;
        return false;
    }

    if app.chat.show_help {
        app.chat.show_help = false;
        return false;
    }

    match key.code {
        KeyCode::Up => { app.chat.scroll_up(1); return false; }
        KeyCode::Down => { app.chat.scroll_down(1); return false; }
        KeyCode::PageUp => { app.chat.scroll_up(10); return false; }
        KeyCode::PageDown => { app.chat.scroll_down(10); return false; }
        KeyCode::Home => { app.chat.scroll_to_top(); return false; }
        KeyCode::End => { app.chat.scroll_to_bottom(); return false; }
        _ => {}
    }

    match key.code {
        KeyCode::Enter => {
            let text = app.chat.input.trim().to_string();
            if !text.is_empty() {
                    if let Some(tx) = &app.net_cmd_tx {
                        if text == "/mute" {
                            app.notifications_muted = true;
                            app.chat.input.clear();
                            app.chat.input_cursor = 0;
                            app.chat.messages.push(crate::app::ChatMessage {
                                from: "─ sys ─".into(),
                                text: "Notifications muted. Type /unmute to re-enable.".into(),
                                timestamp: chrono::Local::now().format("%H:%M").to_string(),
                                is_system: true,
                                is_admin: false,
                            });
                            return false;
                        } else if text == "/unmute" {
                            app.notifications_muted = false;
                            app.chat.input.clear();
                            app.chat.input_cursor = 0;
                            app.chat.messages.push(crate::app::ChatMessage {
                                from: "─ sys ─".into(),
                                text: "Notifications unmuted.".into(),
                                timestamp: chrono::Local::now().format("%H:%M").to_string(),
                                is_system: true,
                                is_admin: false,
                            });
                            return false;
                        } else if let Some(cmd) = text.strip_prefix("/admin ") {
                            let action = cmd.trim().to_string();
                            if !action.is_empty() {
                                let _ = tx.send(crate::net::NetCommand::SendAdminCmd(action));
                            }
                        } else if !text.starts_with('/') || text.starts_with("/admin ") {
                            let _ = tx.send(crate::net::NetCommand::SendMessage(text));
                        } else {
                            let _ = tx.send(crate::net::NetCommand::SendMessage(text));
                        }
                    }
                app.chat.input.clear();
                app.chat.input_cursor = 0;
                app.chat.scroll_to_bottom();
            }
        }
        KeyCode::Backspace => {
            if app.chat.input_cursor > 0 {
                let byte_pos = app.chat.input.char_indices()
                    .nth(app.chat.input_cursor - 1)
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                app.chat.input.remove(byte_pos);
                app.chat.input_cursor -= 1;
            }
        }
        KeyCode::Delete => {
            if app.chat.input_cursor < app.chat.input.chars().count() {
                let byte_pos = app.chat.input.char_indices()
                    .nth(app.chat.input_cursor)
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                app.chat.input.remove(byte_pos);
            }
        }
        KeyCode::Left => {
            if app.chat.input_cursor > 0 { app.chat.input_cursor -= 1; }
        }
        KeyCode::Right => {
            if app.chat.input_cursor < app.chat.input.chars().count() {
                app.chat.input_cursor += 1;
            }
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.chat.input.clear();
            app.chat.input_cursor = 0;
        }
        KeyCode::Char(c) => {
            let byte_pos = app.chat.input.char_indices()
                .nth(app.chat.input_cursor)
                .map(|(i, _)| i)
                .unwrap_or(app.chat.input.len());
            app.chat.input.insert(byte_pos, c);
            app.chat.input_cursor += 1;
        }
        _ => {}
    }
    false
}
