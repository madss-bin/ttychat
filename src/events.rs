use crossterm::event::{Event, EventStream, KeyEvent};
use futures::StreamExt;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;

#[derive(Clone)]
pub enum AppEvent {
    Key(KeyEvent),
    Tick,
    Resize,
    FocusGained,
    FocusLost,
}

pub fn spawn_event_task(tx: mpsc::UnboundedSender<AppEvent>) {
    tokio::spawn(async move {
        let mut reader = EventStream::new();
        let mut tick = time::interval(Duration::from_millis(60));

        loop {
            tokio::select! {
                _ = tick.tick() => {
                    if tx.send(AppEvent::Tick).is_err() { break; }
                }
                maybe_event = reader.next() => {
                    match maybe_event {
                        Some(Ok(Event::Key(key))) => {
                            if tx.send(AppEvent::Key(key)).is_err() { break; }
                        }
                        Some(Ok(Event::Resize(_, _))) => {
                            if tx.send(AppEvent::Resize).is_err() { break; }
                        }
                        Some(Ok(Event::FocusGained)) => {
                            if tx.send(AppEvent::FocusGained).is_err() { break; }
                        }
                        Some(Ok(Event::FocusLost)) => {
                            if tx.send(AppEvent::FocusLost).is_err() { break; }
                        }
                        Some(Err(_)) | None => break,
                        _ => {}
                    }
                }
            }
        }
    });
}
