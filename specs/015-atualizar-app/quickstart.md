# Quickstart — validar a 015

## Uma vez só (usuário): a chave de assinatura

```
! npx tauri signer generate -w %USERPROFILE%\.tauri\espia.key
```

- A chave pública impressa vai em `src-tauri/tauri.conf.json` → `plugins.updater.pubkey`.
- **Faça uma cópia** de `espia.key` (pendrive ou nuvem pessoal). Sem ela, o app nunca mais se atualiza sozinho (research R3).
- A chave privada nunca entra no repositório.

## Automático

```bash
npm run check && npm test && npm run build
cd src-tauri && cargo test
```

- `update::decide`: todas as linhas da tabela do contrato. Ao vivo nunca instala, abertura passada de 10 s vira aviso, bandeja < 30 min não instala, `required` ignora "Depois", versão igual ou menor não faz nada.
- Leitura do `minimum` (ausente, inválido, maior, menor) e do `update-resume.json` (velho, inválido, sala inválida).
- Página: `app-update.svelte.ts` (fases → o que o aviso mostra, `canLater`, confirmação ao vivo).
- `scripts/copy-installer.mjs`: gera `latest.json` com a URL por versão e a assinatura do `.sig`, e apaga instaladores antigos.

## Mocks

| URL | Deve mostrar |
|---|---|
| `?mock=home&scene=update` | aviso "Tem versão nova do app" com "Atualizar agora" e "Depois" |
| `?mock=room&scene=update` | o mesmo aviso no canto, sem cobrir as telas |
| `?mock=room&scene=update-required` | "Essa versão do app não funciona mais…", sem "Depois" |
| `?mock=room&scene=update-confirm-live` | confirmação "Sua transmissão vai parar por alguns segundos" |
| `?mock=room&scene=update-installing` | "Atualizando, o Espia volta em instantes" |
| `?mock=home&scene=update-failed` | "Não deu pra atualizar" com o link "Baixar pelo site" |

## Ponta a ponta local (app de teste instalado, site local)

1. `npm run dev:worker`, e build do app **0.1.9 de teste** com `tauri.test.conf.json` (aceita http) e `ESPIA_UPDATE_URL=http://127.0.0.1:8787/download/latest.json`. Instalar.
2. Gerar um **0.1.10** de teste assinado e colocar o `latest.json` dele em `public/download` do site local.
3. **Abertura (R6)**: abrir o app 0.1.9. Em até ~15 s, ele reabre como 0.1.10 (versão nos detalhes do `Espia.exe`), sem clique e sem janela do instalador.
4. **Aviso (US1)**: com o 0.1.9 aberto numa sala, publicar o 0.1.10 local e forçar a checagem (debug: `ESPIA_UPDATE_EVERY=20s`). O aviso aparece. "Atualizar agora" → reabre **na mesma sala**, e o outro participante não vê "saiu/entrou".
5. **Ao vivo**: transmitindo a câmera falsa, "Atualizar agora" pede a confirmação. "Cancelar" não faz nada.
6. **Depois (US2)**: "Depois" → o aviso some e não volta. Sair pela bandeja e abrir: abre como 0.1.10.
7. **Bandeja (US4)**: janela no X, fora de sala, com `ESPIA_HIDDEN_UPDATE_AFTER=60s` (debug). O app se atualiza e continua só na bandeja: nenhuma janela abre. Clicar no ícone abre a versão nova.
8. **Em sala escondido**: no X, mas numa sala → não se atualiza.
9. **Assinatura (SC-005)**: trocar 1 byte do instalador servido → o app recusa, mostra `failed` (se havia aviso) e continua no 0.1.9.
10. **Mínimo (US3)**: `minimum` = 0.1.10 → aviso sem "Depois". Entrar numa sala e transmitir continua funcionando.

## Manual (depois de publicar)

- **M1**: o grupo baixa o 0.1.9 pelo site uma última vez (quem tem 0.1.8 não se atualiza sozinho, research R11).
- **M2**: no lançamento seguinte (0.1.10 de verdade), conferir em uma semana que todos estão nele (SC-002).
- **M3**: SmartScreen ou antivírus não barram a atualização feita pelo app (research R3).
