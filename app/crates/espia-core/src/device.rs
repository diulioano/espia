//! Dados do aparelho do app nativo (`device.json`): modelo, leitura tolerante, validação, junção e
//! corte. Sem disco e sem relógio escondido: quem lê e grava o arquivo é o `espia-shell`, e o
//! "agora" entra sempre por parâmetro, em milissegundos desde 1970. Ver `specs/019-app-rust-convite`
//! (`data-model.md`, R9 e R12); a junção dá o mesmo resultado de `src/lib/origin-move.ts`.
use crate::room_id::is_valid_room_id;
use serde::Serialize;
use serde_json::{Map, Value};
use std::collections::BTreeMap;

// espelha src/shared/protocol.ts
pub const NICKNAME_MAX: usize = 24;
// espelha src/shared/protocol.ts
pub const ROOM_NAME_MAX: usize = 40;
// espelha src/shared/protocol.ts
pub const SAVED_ROOMS_MAX: usize = 5;

/// Uma sessão guardada vale 24 h (`SESSION_MAX_AGE_MS` de `src/lib/prefs.ts`).
pub const SESSION_MAX_AGE_MS: i64 = 24 * 60 * 60_000;

/// Superfície escolhida para transmitir. Qualquer outro texto lido vira `Window`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Surface {
    Window,
    Monitor,
    Camera,
}

/// Papel da entrada de som lembrada.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioRole {
    Voice,
    Game,
}

/// Preferências do seletor de transmissão (`GoLivePrefs`, `src/lib/types.ts`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoLive {
    /// 720 ou 1080.
    pub resolution: u16,
    /// 30 ou 60.
    pub fps: u16,
    pub audio: bool,
    pub surface: Surface,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_source_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_video_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_audio_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_audio_role: Option<AudioRole>,
}

impl Default for GoLive {
    /// `DEFAULT_GO_LIVE` de `src/lib/prefs.ts`.
    fn default() -> Self {
        GoLive {
            resolution: 1080,
            fps: 30,
            audio: true,
            surface: Surface::Window,
            last_source_key: None,
            last_video_label: None,
            last_audio_label: None,
            last_audio_role: None,
        }
    }
}

/// Sala fixa em que se entrou neste aparelho (`SavedRoom`, `src/lib/saved-rooms.ts`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedRoom {
    pub code: String,
    /// Último nome conhecido.
    pub name: String,
    /// Milissegundos desde 1970 da última entrada neste aparelho.
    pub last_used: i64,
}

/// Identidade para retomar o lugar na sala depois de uma queda ou atualização.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    /// 16 hex minúsculos.
    pub peer_id: String,
    pub resume_token: String,
    /// Momento da gravação, em milissegundos desde 1970.
    pub at: i64,
}

/// Conteúdo do `device.json`.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceData {
    /// Sempre 1.
    pub version: u32,
    /// Até 24 caracteres; `None` = nunca escolheu.
    pub nickname: Option<String>,
    /// `None` = usa `GoLive::default()` (ver `golive_or_default`). Fica `Option` para a junção
    /// saber se o destino já tinha o valor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub golive: Option<GoLive>,
    /// Apelido em minúsculas → volume entre 0 e 1.
    pub volumes: BTreeMap<String, f64>,
    /// No máximo 5, do `lastUsed` mais recente para o mais antigo.
    pub rooms: Vec<SavedRoom>,
    /// Código → chave de dono (64 hex). Nunca é cortada.
    pub owners: BTreeMap<String, String>,
    /// Código → sessão. Vale 24 h.
    pub sessions: BTreeMap<String, Session>,
    pub imported_from_webview: bool,
}

impl Default for DeviceData {
    fn default() -> Self {
        DeviceData {
            version: 1,
            nickname: None,
            golive: None,
            volumes: BTreeMap::new(),
            rooms: Vec::new(),
            owners: BTreeMap::new(),
            sessions: BTreeMap::new(),
            imported_from_webview: false,
        }
    }
}

// ---- Validação ----

