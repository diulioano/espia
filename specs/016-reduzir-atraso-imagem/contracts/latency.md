# Contracts: Latência e compatibilidade

## Interfaces externas preservadas

- API HTTP/WebSocket do Worker e protocolo de sala: nenhuma alteração.
- Canal ctl: continua view com full/thumb/hidden e off codificado como hidden + sound:false. Nenhum relógio, dado de diagnóstico ou pedido de reset novo trafega nesse canal.
- Ponte nativa: capture_supported/version 1, capture_list, capture_start, capture_set_quality e capture_stop mantêm argumentos, respostas e permissões da [005](../../005-seletor-nativo-desktop/contracts/native-capture.md) e transporte da [010](../../010-transmissao-leve/contracts/native-capture-delta.md).
- FrameMsg mantém gen/slot/seq/ts; ts continua relativo à captura em microssegundos. O site muda seu mapeamento local, sem redefinir silenciosamente a época enviada por apps antigos.
- Layout NV12 e os três slots permanecem iguais. Áudio preserva PCM f32 48 kHz estéreo e formatos de transporte existentes. A duração é calculada pelo tamanho recebido, não por pressupor pacotes de 40 ms.

## Contrato entre captura e geradores

CaptureTiming recebe a hora monotônica injetada e timestamps nativos, nunca Date.now. Nova sessão reseta a época; mudança de qualidade mantém a época. O limite de 200 ms vale para filas controladas, não representa uma promessa sobre toda a cadeia de mídia.

A propriedade dos VideoFrame/AudioData segue o comportamento do writer do motor. Frames rejeitados antes de transferir propriedade devem ser fechados; cancelamento/rejeição de escrita precisa liberar recursos sem usar objetos já consumidos. Verificar o comportamento da API Chromium existente, sem pressupor identidade com a API VideoTrackGenerator mais recente.

Stop é idempotente e terminal. Um resultado tardio de capture_start, write, setQuality ou replaceTrack não pode publicar mídia após stop/close; se o recurso terminar de abrir após cancelamento, deve ser fechado imediatamente. Esse caso precisa de teste de startup interrompido.

## Recepção e fallback

PeerLink configura jitterBufferTarget=0 apenas quando o receiver expõe a propriedade; erro não impede ontrack, negociação ou reprodução. Aplicar de forma coerente a áudio/vídeo, sem modificar SDP. O retorno de full/thumb/hidden/off continua usando os controles existentes, preservando áudio em hidden, corte de ambos em off e PiP ativo em segundo plano.

Não há contrato de seek ou playbackRate em MediaStream. Não reiniciar ICE ou toda a malha como mecanismo periódico de limpeza. Se a validação exigir outro mecanismo, documentá-lo com evidência antes de alterar este contrato.

## Diagnóstico local

media-stats normaliza contadores opcionais em amostras descritas em [data-model.md](../data-model.md). Ausência de suporte retorna null, sem bloquear a transmissão. PeerLink fornece dados; Mesh agrega; RoomStore controla ciclo de coleta, inclusive para quem apenas assiste.

As estatísticas não representam atraso ponta a ponta. A interface visual não ganha rótulo de atraso baseado em RTT/jitter, e nenhum dado é transmitido ao servidor. A amostragem é limitada a uma chamada pendente por ciclo e ring de 60 entradas por link.

## Contrato observável para quem usa

| Ação / condição | Resultado exigido | Referência |
|---|---|---|
| Assistir em condições estáveis | p95 ≤ 1 s; nenhuma amostra ≥ 5 s | FR-001/002, SC-001 |
| Comparar com Discord | p95 no máximo 500 ms maior | SC-002 |
| Recuperar após perturbação | ≤ 1 s de atraso em ≤ 5 s | FR-003/004, SC-003 |
| Entrar, voltar, trocar modo/qualidade | recuperar conforme história 3 | FR-006, SC-004 |
| Ouvir a transmissão recuperada | diferença A/V ≤ 200 ms | FR-005, SC-005 |
| Outro espectador tem dificuldade | demais mantêm o limite estável | FR-007, SC-001/004 |
| Continuar assistindo | sem recarregar, reentrar ou configurar atraso | FR-008, SC-005 |
| Encerrar durante engasgo | nenhuma reativação por callback tardio | FR-009, SC-006 |

## Verificação de compatibilidade

Executar testes com bridge falsa nos formatos antigos e atuais, receiver com/sem suporte e setter que lança. Em aparelhos reais, testar app instalado com site novo e app reconstruído, browser↔browser e app↔browser. Não aumentar versão mínima nem forçar instalação para contornar falha de aceitação.
