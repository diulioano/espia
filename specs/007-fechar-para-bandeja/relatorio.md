# Relatório: Fechar o app sem sair (bandeja) — v0.1.4

## O que foi feito

- **O X esconde a janela**: o Espia some da barra de tarefas e fica com um ícone perto do relógio. Continua na sala e transmitindo.
- **Trazer de volta**: clicar no ícone, abrir o atalho de novo ou abrir um link `espia://`. O link de sala segue o fluxo da 003 (pergunta se você está transmitindo).
- **Aviso da primeira vez**: uma notificação do Windows: "O Espia continua aberto". Fica gravado e não aparece de novo.
- **Menu do ícone** (botão direito): Abrir o Espia, Parar transmissão (só ao vivo), Fechar no X (marcável) e Sair.
- **Ao vivo**: o ícone ganha um ponto na cor de destaque do design (a mesma do contorno "ao vivo") e a dica vira "Espia · ao vivo".
- **Sair transmitindo**: uma caixa do Windows pergunta "Você está transmitindo. Sair mesmo assim?" (padrão: Não). O mesmo vale para o X com "Fechar no X" ligado.
- **Desligar o Windows**: não pergunta nada (o Windows não manda o "fechar" da janela).
- **Preferências**: `%APPDATA%\dev.espia.app\tray.json`.
- **Site**: avisa o app quando a transmissão própria começa e para (tela nativa, captura do navegador e câmera), pelo comando `app_set_live`. No navegador, nada muda.
- **Constituição 1.4.0**: `app_set_live` é a única API nova exposta à página.

## Diferenças em relação ao plano

1. O aviso de ao vivo saiu do `RoomRoute.svelte` (um efeito sobre `myBroadcast`), não do `room.svelte.ts`. Cobre todas as fontes sem mexer no store.
2. O "ponto vermelho" do ícone ao vivo usa a cor de destaque do design (`--color-accent`), que é a cor de "ao vivo" do Espia.

## Testes

- `npm run check`: 0 erros. `npm test`: **106 passando** (4 novos em `desktop.test.ts`: `reportLive` dentro e fora do app, `espiaStopLive`, troca de gancho).
- `cargo test`: **40 passando** (6 novos: `tray.json` ausente/inválido/campos extras/ida e volta, o X e o Sair). `cargo clippy --all-targets`: 0 avisos.
- **No app compilado** (sem capturar a tela): mandar o "fechar" para a janela → a janela some e o processo continua; o `tray.json` grava o aviso; abrir o app de novo mostra a janela; com `closeOnX: true`, o X fecha o app.
- **Não testado**:
  - a transmissão com a janela escondida (R2) e o menu e a confirmação com o ícone ao vivo. Isso exige transmitir de verdade e alguém assistindo (roteiro B1–B8);
  - o clique no ícone e no menu, que é um clique na bandeja do Windows.

## Tamanho

Instalador `Espia_0.1.4_x64-setup.exe`: **1,60 MiB** (1.675.076 bytes), +164 KB em relação à v0.1.3 (meta: + < 0,3 MB).

## Publicado

- Site: versão do Worker `2eca8fb5` (2026-10-29), com o `app_set_live`.

## ⚠️ O que precisa de você

1. **Roteiro B1–B8** (`quickstart.md`), principalmente o B2: transmitir, clicar no X, esperar 5 min e conferir com quem assiste que nada travou.
2. **Tag `v0.1.4`** depois do roteiro (o workflow Release gera o instalador).