/// Chave de dono: exatamente 64 hex minúsculos.
pub fn is_valid_owner_key(key: &str) -> bool {
    key.len() == 64 && key.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// Identificador de pessoa na sala: 16 hex minúsculos.
pub fn is_valid_peer_id(id: &str) -> bool {
    id.len() == 16 && id.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// Apelido: de 1 a 24 caracteres (o servidor recusa vazio).
pub fn is_valid_nickname(name: &str) -> bool {
    let n = name.chars().count();
    (1..=NICKNAME_MAX).contains(&n)
}

/// Volume: número finito de 0 a 1, inclusive.
pub fn is_valid_volume(v: f64) -> bool {
    v.is_finite() && (0.0..=1.0).contains(&v)
}

/// Sala guardada: código `xxx-xxx` e nome de até 40 caracteres (vazio passa, como no site).
pub fn is_valid_saved_room(r: &SavedRoom) -> bool {
    is_valid_room_id(&r.code) && r.name.chars().count() <= ROOM_NAME_MAX
}

fn is_valid_session(code: &str, s: &Session, now: i64) -> bool {
    is_valid_room_id(code) && is_valid_peer_id(&s.peer_id) && now.saturating_sub(s.at) <= SESSION_MAX_AGE_MS
}

fn is_valid_golive(g: &GoLive) -> bool {
    matches!(g.resolution, 720 | 1080) && matches!(g.fps, 30 | 60)
}

// ---- Salas (porte de src/lib/saved-rooms.ts) ----

/// Entrou numa sala fixa: vai para o topo. Passou de 5, sai a usada há mais tempo.
pub fn remember_room(list: Vec<SavedRoom>, code: &str, name: &str, now: i64) -> Vec<SavedRoom> {
    let mut out: Vec<SavedRoom> = Vec::with_capacity(list.len() + 1);
    out.push(SavedRoom { code: code.to_string(), name: name.to_string(), last_used: now });
    out.extend(list.into_iter().filter(|r| r.code != code));
    out.sort_by(|a, b| b.last_used.cmp(&a.last_used));
    out.truncate(SAVED_ROOMS_MAX);
    out
}

/// Tira da lista. Não tira a posse (`owners` fica como está).
pub fn forget_room(list: Vec<SavedRoom>, code: &str) -> Vec<SavedRoom> {
    list.into_iter().filter(|r| r.code != code).collect()
}

pub fn rename_saved(list: Vec<SavedRoom>, code: &str, name: &str) -> Vec<SavedRoom> {
    list.into_iter()
        .map(|r| if r.code == code { SavedRoom { name: name.to_string(), ..r } } else { r })
        .collect()
}

// ---- Leitura tolerante ----

fn number(v: &Value) -> Option<f64> {
    v.as_f64().filter(|n| n.is_finite())
}

/// `!!valor` do JavaScript.
fn js_truthy(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().is_some_and(|f| f != 0.0 && !f.is_nan()),
        Some(Value::String(s)) => !s.is_empty(),
        Some(_) => true,
    }
}

fn opt_string(o: &Map<String, Value>, key: &str) -> Option<String> {
    o.get(key).and_then(Value::as_str).map(str::to_string)
}

/// Regras de `loadGoLivePrefs`: resolução e fps inválidos invalidam o objeto inteiro.
fn parse_golive(v: &Value) -> Option<GoLive> {
    let o = v.as_object()?;
    let resolution = match number(o.get("resolution")?)? {
        r if r == 720.0 => 720,
        r if r == 1080.0 => 1080,
        _ => return None,
    };
    let fps = match number(o.get("fps")?)? {
        f if f == 30.0 => 30,
        f if f == 60.0 => 60,
        _ => return None,
    };
    Some(GoLive {
        resolution,
        fps,
        audio: js_truthy(o.get("audio")),
        surface: match o.get("surface").and_then(Value::as_str) {
            Some("monitor") => Surface::Monitor,
            Some("camera") => Surface::Camera,
            _ => Surface::Window,
        },
        last_source_key: opt_string(o, "lastSourceKey"),
        last_video_label: opt_string(o, "lastVideoLabel"),
        last_audio_label: opt_string(o, "lastAudioLabel"),
        last_audio_role: match o.get("lastAudioRole").and_then(Value::as_str) {
            Some("voice") => Some(AudioRole::Voice),
            Some("game") => Some(AudioRole::Game),
            _ => None,
        },
    })
}

fn parse_rooms(v: &Value) -> Vec<SavedRoom> {
    let Some(list) = v.as_array() else { return Vec::new() };
    list.iter()
        .filter_map(|item| {
            let o = item.as_object()?;
            Some(SavedRoom {
                code: o.get("code")?.as_str()?.to_string(),
                name: o.get("name")?.as_str()?.to_string(),
                last_used: number(o.get("lastUsed")?)? as i64,
            })
        })
        .collect()
}

fn parse_sessions(v: &Value) -> BTreeMap<String, Session> {
    let Some(map) = v.as_object() else { return BTreeMap::new() };
    map.iter()
        .filter_map(|(code, item)| {
            let o = item.as_object()?;
            let session = Session {
                peer_id: o.get("peerId")?.as_str()?.to_string(),
                resume_token: o.get("resumeToken")?.as_str()?.to_string(),
                at: number(o.get("at")?)? as i64,
            };
            Some((code.clone(), session))
        })
        .collect()
}

impl DeviceData {
    /// Lê o texto do `device.json`. Nunca falha: arquivo vazio, JSON inválido, que não é objeto ou
    /// com `version` diferente de 1 vira o padrão; campo faltando ou inválido vira o padrão só
    /// daquele campo; item inválido é ignorado. `version` faltando é aceita como 1.
    pub fn from_json(text: &str, now_ms: i64) -> DeviceData {
        let Ok(Value::Object(o)) = serde_json::from_str::<Value>(text) else { return DeviceData::default() };
        if let Some(version) = o.get("version") {
            if version.as_i64() != Some(1) {
                return DeviceData::default();
            }
        }
        let mut volumes = BTreeMap::new();
        if let Some(map) = o.get("volumes").and_then(Value::as_object) {
            for (name, v) in map {
                if let Some(n) = number(v) {
                    volumes.entry(name.to_lowercase()).or_insert(n);
                }
            }
        }
        let owners = o
            .get("owners")
            .and_then(Value::as_object)
            .map(|m| m.iter().filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string()))).collect())
            .unwrap_or_default();
        DeviceData {
            version: 1,
            nickname: o.get("nickname").and_then(Value::as_str).map(str::to_string),
            golive: o.get("golive").and_then(parse_golive),
            volumes,
            rooms: o.get("rooms").map(parse_rooms).unwrap_or_default(),
            owners,
            sessions: o.get("sessions").map(parse_sessions).unwrap_or_default(),
            imported_from_webview: o.get("importedFromWebview").and_then(Value::as_bool).unwrap_or(false),
        }
        .sanitize(now_ms)
    }

    /// Texto para gravar: já limpo (`sanitize`), então sessões velhas e itens inválidos não vão.
    pub fn to_json(&self, now_ms: i64) -> String {
        serde_json::to_string_pretty(&self.clone().sanitize(now_ms)).unwrap_or_else(|_| "{}".to_string())
    }

    /// As preferências de transmissão, ou o padrão se não há.
    pub fn golive_or_default(&self) -> GoLive {
        self.golive.clone().unwrap_or_default()
    }

    /// Tira o que é inválido, une salas repetidas (vale a de maior `lastUsed`), ordena as salas
    /// do mais recente para o mais antigo e corta em 5, e descarta sessões com mais de 24 h.
    /// `owners` nunca é cortada. Idempotente.
    pub fn sanitize(mut self, now_ms: i64) -> DeviceData {
        self.version = 1;
        self.nickname = self.nickname.filter(|n| is_valid_nickname(n));
        self.golive = self.golive.filter(is_valid_golive);

        let mut volumes = BTreeMap::new();
        for (name, v) in self.volumes {
            if is_valid_volume(v) {
                volumes.entry(name.to_lowercase()).or_insert(v);
            }
        }
        self.volumes = volumes;

        let mut rooms: Vec<SavedRoom> = Vec::new();
        for r in self.rooms.into_iter().filter(is_valid_saved_room) {
            match rooms.iter().position(|x| x.code == r.code) {
                Some(i) if r.last_used > rooms[i].last_used => rooms[i] = r,
                Some(_) => {}
                None => rooms.push(r),
            }
        }
        rooms.sort_by(|a, b| b.last_used.cmp(&a.last_used));
        rooms.truncate(SAVED_ROOMS_MAX);
        self.rooms = rooms;

        self.owners.retain(|code, key| is_valid_room_id(code) && is_valid_owner_key(key));
        self.sessions.retain(|code, s| is_valid_session(code, s, now_ms));
        self
    }
}

