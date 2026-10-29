# Quickstart: validar a 008

## Automático

```powershell
npm run check
npm test                   # inclui test/lib/app-invite.test.ts
npm run tauri build        # instalador com a arte; anotar o tamanho (meta: + < 150 KB)
```

## Mocks (Edge, `npm run dev`)

- `?mock=home&scene=default`: sem convite (como no Mac, no celular, no app ou depois do "Agora não").
- `?mock=home&scene=invite`: convite com "Baixar o app" e "Agora não". Em 1366×768, sem rolagem (SC-804).
- `?mock=home&scene=invitehint`: dica do SmartScreen e o link de reserva.

## Ícone (US4)

- `curl -sI https://espia.espia.workers.dev/favicon.ico` → `content-type: image/x-icon` (não `text/html`).
- A aba do Edge e do Chrome mostra o castor, no tema claro e no escuro.

## Download (contrato)

- `curl -sIL https://github.com/DiulianoVargas/espia/releases/latest/download/Espia-setup.exe | grep -i "^HTTP"` termina em `200`.

## Roteiro manual I1–I6

| # | Passo | Esperado |
|---|---|---|
| I1 | Tela inicial no Chrome/Edge do Windows | Convite abaixo de "Tem um link?". "Criar sala" continua o destaque. |
| I2 | "Baixar o app" | O download do `Espia-setup.exe` começa na hora, e aparece a dica do SmartScreen. |
| I3 | "Agora não" e recarregar | O convite não volta. |
| I4 | Tela inicial no celular e dentro do app | Sem convite. |
| I5 | Rodar o instalador novo num PC sem o app | Boas-vindas "Bora instalar o Espia" com o castor; faixa do topo nas páginas do meio; fim "Pronto!" com "Abrir o Espia" marcado, que abre o app. |
| I6 | Rodar de novo com o app aberto (escondido na bandeja) | Avisa em tom informal que vai fechar o Espia, atualiza e reabre. |
