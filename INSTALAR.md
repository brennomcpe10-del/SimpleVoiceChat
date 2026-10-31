# Instalar RedeScreen Voice no Horizon ORIGINAL 26.3

**Versão experimental 0.4.0-beta.1.** Esta integração não usa Minecraft Java, Fabric, Forge ou um executável Horizon substituto. Ainda precisa ser testada em um servidor real.

## Download

Abra o workflow **Windows Voice Helper (original Horizon)** da branch `native-hnmod`, baixe o artefato `redescreen-voice-0.4.0-beta.1-windows-x64-KIT` e extraia. Ele contém:

- `redescreen_voice-0.4.0-beta.1.hnmod`: código nativo do mod, compilado pelo launcher oficial.
- `redescreen-voice-helper.exe`: programa pequeno que executa Opus, AES-GCM e captura/reprodução de áudio. **Não é outro cliente Minecraft.**
- `SHA256.txt`: hashes para conferência.

## Instalação

1. Abra o **Horizon Launcher 26.3** oficial e selecione sua instância.
2. Na página **Mods**, use **Add from GitHub** com `https://github.com/brennomcpe10-del/SimpleVoiceChat/tree/native-hnmod` e selecione o mod `redescreen_voice`. A alternativa por linha de comando é `horizon-launcher native-build "CAMINHO\\redescreen_voice-0.4.0-beta.1.hnmod" --trust-native-code` (o próprio usuário precisa decidir se confia no código).
3. Execute **Native build** no launcher, leia o aviso de código nativo e aguarde a compilação. O primeiro build pode consumir alguns gigabytes de armazenamento temporário; não é outro Horizon separado para iniciar manualmente.
4. Faça a verificação do arquivo `.exe` pelo Windows Defender. **Não desative a proteção**.
5. Execute `redescreen-voice-helper.exe` e deixe a janela aberta.
6. Inicie a instância pelo **Horizon Launcher**, entre no RedeScreen e pressione **V** para ver o status.
7. Segure **Alt esquerdo** para falar; em **Opções > Mods**, alterne entre push-to-talk e microfone aberto e ajuste o volume.

## Diagnóstico

- **Chave recebida; inicie o auxiliar de voz**: o programa auxiliar não está aberto, não conseguiu escutar em 127.0.0.1:37462 ou foi bloqueado.
- **Solicitando chave**: ainda não recebemos `voicechat:secret` do plugin do servidor; confira se o Simple Voice Chat está instalado no servidor e se aceita os protocolos 20 ou 21.
- **Autenticando UDP**: o auxiliar está aberto, mas ainda não confirmou o canal de voz UDP. Pode haver firewall, porta bloqueada ou configuração do servidor.
- **Conectado**: o auxiliar conseguiu estabelecer a sessão UDP; ainda é necessário testar microfone, som e proximidade com outra pessoa.

O programa auxiliar só aceita pacotes de controle em `127.0.0.1` e não grava a chave de voz em arquivo. O áudio é enviado ao servidor de voz indicado pelo próprio Simple Voice Chat.

**Limitações:** nenhum teste real no RedeScreen ou nos dispositivos de áudio do usuário foi realizado. O mod depende de suporte a mods nativos no launcher instalado.