// ---- Junção ----

/// Junta o que veio de fora (app de antes, `#mv=`) ao que o aparelho já tem. Só acrescenta:
/// valor que o destino já tem fica. `rooms` é a união por código; no mesmo código vale a de maior
/// `lastUsed`, com o nome dela, e no empate fica a que já estava; depois ordena e corta em 5.
/// `owners` entra inteira, mesmo de sala que ficou fora da lista. Item inválido de qualquer lado
/// é ignorado. Idempotente: repetir a junção com o mesmo pacote não muda nada.
/// `importedFromWebview` fica como está no destino (quem a marca é o shell).
pub fn merge_device(current: DeviceData, incoming: &DeviceData, now_ms: i64) -> DeviceData {
    let mut out = current.sanitize(now_ms);
    let inc = incoming.clone().sanitize(now_ms);

    if out.nickname.is_none() {
        out.nickname = inc.nickname;
    }
    if out.golive.is_none() {
        out.golive = inc.golive;
    }
    for (k, v) in inc.volumes {
        out.volumes.entry(k).or_insert(v);
    }
    for (k, v) in inc.owners {
        out.owners.entry(k).or_insert(v);
    }
    for (k, v) in inc.sessions {
        out.sessions.entry(k).or_insert(v);
    }

    for room in inc.rooms {
        match out.rooms.iter().position(|r| r.code == room.code) {
            Some(i) if room.last_used > out.rooms[i].last_used => out.rooms[i] = room,
            Some(_) => {}
            None => out.rooms.push(room),
        }
    }
    out.rooms.sort_by(|a, b| b.last_used.cmp(&a.last_used));
    out.rooms.truncate(SAVED_ROOMS_MAX);
    out
}

#[cfg(test)]
mod tests {
    //! Testes do `device.json`. Portados de:
    //! - `test/lib/saved-rooms.test.ts` (13 casos; portados os 3 de `rememberRoom`/tirar/renomear,
    //!   os 10 restantes são de textos e painel e não pertencem a esta tarefa);
    //! - `test/lib/origin-move.test.ts` bloco `mergeMove` (7 casos, todos portados, adaptados ao
    //!   modelo tipado) e a parte de validação do `unpackMove` (itens inválidos ignorados);
    //!   o resto do arquivo (28 casos no total) é de empacotar/URL/base64 e fica no site.
    //! Mais os casos novos do formato do arquivo, da leitura tolerante e da validade das sessões.
    use super::*;
    use serde_json::json;

    const NOW: i64 = 1_762_860_000_000;
    const DAY: i64 = 24 * 60 * 60_000;
    const OWNER_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const OWNER_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn room(code: &str, last_used: i64) -> SavedRoom {
        SavedRoom { code: code.into(), name: format!("Sala {code}"), last_used }
    }

    fn named(code: &str, last_used: i64, name: &str) -> SavedRoom {
        SavedRoom { code: code.into(), name: name.into(), last_used }
    }

