use anyhow::Result;
use base64::Engine;
use crossterm::{event::{KeyCode, KeyModifiers}, execute};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::app::{App, Screen, ChatMessage};
use crate::config::Config;
use crate::crypto;
use crate::events::{spawn_event_task, AppEvent};
use crate::net::{self, NetCommand, NetEvent, ServerMsg};
use crate::ui;

impl App {
    pub async fn run(
        mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        self.event_loop(terminal).await
    }

    pub fn reset_identity(&mut self) {
        let username = self.connect_form.username.trim().to_string();
        if username.is_empty() { return; }
        let key_path = Config::key_path(&username);
        if key_path.exists() && std::fs::remove_file(&key_path).is_ok() {
            self.status_msg = Some(format!("Deleted identity for {}", username));
            self.signing_key = None;
            self.pubkey_b64 = None;
        }
    }

    pub fn start_connection(&mut self, enroll_code: Option<String>) {
        let username = self.connect_form.username.trim().to_string();
        if username.is_empty() { return; }

        let key_path = Config::key_path(&username);
        if !self.connect_form.manual_key.trim().is_empty() {
            let key_str = self.connect_form.manual_key.trim();
            if let Ok(seed) = base64::engine::general_purpose::STANDARD.decode(key_str) {
                let seed: Vec<u8> = seed;
                if seed.len() == 32 {
                    if let Some(parent) = key_path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    let _ = std::fs::write(&key_path, seed);
                    self.status_msg = Some("Imported manual key".into());
                } else {
                    self.screen = Screen::Error("Invalid manual key length (expected 32 bytes)".into());
                    return;
                }
            } else {
                self.screen = Screen::Error("Invalid manual key format (expected Base64)".into());
                return;
            }
        }

        match crypto::load_or_generate(&key_path) {
            Ok((key, is_new)) => {
                let pubkey = crypto::pubkey_b64(&key);
                self.pubkey_b64 = Some(pubkey);
                self.is_new_key = is_new;
                self.signing_key = Some(Arc::new(key));
                
                if is_new && enroll_code.is_none() {
                    self.screen = Screen::KeyInfo;
                    return;
                }
            }
            Err(e) => {
                self.screen = Screen::Error(format!("Key error: {e}"));
                return;
            }
        }

        let signing_key = Arc::clone(self.signing_key.as_ref().unwrap());
        let pubkey_b64 = self.pubkey_b64.clone().unwrap_or_default();

        let raw_server = self.connect_form.server.trim().to_string();
        let server = if raw_server.contains(':') {
            raw_server
        } else {
            format!("{}:7000", raw_server)
        };
        let insecure = self.connect_form.insecure;

        let (event_tx, event_rx) = mpsc::unbounded_channel::<NetEvent>();
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<NetCommand>();
        self.net_cmd_tx = Some(cmd_tx);
        self.net_event_rx = Some(event_rx);

        let params = net::NetParams {
            server: server.clone(),
            username: username.clone(),
            pubkey_b64: pubkey_b64.clone(),
            sig_fn: Box::new(move |nonce: &str| crypto::sign_nonce(&signing_key, nonce)),
            enroll_code,
            insecure,
        };

        self.chat.username = username;
        self.chat.server = server;
        self.screen = Screen::Auth;
        self.status_msg = Some("Connecting...".into());

        tokio::spawn(async move {
            net::connect(params, event_tx, cmd_rx).await;
        });
    }

