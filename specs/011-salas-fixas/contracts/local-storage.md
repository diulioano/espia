# Contrato — Armazenamento no aparelho

Origem do site (`espia.espia.workers.dev`). O app desktop tem o próprio armazenamento (perfil do WebView2), então as listas são separadas sem nenhuma API nativa nova.

| Chave (`localStorage`) | Formato | Notas |
|---|---|---|
| `espia:rooms` | JSON `SavedRoom[]` (`{ code, name, lastUsed }`), máx. 5 | Valor inválido ou corrompido → lista vazia. Itens com código inválido são descartados |
| `espia:owner:<code>` | `ownerKey` hex (64) | Nunca sai do aparelho além do `Authorization` de PATCH/DELETE. Apagada quando o servidor responde `410`/`404` pra essa sala |

Sem armazenamento (aba privada bloqueada etc.): Suas salas fica vazia e criar sala fixa funciona, mas o aparelho não lembra a posse (aviso nenhum; a sala ainda expira sozinha).
