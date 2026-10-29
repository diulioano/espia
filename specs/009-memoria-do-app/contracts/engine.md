# Contract: motor de página do app (009)

Complementa [`specs/007-fechar-para-bandeja/contracts/tray.md`](../../007-fechar-para-bandeja/contracts/tray.md). Constituição 1.5.0.

## Comando página → app: `app_set_room`

```ts
invoke('app_set_room', { inRoom: boolean }): Promise<void>
```

- Só para a origem do site (e o Worker local em debug), só na janela `main`. É a mesma capability de `capture_*` e `app_set_live`.
- Sem retorno, sem erro visível: a página chama e ignora falhas (`.catch(() => {})`), como o `reportLive`.
- A página (`src/routes/RoomRoute.svelte`) chama:
  - `inRoom = connected && !ended` do store da sala, sempre que mudar;
  - `false` quando a rota da sala é destruída.
  - Pode repetir o mesmo valor (o app ignora repetições).
- O app volta a `false` sozinho em `PageLoadEvent::Started`.

`src/lib/desktop.ts`:

```ts
export function reportRoom(inRoom: boolean): void // invoke só dentro do app; no navegador não faz nada
```

## Ciclo da janela (`engine.rs`)

| Evento | Efeito |
|---|---|
| Minimizar / restaurar (`WindowEvent::Resized` + `is_minimized()`) | `window = Minimized` / `Visible` → `apply` |
| X (esconder, 007) | `window = Hidden` → `apply` |
| Clique no ícone / "Abrir o Espia" | janela existe: mostra, `Visible`, `apply`. Não existe: `create_main(None)` |
| Segunda abertura (atalho / `espia://s/…`) | janela existe: como hoje (003) + `Visible`. Não existe: `create_main(sala do link)` |
| `app_set_room` / `app_set_live` mudou | `apply` |
| Timer de 30 s expira (mesma geração, regra ainda `ReleaseAfter`) | `capture::stop_all` + salvar posição da janela + `window.destroy()` → motor sai |
| Última janela destruída sem "Sair" | `ExitRequested{code: None}` → `prevent_exit()` |
| "Sair" (007) | `app.exit(0)` → sai de fato |

Menu da bandeja: sem mudança. "Parar transmissão" só aparece ao vivo, e com o motor solto nunca se está ao vivo.

## Processos

- Nenhum Job Object: o processo principal do motor sai sozinho quando o app que o abriu morre. A conferência é o roteiro M9. Só se sobrar processo, voltar ao research R5 (job com *breakaway* para não levar junto o navegador aberto por links).
- O app nunca abre, mata nem mexe em processos que não são do motor dele (FR-908).
