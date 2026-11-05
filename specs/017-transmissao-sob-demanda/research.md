# Pesquisa: transmissão sob demanda

Data: 2026-11-05. Inspeção local e fontes primárias; nenhum serviço provisionado.

## Escolha positiva por transmissão

**Decision**: substituir `unwatched` por escolhas explícitas em memória, ligadas a peer e início da transmissão. Ausência significa não assistir.
**Rationale**: `buildView()` hoje considera assistir quando a pessoa não está em `unwatched`; `pruneUnwatched()` remove a escolha ao reiniciar, reativando recepção. Isso contraria FR-001/004.
**Alternatives considered**: preencher antecipadamente lista negativa manteria a dependência de eventos na ordem certa; flag global impediria escolher uma dentre duas telas. Preservar escolhas na retomada, não em nova entrada.

## Primeiro envio e identidade

**Decision**: default remoto off, autorização para uma transmissão específica e sender sem track enquanto não autorizado. Canal de controle pode existir sem mídia.
**Rationale**: `PeerLink.theirView` começa full, fallback mesh é hidden e hidden mantém som. A ligação de tracks precede o ajuste assíncrono de encodings. Apenas esconder UI ou chamar active=false depois cria janela de envio.
**Alternatives considered**: manter mensagem legada de view não evita autorização antiga de outra transmissão. Reutilizar filas de setLocal/encoding, mas verificar identidade e revisão após cada await.

## Versão de protocolo

**Decision**: sinalização anuncia versão de mídia 2; clientes sem suporte são recusados com pedido de atualizar, antes de entrar. Atualização coerente de cliente/servidor; sockets antigos sem versão válida são encerrados na transição.
**Rationale**: um emissor antigo pode mandar mídia antes de ler off. Não existe garantia de zero conteúdo misturando esse emissor com receptores novos.
**Alternatives considered**: tradução para hidden/sound=false mantém utilidade apenas em legado, sem comprovar primeiro envio. Não prometer compatibilidade comportamental invisível.

## Proteção do custo

**Decision**: STUN-only nesta release; política sem prova equivale a indisponível. Não emitir nem renovar credenciais TURN e não construir ledger sem autoridade para limitar consumo.
**Rationale**: credenciais atuais valem seis horas. Analytics não é um limitador, e revogação não traz prazo documentado de corte de tráfego ativo. A FAQ documenta fim de faturamento ao vencer TTL, mas não permite assumir corte imediato por revoke. Limites por allocation não são teto agregado da conta.
**Alternatives considered**: alertas, estatísticas do navegador e reserva estimada não garantem saldo compartilhado nem clientes legados. Um controlador global só faz sentido após demonstrar limite efetivo, considerando todas as allocations e outros usuários da conta.

Na transição, parar de emitir primeiro, inventariar usos e registrar último instante possível de emissão + TTL máximo. A expiração limita faturamento conforme documentação; não prova que o saldo prévio era suficiente durante a drenagem. Revogar credenciais conhecidas é defesa adicional, não evidência de custo zero. Não remover chaves ou encerrar salas em produção durante planejamento.

Fontes: [FAQ TURN, cobrança e expiração](https://developers.cloudflare.com/realtime/turn/faq/), [substituição de TURN e analytics](https://developers.cloudflare.com/realtime/turn/replacing-existing/), [limites de serviço TURN](https://developers.cloudflare.com/realtime/turn/), [franquia compartilhada](https://developers.cloudflare.com/realtime/sfu/platform/pricing/).

## SFU e governança

**Decision**: pesquisa documental nesta entrega. Uma futura prova reaproveita os tracks e publica uma vez, autorizando assinaturas no backend. Só depois de emenda II e decisão de criptografia, com custo controlável.
**Rationale**: a constituição proíbe SFU sem emenda, e a restrição de gratuidade não foi relaxada. Mais de 60 horas não permite presumir que 1.000 GB bastem.
**Alternatives considered**: distribuição global por SFU e hospedagem própria mudam custo e operação sem necessidade demonstrada. Migrar para Rust não elimina multiplicação de upload por espectador.

Fontes: [sessões e tracks](https://developers.cloudflare.com/realtime/sfu/concepts/sessions-tracks/), [Connection API](https://developers.cloudflare.com/realtime/sfu/api/), [ordem de negociação](https://developers.cloudflare.com/realtime/sfu/concepts/negotiation/).

## Qualidade e evidência

**Decision**: preservar captura nativa e limites atuais inicialmente; derivar perfil de hints de conteúdo/origem, com fallback explícito. Congestionamento por receptor; pressão real de encoder local pode reduzir captura global.
**Rationale**: a 016 corrigiu filas e timestamps mas ainda não comprovou atraso ponta a ponta. Evitar nova otimização sem baseline. Perfis não garantem identificação de cena; medir cenários mistos.
**Alternatives considered**: análise de pixels adicionaria cópia/carga; GPU encode nativo amplia escopo. Modificar resolução por uma rede individual pioraria todos.

Referências: `src/lib/quality.ts`, `cpu-governor.ts`, `peer.ts`, `media-stats.ts`, `src-tauri/src/capture/video.rs` e relatório da 016. Fonte de contrato: [WebRTC](https://www.w3.org/TR/webrtc/). Não há benchmark novo nesta etapa.

## Divergências resolvidas

O pedido do clique substitui decisão da 014. A emenda separada da constituição para 3.0.0 em 2026-11-05 distinguiu entrar na sala e escolher assistir, resolvendo C1 da análise. O princípio II permanece vigente e a prova remota SFU fica fora do caminho executável enquanto seu gate não for atendido.

I1: supressão de emissão/renovação TURN passa aos fundamentos antes de welcome v2/direct-only. US3 audita a política e integra o cliente, sem adiar o bloqueio do servidor.

A1: cobertura instrumental mínima de 99%, lacuna máxima de dois instantes e denominador fixo de 1.800; falta de vídeo é falha de entrega, não perda do instrumento. Atraso ou falta de imagem ≥5 s reprova. A2: movimento ≥27 fps com fonte ≥30, interrupções ≤1% e pausa máxima de 1 s; FPS mediano do jogo deve preservar ≥95% da baseline. Valores são critérios escolhidos para o ensaio, não medições obtidas.

U1: broadcastSeq monotônico e marca terminal por membro impedem que requestId encerrado volte a ser aceito. Manter apenas a marca mais recente no estado existente de retomada evita coleção de histórico. Essa ordem é independente da revision dos grants e preservada após hibernação/reinício enquanto o membro existir.

Nenhum item de pesquisa ficou como decisão técnica sem resposta; resultados de desempenho e viabilidade de relay são condições de aceitação, não fatos demonstrados.
