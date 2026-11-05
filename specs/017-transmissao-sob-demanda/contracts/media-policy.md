# Política de mídia e limite financeiro

## Política executável da 017

`mintIceServers` devolve STUN_ONLY antes de verificar chaves ou fazer requisição externa. Welcome, resume e refresh obedecem à mesma função. Segredos configurados não habilitam relay. O cliente aceita somente URLs STUN/STUNS neste modo; não injeta servidores TURN locais nem usa configuração ICE antiga em nova conexão.

Se uma conexão existente desta versão descobrir rota relay incompatível com a política, fechar a conexão e tentar reconstruir com STUN-only, preservando somente escolhas válidas. Não entrar em ciclo ilimitado: após falha, mostrar indisponibilidade e Tentar novamente. O fallback não garante conectividade em redes restritivas.

O cliente não interpreta STUN-only como prova de que terceiros com credenciais anteriores deixaram de consumir a conta. Esta release não promete interromper allocations de clientes externos nem consumo de outros produtos.

## Drenagem e escopo da proteção

Antes de afirmar proteção financeira em produção: parar emissão, inventariar chaves e usos compartilhados, registrar último issuedAt e TTL máximo de credenciais antigas, revogar conhecidas e comprovar expiração financeira. Credenciais emitidas pela versão atual têm TTL de seis horas; confirmar em toda fonte emissora. Verificar saldo suficiente para a transição. Sem inventário ou controle, marcar proteção de conta pendente. Nenhuma operação de conta faz parte de /speckit-plan.

## Gate para habilitar relay ou SFU no futuro

Exigir consumo agregado confiável da franquia, limite superior de consumo autorizado enquanto medições atrasam, reservas atômicas compartilhadas, interrupção/expiração com efeito financeiro demonstrável, controle das credenciais legadas e conta sem emissores fora do orçamento. O limite aplicado deve descontar margem para todo consumo autorizado, não só bytes já reportados. Reinício não recupera saldo sem confirmação. Testar falhas, concorrência e atraso de contabilização.

Sem essas provas, a única configuração aceita é direct-only. Não criar rotas, tabelas de reserva, feature flag que habilite TURN ou adaptador SFU no código desta entrega. Uma futura proposta apresenta os contratos novos e reabre o gate; a história de saldo positivo permanece não exercitada.
