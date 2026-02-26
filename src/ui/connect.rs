use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, BorderType},
    Frame,
};

use crate::app::App;
use crate::ui::centered_rect;

pub fn draw_connect(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(80),
        ])
        .split(area);

    draw_sidebar(frame, app, layout[0]);
    draw_join_form(frame, app, layout[1]);
}

fn draw_sidebar(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(" RECENT ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
    
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = Vec::new();
    if app.profiles.is_empty() {
        lines.push(Line::from(Span::styled("  No history yet", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC))));
    } else {
        for (i, p) in app.profiles.iter().enumerate() {
            let is_selected = app.selected_profile == Some(i);
            let is_focused = app.focus_on_profiles && is_selected;
            
            let (prefix, style) = if is_focused {
                (" ▶ ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            } else if is_selected {
                ("   ", Style::default().fg(Color::White))
            } else {
                ("   ", Style::default().fg(Color::Gray))
            };

            lines.push(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(&p.username, style),
                Span::styled("@", Style::default().fg(Color::DarkGray)),
                Span::styled(&p.server, style),
            ]));
        }
    }

    frame.render_widget(Paragraph::new(Text::from(lines)), inner);
}

fn draw_join_form(frame: &mut Frame, app: &App, area: Rect) {
    let center = centered_rect(area, 58, 22);

    let verts = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(3),
        ])
        .split(center);

    let title = Paragraph::new(Text::from(vec![
        Line::from(Span::styled(
            "TTYCHAT",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "WELCOME BACK",
            Style::default().fg(Color::Gray).add_modifier(Modifier::DIM),
        )),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(title, verts[0]);

    let cursor_visible = (app.tick_count / 6).wrapping_rem(2) == 0;
    let focused = app.connect_form.focused_field;
    let form_focused = !app.focus_on_profiles;

    let render_field = |frame: &mut Frame, area: Rect, label: &str, val: &str, idx: usize, is_pass: bool| {
        let is_active = form_focused && focused == idx;
        let display_val = if is_pass && !val.is_empty() {
            "*".repeat(val.len())
        } else {
            val.to_string()
        };
        let field_val = format!(
            "{}{}",
            display_val,
            if is_active && cursor_visible { "█" } else { " " }
        );
        
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(if is_active { Color::Cyan } else { Color::DarkGray }))
            .title(Span::styled(format!(" {label} "), Style::default().fg(if is_active { Color::Cyan } else { Color::Gray })));

        let p = Paragraph::new(Span::styled(
            format!(" {field_val}"),
            Style::default().fg(if is_active { Color::White } else { Color::Gray }),
        )).block(block);
        frame.render_widget(p, area);
    };

    render_field(frame, verts[1], "SERVER", &app.connect_form.server, 0, false);
    render_field(frame, verts[2], "USERNAME", &app.connect_form.username, 1, false);
    render_field(frame, verts[3], "PRIVATE KEY (Optional)", &app.connect_form.manual_key, 2, true);

    let insecure_on = app.connect_form.insecure;
    let is_tls_active = form_focused && focused == 3;
    let tls_str = if insecure_on { "  [✔] Skip TLS cert verification" } else { "  [ ] Skip TLS cert verification" };
    let tls_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if is_tls_active { Color::Cyan } else { Color::DarkGray }))
        .title(Span::styled(" TLS MODE ", Style::default().fg(if is_tls_active { Color::Cyan } else { Color::Gray })));
    
    frame.render_widget(Paragraph::new(Span::styled(tls_str, Style::default().fg(if insecure_on { Color::Yellow } else { Color::Gray }))).block(tls_block), verts[4]);

    let is_reset_active = form_focused && focused == 4;
    let reset_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if is_reset_active { Color::Red } else { Color::DarkGray }))
        .title(Span::styled(" DANGER ", Style::default().fg(if is_reset_active { Color::Red } else { Color::Gray })));

    frame.render_widget(Paragraph::new(Span::styled("  [ Wipe Local Identity ]", Style::default().fg(if is_reset_active { Color::Red } else { Color::DarkGray }))).block(reset_block), verts[5]);

    let hints = Line::from(vec![
        Span::styled(if app.focus_on_profiles { " ←/→ Switch to FORM " } else { " ←/→ Switch to RECENT " }, Style::default().fg(Color::DarkGray)),
        Span::styled("  │  ", Style::default().fg(Color::Rgb(30, 30, 30))),
        Span::styled(" Tab = Move ", Style::default().fg(Color::DarkGray)),
    ]);
    frame.render_widget(Paragraph::new(hints).alignment(Alignment::Center), verts[7]);
}

pub fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) {
    use crossterm::event::KeyCode;
    match key.code {
        KeyCode::Tab => {
            app.connect_form.focused_field = (app.connect_form.focused_field + 1) % 5;
        }
        KeyCode::BackTab => {
            app.connect_form.focused_field = (app.connect_form.focused_field + 4) % 5;
        }
        KeyCode::Enter => {
            if app.connect_form.focused_field == 4 {
                app.reset_identity();
                return;
            }
            if !app.connect_form.server.trim().is_empty()
                && !app.connect_form.username.trim().is_empty()
            {
                app.start_connection(None);
            }
        }
        KeyCode::Char(' ') if app.connect_form.focused_field == 3 => {
            app.connect_form.insecure = !app.connect_form.insecure;
        }
        KeyCode::Backspace => {
            match app.connect_form.focused_field {
                0 => { app.connect_form.server.pop(); }
                1 => { app.connect_form.username.pop(); }
                2 => { app.connect_form.manual_key.pop(); }
                _ => {}
            }
        }
        KeyCode::Char(c) => {
            match app.connect_form.focused_field {
                0 => app.connect_form.server.push(c),
                1 => app.connect_form.username.push(c),
                2 => app.connect_form.manual_key.push(c),
                _ => {}
            }
        }
        _ => {}
    }
}
