# Sonda G2: a tela da sala em Slint (019, T011)

Prova se a Slint 1.17 (R3) reproduz a tela da sala do Espia com fidelidade e aguenta um vídeo 1080p60 mais duas miniaturas. Sem rede e sem captura: tudo é dado falso e vídeo gerado em código.

A tela é a cena `focus` do mock da sala (`src/mocks/room.ts`): Bia em foco no palco, Lu e Tomás nas miniaturas, aviso "Caio saiu da sala", convite de teste local. É a cena do site que tem palco e miniaturas ao mesmo tempo.

## Como rodar

```powershell
cd app/spikes/ui-probe
cargo build --release -j 6        # nunca dois builds ao mesmo tempo (o PC já reiniciou por isso)
target\release\ui-probe.exe                    # roda até fechar a janela
target\release\ui-probe.exe --exit-after 15    # fecha sozinha depois de 15 s
```

A cada 5 s o terminal imprime uma linha como:

```
[   5s] memória privada (conjunto de trabalho privado): 95.2 MB [confirmada 157.4 MB, conjunto total 133.2 MB] | FPS pintados: 116.3 | entregues: 57.2/s | descartados: 14
```

- **memória privada**: conjunto de trabalho privado (`QueryWorkingSet` sem as páginas compartilhadas), o número da coluna "Memória" do Gerenciador de Tarefas. A **confirmada** (`PrivateUsage`) e o conjunto total vêm junto para conferência. O critério de R3 (150 MB) deve usar o primeiro número; registre os três.
- **FPS pintados**: quantas vezes a Slint terminou uma renderização (`RenderingState::AfterRendering`).
- **entregues**: quadros gerados que chegaram à tela por segundo (alvo 60). **descartados**: quadros que o gerador jogou fora porque a tela ainda não tinha consumido o anterior (sinal de engasgo).

### Atenção ao "FPS pintados"

Na máquina de teste ele marcou ~116 a 122, o dobro dos 60 entregues. A Slint repinta mais de uma vez por quadro de vídeo (a janela do teste pode estar a 120 Hz, ou o notificador conta duas passagens). Não é prova de 120 quadros de vídeo. Para o veredito de "sem engasgo" use **entregues** (precisa ficar perto de 60) e **descartados** (perto de 0), mais o olho na barra de cores rolando e no contador de quadros no centro do palco.

## Comparar lado a lado com o site

1. Na raiz do repositório: `npm run dev` (Vite, porta 5173).
2. Abra `http://localhost:5173/?mock=room&scene=focus` em uma janela do Chrome.
3. Rode a sonda e ponha as duas janelas lado a lado, com o mesmo tamanho (a sonda abre em 1280x800 lógicos; redimensione o Chrome para o mesmo, com zoom 100%). Tema escuro (é o único que a sonda implementa).
4. Confira: barra lateral (logo, "Na sala", 4 pessoas, selo AO VIVO, anel no avatar de quem transmite, "você"), cabeçalho (título, subtítulo, convite com "Copiar convite"), palco com foco e anel de destaque, selo de mudo, duas miniaturas com chip de nome, dica, aviso no canto, rodapé ("Transmitir", dica, "Sair da sala").

No palco o site mostra um quadro falso (padrão); a sonda mostra as barras geradas. O conteúdo do vídeo, naturalmente, difere.

## Origem do que está aqui

- `ui/tokens.slint`: cópia à mão de `src/ui/tokens.css` (tema escuro), com o nome do token no nome da propriedade (`color-accent`, `color-stage`, `radius-stage`...). Em R18 a produção gera esse arquivo a partir do CSS; aqui não. Cores fora de token e por quê: olho, nariz e dentes do castor (vêm do próprio `Mascot.svelte`, não são tokens) e os pequenos fundos translúcidos `rgba(245,234,216,x)` da linha "assistindo" e da pílula "você" (o CSS os escreve inline). As barras do vídeo usam só cores das rampas de `tokens.css`.
- `ui/widgets.slint`: ícone, selo AO VIVO, avatar, botão, castor, logo. `ui/room.slint`: a tela. `src/video.rs`: gerador. `src/mem.rs`: memória do processo.
- `assets/icons/*.svg`: ícones Lucide usados pela sala (traço 2.75 como no site).

### Fontes (procedência e confiabilidade)

