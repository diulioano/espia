//! Arquivos do aparelho (R9, data-model.md): `device.json`, `tray.json`, `window.json` e `resume.json`.
//!
//! Regras:
//! - A pasta base é parâmetro; em produção vem de [`pasta_do_app`] (`%APPDATA%\dev.espia.app\`).
//! - Escrita atômica: grava num temporário na mesma pasta, sincroniza e renomeia por cima.
//! - Arquivo ausente, vazio ou inválido vira o padrão, sem erro e sem apagar o arquivo.
//! - Campos desconhecidos são IGNORADOS na leitura (como o `parse_prefs` do app de antes) e,
//!   por isso, não sobrevivem a uma regravação. Foi a escolha para manter o `tray.json` idêntico
//!   ao que o app de antes escreve.
//! - Nada daqui vai à rede.

use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};

pub const NOME_DEVICE: &str = "device.json";
pub const NOME_TRAY: &str = "tray.json";
pub const NOME_WINDOW: &str = "window.json";
pub const NOME_RESUME: &str = "resume.json";

/// Pasta dos dados do app: `%APPDATA%\dev.espia.app\`. `None` se `APPDATA` não existir ou estiver vazia.
pub fn pasta_do_app() -> Option<PathBuf> {
    let appdata = std::env::var_os("APPDATA").filter(|v| !v.is_empty())?;
    Some(PathBuf::from(appdata).join("dev.espia.app"))
}

/// Lê `base/nome`. Ausente, vazio ou inválido vira `T::default()`; o arquivo nunca é tocado.
pub fn ler<T: DeserializeOwned + Default>(base: &Path, nome: &str) -> T {
    fs::read_to_string(base.join(nome))
        .ok()
        .and_then(|texto| serde_json::from_str::<T>(&texto).ok())
        .unwrap_or_default()
}

/// Grava `valor` em `base/nome` de forma atômica. Cria a pasta se preciso.
pub fn gravar<T: Serialize>(base: &Path, nome: &str, valor: &T) -> io::Result<()> {
    let texto = serde_json::to_string(valor).map_err(io::Error::other)?;
    gravar_texto(base, nome, &texto)
}

/// Apaga `base/nome`. Arquivo que já não existe não é erro.
pub fn apagar(base: &Path, nome: &str) -> io::Result<()> {
    match fs::remove_file(base.join(nome)) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
        _ => Ok(()),
    }
}

static CONTADOR: AtomicU64 = AtomicU64::new(0);

fn gravar_texto(base: &Path, nome: &str, texto: &str) -> io::Result<()> {
    fs::create_dir_all(base)?;
    let destino = base.join(nome);
    // Nome único (processo + contador) para duas gravações simultâneas não se pisarem.
    let n = CONTADOR.fetch_add(1, Ordering::Relaxed);
    let temporario = base.join(format!("{nome}.{}.{n}.tmp", std::process::id()));
    let resultado = (|| {
        let mut f = File::create(&temporario)?;
        f.write_all(texto.as_bytes())?;
        f.flush()?;
        f.sync_all()?;
        drop(f);
        fs::rename(&temporario, &destino)
    })();
    if resultado.is_err() {
        let _ = fs::remove_file(&temporario);
    }
    resultado
}

// ---------------------------------------------------------------- tray.json

/// Preferências da bandeja. Mesmo formato do app de antes: `{ "closeOnX": bool, "trayNoticeShown": bool }`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TrayPrefs {
    /// `false` (padrão): o X esconde. `true`: o X fecha.
    pub close_on_x: bool,
    /// O aviso da primeira vez já apareceu.
    pub tray_notice_shown: bool,
}

/// Mesma regra do `parse_prefs` de antes: JSON inválido ou que não é objeto vira o padrão;
/// campo que não é booleano vira `false`; campo desconhecido é ignorado.
pub fn parse_tray(texto: &str) -> TrayPrefs {
    let Ok(v) = serde_json::from_str::<Value>(texto) else {
        return TrayPrefs::default();
    };
    let flag = |k: &str| v.get(k).and_then(Value::as_bool).unwrap_or(false);
    TrayPrefs { close_on_x: flag("closeOnX"), tray_notice_shown: flag("trayNoticeShown") }
}

/// Texto idêntico ao `prefs_json` de antes.
pub fn tray_json(p: &TrayPrefs) -> String {
    json!({ "closeOnX": p.close_on_x, "trayNoticeShown": p.tray_notice_shown }).to_string()
}

pub fn ler_tray(base: &Path) -> TrayPrefs {
    fs::read_to_string(base.join(NOME_TRAY)).map(|t| parse_tray(&t)).unwrap_or_default()
}

pub fn gravar_tray(base: &Path, prefs: &TrayPrefs) -> io::Result<()> {
    gravar_texto(base, NOME_TRAY, &tray_json(prefs))
}

// -------------------------------------------------------------- window.json

