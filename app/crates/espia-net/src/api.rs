//! Chamadas HTTP do Worker (espelha `src/lib/api.ts`; rotas em `contracts/sinalizacao-nativa.md`).
//!
//! Toda função recebe a base do servidor (`server::server_base()` no app; o servidor falso nos testes)
//! e manda `Origin` igual à origem da base: o Worker só aceita pedido do próprio host (`sameOrigin`).
//! Falhas viram os mesmos motivos do site; nada aqui entra em pânico por causa da rede.

use std::sync::OnceLock;
use std::time::Duration;

use reqwest::{Client, Method, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;

use espia_core::protocol::Tone;

use crate::server::{install_crypto, origin_of};

const TEMPO_MAXIMO: Duration = Duration::from_secs(15);

fn cliente() -> &'static Client {
    static CLIENTE: OnceLock<Client> = OnceLock::new();
    CLIENTE.get_or_init(|| {
        install_crypto();
        Client::builder().timeout(TEMPO_MAXIMO).build().expect("cliente HTTP")
    })
}

/// Resposta de uma chamada: status e corpo (já lido por inteiro).
struct Resposta {
    status: StatusCode,
    corpo: Vec<u8>,
}

impl Resposta {
    fn json<T: for<'de> Deserialize<'de>>(&self) -> Option<T> {
        serde_json::from_slice(&self.corpo).ok()
    }
}

/// Faz o pedido com `Origin` da base e, se houver, corpo JSON e chave de dono.
async fn pedir(base: &str, metodo: Method, caminho: &str, corpo: Option<String>, chave: Option<&str>) -> Option<Resposta> {
    let url = format!("{}{}", base.trim_end_matches('/'), caminho);
    let mut req = cliente().request(metodo, url).header("Origin", origin_of(base));
    if let Some(k) = chave {
        req = req.header("Authorization", format!("Bearer {k}"));
    }
    if let Some(c) = corpo {
        req = req.header("Content-Type", "application/json").body(c);
    }
    let res = req.send().await.ok()?;
    let status = res.status();
    let corpo = res.bytes().await.ok()?.to_vec();
    Some(Resposta { status, corpo })
}

// ---------------------------------------------------------------- sala rápida

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateFailure {
    RateLimited,
    Network,
}

/// Sala rápida: o código novo, ou o motivo da falha.
pub type CreateResult = Result<String, CreateFailure>;

#[derive(Deserialize)]
struct CorpoCodigo {
    code: String,
}

/// `POST /api/rooms` sem corpo.
pub async fn create_room(base: &str) -> CreateResult {
    let Some(r) = pedir(base, Method::POST, "/api/rooms", None, None).await else {
        return Err(CreateFailure::Network);
    };
    if r.status == StatusCode::CREATED {
        return r.json::<CorpoCodigo>().map(|c| c.code).ok_or(CreateFailure::Network);
    }
    Err(if r.status == StatusCode::TOO_MANY_REQUESTS { CreateFailure::RateLimited } else { CreateFailure::Network })
}

// ---------------------------------------------------------------- sala fixa

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateFixedFailure {
    RateLimited,
    Name,
    Network,
}

/// Sala fixa criada: a chave do dono vem só nesta resposta.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixedCreated {
    pub code: String,
    pub owner_key: String,
}

pub type CreateFixedResult = Result<FixedCreated, CreateFixedFailure>;

/// `POST /api/rooms` com `{ "kind": "fixed", "name": ... }`.
pub async fn create_fixed_room(base: &str, name: &str) -> CreateFixedResult {
    let corpo = json!({ "kind": "fixed", "name": name }).to_string();
    let Some(r) = pedir(base, Method::POST, "/api/rooms", Some(corpo), None).await else {
        return Err(CreateFixedFailure::Network);
    };
    if r.status == StatusCode::CREATED {
        return r.json::<FixedCreated>().ok_or(CreateFixedFailure::Network);
    }
    Err(match r.status {
        StatusCode::TOO_MANY_REQUESTS => CreateFixedFailure::RateLimited,
        StatusCode::BAD_REQUEST => CreateFixedFailure::Name,
        _ => CreateFixedFailure::Network,
    })
}

// ---------------------------------------------------------------- Suas salas

/// Por que uma sala fixa deixou de existir (o link responde 410).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GoneReason {
    Deleted,
    Expired,
}

