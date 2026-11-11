# Relatório da feature 019

## US1 — Convite com o endereço oficial (T043, concluída)

Publicação: `npm run deploy` em 2026-11-11, versão `fb38d279-ab88-4a59-a5f9-45533a1c16e6`.

| Cenário | Resultado | Evidência |
|---|---|---|
| 1. Sala aberta pelo endereço oficial: convite copiado e mostrado começam com `espia.diuliano.com` | Passou | Página aberta em `https://espia.diuliano.com/s/cfr-dv2`. Convite mostrado: `espia.diuliano.com/s/cfr-dv2`. Copiado: `https://espia.diuliano.com/s/cfr-dv2`. Observação: no endereço oficial, o código anterior também produziria este resultado; não prova a mudança sozinho. |
| 2. Sala aberta pelo endereço antigo | Passou | Após o deploy (versão `fb38d279-ab88-4a59-a5f9-45533a1c16e6`), aberta em `espia.espia.workers.dev/s/cfr-dv2`, o convite mostrado é `espia.diuliano.com/s/cfr-dv2` e o copiado é `https://espia.diuliano.com/s/cfr-dv2`. Antes do deploy, o mesmo teste mostrava `espia.espia.workers.dev/s/cfr-dv2` (linha de base). |
| 3. App já instalado (0.1.10) | Passou | Janela do app desktop mostra `espia.diuliano.com/s/cfr-dv2`; copiado: `https://espia.diuliano.com/s/cfr-dv2`. Versão instalada confirmada pela pessoa. |
| 4. Link antigo de sala abre a mesma sala | Passou | Link antigo abriu a mesma sala, como antes. |
| 5. Criar sala fixa copia sozinho com o endereço oficial | Passou | Sala fixa `hg2-pc4` criada; a cópia automática ao colar trouxe `https://espia.diuliano.com/s/hg2-pc4`. |

Prova SC-001 e SC-002: os cinco cenários passaram após o deploy. História 1 concluída.
