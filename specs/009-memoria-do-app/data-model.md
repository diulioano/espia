# Data Model: Memória do app (009)

Tudo em memória do `Espia.exe`. Nada é gravado.

## EngineState (Rust, `engine.rs`)

| Campo | Tipo | Origem | Padrão |
|---|---|---|---|
| `window` | `Visible` \| `Minimized` \| `Hidden` | `tray::show`/`hide` (X), `WindowEvent::Resized` + `is_minimized()` | `Visible` ao criar a janela |
| `in_room` | bool | `app_set_room` (página) | `false`; volta a `false` em `PageLoadEvent::Started` |
| `live` | bool | `app_set_live` (já existe, 007) | `false` |
| `engine` | `Loaded` \| `Released` | a janela `main` existe ou não | `Loaded` |
| `timer_gen` | u64 | cada `apply` incrementa; o timer de 30 s só age se a geração não mudou | 0 |

## Regra (função pura, com testes)

```text
engine_action(window, in_room, live) ->
  live                               → Normal            (FR-905: ao vivo nada muda)
  Visible                            → Normal
  Minimized                          → Low               (FR-904c; nunca descarrega)
  Hidden && in_room                  → Low               (FR-904a)
  Hidden && !in_room                 → ReleaseAfter(30 s) (FR-904, FR-904b)
```

`Normal` / `Low` = nível de memória pedido ao motor (research R3). `ReleaseAfter` também pede `Low` enquanto espera.

## Transições

```text
Loaded+Normal ──minimizar──► Low ──restaurar──► Normal
      │
      └──X──► Hidden ─┬─ numa sala ──► Low ──mostrar──► Normal
                      └─ fora de sala ──► Low + timer 30 s ──expira──► Released (janela destruída, 0 motor)
                                               └─ mostrar antes / entrar numa sala ──► cancela o timer
Released ──ícone / Abrir / atalho / espia://──► create_main() ──► Loaded+Normal (tela inicial ou sala do link)
Hidden + in_room true→false ──► começa o timer (FR-904b)
"Sair" ──► app.exit(0)
```

Qualquer mudança de estado chama `apply`, que aumenta `timer_gen`. Um timer antigo que expira não faz nada.

## Tempos e limites

| Nome | Valor | Por quê |
|---|---|---|
| `RELEASE_AFTER` | 30 s | reabrir logo não recarrega; SC-903 mede em 1 min |
| Meta escondido fora de sala | ≤ 40 MB, 0 motor | SC-903 |
| Base (Gerenciador, minimizado numa sala) | ~228 MB | spec, Contexto |
