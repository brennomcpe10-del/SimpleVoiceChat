// Native settings UI; values are stored in the same per-mod folder as official mods.
fn redescreen_voice_settings(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    use crate::game::redescreen_voice as voice;
    if mod_id != voice::ID {
        return;
    }
    let mode = game.native_setting(voice::ID, "mode").unwrap_or_else(|| "ptt".into());
    let volume = game.native_setting(voice::ID, "volume").unwrap_or_else(|| "100".into());
    buttons.extend([
        format!("Modo futuro: {}", if mode == "open" { "Microfone aberto" } else { "Push-to-talk" }),
        format!("Volume futuro: {}%", volume),
        format!("Estado: {}", voice::status()),
    ]);
}