/// Item de `POST /api/rooms/status`: só números, nunca apelidos.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "lowercase")]
pub enum RoomSummary {
    Ok { name: Option<String>, people: u32, streaming: u32 },
    Gone { reason: GoneReason },
    NotFound,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomStatus {
    pub code: String,
    #[serde(flatten)]
    pub summary: RoomSummary,
}

#[derive(Deserialize)]
struct CorpoStatus {
    rooms: Vec<RoomStatus>,
}

/// Quantas pessoas em cada sala, numa consulta só. `None` = não deu (rede, limite, resposta ruim).
pub async fn fetch_statuses(base: &str, codes: &[String]) -> Option<Vec<RoomStatus>> {
    if codes.is_empty() {
        return Some(Vec::new());
    }
    let corpo = json!({ "codes": codes }).to_string();
    let r = pedir(base, Method::POST, "/api/rooms/status", Some(corpo), None).await?;
    if !r.status.is_success() {
        return None;
    }
    r.json::<CorpoStatus>().map(|c| c.rooms)
}

// ---------------------------------------------------------------- ações de dono

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnerFailure {
    Forbidden,
    Gone,
    Name,
    RateLimited,
    Network,
}

/// Mesmos textos de `ownerErrorText` do site, caractere por caractere.
pub fn owner_error_text(motivo: OwnerFailure) -> &'static str {
    match motivo {
        OwnerFailure::Forbidden => "Só quem criou a sala pode fazer isso.",
        OwnerFailure::Gone => "Essa sala não existe mais.",
        OwnerFailure::Name => "Esse nome não dá. Use letras, números e espaços.",
        OwnerFailure::RateLimited => "Muitas mudanças daqui agora. Tenta de novo em um minuto.",
        OwnerFailure::Network => "Não deu pra falar com o servidor. Confere a internet e tenta de novo.",
    }
}

async fn pedido_de_dono(
    base: &str,
    codigo: &str,
    chave: &str,
    metodo: Method,
    corpo: Option<String>,
    sub: &str,
) -> Result<Resposta, OwnerFailure> {
    let caminho = format!("/api/rooms/{codigo}{sub}");
    let r = pedir(base, metodo, &caminho, corpo, Some(chave)).await.ok_or(OwnerFailure::Network)?;
    if r.status.is_success() {
        return Ok(r);
    }
    Err(match r.status {
        StatusCode::FORBIDDEN => OwnerFailure::Forbidden,
        StatusCode::NOT_FOUND | StatusCode::GONE => OwnerFailure::Gone,
        StatusCode::BAD_REQUEST => OwnerFailure::Name,
        StatusCode::TOO_MANY_REQUESTS => OwnerFailure::RateLimited,
        _ => OwnerFailure::Network,
    })
}

#[derive(Deserialize)]
struct CorpoNome {
    name: String,
}

/// `PATCH /api/rooms/:code`: renomeia a sala fixa; devolve o nome já normalizado pelo servidor.
pub async fn rename_room(base: &str, codigo: &str, chave: &str, nome: &str) -> Result<String, OwnerFailure> {
    let corpo = json!({ "name": nome }).to_string();
    let r = pedido_de_dono(base, codigo, chave, Method::PATCH, Some(corpo), "").await?;
    r.json::<CorpoNome>().map(|c| c.name).ok_or(OwnerFailure::Network)
}

/// `DELETE /api/rooms/:code`: apaga a sala fixa; quem está dentro sai com aviso.
pub async fn delete_room(base: &str, codigo: &str, chave: &str) -> Result<(), OwnerFailure> {
    pedido_de_dono(base, codigo, chave, Method::DELETE, None, "").await.map(|_| ())
}

/// `POST /api/rooms/:code/remove`: o dono tira alguém da sala.
pub async fn remove_person(base: &str, codigo: &str, chave: &str, peer_id: &str) -> Result<(), OwnerFailure> {
    let corpo = json!({ "peerId": peer_id }).to_string();
    pedido_de_dono(base, codigo, chave, Method::POST, Some(corpo), "/remove").await.map(|_| ())
}