/// Posição e tamanho da janela. Padrão 1280x800, sem posição guardada (o chamador centraliza).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct JanelaSalva {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub maximized: bool,
}

impl Default for JanelaSalva {
    fn default() -> Self {
        Self { x: 0, y: 0, width: 1280, height: 800, maximized: false }
    }
}

pub fn ler_janela(base: &Path) -> JanelaSalva {
    ler(base, NOME_WINDOW)
}

pub fn gravar_janela(base: &Path, janela: &JanelaSalva) -> io::Result<()> {
    gravar(base, NOME_WINDOW, janela)
}

// -------------------------------------------------------------- resume.json

/// Volta à sala depois de atualizar: `{ "room": "<código>", "hidden": bool, "at": ms }`.
/// O padrão (sala vazia) significa "nada para retomar". Validar o código e os 5 minutos de
/// validade não é papel desta camada.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Retomada {
    pub room: String,
    pub hidden: bool,
    pub at: u64,
}

pub fn ler_retomada(base: &Path) -> Retomada {
    ler(base, NOME_RESUME)
}

pub fn gravar_retomada(base: &Path, retomada: &Retomada) -> io::Result<()> {
    gravar(base, NOME_RESUME, retomada)
}

// -------------------------------------------------------------------- testes

#[cfg(test)]
mod testes {
    use super::*;

    /// Pasta temporária própria, apagada ao sair do escopo.
    struct Pasta(PathBuf);

    impl Pasta {
        fn nova() -> Self {
            let n = CONTADOR.fetch_add(1, Ordering::Relaxed);
            let p = std::env::temp_dir().join(format!("espia-storage-{}-{n}", std::process::id()));
            let _ = fs::remove_dir_all(&p);
            Self(p)
        }
    }

    impl Drop for Pasta {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
    #[serde(default)]
    struct Aparelho {
        nickname: String,
        volume: u32,
    }

    #[test]
    fn device_ida_e_volta_e_cria_a_pasta() {
        let p = Pasta::nova();
        let base = p.0.join("fundo").join("dev.espia.app");
        let v = Aparelho { nickname: "Sala".into(), volume: 7 };
        gravar(&base, NOME_DEVICE, &v).unwrap();
        assert_eq!(ler::<Aparelho>(&base, NOME_DEVICE), v);
    }

    #[test]
    fn tray_ida_e_volta() {
        let p = Pasta::nova();
        let t = TrayPrefs { close_on_x: true, tray_notice_shown: true };
        gravar_tray(&p.0, &t).unwrap();
        assert_eq!(ler_tray(&p.0), t);
    }

    #[test]
    fn janela_ida_e_volta() {
        let p = Pasta::nova();
        let j = JanelaSalva { x: -20, y: 80, width: 900, height: 600, maximized: true };
        gravar_janela(&p.0, &j).unwrap();
        assert_eq!(ler_janela(&p.0), j);
    }

    #[test]
    fn retomada_ida_e_volta() {
        let p = Pasta::nova();
        let r = Retomada { room: "abc-def".into(), hidden: true, at: 1762860000000 };
        gravar_retomada(&p.0, &r).unwrap();
        assert_eq!(ler_retomada(&p.0), r);
        let texto = fs::read_to_string(p.0.join(NOME_RESUME)).unwrap();
        let v: Value = serde_json::from_str(&texto).unwrap();
        assert_eq!(v, json!({ "room": "abc-def", "hidden": true, "at": 1762860000000u64 }));
    }

    #[test]
    fn arquivo_ausente_vira_padrao() {
        let p = Pasta::nova();
        assert_eq!(ler_tray(&p.0), TrayPrefs::default());
        assert_eq!(ler_janela(&p.0), JanelaSalva::default());
        assert_eq!(ler_retomada(&p.0), Retomada::default());
        assert_eq!(ler::<Aparelho>(&p.0, NOME_DEVICE), Aparelho::default());
    }

    #[test]
    fn arquivo_vazio_ou_corrompido_vira_padrao_sem_apagar() {
        let p = Pasta::nova();
        fs::create_dir_all(&p.0).unwrap();
        for (nome, conteudo) in [
            (NOME_TRAY, "{ isso não é json"),
            (NOME_WINDOW, ""),
            (NOME_RESUME, "[1, 2, 3]"),
            (NOME_DEVICE, "\u{0}\u{0}\u{0}"),
        ] {
            fs::write(p.0.join(nome), conteudo).unwrap();
        }
        assert_eq!(ler_tray(&p.0), TrayPrefs::default());
        assert_eq!(ler_janela(&p.0), JanelaSalva::default());
        assert_eq!(ler_retomada(&p.0), Retomada::default());
        assert_eq!(ler::<Aparelho>(&p.0, NOME_DEVICE), Aparelho::default());
        // O arquivo ruim continua lá, intacto.
        assert_eq!(fs::read_to_string(p.0.join(NOME_TRAY)).unwrap(), "{ isso não é json");
        assert_eq!(fs::read_to_string(p.0.join(NOME_RESUME)).unwrap(), "[1, 2, 3]");
        assert!(p.0.join(NOME_WINDOW).exists());
    }

