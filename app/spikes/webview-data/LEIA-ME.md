# Sonda G3: dados do app de antes (T013)

Lê o `localStorage` do motor de página (WebView2/Chromium) do app de antes, a partir de uma
**cópia** da pasta `leveldb`, com a crate `rusty-leveldb` 3.0.3. Prova a base do R10 e de SC-003.
Crate isolada (`[workspace]` vazio no `Cargo.toml`): não entra no workspace de `app/`. Será apagada
no fim da feature, junto com os outros `spikes/`.

## O que a sonda faz

1. Recebe por argumento o caminho de uma cópia da pasta `leveldb`.
2. Recusa (código 2) caminhos dentro de `%LOCALAPPDATA%\dev.espia.app\`.
3. Copia os arquivos da pasta (sem o `LOCK`) para uma pasta temporária e abre **só essa segunda
   cópia**, porque o `rusty-leveldb` escreve ao abrir (`LOG`, `MANIFEST`, recuperação do `.log`).
   A pasta temporária é apagada no fim.
4. Lista as chaves das origens `https://espia.espia.workers.dev` e `https://espia.diuliano.com`,
   imprime `espia:rooms` e `espia:nickname`, e, para cada `espia:owner:<código>`, só o código e se o
   valor tem 64 hex minúsculos (e quantos caracteres). **Nunca** imprime valor de `espia:owner:*`
   nem de `espia:session:*` (só a contagem destas). As chaves `espia:volume:*` saem só como contagem.
5. Mostra também todas as origens que existem no banco, com a contagem de chaves. Serve para
   diagnosticar caso o formato das chaves seja diferente do suposto.

Códigos de saída: 0 leu e achou dados do Espia; 1 não conseguiu ler (mensagem diz por quê);
2 uso errado ou caminho recusado; 3 abriu o banco mas não achou dados das origens do Espia.

## Comandos

```powershell
cd app\spikes\webview-data
cargo build --release
cargo test

# Fixture de teste (pasta nova; não existe perfil real aqui)
.\target\release\espia-webview-data-probe.exe --criar-fixture C:\teste-g3\fixture
.\target\release\espia-webview-data-probe.exe C:\teste-g3\fixture
```

## T014: como a pessoa roda com o perfil real

Com o app de antes **fechado** (inclusive na bandeja; confira no Gerenciador de Tarefas):

```powershell
New-Item -ItemType Directory -Force C:\teste-g3 | Out-Null
Copy-Item -Recurse "$env:LOCALAPPDATA\dev.espia.app\EBWebView\Default\Local Storage\leveldb" C:\teste-g3\leveldb
cd C:\Users\Diuli\Documents\espia\app\spikes\webview-data
cargo run --release -- C:\teste-g3\leveldb
```

Anotar no `relatorio.md` (seção "G3") a linha `RESULTADO:` (salas lidas ou não, quantas chaves de
dono e quantas com 64 hex, apelido lido ou não) e a data. Não copiar valores para o relatório.
Se `Copy-Item` reclamar de arquivo em uso, o app ou o motor ainda está aberto: feche e repita.
Se o app instalado usa outra pasta de perfil, procure `Local Storage\leveldb` dentro de
`%LOCALAPPDATA%\dev.espia.app\` e copie essa. Quando terminar, apague `C:\teste-g3`.

## Formato do Chromium: confirmado e não confirmado

Suposto (pista do pedido; coincide com o que o autor conhece do Chromium, mas **não foi conferido
contra dados reais nesta máquina**):

- chave de dado = `_` + origem + byte `0x00` + byte de formato + nome da chave;
- valor = byte de formato + texto; formato `0x01` = Latin-1, `0x00` = UTF-16LE;
- chaves `META:<origem>` (e `METAACCESS:<origem>`) e `VERSION`;
- comparador padrão por bytes (`leveldb.BytewiseComparator`).

**Confirmado** (só contra a fixture e o código da crate, o que NÃO prova compatibilidade com o
Chromium real):

- a sonda decodifica os dois formatos, com acento (`João Ação` em Latin-1; `Ωmega João` em UTF-16LE);
- a crate usa o comparador `leveldb.BytewiseComparator` e lê tabelas `.ldb` e `.sst`
  (`types.rs`/`table_cache.rs` da crate) e registros `.log`;
- a crate registra Snappy (id 1) e None (id 0) por padrão e decodifica pelo id gravado no bloco;
  a fixture usa Snappy;
- a sonda não altera a pasta recebida (há teste).

**Não confirmado** (decide o T014):

- o prefixo exato das chaves (`_` + origem, com ou sem `/` final: a sonda aceita os dois);
- que o formato `0x00`/`0x01` é exatamente esse na versão do WebView2 instalada;
- que o Chromium dessa versão não grava o `localStorage` num formato novo (particionamento por
  `StorageKey`, que acrescentaria algo à chave) ou num banco diferente;
- que o comparador gravado no `MANIFEST` do Chromium é o bytewise padrão (a crate falharia ao abrir
  se fosse outro; a mensagem de erro apareceria como "o leveldb não abriu");
- leitura de um banco com compactação Snappy real do Chromium (a fixture é gravada pela própria
  crate, então não exercita a diferença de implementação).

Não foi usado nenhum Chrome/Edge/WebView2 da máquina para conferir o formato: isso exigiria abrir
dados da pessoa.

## Riscos para a importação real

- **Arquivos travados**: com o app (ou outro programa com WebView2 do mesmo perfil) aberto, o
  Windows trava `LOCK` e, às vezes, o `.log` e o `MANIFEST`. A sonda pula o `LOCK`; os demais podem
  falhar na cópia. No app nativo, a importação deve rodar só quando o app antigo não estiver
  rodando, e tratar erro de cópia como "tentar de novo na próxima abertura" (R10).
- **Cópia inconsistente**: copiar arquivo por arquivo com o motor ativo pode pegar `MANIFEST` e
  `.ldb` de momentos diferentes (erro "missing live files"/corrupção). Só copiar com o app fechado.
- **Snappy**: o Chromium comprime os blocos das tabelas com Snappy. A crate decodifica, mas a
  compatibilidade com o Snappy real só foi exercitada na fixture.
- **Comparador**: se o `MANIFEST` apontar outro comparador, a crate recusa abrir.
- **Registro `.log` não compactado**: dados recentes (provavelmente os mais importantes, como uma
  sala criada há pouco) ficam só no `.log` até o Chromium compactar. A crate o relê ao abrir, mas o
  laço de recuperação dela **para em silêncio** no primeiro registro com erro (`while let Ok(..)`), e
  um `.log` com a cauda escrita pela metade perde só o final, sem aviso. A sonda não detecta isso.
- **Escrita ao abrir**: a crate escreve na pasta; por isso a cópia temporária. O app deve fazer o
  mesmo.
- **Pânico**: a sonda captura pânicos da crate e os mostra como erro. O app deve fazer igual.
- **Valores secretos**: o app real não deve registrar nem imprimir donos e sessões (só a sonda de
  teste mostra o código).
