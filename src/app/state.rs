use std::sync::Arc;
use tokio::sync::mpsc;
use ed25519_dalek::SigningKey;

use crate::config::Config;
use crate::events::AppEvent;
use crate::net::{NetCommand, NetEvent};

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Splash,     
    Connect,    
    KeyInfo,    
    Auth,       
    Enroll,     
    Chat,       
    Error(String),
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub from: String,
    pub text: String,
    pub timestamp: String,
    pub is_system: bool,
    pub is_admin: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ConnectForm {
    pub server: String,
    pub username: String,
    pub manual_key: String,
    pub focused_field: usize,
    pub insecure: bool,
}

#[derive(Default)]
pub struct ChatState {
    pub messages: Vec<ChatMessage>,
    pub online_users: Vec<String>,
    pub scroll_offset: usize,
    pub input: String,
    pub input_cursor: usize,
    pub username: String,
    pub server: String,
    pub show_help: bool,
    pub user_count: u32,
    pub focus_users: bool,
    pub admin_response: Option<String>,
}

impl ChatState {
    pub fn scroll_up(&mut self, n: usize) {
        self.scroll_offset = self.scroll_offset.saturating_add(n);
    }
    pub fn scroll_down(&mut self, n: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(n);
    }
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
    }
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = usize::MAX;
    }
}

#[derive(Debug, Clone, Default)]
pub struct EnrollForm {
    pub invite_code: String,
}

pub struct App {
    pub screen: Screen,
    pub config: Config,
    pub profiles: Vec<crate::config::ServerProfile>,
    pub selected_profile: Option<usize>,
    pub connect_form: ConnectForm,
    pub enroll_form: EnrollForm,
    pub chat: ChatState,
    pub signing_key: Option<Arc<SigningKey>>,
    pub pubkey_b64: Option<String>,
    pub is_new_key: bool,
    pub tick_count: u64,
    pub splash_done: bool,
    pub status_msg: Option<String>,
    pub focus_on_profiles: bool,

    pub net_cmd_tx: Option<mpsc::UnboundedSender<NetCommand>>,
    pub net_event_rx: Option<mpsc::UnboundedReceiver<NetEvent>>,
    pub app_event_tx: Option<mpsc::UnboundedSender<AppEvent>>,

    pub terminal_focused: bool,
    pub unread_count: u32,
    pub notifications_muted: bool,
}

impl App {
    pub fn new() -> Self {
        let config = Config::load();
        let profiles = config.profiles.clone();
        let connect_form = ConnectForm {
            server: config.last_server.clone().unwrap_or_default(),
            username: config.last_username.clone().unwrap_or_default(),
            ..Default::default()
        };

        Self {
            screen: Screen::Splash,
            config,
            profiles,
            selected_profile: None,
            connect_form,
            enroll_form: EnrollForm::default(),
            chat: ChatState::default(),
            signing_key: None,
            pubkey_b64: None,
            is_new_key: false,
            tick_count: 0,
            splash_done: false,
            status_msg: None,
            focus_on_profiles: false,
            net_cmd_tx: None,
            net_event_rx: None,
            app_event_tx: None,
            terminal_focused: true,
            unread_count: 0,
            notifications_muted: false,
        }
    }
}
