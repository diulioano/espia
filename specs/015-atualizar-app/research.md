# Research — 015 Atualizar o app sozinho

## R1 · Mecanismo: plugin oficial de atualização do Tauri

**Decisão**: `tauri-plugin-updater` (v2), usado **só pelo Rust**. A página não ganha nenhuma permissão do plugin.

**Por quê**:
- Ele baixa, confere a assinatura (`download()` e `install()` chamam `verify`) e roda o instalador NSIS com `/UPDATE`.
- No Windows, o app sai sozinho (`std::process::exit(0)`) e o instalador reabre o app: `restart_after_install` vale `true` por padrão, e os modos `passive` e `quiet` passam `/R`.
- Lê um JSON estático. O Worker já serve `/download/*` (008), então não precisa de servidor novo, GitHub nem serviço pago (FR-013).
- `Update.raw_json` guarda os campos que o plugin não conhece. É por ali que vem a versão mínima (R5).

**Alternativas**:
- Atualizador próprio (baixar o `.exe` e rodar): reinventa a conferência da assinatura e o "sair e reabrir". Rejeitado.
- Abrir o site no navegador para a pessoa baixar: é o que existe hoje, e ninguém faz. Rejeitado.

## R2 · Modo de instalação: `quiet`

**Decisão**: `plugins.updater.windows.installMode = "quiet"` (`/S /R`).

**Por quê**:
- A instalação é só do usuário (`installMode: currentUser`), então o modo silencioso não precisa de administrador.
- Nenhum assistente aparece, nem "Avançar" (US1-3), e a atualização na bandeja fica invisível (US4).
- O progresso que a pessoa vê é o aviso do próprio Espia: "Baixando…" e depois "Instalando, o Espia volta em instantes".

**Alternativa**: `passive` mostra uma barrinha do instalador, que apareceria até na atualização silenciosa da bandeja. Rejeitado.

## R3 · Prova de origem: chave própria (minisign do Tauri)

**Decisão**:
- Par de chaves gerado uma vez com `npx tauri signer generate -w %USERPROFILE%\.tauri\espia.key`, rodado pelo usuário.
- A chave pública vai em `tauri.conf.json` (`plugins.updater.pubkey`). A privada fica só no PC de quem publica, fora do repositório.
- `bundle.createUpdaterArtifacts: true` gera o `.sig` ao lado do instalador.
- `npm run desktop:release` passa `TAURI_SIGNING_PRIVATE_KEY` (e a senha, se houver) para o build.

**Por quê**: é o que o plugin confere. Não depende de certificado pago do Windows.

**Risco**: perder a chave privada impede atualizar. Os apps instalados só aceitam a chave antiga, e cada um precisaria reinstalar pelo site. → Fazer uma cópia de segurança da chave (quickstart). Vazar a chave permite empurrar uma "atualização" falsa, mas só se alguém também conseguir publicar no site.

**SmartScreen**: o arquivo baixado pelo app não ganha a marca "veio da internet" (é o Rust que grava). Então o SmartScreen não deve aparecer na atualização. Confirmar no teste manual (quickstart M3).

## R4 · Onde fica o JSON

**Decisão**: `https://espia.espia.workers.dev/download/latest.json`, com o instalador **por versão** (`/download/Espia_<versão>_x64-setup.exe`) e o nome fixo de sempre (`/download/Espia-setup.exe`, botão da 008).

**Por quê**:
- O arquivo por versão evita que um JSON novo aponte para um `.exe` antigo guardado em cache, o que quebraria a assinatura.
- O script de lançamento apaga os `.exe` por versão antigos de `public/download` (pasta fora do git).

**Cache**: o plugin pede o JSON a cada checagem. Na T inicial, conferir os cabeçalhos que o Workers Assets manda para `.json`. Se vier cache longo, o Worker responde `/download/latest.json` com `Cache-Control: no-cache`.

## R5 · Versão mínima

**Decisão**: campo extra `minimum` no `latest.json`, lido de `raw_json`. O valor vem de `package.json` → `"espia": { "appMinimum": "0.1.9" }`, e o script de lançamento copia para o JSON. `required = versão instalada < minimum`. Isso fica no Rust: a página só recebe `required: true/false`.

**Por quê**: o "site" que diz a versão mínima (FR-010) é o próprio arquivo publicado junto com ele, e a página não precisa saber versões.

**Quando subir o mínimo**: só quando o site novo deixar de funcionar com apps antigos (como no caso da 0.1.7). Para mudar só o mínimo, é preciso relançar o JSON, ou seja, rodar `desktop:release` de novo. Mudança rara, aceita.

## R6 · Abrir o app = atualizar na hora (o "Depois" e o "reiniciar")

**Decisão**: em **toda abertura nova do processo** (atalho, link `espia://`, ligar o PC), o app procura versão nova. Se acha em até 10 s, a pessoa ainda não está numa sala e nem ao vivo, ele instala na hora: "Atualizando o Espia…" na tela de carregamento (ou no aviso, se o site já abriu) e reabre já atualizado. Passou de 10 s ou já entrou numa sala: vira o aviso normal (US1).

