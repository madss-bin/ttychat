use std::io::Cursor;
use rodio::{Decoder, Sink, OutputStreamBuilder};

static NOTIF_BYTES: &[u8] = include_bytes!("../assets/notif.mp3");

fn with_stderr_suppressed<F: FnOnce()>(f: F) {
    #[cfg(unix)]
    unsafe {
        use std::ffi::CString;
        let path = CString::new("/dev/null").unwrap();
        let devnull = libc::open(path.as_ptr(), libc::O_WRONLY | libc::O_CLOEXEC);
        if devnull < 0 {
            f();
            return;
        }
        let saved = libc::dup(2);
        libc::dup2(devnull, 2);
        libc::close(devnull);

        f();

        if saved >= 0 {
            libc::dup2(saved, 2);
            libc::close(saved);
        }
    }
    #[cfg(not(unix))]
    f();
}

pub fn play_notification_sound() {
    with_stderr_suppressed(|| {
        let Ok(stream) = OutputStreamBuilder::open_default_stream() else { return };
        let sink = Sink::connect_new(&stream.mixer());
        let cursor = Cursor::new(NOTIF_BYTES);
        let Ok(source) = Decoder::new(cursor) else { return };
        sink.append(source);
        sink.sleep_until_end();
    });
}

pub fn send_desktop_notification(from: &str, text: &str) {
    let body: String = text.chars().take(120).collect();
    let _ = std::process::Command::new("notify-send")
        .arg("--app-name=ttychat")
        .arg("--urgency=normal")
        .arg("--expire-time=5000")
        .arg(format!("ttychat ✉ {from}"))
        .arg(body)
        .spawn();
}