    #[test]
    fn campo_desconhecido_e_ignorado_sem_erro() {
        let p = Pasta::nova();
        fs::create_dir_all(&p.0).unwrap();
        fs::write(p.0.join(NOME_WINDOW), r#"{"x":5,"y":6,"width":800,"height":500,"maximized":true,"futuro":1}"#).unwrap();
        fs::write(p.0.join(NOME_RESUME), r#"{"room":"aaa-bbb","hidden":false,"at":9,"extra":"x"}"#).unwrap();
        fs::write(p.0.join(NOME_TRAY), r#"{"closeOnX":true,"trayNoticeShown":true,"novo":42}"#).unwrap();
        assert_eq!(ler_janela(&p.0), JanelaSalva { x: 5, y: 6, width: 800, height: 500, maximized: true });
        assert_eq!(ler_retomada(&p.0), Retomada { room: "aaa-bbb".into(), hidden: false, at: 9 });
        assert_eq!(ler_tray(&p.0), TrayPrefs { close_on_x: true, tray_notice_shown: true });
    }

    #[test]
    fn tray_no_formato_do_app_de_antes_e_lido_certo() {
        // Texto exato que o `prefs_json` do app de antes escreve.
        let p = Pasta::nova();
        fs::create_dir_all(&p.0).unwrap();
        fs::write(p.0.join(NOME_TRAY), r#"{"closeOnX":true,"trayNoticeShown":false}"#).unwrap();
        assert_eq!(ler_tray(&p.0), TrayPrefs { close_on_x: true, tray_notice_shown: false });
        // Campo que não é booleano vira false; objeto parcial completa com false.
        assert_eq!(parse_tray(r#"{"closeOnX":"sim","trayNoticeShown":true}"#), TrayPrefs { close_on_x: false, tray_notice_shown: true });
        assert_eq!(parse_tray("{}"), TrayPrefs::default());
        // E o que o app nativo escreve tem a mesma forma, legível pelo app de antes.
        let t = TrayPrefs { close_on_x: false, tray_notice_shown: true };
        assert_eq!(tray_json(&t), r#"{"closeOnX":false,"trayNoticeShown":true}"#);
        gravar_tray(&p.0, &t).unwrap();
        assert_eq!(fs::read_to_string(p.0.join(NOME_TRAY)).unwrap(), r#"{"closeOnX":false,"trayNoticeShown":true}"#);
    }

    #[test]
    fn temporario_orfao_nao_estraga_o_arquivo_final() {
        // Simula uma escrita interrompida: sobrou um temporário pela metade ao lado do arquivo bom.
        let p = Pasta::nova();
        let bom = JanelaSalva { x: 1, y: 2, width: 1000, height: 700, maximized: false };
        gravar_janela(&p.0, &bom).unwrap();
        fs::write(p.0.join("window.json.99999.0.tmp"), r#"{"x":9,"y"#).unwrap();
        fs::write(p.0.join("window.json.tmp"), "lixo").unwrap();

        assert_eq!(ler_janela(&p.0), bom);

        // Uma nova gravação continua funcionando e não deixa temporário próprio para trás.
        let novo = JanelaSalva { x: 3, ..bom };
        gravar_janela(&p.0, &novo).unwrap();
        assert_eq!(ler_janela(&p.0), novo);
        let proprios = fs::read_dir(&p.0)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(&format!(".{}.", std::process::id())))
            .count();
        assert_eq!(proprios, 0);
    }

    #[test]
    fn regravar_troca_o_conteudo_por_inteiro() {
        let p = Pasta::nova();
        gravar_retomada(&p.0, &Retomada { room: "aaa-bbb".into(), hidden: false, at: 1_000_000_000_000 }).unwrap();
        gravar_retomada(&p.0, &Retomada { room: "ccc-ddd".into(), hidden: true, at: 1 }).unwrap();
        assert_eq!(ler_retomada(&p.0), Retomada { room: "ccc-ddd".into(), hidden: true, at: 1 });
    }

    #[test]
    fn apagar_remove_e_ignora_ausente() {
        let p = Pasta::nova();
        gravar_retomada(&p.0, &Retomada::default()).unwrap();
        apagar(&p.0, NOME_RESUME).unwrap();
        assert!(!p.0.join(NOME_RESUME).exists());
        apagar(&p.0, NOME_RESUME).unwrap();
    }

    #[test]
    fn pasta_do_app_termina_em_dev_espia_app() {
        if let Some(p) = pasta_do_app() {
            assert!(p.ends_with("dev.espia.app"));
        }
    }
}
