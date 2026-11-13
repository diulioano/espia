# Contrato: atualização do app nativo (R14, R15)

Substitui `specs/015-atualizar-app/contracts/update.md`, que passa a ser referência histórica (T104). Vale só para o app nativo. O app de antes fica congelado e não ganha atualização.

O app nativo não recebe as mudanças do site na hora, porque não carrega o site. Cada mudança de tela é uma versão nova do app. Por isso avisar e atualizar faz parte da feature (FR-022).

## Arquivo publicado: `GET /download/latest.json`

Gerado e publicado por `scripts/release-app.mjs` (T094), junto do instalador, em `public/download/`.

```json
{
  "version": "0.2.0",
  "notes": "",
  "pub_date": "2026-11-20T12:00:00Z",
  "minimum": "0.2.0",
  "url": "https://espia.diuliano.com/download/Espia-setup.exe",
  "signature": "<conteúdo da assinatura minisign do instalador>"
}
```

(Exemplo. A data e o nome do arquivo do instalador são ilustrativos.)

| Campo | Tipo | Regra |
|---|---|---|
| `version` | texto | Versão mais nova publicada. Primeira do app nativo: `0.2.0` (R15). |
| `notes` | texto | Notas da versão. Pode ser vazio. |
| `pub_date` | texto | Data e hora ISO 8601 em UTC. |
| `minimum` | texto | Menor versão que ainda serve. Faltando ou inválido: "sem mínimo" (regra herdada da 015). |
| `url` | texto | Endereço do instalador, no endereço oficial. |
| `signature` | texto | Assinatura minisign do arquivo em `url`. Obrigatória: sem ela o app não instala. |

Mudança em relação à 015: o formato é plano (`url` e `signature` ficam no nível de cima, sem `platforms.windows-x86_64`), porque o app nativo só existe para Windows x64. São seis campos, os da tabela acima.

Cache: sem cache longo, como na 015. O `public/_headers` ganha `Cache-Control: no-cache` para `/download/latest.json` (T094).

## Assinatura

| Item | Onde fica |
|---|---|
| Algoritmo | minisign |
| Conferência no app | crate `minisign-verify`, antes de instalar |
| Chave pública | embutida no executável (não é segredo) |
| Chave privada | `%USERPROFILE%\.espia\update.key`, fora do repositório, gerada e guardada pela pessoa (T092) |

A chave privada nunca entra no repositório nem no executável (constituição VI, R14). Instalador com assinatura que não confere: o app não instala e trata como falha. A origem também é verificada: o `url` tem que ser do endereço oficial (constituição VI: "Uma atualização do app MUST ter a origem verificada antes de ser instalada"). Regra: o `url` precisa começar com `https://espia.diuliano.com/download/`; outro endereço é recusado (T095).

## Onde cada arquivo é servido

| Caminho | O que é |
|---|---|
| `/download/latest.json` | Descrição da versão mais nova do app nativo |
| `/download/Espia-setup.exe` | Instalador atual. Hoje é o do app de antes (0.1.10, montado em 2026-11-05). Na troca, passa a ser o nativo (T109) |
| `/download/Espia-setup-0.1.10.exe` | App de antes, guardado ao lado depois da troca (T099, T109) |

`src/lib/app-invite.ts` continua apontando para `Espia-setup.exe` (T099). Todos os arquivos ficam em `public/download/` e saem com `npm run deploy`.

## Fluxo

1. **Abrir.** O app consulta `https://espia.diuliano.com/download/latest.json`. O app consulta na abertura e a cada 6 horas enquanto está aberto.
2. **Comparar.** Versão publicada maior que a instalada: há versão nova. Igual ou menor: nada a mostrar.
3. **Avisar.** Aparece "Tem versão nova" com "Atualizar agora" e "Depois". Abaixo de `minimum`, o aviso não tem "Depois".
4. **Baixar.** "Atualizar agora" baixa o instalador de `url`.
5. **Conferir.** Confere a assinatura com a chave pública embutida. Não confere: não instala, mostra a falha.
6. **Instalar.** Grava `resume.json` (`room` = sala atual, `hidden` = se estava escondido na bandeja, `at` = momento da gravação), abre o instalador, o app fecha, o instalador roda por cima (mesmo identificador `dev.espia.app`, instalação por usuário).
7. **Voltar.** O app reabre, lê `resume.json` e volta para a mesma sala, com o mesmo apelido (que está em `device.json`). O `resume.json` vale 5 minutos e é apagado ao ser lido, como na 015.

