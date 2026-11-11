# Contrato de UI: visualizador em tela cheia e sala de celular

Contrato entre a lógica (`src/lib`) e as telas (`src/ui`), seguindo a constituição IV. Os componentes recebem estado pronto e emitem eventos.

## `Viewer.svelte` (novo, `src/ui/components/`)

```ts
props: {
  view: StreamView;                 // a transmissão aberta (viewerStream(roomView))
  link: RoomView['link'];           // 'reconnecting' mostra o ReconnectPill compacto
  volumeAdjustable: boolean;        // canSetVolume(); false = só mudo
  initial?: { controls: boolean; zoom: ZoomState };   // só mocks (RoomView.mockViewer)
  onclose: () => void;              // botão sair, voltar, saída da tela cheia do sistema
  ontogglemute: () => void;
  onvolume: (v: number) => void;
  onpip: () => void;                // abre o PiP; a página fecha o visualizador
}
```

**Comportamento obrigatório**:

1. **Camada**: `position: fixed; inset: 0; z-index` acima de toda a sala, com fundo `--color-stage`. A imagem fica com `object-fit: contain` e aparece inteira, com faixas quando a proporção não bate (FR-002).
2. **Tela cheia do sistema**: a página pede a tela cheia no mesmo toque de abrir, chamando `requestFullscreen` na camada. Se não der, fica no modo `page`, sem erro visível (FR-006).
3. **Controles**: um toque na imagem mostra os controles (mudo, volume se ajustável, PiP se houver, sair). Eles somem 3 s depois do último toque. Os botões têm pelo menos 48 px e ficam dentro das áreas seguras (FR-003, FR-013, FR-014).
4. **Saída**: `Esc`, o botão de sair, o voltar (R3) e o `fullscreenchange` de saída chamam `onclose` uma vez só (FR-004).
5. **Zoom**: segue o [data-model.md](../data-model.md). `touch-action: none` vale só na camada. Qualquer `resize` volta à imagem inteira (FR-009).
6. **Vídeo e som**: o `<video>` é mudo, com `playsinline`, e recebe o mesmo `view.stream`. O som continua no `<audio>` do `StreamTile` da sala, que fica montado por baixo, para abrir e fechar não reiniciar o som (FR-005).
   - Um `MediaStream` pode alimentar dois `<video>` sem nova recepção.
   - Enquanto o visualizador está aberto, o `StreamTile` da sala esconde o próprio `<video>`, para não decodificar à toa.
7. **Rótulos**:
   - sair: `aria-label="Sair da tela cheia"`;
   - mudo: "Sem som" / "Com som";
   - região: `aria-label="Transmissão de {nome} em tela cheia"`.

## Mudanças em componentes existentes

**`StreamTile.svelte`**:
- Nova prop `onfullscreen?: () => void`. Quando presente, o botão "Tela cheia" chama a prop em vez de `container.requestFullscreen()`. O computador não passa a prop e fica como hoje.
- Nova prop `videoHidden?: boolean`, que não monta o `<video>` enquanto o visualizador está aberto. O `<audio>` continua montado.
- Nova prop `volumeAdjustable?: boolean` (padrão `true`).
- `@media (pointer: coarse)`: `.ctl` com 44 px e controles que somem em 3 s.

**`MobileRoomPage.svelte`**:
- Passa `onfullscreen={() => openViewer(selected.id)}` ao tile do palco. `openViewer` chama `actions.fullscreen(id)` e pede a tela cheia do sistema no mesmo gesto.
- Renderiza `<Viewer>` quando `viewerStream(view)` existe.
- Ganha o layout `@media (orientation: landscape)` (research R6) e as margens de área segura (R7).

**`RoomActions`** ganha `fullscreen(id: string | null): void`, e **`RoomView`** ganha `fullscreenId` e `mockViewer?` (ver [data-model.md](../data-model.md)).

## Mocks (`src/mocks/mobile.ts`, tela `mobile`)

| Cena | Código | Estado |
|------|--------|--------|
| `viewer` | M·F1 | Tela cheia em pé, controles escondidos |
| `viewer-controls` | M·F2 | Tela cheia com controles visíveis |
| `viewer-zoom` | M·F3 | Tela cheia aproximada 2,5× |
| `viewer-reconnecting` | M·F4 | Tela cheia com a ligação voltando |
| `viewer-nosound` | M·F5 | Controles sem volume (iPhone) |
| `landscape` | M·L1 | Sala deitada, assistindo, 2 ao vivo |
| `viewer-landscape` | M·L2 | Tela cheia deitada |
| `landscape-camera` | M·L3 | Sala deitada transmitindo a câmera (Parar câmera à mão) |

As cenas `landscape*` usam `device: 'mobile-landscape'`. O `MockApp` desenha o quadro em 844×390 para esse `device`, e em 390×844 para `mobile`.