**Por quê**:
- Atende o "atualizar quando reinicializar" sem guardar nada no disco: o "Depois" só esconde o aviso, e a próxima abertura instala de qualquer jeito (US2-2).
- Também atualiza quem nunca viu o aviso.
- Acabou de abrir, então ninguém perde nada. São uns 5 s a mais (SC-006 ≤ 15 s).

**Muda na spec**: no US1-1, o aviso aparece com o app **já aberto** quando a versão sai, ou se a checagem da abertura demorar. Ajustado na spec.

**Alternativa**: marca "pendente" em arquivo e instalar só depois de um "Depois". Isso deixaria sem atualização quem nunca viu o aviso. Rejeitado.

## R7 · Voltar para onde estava

**Decisão**:
- Antes de instalar, o Rust grava `app_config_dir/update-resume.json` com `{ room: "/s/xxx-xxx" | null, hidden: bool, at: ms }`.
- A sala vem da URL atual da janela (`window.url()`), validada por `links::room_path`.
- Na abertura, se o arquivo existe e tem menos de 5 min, ele é lido e apagado:
  - `room` vira o endereço inicial (`index.html?to=…`, igual a um link);
  - com `hidden`, o app não cria janela: fica só na bandeja, como depois da 009.
- Os argumentos que o instalador repassa (`/ARGS`) continuam valendo (um link `espia://` da abertura).

**Por quê**:
- O instalador reabre o app com os mesmos argumentos do processo velho. Não dá para passar argumentos novos (`current_exe_args` é interno do plugin), então o arquivo é o caminho.
- Ao abrir `/s/<código>` com uma sessão salva, a sala já entra sozinha com o apelido salvo e retoma o lugar (`RoomRoute`: `directNickname`). A queda dura poucos segundos, dentro dos 60 s de graça, então ninguém vê "saiu/entrou".

## R8 · Aviso na página: um comando novo e um gancho

**Decisão**:
- **App → página**: `window.espiaUpdate(state)` por `eval`, como o `espiaStopLive` da 007. O app reenvia o estado a cada carregamento da página (`PageLoadEvent::Finished`), assim recarregar ou recriar o motor não perde o aviso.
- **Página → app**: um comando, `app_update(now: boolean)`.
  - `true` = instalar agora;
  - `false` = "Depois" (para de avisar nessa execução; ignorado se `required`).
- A confirmação "Sua transmissão vai parar por alguns segundos" fica na página, que sabe se está ao vivo.
- Contrato completo em [contracts/update.md](contracts/update.md).

**Por quê**: a spec pede o aviso com o visual do Espia e com mock (FR-016). Uma caixa nativa do Windows não teria nenhum dos dois. O app só decide; a página só mostra (princípio IV).

**Constituição**: comando novo para a página = emenda **2.1.0 (MINOR)**: o app pode se atualizar a partir do site, com um comando `app_update(now)` e assinatura própria.

## R9 · Atualização silenciosa na bandeja (US4)

**Decisão**:
- `EngineState` ganha `hidden_out_since: Option<Instant>`. Ele começa a contar quando a janela fica `Hidden` com `in_room == false` (ou quando o motor foi solto) e zera ao mostrar a janela ou entrar numa sala.
- A cada checagem (de hora em hora, e quando uma versão nova é encontrada): com versão nova, contando há ≥ 30 min e sem estar ao vivo, instala em silêncio com `hidden: true` no `update-resume.json`.
- Falhou: nada aparece. Tenta na próxima hora.

**Por quê**: o X só esconde (007), então o app quase nunca reinicia. Escondido e fora de sala, ninguém está usando (é o mesmo critério da 009 para soltar o motor).

## R10 · Testar sem publicar

**Decisão**:
- Em builds de debug, `ESPIA_UPDATE_URL` troca o endereço do JSON (`updater_builder().endpoints(...)`), e o build de teste usa um config extra (`src-tauri/tauri.test.conf.json`) com `dangerousInsecureTransportProtocol: true` para aceitar o `http://127.0.0.1:8787` do `wrangler dev`.
- O teste ponta a ponta instala de verdade um 0.1.9 de teste e serve um "0.1.10" local (quickstart).
- O teste de assinatura usa um `.exe` com 1 byte trocado: o app tem que recusar.
- A regra de decisão (quando instalar, avisar ou ignorar) é uma função pura em `src-tauri/src/update.rs`, com testes `cargo test`, como `engine_action` na 009.

## R11 · Quem tem 0.1.8 ou mais antigo

Essas versões não têm o plugin, então não leem o JSON e nunca veem aviso. Elas continuam funcionando com o site como hoje. O `appMinimum` começa em `0.1.9` e só vale para apps que já sabem se atualizar. → Avisar o grupo uma última vez para baixar pelo site (o botão "Baixar o app" já existe). Daí em diante, é sozinho.

Opcional, fica fora deste plano: o site poderia avisar quem está num app sem `espiaUpdate`. Não entra: a página teria que adivinhar a versão do app, e são só ~10 amigos.
