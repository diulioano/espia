# Quickstart — validar a 006

## Automático

```powershell
npm run check; npm test   # devices.ts (sugestão de som, voz×jogo, constraints), Worker com source/'device'
```

## Mocks

Abra `/?mock=room&scene=` com cada uma das cenas: `pickercamperm`, `pickercams`, `pickercamnone`, `pickercamdenied`, `pickercammic`, `mobilecam`, `livecam`. Confira:
- a aba **Câmeras** com "Permitir câmeras";
- os cartões com prévia;
- o seletor de entrada de som com "Som de voz / Som de jogo";
- a dica do Discord/fone;
- o celular só com a aba Câmeras;
- o card ao vivo "câmera · …".

## E2E sem hardware (Edge)

Use `--use-fake-device-for-media-stream`, que gera uma câmera **falsa** com um padrão de teste. **Nunca** use `--use-fake-ui-for-media-stream` sem a câmera falsa, nem nada que capture a tela real.
- Transmitir a câmera falsa → a outra aba recebe 1 faixa de vídeo com o padrão e "câmera de <apelido>".
- Com som "jogo" → `echoCancellation: false` nas settings da faixa. Com "voz" → `true`.
- Fechar o seletor → nenhuma faixa de prévia continua viva.

## Manual (hardware real)

| # | Passos | Esperado |
|---|---|---|
| C1 | Site no Chrome: aba Câmeras → Permitir → escolher a webcam → Transmitir | Outra pessoa vê a webcam com "câmera de …". A luz da câmera só acende depois do clique em Permitir. |
| C2 | App 0.1.3, com o site já publicado: mesmo do C1 | O WebView2 pede a permissão uma vez e depois lembra. |
| C3 | Placa com o console ligado: escolher a placa e deixar o som sugerido | O som sugerido é o da placa, como "Som de jogo". Quem assiste ouve o console sem cortes/abafamento. |
| C4 | Webcam + microfone ("Som de voz"), duas pessoas transmitindo, uma pelas caixas | Ninguém ouve o próprio eco de forma perceptível (SC-605). |
| C5 | Celular: Transmitir → câmera traseira | O PC vê a imagem em 720p30. Apagar a tela do celular encerra a transmissão. |
| C6 | Câmera em uso no OBS/Discord | O cartão mostra "Em uso por outro programa". |
| C7 | Tirar o cabo da câmera no meio | A transmissão acaba sozinha. |
