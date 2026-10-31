// Tiny, bounded handoff from Horizon's network crate to the game-thread
// native mixin. No secret is logged, saved, or exposed outside this process.
use std::sync::Mutex;

static INCOMING: Mutex<Vec<Vec<u8>>> = Mutex::new(Vec::new());

pub fn push(secret: &[u8]) {
    if secret.len() > 65536 {
        return;
    }
    if let Ok(mut pending) = INCOMING.lock() {
        if pending.len() < 4 {
            pending.push(secret.to_vec());
        }
    }
}

pub fn drain() -> Vec<Vec<u8>> {
    INCOMING.lock().map(|mut pending| std::mem::take(&mut *pending)).unwrap_or_default()
}

pub fn clear() {
    if let Ok(mut pending) = INCOMING.lock() {
        pending.clear();
    }
}
