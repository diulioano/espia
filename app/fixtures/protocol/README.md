# Fixtures do protocolo

Cada JSON desta pasta é uma mensagem de exemplo do protocolo de sinalização. Os dois lados leem os mesmos arquivos, para o app nativo não se afastar do site:

- o site, no teste Vitest `test/shared/protocol-fixtures.test.ts` (confere contra `src/worker/validate.ts`);
- o app, em `cargo test` na crate `espia-core`.

## Nome dos arquivos

- `client-<t>.json`: mensagem que o cliente manda.
- `server-<t>.json`: mensagem que o servidor manda.

`<t>` é o valor do campo `t` da mensagem.

## Regras

- Mudar o protocolo exige mudar os dois lados e as fixtures juntos; os dois testes têm de continuar verdes.
- Nesta feature (019) o Worker e `src/shared/protocol.ts` não mudam. O app só espelha o protocolo que já existe.