// ---------------------------------------------------------------- prévia

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PreviewPerson {
    pub nickname: String,
    pub tone: Tone,
    pub live: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct FixedInfo {
    pub name: String,
}

/// Resposta de `GET /api/rooms/:code`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RoomPreview {
    pub code: String,
    pub people: Vec<PreviewPerson>,
    pub capacity: u32,
    /// Só em sala fixa.
    #[serde(default)]
    pub fixed: Option<FixedInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewResult {
    Ready(RoomPreview),
    Full(RoomPreview),
    NotFound,
    /// Sala fixa apagada ou expirada (410).
    Gone(GoneReason),
    /// Não deu pra perguntar (rede, limite): a tela esconde a prévia e deixa tentar.
    Unknown,
}

#[derive(Deserialize)]
struct CorpoMotivo {
    reason: String,
}

/// `GET /api/rooms/:code`.
pub async fn fetch_preview(base: &str, codigo: &str) -> PreviewResult {
    let Some(r) = pedir(base, Method::GET, &format!("/api/rooms/{codigo}"), None, None).await else {
        return PreviewResult::Unknown;
    };
    if r.status == StatusCode::NOT_FOUND {
        return PreviewResult::NotFound;
    }
    if r.status == StatusCode::GONE {
        return match r.json::<CorpoMotivo>() {
            // Como o site: qualquer motivo que não seja "expired" vale "deleted".
            Some(m) => PreviewResult::Gone(if m.reason == "expired" { GoneReason::Expired } else { GoneReason::Deleted }),
            None => PreviewResult::Unknown,
        };
    }
    if !r.status.is_success() {
        return PreviewResult::Unknown;
    }
    match r.json::<RoomPreview>() {
        Some(p) if p.people.len() as u64 >= u64::from(p.capacity) => PreviewResult::Full(p),
        Some(p) => PreviewResult::Ready(p),
        None => PreviewResult::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::testutil::{Req, resp, serve};
    use serde_json::Value;

    const CHAVE: &str = "ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12";

    fn corpo_json(req: &Req) -> Value {
        serde_json::from_str(&req.body).expect("corpo JSON")
    }

    #[tokio::test]
    async fn sala_rapida_manda_post_vazio_com_origin() {
        let fake = serve(|_| resp(201, r#"{"code":"abc-def"}"#)).await;
        assert_eq!(create_room(&fake.base).await, Ok("abc-def".to_string()));
        let req = fake.ultimo();
        assert_eq!((req.method.as_str(), req.path.as_str()), ("POST", "/api/rooms"));
        assert_eq!(req.header("origin"), Some(fake.base.as_str()));
        assert_eq!(req.body, "");
    }

    #[tokio::test]
    async fn sala_rapida_motivos_de_falha() {
        let fake = serve(|_| resp(429, r#"{"error":"rate-limited"}"#)).await;
        assert_eq!(create_room(&fake.base).await, Err(CreateFailure::RateLimited));
        let fake = serve(|_| resp(503, r#"{"error":"busy"}"#)).await;
        assert_eq!(create_room(&fake.base).await, Err(CreateFailure::Network));
        let fake = serve(|_| resp(403, r#"{"error":"origin"}"#)).await;
        assert_eq!(create_room(&fake.base).await, Err(CreateFailure::Network));
        let fake = serve(|_| resp(201, "não é json")).await;
        assert_eq!(create_room(&fake.base).await, Err(CreateFailure::Network));
        // porta fechada: erro de rede, sem pânico
        assert_eq!(create_room("http://127.0.0.1:9").await, Err(CreateFailure::Network));
    }

    #[tokio::test]
    async fn sala_fixa_corpo_origin_e_chave() {
        let fake = serve(|_| resp(201, &format!(r#"{{"code":"abc-def","ownerKey":"{CHAVE}"}}"#))).await;
        let ok = create_fixed_room(&fake.base, "Turma").await.unwrap();
        assert_eq!(ok, FixedCreated { code: "abc-def".into(), owner_key: CHAVE.into() });
        let req = fake.ultimo();
        assert_eq!(req.header("origin"), Some(fake.base.as_str()));
        assert_eq!(req.header("content-type"), Some("application/json"));
        assert_eq!(corpo_json(&req), json!({"kind":"fixed","name":"Turma"}));
        assert!(req.header("authorization").is_none());
    }

    #[tokio::test]
    async fn sala_fixa_motivos_de_falha() {
        for (status, esperado) in [
            (400, CreateFixedFailure::Name),
            (429, CreateFixedFailure::RateLimited),
            (503, CreateFixedFailure::Network),
            (403, CreateFixedFailure::Network),
        ] {
            let fake = serve(move |_| resp(status, "{}")).await;
            assert_eq!(create_fixed_room(&fake.base, "x").await, Err(esperado), "status {status}");
        }
    }

    #[tokio::test]
    async fn status_das_salas() {
        let corpo = r#"{"rooms":[
            {"code":"aaa-aaa","state":"ok","name":"Turma","people":2,"streaming":1},
            {"code":"bbb-bbb","state":"ok","name":null,"people":0,"streaming":0},
            {"code":"ccc-ccc","state":"gone","reason":"expired"},
            {"code":"ddd-ddd","state":"notfound"}]}"#;
        let fake = serve(move |_| resp(200, corpo)).await;
        let codes: Vec<String> = ["aaa-aaa", "bbb-bbb", "ccc-ccc", "ddd-ddd"].iter().map(|s| s.to_string()).collect();
        let salas = fetch_statuses(&fake.base, &codes).await.unwrap();
        assert_eq!(salas.len(), 4);
        assert_eq!(salas[0].summary, RoomSummary::Ok { name: Some("Turma".into()), people: 2, streaming: 1 });
        assert_eq!(salas[1].summary, RoomSummary::Ok { name: None, people: 0, streaming: 0 });
        assert_eq!(salas[2].summary, RoomSummary::Gone { reason: GoneReason::Expired });
        assert_eq!(salas[3], RoomStatus { code: "ddd-ddd".into(), summary: RoomSummary::NotFound });
        let req = fake.ultimo();
        assert_eq!((req.method.as_str(), req.path.as_str()), ("POST", "/api/rooms/status"));
        assert_eq!(req.header("origin"), Some(fake.base.as_str()));
        assert_eq!(corpo_json(&req), json!({"codes": codes}));
    }

    #[tokio::test]
    async fn status_sem_codigos_nao_chama_e_falha_vira_none() {
        let fake = serve(|_| resp(200, r#"{"rooms":[]}"#)).await;
        assert_eq!(fetch_statuses(&fake.base, &[]).await, Some(vec![]));
        assert!(fake.reqs.lock().unwrap().is_empty());
        let fake = serve(|_| resp(429, "{}")).await;
        assert_eq!(fetch_statuses(&fake.base, &["aaa-aaa".to_string()]).await, None);
        assert_eq!(fetch_statuses("http://127.0.0.1:9", &["aaa-aaa".to_string()]).await, None);
    }

    #[tokio::test]
    async fn dono_renomear_apagar_e_tirar() {
        let fake = serve(|req| match req.method.as_str() {
            "PATCH" => resp(200, r#"{"name":"Turma B"}"#),
            _ => resp(204, ""),
        })
        .await;
        assert_eq!(rename_room(&fake.base, "abc-def", CHAVE, "Turma  B").await, Ok("Turma B".to_string()));
        let req = fake.ultimo();
        assert_eq!((req.method.as_str(), req.path.as_str()), ("PATCH", "/api/rooms/abc-def"));
        assert_eq!(req.header("origin"), Some(fake.base.as_str()));
        assert_eq!(req.header("authorization"), Some(format!("Bearer {CHAVE}").as_str()));
        assert_eq!(corpo_json(&req), json!({"name":"Turma  B"}));

        assert_eq!(delete_room(&fake.base, "abc-def", CHAVE).await, Ok(()));
        let req = fake.ultimo();
        assert_eq!((req.method.as_str(), req.path.as_str()), ("DELETE", "/api/rooms/abc-def"));
        assert_eq!(req.header("origin"), Some(fake.base.as_str()));
        assert_eq!(req.header("authorization"), Some(format!("Bearer {CHAVE}").as_str()));

        assert_eq!(remove_person(&fake.base, "abc-def", CHAVE, "0123456789abcdef").await, Ok(()));
        let req = fake.ultimo();
        assert_eq!((req.method.as_str(), req.path.as_str()), ("POST", "/api/rooms/abc-def/remove"));
        assert_eq!(req.header("origin"), Some(fake.base.as_str()));
        assert_eq!(corpo_json(&req), json!({"peerId":"0123456789abcdef"}));
    }

    #[tokio::test]
    async fn dono_motivos_de_falha() {
        for (status, esperado) in [
            (403, OwnerFailure::Forbidden),
            (404, OwnerFailure::Gone),
            (410, OwnerFailure::Gone),
            (400, OwnerFailure::Name),
            (429, OwnerFailure::RateLimited),
            (500, OwnerFailure::Network),
            (503, OwnerFailure::Network),
        ] {
            let fake = serve(move |_| resp(status, "{}")).await;
            assert_eq!(delete_room(&fake.base, "abc-def", CHAVE).await, Err(esperado), "status {status}");
            assert_eq!(rename_room(&fake.base, "abc-def", CHAVE, "x").await, Err(esperado), "status {status}");
            assert_eq!(remove_person(&fake.base, "abc-def", CHAVE, "p").await, Err(esperado), "status {status}");
        }
        assert_eq!(delete_room("http://127.0.0.1:9", "abc-def", CHAVE).await, Err(OwnerFailure::Network));
        // 200 sem JSON de nome: não deu pra ler
        let fake = serve(|_| resp(200, "x")).await;
        assert_eq!(rename_room(&fake.base, "abc-def", CHAVE, "x").await, Err(OwnerFailure::Network));
    }

    #[test]
    fn textos_de_erro_iguais_aos_do_site() {
        assert_eq!(owner_error_text(OwnerFailure::Forbidden), "Só quem criou a sala pode fazer isso.");
        assert_eq!(owner_error_text(OwnerFailure::Gone), "Essa sala não existe mais.");
        assert_eq!(owner_error_text(OwnerFailure::Name), "Esse nome não dá. Use letras, números e espaços.");
        assert_eq!(owner_error_text(OwnerFailure::RateLimited), "Muitas mudanças daqui agora. Tenta de novo em um minuto.");
        assert_eq!(
            owner_error_text(OwnerFailure::Network),
            "Não deu pra falar com o servidor. Confere a internet e tenta de novo."
        );
    }

    #[tokio::test]
    async fn previa_pronta_cheia_fixa() {
        let pronta = r#"{"code":"abc-def","people":[{"nickname":"Ana","tone":"sage","live":true}],"capacity":10}"#;
        let fake = serve(move |_| resp(200, pronta)).await;
        match fetch_preview(&fake.base, "abc-def").await {
            PreviewResult::Ready(p) => {
                assert_eq!(p.people[0].nickname, "Ana");
                assert!(p.people[0].live && p.fixed.is_none());
            }
            outro => panic!("esperava Ready: {outro:?}"),
        }
        let req = fake.ultimo();
        assert_eq!((req.method.as_str(), req.path.as_str()), ("GET", "/api/rooms/abc-def"));

        let cheia = r#"{"code":"abc-def","people":[{"nickname":"A","tone":"sand","live":false},{"nickname":"B","tone":"deep","live":false}],
            "capacity":2,"fixed":{"name":"Turma"}}"#;
        let fake = serve(move |_| resp(200, cheia)).await;
        match fetch_preview(&fake.base, "abc-def").await {
            PreviewResult::Full(p) => assert_eq!(p.fixed, Some(FixedInfo { name: "Turma".into() })),
            outro => panic!("esperava Full: {outro:?}"),
        }
    }

    #[tokio::test]
    async fn previa_outros_resultados() {
        let fake = serve(|_| resp(404, r#"{"error":"not-found"}"#)).await;
        assert_eq!(fetch_preview(&fake.base, "abc-def").await, PreviewResult::NotFound);
        let fake = serve(|_| resp(410, r#"{"reason":"expired"}"#)).await;
        assert_eq!(fetch_preview(&fake.base, "abc-def").await, PreviewResult::Gone(GoneReason::Expired));
        let fake = serve(|_| resp(410, r#"{"reason":"deleted"}"#)).await;
        assert_eq!(fetch_preview(&fake.base, "abc-def").await, PreviewResult::Gone(GoneReason::Deleted));
        let fake = serve(|_| resp(410, r#"{"reason":"outra"}"#)).await;
        assert_eq!(fetch_preview(&fake.base, "abc-def").await, PreviewResult::Gone(GoneReason::Deleted));
        let fake = serve(|_| resp(410, "x")).await;
        assert_eq!(fetch_preview(&fake.base, "abc-def").await, PreviewResult::Unknown);
        for status in [400, 429, 500] {
            let fake = serve(move |_| resp(status, "{}")).await;
            assert_eq!(fetch_preview(&fake.base, "abc-def").await, PreviewResult::Unknown, "status {status}");
        }
        assert_eq!(fetch_preview("http://127.0.0.1:9", "abc-def").await, PreviewResult::Unknown);
    }
}
