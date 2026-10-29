# Data Model: Instalar o app pela tela inicial

## Convite do app (`HomeView.appInvite`)

| Valor | Quando | O que mostra |
|---|---|---|
| `undefined` | `appInviteVisible(...) === false` (R2) | nada |
| `'offer'` | Windows desktop, fora do app, não dispensado | título, o que o app traz, "Baixar o app", "Agora não" |
| `'hint'` | depois do clique em "Baixar o app" | dica do SmartScreen + "Não baixou? Abra a página de versões" |

Transições: `offer --baixar--> hint`; `offer --agora não--> undefined` (grava a preferência); `hint` dura até recarregar.

## Entrada de `appInviteVisible`

| Campo | Origem |
|---|---|
| `platform` | `navigator.userAgentData?.platform` (pode faltar) |
| `mobile` | `navigator.userAgentData?.mobile` (pode faltar) |
| `ua` | `navigator.userAgent` |
| `inApp` | `'__TAURI_INTERNALS__' in window` |
| `dismissed` | preferência abaixo |

## Preferência (`localStorage`)

| Chave | Valor | Regra |
|---|---|---|
| `espia:appinvite` | `"dismissed"` | gravada no "Agora não"; nunca apagada pelo site (FR-805) |

## Ações (`HomeActions`)

- `downloadApp()`: navega para `DOWNLOAD_URL` (download direto) e passa o convite para `'hint'`.
- `dismissAppInvite()`: grava a preferência e esconde o convite.

## Textos

| Onde | Texto |
|---|---|
| Título | Tem Windows? Baixe o app do Espia |
| Linha | Escolha a janela numa grade com prévias, som sem vazar o Discord e feche sem sair da sala. |
| Botões | Baixar o app · Agora não |
| Dica | Baixando… Se o Windows avisar que protegeu o computador, clique em **Mais informações → Executar assim mesmo**. |
| Reserva | Não baixou? Abra a página de versões |
| Instalador: boas-vindas | **Bora instalar o Espia** / O app abre o mesmo Espia do site, com o seletor de janelas com prévias, o som sem o Discord e a bandeja. Não precisa de administrador. |
| Instalador: fim | **Pronto!** / O Espia está instalado. Links `espia://` agora abrem direto nele. · ☑ Abrir o Espia |
