# Relatório — 011 Salas fixas e chamada que sobrevive a quedas

## Causa da queda de 2026-10-29 (research R1)

Uma publicação reinicia o Durable Object e corta os WebSockets **sem rodar `webSocketClose`**. Quem estava na sala só existia no attachment do socket, então a sala reiniciava "vazia": `resume` → `resume-failed`, `join` → `room-not-found`. Provado no teste "publicação: todo mundo volta com a mesma identidade" (`evictDurableObject(stub, { webSockets: 'close' })`): **falha sem o `roster` e passa com ele**.

## Entregas

| Parte | Commit | Publicada |
|---|---|---|
| MVP US1 — queda não é sair | `c84ad06` | 2026-10-30, versão `ed41d3d1` |
| US2 — sala fixa | `5087b51` | — |
| US3 — Suas salas (máx. 5, pedido de 30/10) | `d1d1a1c` | — |
| US4 — dono renomeia/apaga + acabamento | este commit | — |

## Testes automáticos

`npm test`: 174 testes (Worker no runtime real + funções puras). Destaques:
- publicação com 3 pessoas → todas voltam com a mesma identidade, ninguém vê saída/entrada;
- sala rápida segurada 5 min (`HOLD_MS`) quando todos caem; some na hora quando todos saem;
- sala fixa existe vazia, expira 30 dias depois do último `join` (prévia e status não renovam), lápide sem nome (410), código não reaproveitado;
- `status` nunca devolve apelidos; até 5 códigos;
- renomear/apagar sem chave → 403; apagar com gente dentro → `room-deleted` + 4410, TURN revogado, **sobra só a lápide**.

## Ponta a ponta (Worker local + Edge headless, 2–3 pessoas, sem captura de tela real)

Scripts no scratchpad da sessão (queda.mjs, fixa.mjs, suas.mjs, dono.mjs, dono-home.mjs). Todos os passos OK:

- **Queda**: recarregar o `wrangler dev` (reinício igual a uma publicação) → "Reconectando…" nas duas telas e "De volta!" em ~2 s, 2 de 10 mantidos. Worker parado > 60 s → "Não deu pra voltar pra sala".
- **Sala fixa**: criar pela tela (nome normalizado), convite copiado, chave só em quem criou, entrada por link com selo, vazia continua existindo, link inexistente → N4.
- **Suas salas**: aparece com "Sua", "Vazia · usada hoje", "1 pessoa agora" ao vivo, status sem apelidos, quem entrou pelo link tem a sala sem "Sua", "Entrar" direto, "Tirar da lista" só deste aparelho.
- **Dono**: sem "⋯" pra quem não é dono, renomear muda na tela dos outros sem recarregar (link igual), aviso "Tem 1 pessoa aqui agora…", apagar → dono volta pro início, quem estava dentro vê N2 com o nome e "Tirar de Suas salas", link → N1. Renomear/apagar pelo "⋯" de Suas salas.

## Problemas achados nos testes e corrigidos

1. Sala fixa criada e abandonada antes de conectar não entrava em Suas salas → agora é guardada já na criação.
2. Dono apagando: o fechamento 4410 podia chegar antes da resposta do servidor e a inicial mostrava a sala ainda na lista → a lista é limpa antes de sair.
3. Apagar avisava todo mundo e revogava o TURN (~0,5 s) **antes** de gravar a lápide: quem abrisse o link nessa janela via a sala viva; e o fechamento recriava fantasmas (com apelidos) e a espera de sala rápida → lápide primeiro, sockets marcados fora da sala antes de fechar. Teste garante que só sobra a lápide.

## Pendências

- Publicar (`npm run deploy`) e conferir em produção: publicar de novo com a galera numa sala (todos voltam sozinhos), e sala fixa pelo app 0.1.7 (lista separada da do navegador).
- Limites por IP (`RL_CREATE` 10/min, `RL_LOOKUP` 20/min) atrapalham testes locais que criam/consultam muito; em uso normal sobram.
