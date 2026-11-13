# Contrato: o codificador (R1 a R4, R13)

**Feature**: [spec.md](../spec.md) · **Modelo**: [data-model.md](../data-model.md) · **Decisões**: [research.md](../research.md)

Este contrato diz o que o publicador (`espia-media/src/publish.rs`) entrega a um codificador, o que recebe de volta e o que cada caminho garante. Não é implementação: as assinaturas abaixo são só para ilustrar e os nomes podem mudar. O que não foi confirmado fica marcado, com o gate que resolve.

Dois codificadores cumprem o mesmo contrato, em `app/crates/espia-media/src/encode/`:

| Arquivo | Caminho | Quem comprime |
|---|---|---|
| `hardware.rs` | placa de vídeo | o codificador de H.264 por hardware do Windows (Media Foundation), uma vez, para todos |
| `software.rs` | processador | a biblioteca (`libwebrtc` 0.4.0), uma vez por espectador |
| `mod.rs` | | a interface e o quadro comprimido |

## Interface (ilustrativa)

```text
Codificador
  iniciar(config)                      -> ok | erro de início
  entregar(quadro)                     -> ok | erro de compressão
  mudar_alvo(taxa_em_bps)              -> ok | não aceito
  pedir_quadro_inteiro()               -> (o próximo quadro sai inteiro)
  receber() -> QuadroComprimido        (um por quadro entregue, ou nenhum)
  parar()
```

No caminho pelo processador, quem comprime é a biblioteca. O "codificador" desse caminho entrega o quadro cru e não devolve quadro comprimido (ver "O que cada caminho garante").

## O que o codificador recebe

| Entrada | Valores | Regra |
|---|---|---|
| Quadro | textura NV12 na placa, ou quadro cru em memória | Caminho pela placa: a textura que a captura já produz, no mesmo dispositivo gráfico (R3). Na sonda (G4, G5) o quadro é gerado no processador e enviado para uma textura, porque ali se prova a ligação e não a captura. Caminho pelo processador: quadro cru em memória (na sonda do G1, `I420Buffer`). |
| Tamanho | largura × altura | Igual ao da qualidade a que o codificador serve (R4). Fixo enquanto a qualidade não muda. |
| Quadros por segundo | 15, 30 ou 60 | Igual ao da qualidade (R4). |
| Taxa alvo | bits por segundo | **Muda ao vivo**: quando a qualidade efetiva muda, ou quando a regra de internet fraca baixa a taxa da cheia. |
| Pedido de quadro inteiro | sim, sem dados | O próximo quadro daquela qualidade sai inteiro. O publicador já juntou os pedidos (300 ms por qualidade, ver [qualidades.md](./qualidades.md)). |

Um codificador serve **uma qualidade**. No caminho pela placa há no máximo dois ao mesmo tempo, um para a cheia e um para a reduzida (R6). Cada um só existe enquanto alguém recebe a qualidade dele.

## O que o codificador devolve

