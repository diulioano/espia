# Espia

Compartilhamento de tela entre amigos, estilo "Go Live" do Discord. Cria uma sala, manda o link, e
quem quiser clica em **Transmitir**. O vídeo vai direto de um PC pro outro (WebRTC P2P); o servidor só
apresenta as pessoas. Cada transmissão aparece com **Assistir**: vídeo e som só chegam após esse clique.

A versão 0.1.10 usa somente conexão direta, sem TURN enquanto o limite de custo de mídia não for comprovado. Algumas redes podem não conectar; o Espia informa a limitação.

- **Site**: https://espia.diuliano.com (Chrome/Edge para transmitir; qualquer navegador para assistir)
- **App para Windows (opcional, leve)**: [baixar o instalador mais recente](https://espia.diuliano.com/download/Espia-setup.exe).

## App desktop

O app é uma janela própria para o Espia, sem abas do navegador, com o ícone do castor. Ele usa o
WebView2 que já vem no Windows, então o instalador tem poucos MB.

- Instala sem administrador. Se o PC não tiver o WebView2, o instalador baixa sozinho.
- O Windows pode avisar **"O Windows protegeu o computador"** porque o app não é assinado. Clique em
  **Mais informações → Executar assim mesmo**.
- Links `espia://s/<código>` abrem a sala no app. Você também pode colar o link `https://…/s/<código>` na
  tela inicial.
- Ao transmitir, o app não mostra a barra "… está compartilhando …" do navegador. Para parar, use o botão
  **Parar** do Espia.
- O app sempre abre a versão mais nova do site, então só é preciso reinstalar quando sair uma versão nova
  do app em si.
- Memória: no Gerenciador de Tarefas, o motor do app aparece como **"WebView2 Manager"** (com o castor).
  Fechado no X e fora de sala, depois de 30 s o app desliga esse motor e fica só o ícone na bandeja
  (poucos MB); ao abrir de novo, a janela volta em segundos. Minimizado ou escondido numa sala, ele pede
  ao motor para gastar menos. Para medir: `powershell -File scripts/memoria.ps1`.

## Desenvolvimento

Requisitos: Node 24. Para o app desktop: Rust (stable) e Windows.

```sh
npm ci
npm run dev            # site com Vite (a API fica no `wrangler dev`: npm run dev:worker)
npm run check          # svelte-check + tsc
npm test               # Vitest (lib + Worker)
node --test test/scripts/acceptance-017.test.mjs  # métricas de aceitação da 017
npm run desktop:dev    # app desktop apontando para o site publicado
npm run desktop:build  # instalador em src-tauri/target/release/bundle/nsis/
```

Telas com dados falsos: `/?mock` (ex.: `/?mock=room&scene=grid`).

Documentação: [`DEFINICOES.md`](DEFINICOES.md) (decisões) e `specs/` (Spec Kit: 001 site, 002 áudio,
003 app desktop).

## Publicar

- **Site + Worker**: `npm run deploy`. Também roda sozinho a cada push na `main` se o repositório tiver os
  segredos `CLOUDFLARE_API_TOKEN` e `CLOUDFLARE_ACCOUNT_ID`.
- **App**: suba `version` em `src-tauri/tauri.conf.json` e `src-tauri/Cargo.toml`, faça o commit e rode
  `git tag vX.Y.Z && git push --tags`. O GitHub Actions gera o instalador e publica a release.
