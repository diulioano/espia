# Conexão direta como padrão e mídia limitada à franquia gratuita

O Espia preserva site e app e mantém conexão direta como transporte padrão. A pessoa usuária informou mais de 60 horas mensais de uso e deseja permanecer na franquia gratuita, com duas transmissões e até sete espectadores assistindo às duas. Uma distribuição integral por servidor pode exceder a franquia nesse cenário.

A direção acordada é bloquear SFU e TURN sem saldo gratuito, aceitando indisponibilidade nas redes que dependem de relay. A implementação futura precisa interromper também o consumo ativo e reservar margem para tráfego ainda não contabilizado. Essa política ainda não está implementada.

A SFU permanece como alternativa para uma prova limitada, condicionada às medições de upload e codificação e ao controle de consumo. A migração ampla do motor para Rust foi adiada em favor de melhorias graduais. A meta de até 500 ms na maior parte do tempo é um objetivo de validação, não uma garantia de desempenho.

Atualizado em 2026-11-11: ver [ADR 0002](./0002-app-nativo-em-rust.md) sobre a migração para Rust.
