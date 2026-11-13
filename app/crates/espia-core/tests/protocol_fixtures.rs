//! Confere as fixtures de `app/fixtures/protocol/` (as mesmas que o teste Vitest do site lê):
//! cada uma tem de decodificar e recodificar sem perder campo.

use std::fs;
use std::path::PathBuf;

use espia_core::protocol::{ClientMessage, ServerMessage};
use serde_json::Value;

const CLIENT: [&str; 6] = ["join", "resume", "signal", "live", "unlive", "turn-refresh"];
const SERVER: [&str; 15] = [
    "welcome",
    "live-ack",
    "room-not-found",
    "room-full",
    "resume-failed",
    "peer-joined",
    "peer-left",
    "peer-live",
    "signal",
    "ice-servers",
    "error",
    "room-renamed",
    "room-deleted",
    "removed",
    "pong",
];

fn ler(lado: &str, t: &str) -> (String, Value) {
    let caminho = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/protocol")
        .join(format!("{lado}-{t}.json"));
    let texto = fs::read_to_string(&caminho).unwrap_or_else(|e| panic!("sem fixture {caminho:?}: {e}"));
    let valor: Value = serde_json::from_str(&texto).expect("fixture não é JSON");
    (texto, valor)
}

#[test]
fn fixtures_do_cliente_ida_e_volta_sem_perder_campo() {
    for t in CLIENT {
        let (texto, original) = ler("client", t);
        let msg: ClientMessage = serde_json::from_str(&texto).unwrap_or_else(|e| panic!("client-{t}: {e}"));
        let de_volta = serde_json::to_value(&msg).unwrap();
        assert_eq!(de_volta, original, "client-{t} perdeu ou mudou campo");
        assert_eq!(original["t"], t, "client-{t}: campo t diferente do nome do arquivo");
    }
}

#[test]
fn fixtures_do_servidor_ida_e_volta_sem_perder_campo() {
    for t in SERVER {
        let (texto, original) = ler("server", t);
        let msg: ServerMessage = serde_json::from_str(&texto).unwrap_or_else(|e| panic!("server-{t}: {e}"));
        let de_volta = serde_json::to_value(&msg).unwrap();
        assert_eq!(de_volta, original, "server-{t} perdeu ou mudou campo");
        assert_eq!(original["t"], t, "server-{t}: campo t diferente do nome do arquivo");
    }
}

#[test]
fn existem_exatamente_21_fixtures() {
    let pasta = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/protocol");
    let n = fs::read_dir(pasta)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
        .count();
    assert_eq!(n, CLIENT.len() + SERVER.len());
    assert_eq!(n, 21);
}
