# Quickstart: validar tela cheia e sala responsiva no celular

Referências: [spec.md](./spec.md) · [contracts/viewer.md](./contracts/viewer.md) · [data-model.md](./data-model.md)

## 1. Gates automáticos

```sh
npm run check
npm test        # zoom, isPhoneLayout, viewerStream, fullscreenId na store, router
npm run build
```

**Esperado**: tudo verde.

## 2. Mocks (sem servidor, câmera ou rede)

Com `npm run dev`, abrir as cenas abaixo. Cada uma tem que abrir e mostrar o estado descrito.

| URL | Conferir |
|-----|----------|
| `/?mock=mobile&scene=viewer` | Imagem ocupando a tela toda em pé, inteira, com faixas em cima e embaixo; nada da sala visível |
| `/?mock=mobile&scene=viewer-controls` | Mudo, volume, PiP e sair na base, dentro da área segura, com botões de 48 px |
| `/?mock=mobile&scene=viewer-zoom` | Imagem aproximada 2,5×; arrastar não passa das bordas; toque duplo volta à imagem inteira |
| `/?mock=mobile&scene=viewer-reconnecting` | Visualizador aberto com o aviso compacto de reconexão |
| `/?mock=mobile&scene=viewer-nosound` | Só o botão de mudo, sem barra de volume |
| `/?mock=mobile&scene=landscape` | Quadro 844×390: palco à esquerda com a altura toda, coluna à direita com chips, pessoas e Sair |
| `/?mock=mobile&scene=viewer-landscape` | Imagem 16:9 ocupando 100% da altura |
| `/?mock=mobile&scene=landscape-camera` | "Parar câmera" visível sem rolar |

## 3. Emulação (Playwright no scratchpad)

Usar Pixel 7 e iPhone 14, em pé e deitado. Rodar nas larguras 320, 360, 390, 430, 844 e 932, com os mocks `home`, `join`, `mobile` (todas as cenas) e `notfound`.

**Esperado**:
- `document.documentElement.scrollWidth <= innerWidth` (SC-004);
- todo `button`, `a`, `[role=radio]` e `input[type=range]` visível com caixa de pelo menos 44×44 (SC-005);
- nas larguras 640–932 com toque e altura de até 540, aparece a sala de celular, não a de computador (FR-010).

## 4. Aparelhos reais

Rodar `npm run dev:lan` e abrir `https://<ip-do-pc>:8787` num Android (Chrome) e num iPhone (Safari). No computador, transmitir uma tela com texto pequeno e algo em movimento, com som. No celular, entrar e clicar em **Assistir**.

| # | Passo | Esperado |
|---|-------|----------|
| 1 | Tocar em Tela cheia em pé | Android: tela cheia do sistema. iPhone: camada na página toda. Imagem inteira, som continua (US1-1, FR-006). |
| 2 | Deitar o celular | Reorganiza em até 1 s; som sem corte; imagem a 100% da altura (US1-2, SC-001, SC-003). |
| 3 | Tocar na imagem e esperar | Controles aparecem e somem em cerca de 3 s (US1-3). |
| 4 | Afastar dois dedos, arrastar, toque duplo | Aproxima até 4×; a página não se move; continua ao vivo; volta inteira (US3). |
| 5 | Ler um texto de interface com zoom de 3× | Legível num celular de cerca de 6" (SC-006). |
| 6 | Gesto ou botão de voltar | Sai da tela cheia, continua na sala e assistindo. Um segundo voltar sai da sala, como hoje (US1-4, FR-004). |
| 7 | Entrar de novo e parar a transmissão no PC | Visualizador fecha; a sala mostra o estado sem transmissão (US1-5, FR-007). |
| 8 | Entrar de novo e desligar o Wi-Fi do PC por cerca de 10 s | Visualizador fica aberto com o aviso; a imagem volta nele (US1-6, FR-008). |
| 9 | Girar 10 vezes seguidas, na sala e em tela cheia | Nenhum corte de som; nunca vira a sala de computador (SC-003, US2-3). |
| 10 | Com a trava de giro ligada | Tela cheia funciona em pé; nada pede para destravar. |
| 11 | Sala deitada sem tela cheia, transmitindo a câmera do celular | Palco grande; Parar câmera e Sair alcançáveis; nada sob a ilha ou o entalhe (US2-2, US2-4). |
| 12 | Renomear a sala fixa (dono) com o teclado aberto | Campo e Salvar visíveis (US2-5). |
| 13 | Com texto grande nas configurações de acessibilidade | Sem rolagem lateral; nomes longos abreviados (US2-6). |

Registrar o resultado de cada aparelho em `relatorio.md`, com modelo, sistema e navegador.
