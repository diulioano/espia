# Contrato — atualização do app (015)

Vale só dentro do app desktop, na janela `main`, para a origem do site (`capture::origin_ok`). No navegador, nada disso existe (FR-015).

## Arquivo publicado: `GET /download/latest.json`

Gerado por `npm run desktop:release` (`scripts/copy-installer.mjs`) e publicado com `npm run deploy`.

```json
{
  "version": "0.1.9",
  "minimum": "0.1.9",
  "notes": "",
  "pub_date": "2026-11-01T12:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "url": "https://espia.espia.workers.dev/download/Espia_0.1.9_x64-setup.exe",
      "signature": "<conteúdo do .sig gerado pelo build>"
    }
  }
}
```

- `version`, `platforms.windows-x86_64.url` e `signature` são obrigatórios (plugin).
- `minimum` é nosso (R5). Se faltar ou estiver inválido, vale "sem mínimo".
- Cache: sem cache longo (R4).

## App → página: `window.espiaUpdate(state)`

Chamado por `eval` quando o estado muda, e de novo a cada `PageLoadEvent::Finished`. A página pode ainda não ter o gancho (site antigo, tela de carregamento): por isso o `eval` usa `window.espiaUpdate?.(…)`.

```ts
type UpdateState =
  | { phase: 'none' }                                      // nada a mostrar (ou "Depois" escolhido)
  | { phase: 'available'; version: string; required: boolean }
  | { phase: 'downloading'; version: string; required: boolean }
  | { phase: 'installing'; version: string }               // o app vai fechar em instantes
  | { phase: 'failed'; version: string; required: boolean }; // "Não deu pra atualizar" + link pro site
```

- `required`: a versão instalada é menor que `minimum` (US3). Sem "Depois".
- `failed` volta para `available` na próxima checagem que achar a versão.

A tela de carregamento local (`src-tauri/www`) ganha o mesmo gancho, só para `installing`: mostra "Atualizando o Espia…" (R6).

## Página → app: comando `app_update`

```ts
invoke('app_update', { now: boolean }): Promise<void>
```

| `now` | Efeito |
|---|---|
| `true` | Grava `update-resume.json` (sala atual, `hidden: false`), manda `downloading` → `installing`, instala e o app reabre. Erro → `failed`. |
| `false` | "Depois": manda `none` e para de avisar nessa execução. **Ignorado** se `required`. A próxima abertura instala (R6). |

- Sem retorno útil. Chamado de outra origem ou outra janela: ignorado (como `app_set_live`).
- Confirmação ao vivo é da página, **antes** de chamar `now: true`: "Sua transmissão vai parar por alguns segundos. Atualizar agora?".
- Permissão nova na capability `capture.json`: `allow-app-update`.

## Arquivo local: `app_config_dir/update-resume.json`

```json
{ "room": "/s/k7f-q2x", "hidden": false, "at": 1793550000000 }
```

Lido e **apagado** na abertura. Com mais de 5 min, ou inválido, é ignorado. `room` passa por `links::room_path`. Não guarda apelido nem nada da pessoa: o apelido já está no `localStorage` do site.

## Regra (Rust, função pura `update::decide`)

| Situação | Ação |
|---|---|
| sem versão nova, ou versão publicada ≤ instalada | nada (`none`) |
| abertura do processo, encontrada em ≤ 10 s, fora de sala e sem estar ao vivo, e não é a reabertura pós-atualização | instalar já (R6) |
| janela escondida e fora de sala há ≥ 30 min, sem estar ao vivo | instalar em silêncio, `hidden: true` (R9) |
| "Depois" escolhido e `!required` | `none` |
| qualquer outro caso | `available` (aviso) |

Ao vivo nunca instala sem `app_update(true)` (FR-005).