Um **quadro comprimido** por quadro entregue, com os campos de [data-model.md](../data-model.md#quadro-comprimido):

| Campo | Regra |
|---|---|
| Codec | H.264 |
| Parte | inteiro ou parcial |
| Dados | H.264 com códigos de início; parâmetros de sequência junto de cada quadro inteiro (**não confirmado**, G4) |
| Tamanho da imagem | o da qualidade |
| Tempo de captura | o instante da entrega do quadro ao codificador |
| Qualidade | a que ele serve |

O quadro é entregue à fonte de vídeo daquela qualidade (`NativeVideoSource::new_encoded`, `capture_encoded_frame`, R1). O repassador de cada ligação cuida do resto.

## O que cada caminho garante

| | Placa de vídeo | Processador |
|---|---|---|
| Quem comprime | o app, pelo codificador da placa | a biblioteca, dentro de cada ligação |
| Quantas compressões | uma por qualidade ativa (no máximo duas) | uma por espectador que recebe vídeo |
| O que o app entrega | quadro já comprimido, uma vez, à fonte | quadro cru, uma vez, à fonte de cada ligação |
| Qualidade por espectador | cheia ou reduzida (duas faixas) | sob medida por ligação, como na sonda do G1 e no app de hoje (`send_params`) |
| Quadro inteiro | o app o produz a pedido e o repassador de cada ligação o exige ao entrar no meio, trocar de faixa ou perder quadro | a biblioteca cuida, por ligação |
| Teto | a qualidade efetiva | a qualidade efetiva (sem teto próprio, R18) |
| Custo de processador | quase não cresce com os espectadores | cresce com os espectadores |
| Provado | nada ainda; G4 e G5 | G1 |

Em qualquer caminho: nada é enviado a quem não clicou em Assistir, a quem parou de assistir ou a quem está com a janela escondida, e parar de enviar é `active=false`, nunca tirar a faixa de um canal ativo (R7).

## Configuração obrigatória do caminho pela placa (R4)

| Item | Valor | Por quê |
|---|---|---|
| Baixa latência | ligada | atraso baixo numa transmissão ao vivo (`CODECAPI_AVLowLatencyMode`) |
| Quadros B | nenhum | atraso baixo |
| Perfil | Constrained Baseline | o que o site negocia hoje; Chrome e Safari já recebem |
| Controle de taxa | constante, com alvo que muda ao vivo | banda controlada (`CODECAPI_AVEncCommonRateControlMode`, `CODECAPI_AVEncCommonMeanBitRate`) |
| Quadro inteiro | só a pedido; intervalo automático desligado ou muito longo | `CODECAPI_AVEncMPVGOPSize`, `CODECAPI_AVEncVideoForceKeyFrame` |
| Tamanho e quadros por segundo | os da qualidade efetiva | R4 |
| Entrada | textura no mesmo dispositivo gráfico da captura | `IMFDXGIDeviceManager`, `MFCreateDXGISurfaceBuffer`, `MF_SA_D3D11_AWARE` (R3) |

Perfil High (melhor imagem na mesma banda) não é o que a negociação anuncia; fica para depois da prova (R4, alternativas).

## Erros e o que o chamador faz

| Situação | Quando acontece | O que o chamador faz |
|---|---|---|
| Não achou codificador por hardware | início | segue pelo caminho pelo processador desde o começo ([caminho.md](./caminho.md)) |
| Codificador achado não iniciou | início | idem |
| Erro ao comprimir um quadro | durante | descarta o quadro, conta um erro seguido. Um quadro que sai bem zera a contagem. Três seguidos: troca para o processador |
| Nenhum quadro comprimido em 2 s | durante, com alguma qualidade ativa | troca para o processador |
| Alvo de taxa não aceito ao vivo | durante | Reabrir o codificador com a taxa nova; o próximo quadro sai inteiro. Mais de uma vez por minuto conta como erro para a troca de caminho (R18). Se o codificador da máquina do dono aceita a mudança ao vivo é **não confirmado** (G4). |
| Janela capturada muda de tamanho | durante | As qualidades ativas são reabertas no tamanho novo e o próximo quadro sai inteiro (R18). A pausa que isso causa é **não confirmada** (G6). |
| Sinal de que a placa está no limite | durante | quadros que o codificador não comprimiu a tempo contam como "limitado" e entram no ajuste por processador (R10). Qual sinal o codificador dá é **não confirmado** (G6; plano B: atraso entre entregar o quadro e receber o comprimido). |

Nenhum erro aparece para a pessoa como aviso técnico (FR-014).

## O que não foi confirmado

| Item | Onde | Resolve |
|---|---|---|
| O vídeo comprimido pela placa chega certo ao Chrome | R1, R4 | G4 |
| Formato do quadro que o repassador espera (códigos de início, parâmetros de sequência junto do quadro inteiro) | R1, R4 | G4. Plano B: acrescentar os parâmetros de sequência antes de cada quadro inteiro, no app. |
| Baixa latência, quadro inteiro a pedido e alvo ao vivo no codificador da máquina do dono | R2, R4 | G4 |
| O que a sonda mostra como "encoder" quando o quadro vem pronto do app | G4 item (d) | G4. O Diagnóstico da sonda precisa distinguir o caminho; como ele aparece com `PreEncoded` não foi visto. |
| Várias ligações na mesma fonte | R5 | G5 |
| Captura e codificador no mesmo dispositivo gráfico, e custo no FPS do jogo | R3, R10 | G6. Plano B: entrada pela memória, medindo o custo. |
| Outras marcas de placa (AMD, Intel) | R2 | conferência com o grupo. Marca que não servir cai no caminho pelo processador. |
| Patente do H.264 com o codificador do sistema | R16 | junto da pendência de licenças da 019 |

Falha do G4: volta-se à decisão R2 da 019 (biblioteca modificada ou outra biblioteca).
