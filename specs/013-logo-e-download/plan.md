# Implementation Plan: Logo leva pro início e botão de baixar o app

**Branch**: `013-logo-e-download` (direto na `main`) | **Date**: 2026-10-30 | **Spec**: [spec.md](./spec.md)

## Summary

1. **Logo** (`Logo.svelte`) vira um link para `/` com nome acessível "Espia — início". O clique é **um evento** (`onhome`), e cada tela decide o que fazer (princípio IV):
   - início: volta ao topo;
   - entrada, não encontrada, cheia: `router.go('/')`;
   - sala: a ação `goHome` do store. Ela confirma com `confirm("Você está transmitindo. Parar e sair da sala?")` se você estiver transmitindo, igual ao `leaveGuard` do app (desktop.ts), e depois faz o mesmo que "Sair da sala".
2. **Botão "Baixar o app"** no topo da tela inicial: `HomeView.download: boolean`, calculado com a mesma regra do convite (`appInviteVisible`), mas **ignorando o "Agora não"**. É um link para `DOWNLOAD_URL` com `download`, o mesmo instalador do convite.

## Technical Context

TypeScript 6 + Svelte 5. Só telas e rotas, **sem servidor e sem app**. Testes: Vitest para a regra do botão (`appDownloadVisible`). Mocks: `home` default (com botão) e `invite` (botão + convite). A cena sem botão é a `mobile` (celular).

## Constitution Check

| Princípio | Situação |
|---|---|
| I. Simplicidade | ✅ Um clique pra voltar; baixar continua opcional. |
| II–III, VI | ✅ Nada muda. |
| IV. Camada visual separada | ✅ O logo emite `onhome`; quem decide são as rotas e o store. |
| V. Mocks | ✅ Cenas existentes mostram o botão; o clique no logo funciona nos mocks. |
| App desktop | ✅ Sem API nova; o botão não aparece no app. |

## Source Code

```text
src/lib/app-invite.ts            # + appDownloadVisible(env) (sem "dispensado")
src/lib/types.ts                 # HomeView.download?, JoinActions/NotFoundActions/RoomActions.goHome
src/lib/stores/room.svelte.ts    # goHome: confirma se transmitindo, depois leave
src/ui/components/Logo.svelte    # <a href="/"> + onhome
src/ui/pages/HomePage, JoinPage, NotFoundPage, RoomPage, MobileRoomPage(sem logo: nada)
src/routes/HomeRoute, RoomRoute  # goHome / download
src/mocks/*                      # ações goHome; home com download
test/lib/app-invite.test.ts      # + appDownloadVisible
```