    fn with_rooms(rooms: Vec<SavedRoom>) -> DeviceData {
        DeviceData { rooms, ..DeviceData::default() }
    }

    fn codes(d: &DeviceData) -> Vec<&str> {
        d.rooms.iter().map(|r| r.code.as_str()).collect()
    }

    fn session(at: i64) -> Session {
        Session { peer_id: "0123456789abcdef".into(), resume_token: "tok".into(), at }
    }

    // ---- saved-rooms.test.ts: rememberRoom, tirar e renomear ----

    #[test]
    fn a_sala_nova_vai_pro_topo_e_a_6a_tira_a_usada_ha_mais_tempo() {
        let mut list: Vec<SavedRoom> = vec![];
        for (i, c) in ["aaa-aaa", "bbb-bbb", "ccc-ccc", "ddd-ddd", "eee-eee"].iter().enumerate() {
            list = remember_room(list, c, c, NOW - (10 - i as i64) * DAY);
        }
        list = remember_room(list, "fff-fff", "Nova", NOW);
        let codes: Vec<&str> = list.iter().map(|r| r.code.as_str()).collect();
        assert_eq!(codes, ["fff-fff", "eee-eee", "ddd-ddd", "ccc-ccc", "bbb-bbb"]);
    }

    #[test]
    fn entrar_de_novo_nao_duplica_so_atualiza_nome_e_uso() {
        let list = vec![named("aaa-aaa", NOW - 3 * DAY, "Velho"), named("bbb-bbb", NOW - DAY, "B")];
        let list = remember_room(list, "aaa-aaa", "Novo", NOW);
        assert_eq!(list, vec![named("aaa-aaa", NOW, "Novo"), named("bbb-bbb", NOW - DAY, "B")]);
    }

    #[test]
    fn tirar_e_renomear() {
        let list = vec![named("aaa-aaa", NOW - DAY, "A"), named("bbb-bbb", NOW - 2 * DAY, "B")];
        let sem = forget_room(list.clone(), "aaa-aaa");
        assert_eq!(sem.iter().map(|r| r.code.as_str()).collect::<Vec<_>>(), ["bbb-bbb"]);
        assert_eq!(rename_saved(list, "bbb-bbb", "B2")[1].name, "B2");
    }

    #[test]
    fn tirar_da_lista_nao_tira_a_posse() {
        let mut d = DeviceData::default();
        d.rooms = vec![room("aaa-aaa", 1)];
        d.owners.insert("aaa-aaa".into(), OWNER_A.into());
        d.rooms = forget_room(d.rooms, "aaa-aaa");
        assert!(d.rooms.is_empty());
        assert_eq!(d.owners.get("aaa-aaa").map(String::as_str), Some(OWNER_A));
    }

    // ---- origin-move.test.ts: mergeMove ----

    #[test]
    fn so_acrescenta_o_que_o_destino_ja_tem_fica() {
        let mut cur = DeviceData::default();
        cur.nickname = Some("Oficial".into());
        cur.golive = Some(GoLive { fps: 30, ..GoLive::default() });
        cur.volumes.insert("ana".into(), 0.2);
        cur.owners.insert("kkk-qqq".into(), OWNER_A.into());
        cur.sessions.insert("kkk-qqq".into(), Session { peer_id: "00000000000000aa".into(), ..session(NOW) });
        let mut inc = DeviceData::default();
        inc.nickname = Some("Antigo".into());
        inc.golive = Some(GoLive { fps: 60, ..GoLive::default() });
        inc.volumes.insert("ana".into(), 0.9);
        inc.owners.insert("kkk-qqq".into(), OWNER_B.into());
        inc.sessions.insert("kkk-qqq".into(), Session { peer_id: "00000000000000bb".into(), ..session(NOW) });
        assert_eq!(merge_device(cur.clone(), &inc, NOW), cur);
    }

    #[test]
    fn o_que_o_destino_nao_tem_entra() {
        let mut inc = DeviceData::default();
        inc.nickname = Some("Antigo".into());
        inc.golive = Some(GoLive { fps: 60, ..GoLive::default() });
        inc.volumes.insert("ana".into(), 0.9);
        inc.owners.insert("kkk-qqq".into(), OWNER_B.into());
        inc.sessions.insert("kkk-qqq".into(), session(NOW));
        assert_eq!(merge_device(DeviceData::default(), &inc, NOW), inc);

        // Só o que falta entra: o apelido do destino fica.
        let mut cur = DeviceData::default();
        cur.nickname = Some("Oficial".into());
        let out = merge_device(cur, &inc, NOW);
        assert_eq!(out.nickname.as_deref(), Some("Oficial"));
        assert_eq!(out.golive, inc.golive);
        assert_eq!(out.volumes, inc.volumes);
        assert_eq!(out.owners, inc.owners);
        assert_eq!(out.sessions, inc.sessions);
    }

    #[test]
    fn salas_uniao_por_codigo_mais_recente_primeiro() {
        let cur = with_rooms(vec![room("aaa-aaa", 30), room("bbb-bbb", 10)]);
        let inc = with_rooms(vec![room("ccc-ccc", 20), named("aaa-aaa", 5, "Nome velho")]);
        let out = merge_device(cur, &inc, NOW);
        assert_eq!(out.rooms, vec![room("aaa-aaa", 30), room("ccc-ccc", 20), room("bbb-bbb", 10)]);
    }

