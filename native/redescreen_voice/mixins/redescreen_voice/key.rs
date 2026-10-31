// Native key bindings. Audio is captured only while PTT is held (or open mode is selected).
fn redescreen_voice_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    use crate::game::redescreen_voice as voice;
    if !game.native_on(voice::ID) {
        voice::set_ptt(false);
        return;
    }
    let ptt = game.native_mod_key(voice::ID, "ptt", "key.keyboard.left.alt");
    if key == ptt {
        voice::set_ptt(pressed && game.native_playing());
    }
    let menu = game.native_mod_key(voice::ID, "menu", "key.keyboard.v");
    if key == menu && pressed && game.native_playing() {
        game.native_chat(&format!("RedeScreen Voice: {}. Configuracoes em Opcoes > Mods.", voice::status()));
    }
}