A Slint só carrega TTF/OTF e o pacote `@fontsource` só traz woff/woff2. Os TTF em `assets/fonts/` foram gerados por `tools/woff2ttf.py` (Python puro, sem dependências) a partir de `node_modules/@fontsource/{caprasimo,figtree}/files/*-latin-<peso>-normal.woff` (pacotes 5.3.0): Caprasimo 400 e Figtree 400, 600 e 700. Os `.woff` de origem estão copiados ao lado e conferem byte a byte com os do `node_modules`.

A conversão WOFF 1 para TTF é só descompressão zlib de cada tabela e remontagem do cabeçalho sfnt, sem tocar nos glifos: sem perda, é o mesmo desenho. Conferido: as tabelas têm a soma de verificação original correta nos 4 arquivos e a Slint carregou as quatro (a tela renderiza sem erro). Ressalvas: os arquivos são o subconjunto **latin** (cobre português, "—", "·", "…"; não cobre outros alfabetos, que o app não usa) e a conversão não foi checada com outra ferramenta (fontTools não estava instalado). Confiável o bastante para a sonda; a produção deve conferir de novo.

Licenças (SIL OFL 1.1) guardadas em `assets/fonts/LICENSE-caprasimo.txt` e `LICENSE-figtree.txt`. A OFL exige levar o texto junto quando a fonte é redistribuída, o que vale também para o app final.

## O que a T012 pede para medir

Critérios (de `tasks.md`): (1) tela da sala igual ao site na comparação lado a lado; (2) 1080p60 e duas miniaturas sem engasgar; (3) memória privada do processo, **com o motor do G1 carregado**, em até 150 MB. Registrar: capturas das duas telas, FPS percebido, memória em MB, data (em `relatorio.md`, seção "G2"). Se falhar, plano B de R3: `iced`.

### Sobre a memória final (importante)

Esta sonda sozinha **não prova o item 3**. Ela mede só a interface e o vídeo gerado; o número final precisa do motor de mídia do G1 (WebRTC, decodificador H.264, receptor) vivendo **no mesmo processo**, porque o orçamento de 150 MB é do processo inteiro. Na máquina de teste a sonda, sozinha, ficou perto de 95 a 104 MB de memória privada (confirmada ~150 a 158 MB), então sobra pouco para o motor. Como tratar:

1. Registrar o número desta sonda como "só interface" (piso).
2. Quem fechar o G1 (pessoa ou principal) deve juntar as duas peças num único executável: ou chamar a crate do G1 (`app/spikes/...` do T009) a partir de um `main` que também abra esta tela, ou colocar o gerador de vídeo `src/video.rs` e a janela em cima do receptor do G1, entregando os quadros decodificados no lugar dos gerados (a Slint recebe `Image::from_rgba8_premultiplied`, o ponto de troca é `video::iniciar`). Rodar com o G1 ligado a uma sala real, 1080p60 recebido, e ler a mesma linha de memória a cada 5 s.
3. Medir a memória depois de alguns minutos (estabilizada), não só nos primeiros 5 s, e observar se cresce (vazamento).
4. Anotar as duas memórias (conjunto de trabalho privado e confirmada). Se o conjunto privado do processo completo passar de 150 MB, R3 falha pelo critério e vale discutir se o culpado é a interface ou o motor antes de acionar o `iced`.

## Diferenças conhecidas em relação ao site

Ver o relatório de quem rodou: limites da Slint (sombras, cantos, fontes, animação, custo da imagem por quadro) estão no resumo entregue com esta sonda e devem ir para a seção "G2" do `relatorio.md`.

- Sombra: só o aviso do canto leva sombra (`--shadow-lg`). A Slint tem uma sombra por elemento (`drop-shadow-*`) e não tem o "anel de 1px" do `box-shadow` com spread; o anel foi feito com `border`. O desfoque e a difusão não são idênticos aos do Chrome.
- Castor do logo: cabeça com cantos elípticos (`border-radius` com duas razões) só existe como `Path`; o olho pisca a cada 4,2 s como no site, sem o balanço.
- Só a cena `focus`, tema escuro, sem hover, sem foco de teclado, sem diálogos e sem as outras cenas.
- Convite em Consolas (o CSS pede `ui-monospace, Menlo, monospace`; no Windows o Chrome cai em Consolas, mas isso depende da máquina).
- Texto: o suavizador de bordas e o espaçamento fino de glifos da Slint (femtovg) não são os do Chrome; diferenças de 1 px em quebra e altura de linha são esperadas.