    #[test]
    fn mesmo_codigo_dos_dois_lados_fica_a_de_maior_last_used_com_o_nome_dela() {
        let cur = with_rooms(vec![named("aaa-aaa", 10, "Nome do oficial")]);
        let inc = with_rooms(vec![named("aaa-aaa", 20, "Nome do antigo")]);
        assert_eq!(merge_device(cur.clone(), &inc, NOW).rooms, vec![named("aaa-aaa", 20, "Nome do antigo")]);
        // Empate: fica a que já estava.
        let empate = with_rooms(vec![named("aaa-aaa", 10, "Nome do antigo")]);
        assert_eq!(merge_device(cur.clone(), &empate, NOW), cur);
    }

    #[test]
    fn corta_em_5_e_nunca_descarta_chave_de_dono_de_sala_que_ficou_fora() {
        let cur = with_rooms(vec![room("aaa-aaa", 60), room("bbb-bbb", 50), room("ccc-ccc", 40)]);
        let mut inc = with_rooms(vec![room("ddd-ddd", 55), room("eee-eee", 45), room("fff-fff", 1), room("ggg-ggg", 2)]);
        inc.owners.insert("fff-fff".into(), OWNER_A.into());
        inc.owners.insert("zzz-zzz".into(), OWNER_B.into());
        let out = merge_device(cur, &inc, NOW);
        assert_eq!(codes(&out), ["aaa-aaa", "ddd-ddd", "bbb-bbb", "eee-eee", "ccc-ccc"]);
        assert_eq!(out.owners.get("fff-fff").map(String::as_str), Some(OWNER_A));
        assert_eq!(out.owners.get("zzz-zzz").map(String::as_str), Some(OWNER_B));
    }

    #[test]
    fn lista_corrompida_no_destino_nao_impede_a_do_pacote_de_entrar() {
        // Entradas inválidas no destino (montado à mão, sem passar pela leitura) são ignoradas.
        let cur = with_rooms(vec![room("sala", 99), named("aaa-aaa", 5, &"x".repeat(41))]);
        let inc = with_rooms(vec![room("aaa-aaa", 10)]);
        assert_eq!(merge_device(cur, &inc, NOW).rooms, vec![room("aaa-aaa", 10)]);
    }

    #[test]
    fn pacote_sem_salas_nao_mexe_na_lista_do_destino() {
        let cur = with_rooms(vec![room("aaa-aaa", 10)]);
        let mut inc = DeviceData::default();
        inc.nickname = Some("Rafa".into());
        let out = merge_device(cur, &inc, NOW);
        assert_eq!(out.rooms, vec![room("aaa-aaa", 10)]);
        assert_eq!(out.nickname.as_deref(), Some("Rafa"));
    }

    #[test]
    fn juncao_e_idempotente() {
        let mut cur = with_rooms(vec![room("aaa-aaa", 30), room("bbb-bbb", 10)]);
        cur.nickname = Some("Oficial".into());
        cur.owners.insert("aaa-aaa".into(), OWNER_A.into());
        let mut inc = with_rooms(vec![
            room("ccc-ccc", 20),
            room("aaa-aaa", 40),
            room("ddd-ddd", 1),
            room("eee-eee", 2),
            room("fff-fff", 3),
            room("ggg-ggg", 4),
        ]);
        inc.nickname = Some("Antigo".into());
        inc.volumes.insert("ana".into(), 0.4);
        inc.owners.insert("ccc-ccc".into(), OWNER_B.into());
        inc.sessions.insert("ccc-ccc".into(), session(NOW));
        let uma = merge_device(cur, &inc, NOW);
        let duas = merge_device(uma.clone(), &inc, NOW);
        let tres = merge_device(duas.clone(), &inc, NOW);
        assert_eq!(uma, duas);
        assert_eq!(duas, tres);
        assert_eq!(uma.rooms.len(), 5);
        assert_eq!(uma.rooms[0], room("aaa-aaa", 40));
    }

    #[test]
    fn juncao_ignora_itens_invalidos_do_pacote() {
        let mut inc = DeviceData::default();
        inc.nickname = Some("x".repeat(25));
        inc.owners.insert("kkk-qqq".into(), OWNER_A.into());
        inc.owners.insert("abc-def".into(), OWNER_A.to_uppercase());
        inc.owners.insert("xyz-234".into(), "a".repeat(63));
        inc.owners.insert("K7F-Q2X".into(), OWNER_A.into());
        inc.owners.insert("sala".into(), OWNER_A.into());
        inc.volumes.insert("ana".into(), 0.5);
        inc.volumes.insert("bia".into(), 1.5);
        inc.volumes.insert("caio".into(), -0.1);
        inc.volumes.insert("duda".into(), f64::NAN);
        inc.rooms = vec![room("kkk-qqq", 10), named("abc-def", 9, &"x".repeat(41)), room("sala", 8), room("mnp-789", 5)];
        inc.sessions.insert("nada".into(), session(NOW));
        inc.sessions.insert("kkk-qqq".into(), Session { peer_id: "curto".into(), ..session(NOW) });
        let out = merge_device(DeviceData::default(), &inc, NOW);
        assert_eq!(out.nickname, None);
        assert_eq!(out.owners.keys().map(String::as_str).collect::<Vec<_>>(), ["kkk-qqq"]);
        assert_eq!(out.volumes.keys().map(String::as_str).collect::<Vec<_>>(), ["ana"]);
        assert_eq!(codes(&out), ["kkk-qqq", "mnp-789"]);
        assert!(out.sessions.is_empty());
    }

