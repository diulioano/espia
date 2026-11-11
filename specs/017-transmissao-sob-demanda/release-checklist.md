# Transição de versão e proteção da conta

Procedimento para uma publicação futura, não executado nesta implementação.

## Preparar

Conferir checks locais e critérios reais no relatorio.md. Código local exige protocolo v2 nas duas pontas; servidor e assets devem ser publicados como uma unidade coerente. Preparar aviso Atualize o Espia e garantir que páginas antigas não sejam mantidas como participantes v2. A publicação depende de autorização própria; esta lista não a concede.

## Emitentes e saldo

Inventariar todas as chaves/emissores que usam a franquia SFU/TURN da conta. Conferir consumo e saldo, além de credenciais ainda válidas. Não listar segredos no relatório. A versão anterior usa TTL de seis horas; confirmar o TTL máximo efetivo de cada emissor. Separar chave não cria outra franquia gratuita.

## Parar emissão e drenar

Aplicar a supressão de geração/renovação em todos os emitentes sob controle, registrar o último instante possível de emissão e não reativá-la por presença de segredo. Drenar sockets/attachments legados e impedir retomada sem v2. Credenciais prévias podem continuar com efeito financeiro durante a transição; aguardar o TTL máximo confirmado e conferir o consumo residual. Revogar credenciais conhecidas pode complementar a expiração, mas não prova corte imediato de allocations existentes.

Se não for possível inventariar emissores, confirmar saldo ou comprovar drenagem, manter proteção global da conta pendente e não prometer custo zero. Não converter fim da emissão local em afirmação sobre todos os consumidores da conta.

## Compatibilidade e rollback

Conferir join/resume v2, welcome STUN-only e zero emissão em refresh com segredos configurados. Cliente antigo deve receber 4426; cliente novo diante de welcome antigo deve mostrar atualização e parar. Um rollback de assets não pode restaurar o envio automático ou o gerador TURN antigo. Preferir interromper a publicação e apresentar atualização necessária a misturar versões incompatíveis. Manter cópia das revisões revisadas, não publicar uma revisão antiga sem essas proteções.

## Verificação pós-publicação futura

Confirmar hashes de assets, entrada/retomada, Assistir/Parar em sala exclusiva de teste e ausência de geração TURN. Verificar em rede direta e rede que exige relay: a segunda pode ficar indisponível nesta entrega. Só encerrar a drenagem com evidência da conta. Não reutilizar salas de pessoas para testes.
