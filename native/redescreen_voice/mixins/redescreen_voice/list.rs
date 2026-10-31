// Register the mod with Horizon's native Options > Mods UI.
fn redescreen_voice_list(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::redescreen_voice as voice;
    mods.push(crate::game::native_hooks::NativeMod {
        id: voice::ID.into(),
        title: format!("Simple Voice Chat (native) {}", voice::VERSION),
        description: "Adaptador em desenvolvimento; sem conexao de voz nesta versao.".into(),
        keys: vec![
            ["menu".into(), "Informacoes de voz".into(), "key.keyboard.v".into()],
            ["ptt".into(), "Apertar para falar (futuro)".into(), "key.keyboard.left.alt".into()],
        ],
    });
}
