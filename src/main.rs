use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

mod app;
mod config;
mod crypto;
mod events;
mod net;
mod notify;
mod ui;
mod widgets;

use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls ring crypto provider");

    let args: Vec<String> = std::env::args().collect();

    if args.get(1).map(|s| s.as_str()) == Some("gen") {
        let name = args.get(2).map(|s| s.as_str());
        return crypto::cmd_gen(name);
    }
    
    if args.get(1).map(|s| s.as_str()) == Some("reset") {
        let name = args.get(2).map(|s| s.as_str());
        return crypto::cmd_reset(name);
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        crossterm::cursor::Hide,
        crossterm::event::EnableFocusChange,
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let saved_stderr = unsafe { libc::dup(2) };
    unsafe {
        if saved_stderr >= 0 {
            let devnull = libc::open(
                b"/dev/null\0".as_ptr() as *const libc::c_char,
                libc::O_WRONLY | libc::O_CLOEXEC,
            );
            if devnull >= 0 {
                libc::dup2(devnull, 2);
                libc::close(devnull);
            }
        }
    }

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, crossterm::cursor::Show);
        original_hook(panic_info);
    }));

    let result = App::new().run(&mut terminal).await;

    disable_raw_mode()?;
    unsafe {
        if saved_stderr >= 0 {
            libc::dup2(saved_stderr, 2);
            libc::close(saved_stderr);
        }
    }
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::cursor::Show,
        crossterm::event::DisableFocusChange,
        crossterm::terminal::SetTitle(""),
    )?;
    terminal.show_cursor()?;

    result
}
