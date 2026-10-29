# Feature Specification: Seletor de tela (janela ou tela inteira, com ou sem som)

**Feature Branch**: `004-seletor-de-tela` (feita direto na `main`)

**Created**: 2026-10-28

**Status**: Implementada (ver [tasks.md](./tasks.md))

**Relacionada a**: [002-audio-sem-discord](../002-audio-sem-discord/spec.md) (regras de som, implementadas junto) e ao design "Espia" v1.2, seção **4b · App desktop · seletor de tela** (`SourcePicker.dc.html`, cenas A1–A4).

**Input**: "Faça a seleção da tela com som ou não. Não tenho certeza se configurou de tirar o som do Discord ao compartilhar a tela." + design `Espia.dc.html` (SourcePicker).

## Contexto

- A 002 (som sem vazar o Discord) estava especificada, mas **não implementada**: `capture.ts` ainda pedia `systemAudio: 'include'`, que no Windows manda o som do PC inteiro, inclusive a chamada do Discord.
- O design novo troca o modal "Transmitir" por um seletor de uma etapa: abas **Janelas / Telas inteiras**, grade de miniaturas, "Compartilhar áudio" com texto que muda conforme a escolha, e resolução/FPS na mesma faixa.

## Análise de viabilidade (o que dá e o que não dá)

| Item do design | Viável? | Por quê |
|---|---|---|
| Abas Janelas / Telas inteiras | ✅ | Vira a dica `displaySurface: 'window' \| 'monitor'` do `getDisplayMedia`: o seletor nativo já abre na aba certa. |
| Grade com miniaturas reais das janelas | ❌ | Nem o navegador nem o WebView2 (app desktop) deixam a página listar janelas ou escolher a fonte: o `getDisplayMedia` **sempre** abre o seletor nativo, e o evento `ScreenCaptureStarting` do WebView2 só permite aceitar/cancelar. Mostrar miniaturas obrigaria a pessoa a escolher duas vezes. |
| "Compartilhar áudio" + texto por aba | ✅ | Com a regra da 002. |
| Telas inteiras com "todo o som do computador" | ❌ (por decisão) | É exatamente o caminho que vazou o Discord. Tela inteira vai **sem som**; o texto explica e sugere escolher a janela. |
| Resolução e FPS na faixa de baixo | ✅ | |
| A4 · sem permissão (macOS) | — | O app é só Windows; no Windows não existe essa permissão. Fica o estado E3 que já existe. |

## User Scenarios & Testing

### US1 — Escolher janela com som (P1)
Clico em **Transmitir**, fico na aba **Janelas**, deixo "Compartilhar áudio" ligado e clico **Transmitir**. O seletor do navegador abre na lista de janelas; escolho o jogo. A galera ouve o som do jogo e nada do Discord.

1. **Given** aba Janelas e som ligado, **Then** o texto diz que vai só o som da janela escolhida e que Discord e notificações ficam de fora.
2. **Given** som desligado, **Then** o texto diz "Só a imagem vai pra sala." e nenhuma faixa de áudio é enviada.

### US2 — Tela inteira, sem som (P1)
Troco para **Telas inteiras**. O interruptor de som fica desligado e travado, com o texto "Tela inteira vai sem som, pra não vazar o Discord. Pra levar o som, escolha uma janela." O seletor nativo abre na lista de telas e a transmissão vai sem áudio.

### US3 — Escolha lembrada (P3)
A próxima vez que abro o seletor, ele volta na última aba e com a última escolha de som/qualidade.

### Edge cases
- Escolhi "Janelas" no modal mas, no seletor nativo, cliquei numa tela inteira com som do sistema → a rede de segurança da 002 descarta o som e mostra o aviso.
- Navegador sem suporte à dica `displaySurface` → abre o seletor normal; as regras de som continuam valendo.
- Celular não transmite (não muda).

## Requirements

- **FR-401**: O modal Transmitir MUST seguir o layout do `SourcePicker` (título "Escolha o que compartilhar", abas com ícone, faixa com som/resolução/FPS, rodapé com dica e botões), no app desktop e no navegador.
- **FR-402**: As abas MUST ser "Janelas" e "Telas inteiras"; a escolha MUST ir como `displaySurface` para o `getDisplayMedia`.
- **FR-403**: Na aba Telas inteiras o som MUST ficar desligado e desabilitado; a preferência de som da aba Janelas MUST ser preservada.
- **FR-404**: O texto sob "Compartilhar áudio" MUST mudar com a escolha (janela com som / sem som / tela inteira).
- **FR-405**: A dica do rodapé MUST seguir o design: 1080p+60 → aviso de internet; senão "`<res>`p · `<fps>` fps — leve pra todo mundo."
- **FR-406**: O botão principal MUST se chamar "Transmitir" (decisão de 2026-10-28, não "Go Live").
- **FR-407**: A aba escolhida MUST ser lembrada junto com as outras preferências.
- **FR-408**: Todos os FR-201..FR-209 da 002 MUST valer.

## Success Criteria
- **SC-401**: Transmitindo a janela de um jogo durante uma chamada do Discord, quem assiste ouve o jogo e **nenhuma** voz do Discord (manual, M1 da 002).
- **SC-402**: Transmitindo a tela inteira, nenhuma faixa de áudio chega a quem assiste (automático + manual M3).
- **SC-403**: O mock `?mock=room&scene=golive` e `scene=golivescreen` mostram as duas abas.
