//! Rede do app nativo: API HTTP (`api`) e sinalização por WebSocket (`signaling`).
//! Mesmo servidor e mesmo protocolo do site; sem tipos de tela.

pub mod api;
pub mod server;
pub mod signaling;
