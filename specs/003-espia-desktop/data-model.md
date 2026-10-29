# Data Model: Espia Desktop

Nenhum dado novo no servidor.

| Entidade | Onde | Campos | Regras |
|---|---|---|---|
| Link de sala | argumento de linha de comando / colado | esquema (`espia`/`https`), código | ver [contracts/deep-link.md](./contracts/deep-link.md) |
| Estado da janela | arquivo do plugin window-state em `%APPDATA%\dev.espia.app` | posição, tamanho, maximizada | restaurado ao abrir |
| Perfil do WebView2 | `%LOCALAPPDATA%\dev.espia.app\EBWebView` | `localStorage` do site (apelido, qualidade, volumes) | mesmo conteúdo do site (Princípio III); separado do navegador |
| `router.leaveGuard` | site, memória | `() => string \| null` | a sala aberta devolve a pergunta se você está transmitindo |
