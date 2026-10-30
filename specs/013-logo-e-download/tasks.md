# Tasks: Logo leva pro início e botão de baixar o app

- [X] T1301 [US2] `src/lib/app-invite.ts`: `appDownloadVisible(env: Omit<InviteEnv, 'dismissed'>)` = mesma regra do convite, sem o "dispensado". O `appInviteVisible` passa a usá-la. Testes em `test/lib/app-invite.test.ts`: Windows sim; app, celular, Mac e Linux não; dispensado continua sim.
- [X] T1302 [US1] `src/ui/components/Logo.svelte`: `<a href="/" aria-label="Espia — início">` com prop `onhome?: () => void`. O clique sem modificador chama `preventDefault` + `onhome`; Ctrl/Meta/botão do meio abrem normal. Mesma aparência de hoje, foco visível.
- [X] T1303 [US1] `src/lib/types.ts`: `goHome()` em `JoinActions`, `NotFoundActions` e `RoomActions`; `HomeView.download?: boolean`.
- [X] T1304 [US1] `src/lib/stores/room.svelte.ts`: ação `goHome`. Se `myBroadcast` e `!confirm('Você está transmitindo. Parar e sair da sala?')` → nada; senão, o mesmo que `leave`.
- [X] T1305 [US1] Telas: `HomePage` (logo → `scrollTo(0,0)`), `JoinPage`, `NotFoundPage` e `RoomPage` passam `onhome` = `actions.goHome`.
- [X] T1306 [US1] Rotas e mocks: `RoomRoute` (`joinActions.goHome` e `notFoundActions.goHome` → `router.go('/')`); `MockJoin`, `MockNotFound` e `MockRoom` (`goHome` → `onnavigate('home','default')`).
- [X] T1307 [US2] `HomePage`: botão "Baixar o app" (ícone `Download`, `btn-secondary` pequeno) à direita do logo no topo, quando `view.download`; link `DOWNLOAD_URL` com `download`. `HomeRoute`: `download: appDownloadVisible(readInviteEnv())`. `mocks/home.ts`: `download: true`, menos nas cenas `mobile`.
- [X] T1308 `npm run check`, `npm test`, `npm run build`; prints de `home` default/invite/mobile; commit.