Estados do aviso (também em `data-model.md`): `nenhuma` → `disponível` → `baixando` → `pronta` → `instalando`. "Depois" devolve a `nenhuma` e fecha o aviso até a próxima abertura do app; na abertura seguinte o aviso volta. Não há memória de versão pulada. Falha em `baixando` ou `pronta` volta a `disponível`.

## Regras

| Regra | Efeito |
|---|---|
| Nunca durante transmissão própria | Nada é baixado nem instalado enquanto a pessoa transmite (R14, FR-022). A transmissão não é interrompida. A atualização espera a pessoa decidir (US7, cenário 5). Durante a transmissão o aviso aparece só com o texto, sem botão; os botões aparecem quando ela termina (T098) |
| Abaixo de `minimum` | O aviso não tem "Depois". Só some atualizando. O app continua usável para o que já funciona (regra herdada da 015, Q1) |
| Fechamento 4426 do servidor | Mostra "Atualize o Espia" com o mesmo botão de atualizar (T097). O servidor fecha com 4426 quem não manda `mediaProtocol: 2` |
| Voltar à sala | Pelo `resume.json` (T098) |
| Instalação falha | O app continua na versão instalada; o aviso volta |

A atualização na abertura sem avisar e a atualização escondida na bandeja da 015 (US2 e US4) não fazem parte do app nativo. Ficam fora até uma decisão nova.

## Textos (português informal)

| Onde | Texto |
|---|---|
| Aviso | "Tem versão nova" |
| Botão principal | "Atualizar agora" |
| Botão secundário (some abaixo de `minimum`) | "Depois" |
| Fechamento 4426 | "Atualize o Espia" (com o botão de atualizar) |
| Abaixo de `minimum` | "Essa versão do app não funciona mais com o Espia. Atualize pra continuar" |
| Instalando | "Atualizando o Espia…" |
| Falha | "Não deu pra atualizar" |

O site tem hoje a tela `UpdateRequired` (`src/ui/components/UpdateRequired.svelte`: "Atualize o Espia para continuar", "Esta sala usa uma versão nova da transmissão.", botão "Atualizar"). A tela do app nativo mantém o espírito dela, com o título "Atualize o Espia".

As cenas de dados falsos novas desta parte (`home`/`update-available`, com "Depois", e `home`/`update-minimum`, sem "Depois") estão em `contracts/dados-falsos.md`.

## Diferenças para a spec 015

- Formato de `latest.json` plano: `version`, `notes`, `pub_date`, `minimum`, `url`, `signature`. A 015 usava `platforms.windows-x86_64` do plugin do Tauri.
- Assinatura minisign conferida com `minisign-verify` no próprio app, em vez do plugin de atualização do Tauri.
- Não há ponte com página: sem `window.espiaUpdate`, sem comando `app_update`, sem permissão `allow-app-update`. O aviso é tela do app.
- O arquivo de volta é `resume.json` (era `update-resume.json`), com os mesmos campos `room`, `hidden`, `at`. O campo `room` guarda o código `xxx-xxx`, não o caminho `/s/<código>`.
- Sem atualização automática na abertura e sem atualização escondida na bandeja (US2 e US4 da 015): não fazem parte do app nativo.
- Estados do aviso: `nenhuma`, `disponível`, `baixando`, `pronta`, `instalando` (a 015 tinha `none`, `available`, `downloading`, `installing`, `failed`).
- Continua igual: `minimum` sem "Depois", nunca atualizar durante a transmissão própria, voltar para a sala depois.
