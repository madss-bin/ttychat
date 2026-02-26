pub mod chat;
pub mod connect;
pub mod screens;
pub mod splash;
pub mod assets;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use crate::app::{App, Screen};

pub fn draw(frame: &mut Frame, app: &App) {
    match &app.screen {
        Screen::Splash     => splash::draw_splash(frame, app),
        Screen::Connect    => connect::draw_connect(frame, app),
        Screen::KeyInfo    => screens::draw_key_info(frame, app),
        Screen::Auth       => screens::draw_auth(frame, app),
        Screen::Enroll     => screens::draw_enroll(frame, app),
        Screen::Chat       => chat::draw_chat(frame, app),
        Screen::Error(msg) => screens::draw_error(frame, app, msg),
    }
}

pub fn centered_rect(area: Rect, w: u16, h: u16) -> Rect {
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(h),
            Constraint::Fill(1),
        ])
        .split(area);

    let h_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(w),
            Constraint::Fill(1),
        ])
        .split(vert[1]);
        
    h_layout[1]
}
