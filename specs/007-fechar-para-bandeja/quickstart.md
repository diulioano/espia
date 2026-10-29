# Quickstart: validar a bandeja (007)

## Automático

```powershell
npm run check
npm test                       # inclui test/lib/desktop.test.ts (espiaStopLive, reportLive)
cd src-tauri; cargo test; cargo clippy --all-targets; cd ..
npm run tauri build            # anotar o tamanho do instalador (meta: + < 0,3 MB)
```

`cargo test` cobre as funções puras de `tray.rs`: `on_close`, `on_quit`, ler/gravar `tray.json` (ausente, inválido, campos extras) e os itens do menu com e sem ao vivo.

## Antes do roteiro

- Site publicado com o `app_set_live` (`npm run deploy`). Sem isso, o app nunca fica "ao vivo" (o X esconde, mas Sair não pergunta).
- Apagar `%APPDATA%\<identifier>\tray.json` para ver o aviso da primeira vez.
- Duas pessoas: A no app, B no navegador, na mesma sala.

## Roteiro manual B1–B8

| # | Passo | Esperado |
|---|---|---|
| B1 | A abre o app e clica no X (sem sala) | Janela some, sem botão na barra de tarefas. Ícone perto do relógio. Aviso do Windows aparece **uma vez**. |
| B2 | A entra na sala, transmite uma janela com som, clica no X. Esperar 5 min | B continua vendo e ouvindo sem travar (SC-701). Ícone de A em "ao vivo", dica "Espia · ao vivo". Log `[espia] captura nativa:` sem descarte constante. |
| B3 | Repetir B2 com a câmera (006) | Igual. |
| B4 | Clicar no ícone; depois esconder e abrir pelo atalho; depois por um link `espia://s/...` de outra sala | Janela volta em < 1 s no mesmo lugar e estado. O link pergunta se quer sair da transmissão (fluxo da 003). |
| B5 | Menu → "Parar transmissão" com a janela escondida | B vê a transmissão acabar. O item some do menu e o ícone volta ao normal. |
| B6 | Transmitindo, menu → Sair → "Não"; depois Sair → "Sim" | "Não": nada muda. "Sim": app fecha, B vê A sair da sala. Reabrir: janela no mesmo lugar. |
| B7 | Ligar "Fechar no X", fechar e reabrir o app, clicar no X sem transmitir; depois transmitindo | Opção continua marcada. Sem transmitir: fecha. Transmitindo: pergunta. |
| B8 | Com o app escondido, rodar o instalador novo; e, em outro teste, reiniciar o Windows com o app escondido e transmitindo | Instalador fecha o app sozinho. Reinício: nada pergunta, o Windows desliga. |
