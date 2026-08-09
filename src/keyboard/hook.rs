use rdev::{listen, Event, EventType, Key};

pub fn start<F>(callback: F)
where
    F: Fn(Key) + Send + 'static,
{
    let result = listen(move |event: Event| {
        if let EventType::KeyPress(key) = event.event_type {
            callback(key);
        }
    });

    if let Err(err) = result {
        eprintln!("Keyboard hook error: {err:?}");
    }
}