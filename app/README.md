# App nativo do Espia

App para Windows inteiro em Rust, sem motor de página. O site (`src/`) e o servidor (`src/worker/`) ficam como estão; o app de antes (`src-tauri/`) está congelado até a troca. Decisões em [research.md](../specs/019-app-rust-convite/research.md) e no [ADR 0002](../docs/adr/0002-app-nativo-em-rust.md).

## Crates

| Crate | O que faz | Depende de |
|---|---|---|
| `espia-core` | Lógica pura: protocolo, sala (redutor), qualidade, view-models, dados do aparelho. Sem rede, mídia ou tela. | nada |
| `espia-net` | Sinalização (WebSocket) e API HTTP. | `espia-core` |
| `espia-media` | Ligação de vídeo: par, malha, envio, recepção, som. | `espia-core` |
| `espia-capture` | Captura no Windows: telas, janelas, miniaturas, som por janela, câmera, entradas de som. | `espia-core` |
| `espia-shell` | Bandeja, instância única, link `espia://`, janela, avisos do Windows, arquivos do aparelho, atualização. | `espia-core` |
| `espia-ui` | Telas em Slint e cenas de dados falsos. | só `espia-core` |
| `espia` | Binário `Espia.exe`: junta tudo. | todas |

A tela não fala com rede, captura nem mídia: `espia-ui` só enxerga `espia-core`, então o grafo de dependências garante a regra IV da constituição. Não acrescente outra crate do workspace às dependências de `espia-ui`.

## Outras pastas

- `fixtures/protocol/`: JSON do protocolo, conferido pelo site e pelo app.
- `packaging/`: instalador.
- `spikes/`: provas dos gates G1, G2 e G3 (apagadas no fim).

## Como rodar

Da raiz do repositório:

```sh
npm run app:check   # cargo check --workspace
npm run app:test    # cargo test --workspace
npm run app:dev     # abre o app
npm run app:mock -- home --scene default   # uma tela com dados falsos
```

Ou direto em `app/`: `cargo check --workspace`, `cargo test --workspace`, `cargo run -p espia`.

Precisa de Rust 1.90 ou mais novo (edição 2024).
