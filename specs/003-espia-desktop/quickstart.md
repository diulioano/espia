# Quickstart: validar o Espia Desktop

Pré-requisitos (dev): Node 24, Rust 1.90+, Windows 10/11. `npm ci` na raiz.

## Q1. Testes automáticos
```sh
npm run check && npm test          # inclui test/lib/desktop.test.ts
cd src-tauri && cargo test         # links.rs e share_bar.rs
```

## Q2. Rodar o app
```sh
npm run desktop:dev                # janela "Espia" com o site publicado
npm run desktop:build              # instalador em src-tauri/target/release/bundle/nsis/
```
Esperado: tela "Abrindo o Espia…" com o castor e, em ~1 s, a tela inicial do site.
Sem internet (desligar o Wi-Fi): "Sem conexão com o Espia" + "Tentar de novo".

## Q3. Transmitir sem a barra (manual, precisa de clique real)
1. No app, criar sala → **Transmitir** → escolher uma **janela** (ex.: Bloco de Notas) → Transmitir.
2. Esperado: a transmissão começa e **não** aparece a barra flutuante "espia.espia.workers.dev está
   compartilhando…". Parar pelo botão do Espia.
3. Em outro PC/navegador, abrir o link da sala: vê a janela e ouve o som (regras de áudio do site).
4. Se a barra aparecer: rodar a build de debug (`npm run desktop:dev`) e repetir; após 2 min o console
   lista `[share_bar] não escondida: "<título>" (<altura>px)` — usar isso para ajustar `title_is_bar`.

## Q4. Links
- App fechado: Win+R → `espia://s/k7f-q2x` → abre na tela de entrada da sala.
- App aberto: repetir → a mesma janela vem pra frente e vai pra sala (sem segunda janela).
- Transmitindo: repetir com outro código → pergunta "Você está transmitindo. Parar e ir pra outra sala?".
- `espia://qualquer-coisa` → só foca a janela.
- Link externo dentro do site → abre no navegador padrão.

## Q5. Leveza (SC-001/SC-002)
- Tamanho de `Espia_*_x64-setup.exe` ≤ 10 MB.
- Gerenciador de Tarefas, app numa sala sem transmitir: soma de `Espia` + `msedgewebview2` ≤ ~150 MB.

## Q6. Release
`git tag v0.1.0 && git push --tags` → em ~20 min a Release `v0.1.0` tem o instalador.