    async fn event_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<()> {
        let (app_event_tx, mut app_event_rx) = mpsc::unbounded_channel::<AppEvent>();
        self.app_event_tx = Some(app_event_tx.clone());
        spawn_event_task(app_event_tx);

        loop {
            let mut rx_opt = self.net_event_rx.take();
            if let Some(rx) = &mut rx_opt {
                let mut quit = false;
                let mut events = Vec::new();
                while let Ok(evt) = rx.try_recv() {
                    events.push(evt);
                }
                for evt in events {
                    if self.handle_net_event(evt) {
                        quit = true;
                        break;
                    }
                }
                self.net_event_rx = Some(rx_opt.unwrap());
                if quit { return Ok(()); }
            } else {
                self.net_event_rx = rx_opt;
            }

            terminal.draw(|frame| ui::draw(frame, self))?;

            let Some(event) = app_event_rx.recv().await else {
                break;
            };

            match event {
                AppEvent::Tick => {
                    self.tick_count = self.tick_count.wrapping_add(1);
                    if self.screen == Screen::Splash && self.tick_count >= 30 && !self.splash_done {
                        self.splash_done = true;
                        self.screen = Screen::Connect;
                    }
                }
                AppEvent::Resize => {}
                AppEvent::FocusGained => {
                    self.terminal_focused = true;
                    if self.unread_count > 0 {
                        self.unread_count = 0;
                        let _ = execute!(io::stdout(), crossterm::terminal::SetTitle("ttychat"));
                    }
                }
                AppEvent::FocusLost => {
                    self.terminal_focused = false;
                }
                AppEvent::Key(key) => {
                    if self.handle_key(key) {
                        return Ok(());
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_net_event(&mut self, evt: NetEvent) -> bool {
        match evt {
            NetEvent::Connected => {
                self.status_msg = Some("Authenticating…".into());
            }
            NetEvent::AuthOk { username } => {
                self.screen = Screen::Chat;
                self.status_msg = None;
                self.chat.username = username.clone();
                self.config.last_server = Some(self.chat.server.clone());
                self.config.last_username = Some(self.chat.username.clone());
                
                let mut profiles = self.config.profiles.clone();
                profiles.retain(|p| p.server != self.chat.server || p.username != username);
                profiles.insert(0, crate::config::ServerProfile {
                    server: self.chat.server.clone(),
                    username: username.clone(),
                });
                self.config.profiles = profiles.into_iter().take(10).collect();
                self.profiles = self.config.profiles.clone();

                let _ = self.config.save();
            }
            NetEvent::AuthFail { reason } => {
                self.screen = Screen::Enroll;
                self.status_msg = Some(format!("Auth failed: {reason}. Enter invite code to enroll."));
            }
            NetEvent::Message(msg) => {
                self.handle_server_msg(*msg);
            }
            NetEvent::AdminResponse { action, data } => {
                self.chat.admin_response = Some(format!("[ADMIN >> {action}] {data}"));
                let cm = ChatMessage {
                    from: "ADMIN".into(),
                    text: format!("[{action}] {data}"),
                    timestamp: chrono::Local::now().format("%H:%M").to_string(),
                    is_system: true,
                    is_admin: true,
                };
                self.push_message(cm);
            }
            NetEvent::Error(e) => {
                self.screen = Screen::Error(e);
            }
            NetEvent::Disconnected => {
                if self.screen == Screen::Chat {
                    self.push_system_msg("Disconnected from server");
                    self.net_cmd_tx = None;
                }
            }
        }
        false
    }

    fn handle_server_msg(&mut self, msg: ServerMsg) {
        if let Some(val) = msg.users {
            if val.is_number() {
                if let Some(count) = val.as_u64() {
                    self.chat.user_count = count as u32;
                }
            } else if val.is_array() {
                if let Some(arr) = val.as_array() {
                    self.chat.online_users = arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                }
            }
        }

        if let Some(u) = msg.online { self.chat.online_users = u; }
        if let Some(u) = msg.names { self.chat.online_users = u; }
        if let Some(u) = msg.list { self.chat.online_users = u; }
        if let Some(u) = msg.user_list { self.chat.online_users = u; }

        if msg.msg_type == "presence" {
            return;
        }

        let from = msg.from.unwrap_or_else(|| "system".into());
        let text = msg.text.unwrap_or_default();
        
        let timestamp = if let Some(ts_val) = &msg.timestamp {
            if let Some(ts_str) = ts_val.as_str() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts_str) {
                    dt.with_timezone(&chrono::Local).format("%H:%M").to_string()
                } else {
                    ts_str.chars().take(5).collect()
                }
            } else if let Some(ts_num) = ts_val.as_f64().map(|f| f as i64) {
                let dt = if ts_num > 10_000_000_000 {
                    chrono::DateTime::from_timestamp(ts_num / 1000, (ts_num % 1000) as u32 * 1_000_000)
                } else {
                    chrono::DateTime::from_timestamp(ts_num, 0)
                };
                dt.and_then(|d| Some(d.with_timezone(&chrono::Local).format("%H:%M").to_string()))
                  .unwrap_or_else(|| chrono::Local::now().format("%H:%M").to_string())
            } else {
                chrono::Local::now().format("%H:%M").to_string()
            }
        } else {
            chrono::Local::now().format("%H:%M").to_string()
        };

        let cm = ChatMessage { from, text: text.clone(), timestamp, is_system: false, is_admin: false };
        self.push_message(cm);
    }

    fn push_message(&mut self, msg: ChatMessage) {
        self.chat.messages.push(msg.clone());

        let is_own = msg.from == self.chat.username;
        if !msg.is_system && !is_own && !self.terminal_focused && !self.notifications_muted {
            self.unread_count += 1;
            let title = format!("({} unread) ttychat", self.unread_count);
            let _ = execute!(io::stdout(), crossterm::terminal::SetTitle(title.as_str()));
            let notif_from = msg.from.clone();
            let notif_text = msg.text.clone();
            tokio::task::spawn_blocking(move || {
                crate::notify::play_notification_sound();
                crate::notify::send_desktop_notification(&notif_from, &notif_text);
            });
        }
    }

    fn push_system_msg(&mut self, text: &str) {
        let cm = ChatMessage {
            from: "─ sys ─".into(),
            text: text.into(),
            timestamp: chrono::Local::now().format("%H:%M").to_string(),
            is_system: true,
            is_admin: false,
        };
        self.push_message(cm);
    }

    fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return true;
        }

        if self.screen == Screen::Chat && key.code == KeyCode::Tab {
            self.chat.focus_users = !self.chat.focus_users;
            return false;
        }

        match &self.screen.clone() {
            Screen::Splash => {
                self.splash_done = true;
                self.screen = Screen::Connect;
            }
            Screen::KeyInfo => {
                if key.code == KeyCode::Enter || key.code == KeyCode::Char('c') {
                    self.screen = Screen::Connect;
                }
            }
            Screen::Connect => {
                match key.code {
                    KeyCode::Up if self.focus_on_profiles => {
                        if !self.profiles.is_empty() {
                            let idx = self.selected_profile.unwrap_or(0);
                            self.selected_profile = Some(idx.saturating_sub(1));
                        }
                    }
                    KeyCode::Down if self.focus_on_profiles => {
                        if !self.profiles.is_empty() {
                            let idx = self.selected_profile.unwrap_or(0);
                            self.selected_profile = Some((idx + 1).min(self.profiles.len() - 1));
                        }
                    }
                    KeyCode::Left | KeyCode::Right if !self.profiles.is_empty() => {
                        self.focus_on_profiles = !self.focus_on_profiles;
                    }
                    KeyCode::Enter if self.focus_on_profiles => {
                        if let Some(idx) = self.selected_profile {
                            if let Some(p) = self.profiles.get(idx).cloned() {
                                self.connect_form.server = p.server;
                                self.connect_form.username = p.username;
                                self.start_connection(None);
                            }
                        }
                    }
                    _ => ui::connect::handle_key(self, key),
                }
            }
            Screen::Auth => {
                if key.code == KeyCode::Esc {
                    self.screen = Screen::Connect;
                    self.net_cmd_tx = None;
                }
            }
            Screen::Enroll => ui::screens::handle_enroll_key(self, key),
            Screen::Chat => {
                if ui::chat::handle_key(self, key) { return true; }
            }
            Screen::Error(_) => {
                if key.code == KeyCode::Enter || key.code == KeyCode::Esc {
                    self.screen = Screen::Connect;
                }
            }
        }
        false
    }
}