    #[test]
    fn limites_validos_passam_apelido_24_nome_40_volume_0_e_1() {
        let mut inc = DeviceData::default();
        inc.nickname = Some("x".repeat(24));
        inc.volumes.insert("ana".into(), 0.0);
        inc.volumes.insert("bia".into(), 1.0);
        inc.rooms = vec![named("aaa-aaa", 1, &"x".repeat(40))];
        let out = merge_device(DeviceData::default(), &inc, NOW);
        assert_eq!(out, inc);
    }

    #[test]
    fn apelido_conta_caracteres_nao_bytes() {
        let mut inc = DeviceData::default();
        inc.nickname = Some("\u{1F3AE}".repeat(24));
        assert!(merge_device(DeviceData::default(), &inc, NOW).nickname.is_some());
        inc.nickname = Some("\u{1F3AE}".repeat(25));
        assert!(merge_device(DeviceData::default(), &inc, NOW).nickname.is_none());
        inc.nickname = Some(String::new());
        assert!(merge_device(DeviceData::default(), &inc, NOW).nickname.is_none());
    }

    // ---- Formato do arquivo ----

    #[test]
    fn padrao_serializa_em_camel_case_com_version_1() {
        let v: serde_json::Value = serde_json::from_str(&DeviceData::default().to_json(NOW)).unwrap();
        assert_eq!(v["version"], 1);
        assert_eq!(v["nickname"], serde_json::Value::Null);
        assert_eq!(v["rooms"], json!([]));
        assert_eq!(v["owners"], json!({}));
        assert_eq!(v["sessions"], json!({}));
        assert_eq!(v["volumes"], json!({}));
        assert_eq!(v["importedFromWebview"], false);
        assert!(v.get("imported_from_webview").is_none());
    }

    #[test]
    fn grava_nomes_do_data_model() {
        let mut d = DeviceData::default();
        d.nickname = Some("Rafa".into());
        d.golive = Some(GoLive { audio: false, surface: Surface::Monitor, last_source_key: Some("m:Tela 1".into()), last_audio_role: Some(AudioRole::Game), ..GoLive::default() });
        d.rooms = vec![named("k7f-q2x", NOW, "Noite de jogo")];
        d.owners.insert("k7f-q2x".into(), OWNER_A.into());
        d.sessions.insert("k7f-q2x".into(), session(NOW));
        d.imported_from_webview = true;
        let v: serde_json::Value = serde_json::from_str(&d.to_json(NOW)).unwrap();
        assert_eq!(v["rooms"], json!([{ "code": "k7f-q2x", "name": "Noite de jogo", "lastUsed": NOW }]));
        assert_eq!(v["sessions"]["k7f-q2x"], json!({ "peerId": "0123456789abcdef", "resumeToken": "tok", "at": NOW }));
        assert_eq!(v["golive"], json!({ "resolution": 1080, "fps": 30, "audio": false, "surface": "monitor", "lastSourceKey": "m:Tela 1", "lastAudioRole": "game" }));
        assert_eq!(v["importedFromWebview"], true);
    }

    #[test]
    fn ida_e_volta_nao_muda_nada() {
        let mut d = with_rooms(vec![named("k7f-q2x", NOW, "Noite de jogo"), room("abc-def", NOW - 5)]);
        d.nickname = Some("Rafa".into());
        d.golive = Some(GoLive { fps: 60, last_video_label: Some("Câmera".into()), ..GoLive::default() });
        d.volumes.insert("ana".into(), 0.75);
        d.owners.insert("k7f-q2x".into(), OWNER_A.into());
        d.sessions.insert("k7f-q2x".into(), session(NOW - 1000));
        d.imported_from_webview = true;
        assert_eq!(DeviceData::from_json(&d.to_json(NOW), NOW), d);
    }

    #[test]
    fn arquivo_ausente_vazio_ou_ilegivel_vira_o_padrao() {
        assert_eq!(DeviceData::from_json("", NOW), DeviceData::default());
        assert_eq!(DeviceData::from_json("   ", NOW), DeviceData::default());
        assert_eq!(DeviceData::from_json("{", NOW), DeviceData::default());
        assert_eq!(DeviceData::from_json("null", NOW), DeviceData::default());
        assert_eq!(DeviceData::from_json("[1,2]", NOW), DeviceData::default());
        assert_eq!(DeviceData::from_json("\"texto\"", NOW), DeviceData::default());
    }

    #[test]
    fn version_desconhecida_e_arquivo_ilegivel() {
        let texto = json!({ "version": 2, "nickname": "Rafa", "importedFromWebview": true }).to_string();
        assert_eq!(DeviceData::from_json(&texto, NOW), DeviceData::default());
        let texto = json!({ "version": "1", "nickname": "Rafa" }).to_string();
        assert_eq!(DeviceData::from_json(&texto, NOW), DeviceData::default());
    }

