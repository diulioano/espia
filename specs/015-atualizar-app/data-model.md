# Data model — 015

## Versão publicada (`/download/latest.json`)

| Campo | Tipo | Regra |
|---|---|---|
| `version` | semver | versão mais nova; o app só aceita se `> instalada` (FR-009) |
| `minimum` | semver? | menor versão que funciona com o site (R5); ausente = sem mínimo |
| `platforms.windows-x86_64.url` | URL | instalador **por versão** no próprio site (R4) |
| `platforms.windows-x86_64.signature` | texto | assinatura minisign; sem bater com a chave pública do app → recusado (FR-008) |

## Estado da atualização no app (Rust, `UpdateState` em memória)

| Campo | Tipo | Notas |
|---|---|---|
| `found` | `Option<Update>` | resultado da última checagem |
| `required` | bool | `instalada < minimum` |
| `dismissed` | bool | "Depois" nessa execução; vale só se `!required` |
| `phase` | `none \| available \| downloading \| installing \| failed` | o que vai pra página (contracts/update.md) |
| `started_at` | Instant | abertura do processo (janela de 10 s do R6) |
| `resumed` | bool | esta abertura veio de uma atualização (não instala de novo) |

Nada disso vai para o disco, exceto o `update-resume.json`.

### Transições

```
none ──(checagem acha versão)──► available
available ──(app_update(true) | abertura ≤10 s | bandeja ≥30 min)──► downloading ──► installing ──► (app sai e reabre)
downloading ──(erro de rede/assinatura)──► failed ──(próxima checagem acha)──► available
available ──(app_update(false), !required)──► none   [até o fim da execução]
```

## Contador da bandeja (em `EngineState`, R9)

| Campo | Tipo | Regra |
|---|---|---|
| `hidden_out_since` | `Option<Instant>` | começa quando a janela fica `Hidden` (ou o motor é solto) com `in_room == false`; zera ao mostrar a janela ou entrar numa sala |

## Retomada (`app_config_dir/update-resume.json`)

| Campo | Tipo | Regra |
|---|---|---|
| `room` | `"/s/xxx-xxx"` ou `null` | validado por `links::room_path` |
| `hidden` | bool | `true` = abrir só na bandeja (atualização silenciosa) |
| `at` | ms desde 1970 | > 5 min → ignorado |

Apagado assim que lido.

## Página (`src/lib/app-update.svelte.ts`)

View-model do aviso, alimentado por `window.espiaUpdate`:

| Campo | Tipo |
|---|---|
| `phase` | igual ao contrato |
| `version` | string |
| `required` | bool |
| `canLater` | `!required && phase === 'available'` |
| `confirmLive` | a página está transmitindo → pedir confirmação antes de `app_update(true)` |
