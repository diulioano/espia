# App para Windows inteiro nativo em Rust

Decisão de 2026-11-11: o app do Espia para Windows passa a ser inteiro nativo em Rust, sem motor de página. O site (Svelte) e o servidor (Worker e Durable Object, em TypeScript) ficam como estão, e o protocolo não muda. O app de antes (`src-tauri/`) fica congelado até o nativo passar nas histórias de transmitir, assistir e paridade da feature 019.

O motivo é o peso. O app de antes carrega um motor de página inteiro só para mostrar o site, e é esse motor que ocupa a memória e disputa processador com o jogo durante a transmissão. As metas de memória e de processador da feature 019 só cabem num app que não carrega esse motor.

O código vive em `app/`, num workspace de sete crates em que a crate das telas só enxerga a de lógica pura. Com isso, a regra de a tela não falar com rede, captura nem mídia é garantida pelo grafo de dependências. A ligação de vídeo usa a crate `libwebrtc` da LiveKit e a interface usa Slint. As duas escolhas dependem de provas feitas antes de construir o resto (gates G1 e G2); se uma falhar, entra o plano B registrado em `specs/019-app-rust-convite/research.md` (R1 a R3).

Alternativas consideradas:

- **Só o envio nativo, mantendo o motor de página para as telas**: descartada, porque o motor continua carregado e a memória fica igual.
- **App, site e servidor em Rust**: descartada, porque o site e o servidor não têm o problema de peso e reescrevê-los só aumentaria o custo e o risco.

Este ADR substitui a frase "A migração ampla do motor para Rust foi adiada" do [ADR 0001](./0001-conexao-direta-e-franquia-de-midia.md). O resto do ADR 0001 continua valendo: conexão direta como padrão, sem SFU, sem TURN pago, meta de atraso de até 500 ms na maior parte do tempo e duas transmissões com até sete espectadores.