    #[test]
    fn version_faltando_e_campo_faltando_nao_derruba_o_resto() {
        let d = DeviceData::from_json(&json!({ "nickname": "Rafa" }).to_string(), NOW);
        assert_eq!(d.nickname.as_deref(), Some("Rafa"));
        assert_eq!(d.version, 1);
        assert!(d.rooms.is_empty());
        assert_eq!(d.golive, None);
        assert_eq!(d.golive_or_default(), GoLive::default());
    }

    #[test]
    fn campo_com_tipo_errado_usa_o_padrao_so_daquele_campo() {
        let texto = json!({
            "version": 1,
            "nickname": 7,
            "golive": "x",
            "volumes": [1],
            "rooms": "k7f-q2x",
            "owners": 3,
            "sessions": null,
            "importedFromWebview": "sim",
        })
        .to_string();
        assert_eq!(DeviceData::from_json(&texto, NOW), DeviceData::default());
        let texto = json!({ "nickname": 7, "owners": { "k7f-q2x": OWNER_A } }).to_string();
        let d = DeviceData::from_json(&texto, NOW);
        assert_eq!(d.nickname, None);
        assert_eq!(d.owners.len(), 1);
    }

    #[test]
    fn campos_desconhecidos_sao_ignorados() {
        let d = DeviceData::from_json(&json!({ "version": 1, "nickname": "Rafa", "futuro": { "a": 1 } }).to_string(), NOW);
        assert_eq!(d.nickname.as_deref(), Some("Rafa"));
    }

    #[test]
    fn leitura_tira_itens_invalidos_sem_derrubar_os_outros() {
        let texto = json!({
            "version": 1,
            "volumes": { "ana": 0.5, "bia": 1.5, "caio": -0.1, "duda": "0.5", "edu": null, "fim": 1, "zero": 0 },
            "rooms": [
                { "code": "k7f-q2x", "name": "x".repeat(40), "lastUsed": 10 },
                { "code": "abc-def", "name": "x".repeat(41), "lastUsed": 9 },
                { "code": "sala", "name": "A", "lastUsed": 8 },
                { "code": "xyz-234", "name": "Sem data" },
                { "code": "xyz-234", "name": 7, "lastUsed": 6 },
                null,
                { "code": "mnp-789", "name": "B", "lastUsed": 5 },
            ],
            "owners": { "k7f-q2x": OWNER_A, "abc-def": OWNER_A.to_uppercase(), "xyz-234": "a".repeat(63), "sala": OWNER_A, "mnp-789": 5 },
            "sessions": {
                "k7f-q2x": { "peerId": "0123456789abcdef", "resumeToken": "t", "at": NOW },
                "abc-def": { "peerId": "0123456789ABCDEF", "resumeToken": "t", "at": NOW },
                "xyz-234": { "peerId": "0123456789abcdef", "at": NOW },
                "mnp-789": { "peerId": "0123456789abcdef", "resumeToken": "t" },
                "sala": { "peerId": "0123456789abcdef", "resumeToken": "t", "at": NOW },
            },
        })
        .to_string();
        let d = DeviceData::from_json(&texto, NOW);
        assert_eq!(d.volumes.keys().map(String::as_str).collect::<Vec<_>>(), ["ana", "fim", "zero"]);
        assert_eq!(codes(&d), ["k7f-q2x", "mnp-789"]);
        assert_eq!(d.owners.keys().map(String::as_str).collect::<Vec<_>>(), ["k7f-q2x"]);
        assert_eq!(d.sessions.keys().map(String::as_str).collect::<Vec<_>>(), ["k7f-q2x"]);
    }

    #[test]
    fn leitura_ordena_corta_em_5_e_une_repetidos_mas_nao_corta_donos() {
        let mut rooms = vec![];
        for (i, c) in ["aaa-aaa", "bbb-bbb", "ccc-ccc", "ddd-ddd", "eee-eee", "fff-fff", "ggg-ggg"].iter().enumerate() {
            rooms.push(json!({ "code": c, "name": c, "lastUsed": i as i64 }));
        }
        rooms.push(json!({ "code": "aaa-aaa", "name": "Repetida", "lastUsed": 99 }));
        let mut owners = serde_json::Map::new();
        for c in ["aaa-aaa", "bbb-bbb", "ccc-ccc", "ddd-ddd", "eee-eee", "fff-fff", "ggg-ggg"] {
            owners.insert(c.into(), json!(OWNER_A));
        }
        let texto = json!({ "version": 1, "rooms": rooms, "owners": owners }).to_string();
        let d = DeviceData::from_json(&texto, NOW);
        assert_eq!(codes(&d), ["aaa-aaa", "ggg-ggg", "fff-fff", "eee-eee", "ddd-ddd"]);
        assert_eq!(d.rooms[0].name, "Repetida");
        assert_eq!(d.owners.len(), 7);
    }

    #[test]
    fn volume_chave_vira_minuscula() {
        let d = DeviceData::from_json(&json!({ "volumes": { "Ana": 0.3 } }).to_string(), NOW);
        assert_eq!(d.volumes.get("ana"), Some(&0.3));
    }

