# Contrato: links de sala no app desktop

## Formatos aceitos

| Link | Resultado |
|---|---|
| `espia://s/<código>` | sala `<código>` |
| `https://espia.diuliano.com/s/<código>` (com ou sem `?…`, barra final) | sala `<código>` |
| `https://espia.espia.workers.dev/s/<código>` (endereço antigo, continua aceito) | sala `<código>` |
| qualquer outro (`espia://qualquer-coisa`, outro host, `http://`, código inválido) | ignorado: abre/foca na tela inicial |

`<código>` = `xxx-xxx` com o alfabeto `abcdefghjkmnpqrstuvwxyz23456789` (o mesmo de `src/lib/room-id.ts`),
sem diferença de maiúsculas. Implementação: `src-tauri/src/links.rs` (`room_path`).

## Fluxo

1. **App fechado**: o Windows roda `Espia.exe espia://s/<código>`. O app abre a página local com
   `index.html?to=/s/<código>`; ela confere a conexão e faz `location.replace(ORIGIN + to)`.
2. **App aberto**: a segunda instância repassa os argumentos à primeira e sai. A primeira foca a janela e
   executa no site:
   ```js
   (window.espiaOpen || ((p) => location.assign("https://espia.diuliano.com" + p)))("/s/<código>")
   ```
3. **`window.espiaOpen(path)`** (site, `src/lib/desktop.ts`):
   - caminho inválido ou igual ao atual → nada;
   - `router.leaveGuard()` devolve texto (você está transmitindo) → `confirm(texto)`; cancelou → nada;
   - senão `router.go(path)` (SPA, sem recarregar).
   Na página local, `espiaOpen` troca o destino e tenta conectar de novo.

## Navegação

- Dentro da janela: `https://espia.diuliano.com/*`, a página local (`http://tauri.localhost`),
  `about:`/`blob:`/`data:`.
- Qualquer outro `http(s)` → navegador padrão; outros esquemas → bloqueados.
- `window.open`/`target=_blank` → navegador padrão (nunca uma segunda janela do app).
