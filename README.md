# Espia

Compartilhamento de tela entre amigos, estilo "Go Live" do Discord. Cria uma sala, manda o link, e
quem quiser clica em **Transmitir**. O vídeo vai direto de um PC pro outro (WebRTC P2P); o servidor só
apresenta as pessoas.

- **Site**: https://espia.espia.workers.dev (Chrome/Edge para transmitir; qualquer navegador para assistir)
- **App para Windows (opcional, leve)**: baixe o `Espia_*_x64-setup.exe` na
  [última release](../../releases/latest).

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

## Desenvolvimento

Requisitos: Node 24. Para o app desktop: Rust (stable) e Windows.

```sh
npm ci
npm run dev            # site com Vite (a API fica no `wrangler dev`: npm run dev:worker)
npm run check          # svelte-check + tsc
npm test               # Vitest (lib + Worker)
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