    #[test]
    fn sessao_vale_24_horas() {
        let texto = json!({ "sessions": {
            "aaa-aaa": { "peerId": "0123456789abcdef", "resumeToken": "t", "at": NOW - DAY },
            "bbb-bbb": { "peerId": "0123456789abcdef", "resumeToken": "t", "at": NOW - DAY - 1 },
            "ccc-ccc": { "peerId": "0123456789abcdef", "resumeToken": "t", "at": NOW + 5 },
        } })
        .to_string();
        let d = DeviceData::from_json(&texto, NOW);
        assert_eq!(d.sessions.keys().map(String::as_str).collect::<Vec<_>>(), ["aaa-aaa", "ccc-ccc"]);
        assert_eq!(SESSION_MAX_AGE_MS, DAY);
    }

    #[test]
    fn gravacao_descarta_sessao_velha() {
        let mut d = DeviceData::default();
        d.sessions.insert("aaa-aaa".into(), session(NOW - 2 * DAY));
        d.sessions.insert("bbb-bbb".into(), session(NOW));
        let v: serde_json::Value = serde_json::from_str(&d.to_json(NOW)).unwrap();
        assert!(v["sessions"].get("aaa-aaa").is_none());
        assert!(v["sessions"].get("bbb-bbb").is_some());
    }

    #[test]
    fn gravacao_corta_salas_e_nao_corta_donos() {
        let mut d = DeviceData::default();
        for (i, c) in ["aaa-aaa", "bbb-bbb", "ccc-ccc", "ddd-ddd", "eee-eee", "fff-fff"].iter().enumerate() {
            d.rooms.push(room(c, i as i64));
            d.owners.insert((*c).into(), OWNER_A.into());
        }
        let volta = DeviceData::from_json(&d.to_json(NOW), NOW);
        assert_eq!(codes(&volta), ["fff-fff", "eee-eee", "ddd-ddd", "ccc-ccc", "bbb-bbb"]);
        assert_eq!(volta.owners.len(), 6);
    }

    // ---- golive ----

    #[test]
    fn golive_valido_e_lido_como_o_site() {
        let texto = json!({ "golive": {
            "resolution": 720, "fps": 60, "audio": 1, "surface": "camera",
            "lastSourceKey": "app|titulo", "lastVideoLabel": "Cam", "lastAudioLabel": "Mic", "lastAudioRole": "voice",
        } })
        .to_string();
        let g = DeviceData::from_json(&texto, NOW).golive.unwrap();
        assert_eq!(g.resolution, 720);
        assert_eq!(g.fps, 60);
        assert!(g.audio);
        assert_eq!(g.surface, Surface::Camera);
        assert_eq!(g.last_source_key.as_deref(), Some("app|titulo"));
        assert_eq!(g.last_video_label.as_deref(), Some("Cam"));
        assert_eq!(g.last_audio_label.as_deref(), Some("Mic"));
        assert_eq!(g.last_audio_role, Some(AudioRole::Voice));
    }

    #[test]
    fn golive_com_resolucao_ou_fps_invalidos_e_ignorado_inteiro() {
        for g in [
            json!({ "resolution": 480, "fps": 30, "audio": true }),
            json!({ "resolution": 1080, "fps": 24, "audio": true }),
            json!({ "fps": 30 }),
            json!({ "resolution": "1080", "fps": 30 }),
            json!(null),
        ] {
            let d = DeviceData::from_json(&json!({ "golive": g }).to_string(), NOW);
            assert_eq!(d.golive, None);
            assert_eq!(d.golive_or_default(), GoLive { resolution: 1080, fps: 30, audio: true, surface: Surface::Window, ..GoLive::default() });
        }
    }

    #[test]
    fn golive_audio_e_surface_seguem_as_regras_do_site() {
        let g = |v: serde_json::Value| DeviceData::from_json(&json!({ "golive": v }).to_string(), NOW).golive.unwrap();
        // audio ausente = false (`!!undefined`); texto vazio e 0 são falsos.
        assert!(!g(json!({ "resolution": 1080, "fps": 30 })).audio);
        assert!(!g(json!({ "resolution": 1080, "fps": 30, "audio": 0 })).audio);
        assert!(!g(json!({ "resolution": 1080, "fps": 30, "audio": "" })).audio);
        assert!(g(json!({ "resolution": 1080, "fps": 30, "audio": "sim" })).audio);
        // surface desconhecida vira window; opcionais com tipo errado somem.
        let x = g(json!({ "resolution": 1080, "fps": 30, "surface": "tela", "lastSourceKey": 5, "lastAudioRole": "outro" }));
        assert_eq!(x.surface, Surface::Window);
        assert_eq!(x.last_source_key, None);
        assert_eq!(x.last_audio_role, None);
        assert_eq!(g(json!({ "resolution": 1080, "fps": 30, "surface": "monitor" })).surface, Surface::Monitor);
    }

    #[test]
    fn padrao_do_golive() {
        assert_eq!(GoLive::default(), GoLive { resolution: 1080, fps: 30, audio: true, surface: Surface::Window, last_source_key: None, last_video_label: None, last_audio_label: None, last_audio_role: None });
    }

    #[test]
    fn constantes_espelham_o_protocolo() {
        assert_eq!(NICKNAME_MAX, 24);
        assert_eq!(ROOM_NAME_MAX, 40);
        assert_eq!(SAVED_ROOMS_MAX, 5);
    }
}
