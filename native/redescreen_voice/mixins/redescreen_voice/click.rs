// Persist settings without pretending that the voice backend is active.
fn redescreen_voice_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use crate::game::redescreen_voice as voice;
    if mod_id != voice::ID {
        return;
    }
    match button {
        0 => {
            let open = game.native_setting(voice::ID, "mode").as_deref() == Some("open");
            game.native_set_setting(voice::ID, "mode", if open { "ptt" } else { "open" });
        }
        1 => {
            let current = game.native_setting(voice::ID, "volume")
                .and_then(|s| s.parse::<u8>().ok())
                .unwrap_or(100);
            let next = if current >= 100 { 0 } else { current.saturating_add(25).min(100) };
            game.native_set_setting(voice::ID, "volume", &next.to_string());
        }
        _ => {}
    }
}
