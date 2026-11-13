//! Sonda do gate G3 (T013): lê o `localStorage` do motor de página (Chromium/WebView2)
//! do app de antes, a partir de uma CÓPIA da pasta `leveldb`.
//!
//! Regras de segurança desta sonda:
//! - nunca abre a pasta recebida: copia para uma pasta temporária e abre só a segunda cópia
//!   (o `rusty-leveldb` escreve ao abrir: LOG, MANIFEST, recuperação do `.log`);
//! - recusa caminhos dentro de `%LOCALAPPDATA%\dev.espia.app\` (perfil vivo do app);
//! - nunca imprime o valor de `espia:owner:*` (só o código e se são 64 hex minúsculos)
//!   nem de `espia:session:*` (nem o valor, só a contagem).
//!
//! O formato das chaves do Chromium NÃO está confirmado contra dados reais (ver LEIA-ME.md).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use rusty_leveldb::{compressor, CompressorId, LdbIterator, Options, DB};

/// Origens do site lidas (R10).
const ORIGENS: [&str; 2] = ["https://espia.espia.workers.dev", "https://espia.diuliano.com"];

/// Pasta do perfil vivo do app de antes (relativa a `%LOCALAPPDATA%`).
const PASTA_VIVA: &str = "dev.espia.app";

// ---------------------------------------------------------------------------
// Formato do Chromium (não confirmado)
// ---------------------------------------------------------------------------

/// Uma entrada de dados do `localStorage` já decodificada.
#[derive(Debug, Clone, PartialEq)]
struct Entrada {
    origem: String,
    nome: String,
    valor: String,
}

/// O que fazer com uma chave crua do banco.
#[derive(Debug, PartialEq)]
enum ChaveCrua {
    /// `_<origem>\0<formato><nome>` com valor `<formato><texto>`.
    Dado { origem: String, nome: String },
    /// `META:<origem>` ou `METAACCESS:<origem>`.
    Meta,
    /// `VERSION`.
    Versao,
    /// Qualquer outra coisa (formato desconhecido).
    Desconhecida,
}

/// Decodifica texto do Chromium: byte de formato 1 = Latin-1, 0 = UTF-16LE.
fn decodificar(formato: u8, bytes: &[u8]) -> Option<String> {
    match formato {
        1 => Some(bytes.iter().map(|&b| b as char).collect()),
        0 => {
            if bytes.len() % 2 != 0 {
                return None;
            }
            let unidades: Vec<u16> = bytes
                .chunks_exact(2)
                .map(|p| u16::from_le_bytes([p[0], p[1]]))
                .collect();
            Some(String::from_utf16_lossy(&unidades))
        }
        _ => None,
    }
}

/// Codifica como o Chromium (usado só pela fixture): Latin-1 se couber, senão UTF-16LE.
fn codificar(texto: &str, forcar_utf16: bool) -> (u8, Vec<u8>) {
    if !forcar_utf16 && texto.chars().all(|c| (c as u32) < 256) {
        (1, texto.chars().map(|c| c as u8).collect())
    } else {
        (0, texto.encode_utf16().flat_map(|u| u.to_le_bytes()).collect())
    }
}

fn classificar_chave(chave: &[u8]) -> (ChaveCrua, Option<u8>, Vec<u8>) {
    // Retorna a classe, o byte de formato do nome e os bytes crus do nome (para decodificar).
    if chave == b"VERSION" {
        return (ChaveCrua::Versao, None, vec![]);
    }
    if chave.starts_with(b"META:") || chave.starts_with(b"METAACCESS:") {
        return (ChaveCrua::Meta, None, vec![]);
    }
    if chave.first() == Some(&b'_') {
        if let Some(pos) = chave.iter().position(|&b| b == 0) {
            let origem = String::from_utf8_lossy(&chave[1..pos]).into_owned();
            let resto = &chave[pos + 1..];
            if let Some((&formato, nome)) = resto.split_first() {
                if let Some(nome) = decodificar(formato, nome) {
                    return (ChaveCrua::Dado { origem, nome }, Some(formato), vec![]);
                }
            }
        }
    }
    (ChaveCrua::Desconhecida, None, vec![])
}

/// Compara origens ignorando uma barra final (a pista não confirma se o Chromium grava `/`).
fn mesma_origem(a: &str, b: &str) -> bool {
    a.trim_end_matches('/') == b.trim_end_matches('/')
}

// ---------------------------------------------------------------------------
// Leitura
// ---------------------------------------------------------------------------

/// Tudo o que a leitura achou, sem passar da camada de decodificação.
#[derive(Debug, Default)]
struct Leitura {
    /// Entradas de dados das origens do Espia.
    entradas: Vec<Entrada>,
    /// Origens vistas no banco inteiro e quantas chaves de dados cada uma tem.
    origens_vistas: BTreeMap<String, usize>,
    meta: usize,
    versao: usize,
    desconhecidas: usize,
    /// Valores de dados que não deu para decodificar (formato inesperado).
    valores_ilegiveis: usize,
    total_chaves: usize,
}

fn eh_do_espia(origem: &str) -> bool {
    ORIGENS.iter().any(|o| mesma_origem(origem, o))
}

/// Abre a cópia temporária e lê todas as chaves.
fn ler_banco(pasta: &Path) -> Result<Leitura, String> {
    let mut opt = Options::default();
    opt.create_if_missing = false;
    opt.reuse_logs = false;
    opt.reuse_manifest = false;
    // Chromium grava as tabelas com Snappy (id 1); o leitor usa o id do bloco, mas deixamos
    // o id padrão explícito para não depender do padrão da crate.
    opt.compressor = compressor::SnappyCompressor::ID;

    let mut db = DB::open(pasta, opt).map_err(|e| format!("o leveldb não abriu: {e}"))?;
    let mut it = db
        .new_iter()
        .map_err(|e| format!("não consegui criar o iterador: {e}"))?;

    let mut leitura = Leitura::default();
    let (mut k, mut v) = (Vec::new(), Vec::new());
    while it.advance() {
        if !it.current(&mut k, &mut v) {
            break;
        }
        leitura.total_chaves += 1;
        let (classe, _, _) = classificar_chave(&k);
        match classe {
            ChaveCrua::Versao => leitura.versao += 1,
            ChaveCrua::Meta => leitura.meta += 1,
            ChaveCrua::Desconhecida => leitura.desconhecidas += 1,
            ChaveCrua::Dado { origem, nome } => {
                *leitura.origens_vistas.entry(origem.clone()).or_insert(0) += 1;
                if !eh_do_espia(&origem) {
                    continue;
                }
                let valor = v
                    .split_first()
                    .and_then(|(&f, resto)| decodificar(f, resto));
                match valor {
                    Some(valor) => leitura.entradas.push(Entrada { origem, nome, valor }),
                    None => leitura.valores_ilegiveis += 1,
                }
            }
        }
    }
    drop(it);
    let _ = db.close();
    Ok(leitura)
}

// ---------------------------------------------------------------------------
// Cópia temporária e proteção da pasta viva
// ---------------------------------------------------------------------------

/// Pasta temporária apagada ao sair de escopo.
struct Temporaria(PathBuf);

impl Drop for Temporaria {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn pasta_temporaria() -> Result<Temporaria, String> {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let p = std::env::temp_dir().join(format!("espia-g3-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&p).map_err(|e| format!("não consegui criar a pasta temporária: {e}"))?;
    Ok(Temporaria(p))
}

/// Recusa caminhos dentro de `%LOCALAPPDATA%\dev.espia.app\`.
fn recusar_pasta_viva(pasta: &Path) -> Result<(), String> {
    recusar_dentro_de(pasta, std::env::var_os("LOCALAPPDATA").map(PathBuf::from))
}

/// Versão com a raiz explícita (testável sem tocar no perfil real).
fn recusar_dentro_de(pasta: &Path, local: Option<PathBuf>) -> Result<(), String> {
    let Some(local) = local else {
        return Ok(());
    };
    let viva = local.join(PASTA_VIVA);
    // Se a pasta viva não existe, nada a proteger; senão compara caminhos reais.
    let Ok(viva) = fs::canonicalize(&viva) else {
        return Ok(());
    };
    let alvo = match fs::canonicalize(pasta) {
        Ok(a) => a,
        Err(e) => {
            // Caminho que não existe: ainda assim recusa se, lido como texto, cai na pasta viva.
            let norm = |p: &Path| p.to_string_lossy().replace('/', "\\").to_lowercase();
            let (n_alvo, n_viva) = (norm(pasta), norm(&viva));
            let n_viva = n_viva.trim_start_matches(r"\\?\").to_string();
            let trecho = format!(r"\{}\", PASTA_VIVA.to_lowercase());
            if n_alvo.contains(&n_viva) || n_alvo.contains(&trecho) {
                return Err(format!("recusado: `{}` aponta para a pasta viva do app.", pasta.display()));
            }
            return Err(format!("caminho inválido `{}`: {e}", pasta.display()));
        }
    };
    if alvo.starts_with(&viva) {
        return Err(format!(
            "recusado: `{}` está dentro de {}. Esta sonda nunca abre o perfil vivo do app. \
             Feche o app, copie a pasta `leveldb` para outro lugar (por exemplo \
             C:\\teste-g3\\leveldb) e aponte a sonda para a cópia.",
            pasta.display(),
            viva.display()
        ));
    }
    Ok(())
}

/// Copia os arquivos (não recursivo) da pasta para `destino`, sem o `LOCK`.
fn copiar_pasta(origem: &Path, destino: &Path) -> Result<usize, String> {
    let meta = fs::metadata(origem)
        .map_err(|e| format!("não consegui ler `{}`: {e}", origem.display()))?;
    if !meta.is_dir() {
        return Err(format!("`{}` não é uma pasta", origem.display()));
    }
    let mut copiados = 0;
    let lista = fs::read_dir(origem)
        .map_err(|e| format!("não consegui listar `{}`: {e}", origem.display()))?;
    for item in lista {
        let item = item.map_err(|e| format!("erro ao listar a pasta: {e}"))?;
        let nome = item.file_name();
        let nome_txt = nome.to_string_lossy();
        if nome_txt.eq_ignore_ascii_case("LOCK") {
            continue; // arquivo de trava: a crate cria o seu na cópia
        }
        let tipo = item
            .file_type()
            .map_err(|e| format!("erro ao ler `{nome_txt}`: {e}"))?;
        if !tipo.is_file() {
            continue;
        }
        fs::copy(item.path(), destino.join(&nome)).map_err(|e| {
            format!(
                "não consegui copiar `{nome_txt}` (arquivo travado por outro programa? \
                 Feche o app de antes e copie a pasta de novo): {e}"
            )
        })?;
        copiados += 1;
    }
    Ok(copiados)
}

// ---------------------------------------------------------------------------
// Relatório (nunca imprime valor de owner nem de session)
// ---------------------------------------------------------------------------

fn eh_hex64(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

struct Resumo {
    salas: bool,
    donos: usize,
    donos_hex64: usize,
    apelido: bool,
}

fn relatorio(l: &Leitura) -> (String, Resumo) {
    use std::fmt::Write;
    let mut s = String::new();
    let mut resumo = Resumo { salas: false, donos: 0, donos_hex64: 0, apelido: false };

    let _ = writeln!(
        s,
        "Banco lido: {} chaves ({} META, {} VERSION, {} fora do formato conhecido).",
        l.total_chaves, l.meta, l.versao, l.desconhecidas
    );
    let _ = writeln!(s, "Origens com dados no banco (nome da origem: quantidade de chaves):");
    if l.origens_vistas.is_empty() {
        let _ = writeln!(s, "  (nenhuma; se o banco tem chaves, o formato `_origem\\0...` não bateu)");
    }
    for (o, n) in &l.origens_vistas {
        let marca = if eh_do_espia(o) { "  <- lida" } else { "" };
        let _ = writeln!(s, "  {o}: {n}{marca}");
    }
    if l.valores_ilegiveis > 0 {
        let _ = writeln!(s, "Aviso: {} valor(es) das origens do Espia com formato ilegível.", l.valores_ilegiveis);
    }

    for origem in ORIGENS {
        let _ = writeln!(s, "\nOrigem {origem}");
        let daqui: Vec<&Entrada> =
            l.entradas.iter().filter(|e| mesma_origem(&e.origem, origem)).collect();
        if daqui.is_empty() {
            let _ = writeln!(s, "  (sem chaves)");
            continue;
        }
        let _ = writeln!(s, "  Chaves ({}):", daqui.len());
        let mut volumes = 0;
        for e in &daqui {
            // Só o nome. Nomes de `owner` e `session` contêm só o código, nunca o valor.
            if e.nome.starts_with("espia:volume:") {
                volumes += 1;
            } else {
                let _ = writeln!(s, "    {}", e.nome);
            }
        }
        if volumes > 0 {
            let _ = writeln!(s, "    espia:volume:* ({volumes} chaves)");
        }
        for e in &daqui {
            if e.nome == "espia:rooms" {
                resumo.salas = true;
                let _ = writeln!(s, "  espia:rooms = {}", e.valor);
            } else if e.nome == "espia:nickname" {
                resumo.apelido = true;
                let _ = writeln!(s, "  espia:nickname = {}", e.valor);
            }
        }
        for e in &daqui {
            if let Some(codigo) = e.nome.strip_prefix("espia:owner:") {
                resumo.donos += 1;
                let ok = eh_hex64(&e.valor);
                if ok {
                    resumo.donos_hex64 += 1;
                }
                let _ = writeln!(
                    s,
                    "  dono {codigo}: 64 hex minúsculos = {} ({} caracteres)",
                    if ok { "sim" } else { "não" },
                    e.valor.chars().count()
                );
            }
        }
        let sessoes = daqui.iter().filter(|e| e.nome.starts_with("espia:session:")).count();
        let _ = writeln!(s, "  espia:session:* = {sessoes} (valores não impressos)");
    }
    (s, resumo)
}

// ---------------------------------------------------------------------------
// Fixture
// ---------------------------------------------------------------------------

fn chave_dado(origem: &str, nome: &str, utf16: bool) -> (Vec<u8>, u8) {
    let (formato, bytes) = codificar(nome, utf16);
    let mut k = Vec::new();
    k.push(b'_');
    k.extend_from_slice(origem.as_bytes());
    k.push(0);
    k.push(formato);
    k.extend_from_slice(&bytes);
    (k, formato)
}

fn valor_dado(texto: &str, utf16: bool) -> Vec<u8> {
    let (formato, bytes) = codificar(texto, utf16);
    let mut v = vec![formato];
    v.extend_from_slice(&bytes);
    v
}

/// Dono falso de teste: 64 hex minúsculos, e outro inválido.
pub const DONO_VALIDO: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
pub const DONO_INVALIDO: &str = "ABC-nao-e-hex";
pub const SESSAO_SECRETA: &str = "sessao-secreta-que-nao-pode-aparecer";

/// Cria uma pasta leveldb de teste no formato (suposto) do Chromium, com valores Latin-1 e
/// UTF-16LE. Parte dos dados vai para um `.ldb` (compactação) e parte fica só no `.log`.
fn criar_fixture(pasta: &Path) -> Result<(), String> {
    if pasta.exists() && fs::read_dir(pasta).map(|mut d| d.next().is_some()).unwrap_or(false) {
        return Err(format!("`{}` já existe e não está vazia; use uma pasta nova", pasta.display()));
    }
    let mut opt = Options::default();
    opt.create_if_missing = true;
    opt.compressor = compressor::SnappyCompressor::ID;
    let mut db = DB::open(pasta, opt).map_err(|e| format!("não consegui criar a fixture: {e}"))?;
    let erro = |e: rusty_leveldb::Status| format!("erro ao gravar a fixture: {e}");

    let mut p = |k: Vec<u8>, v: Vec<u8>| db.put(&k, &v).map_err(erro);

    p(b"VERSION".to_vec(), b"1".to_vec())?;
    for o in ORIGENS {
        p(format!("META:{o}").into_bytes(), vec![0x08, 0x01])?;
    }
    // Outra origem que não deve ser lida.
    p(chave_dado("https://outro.exemplo", "espia:nickname", false).0, valor_dado("intruso", false))?;

    let antiga = ORIGENS[0];
    let oficial = ORIGENS[1];
    // Latin-1 (com acento) e UTF-16LE (caractere fora do Latin-1).
    p(chave_dado(antiga, "espia:nickname", false).0, valor_dado("João Ação", false))?;
    p(
        chave_dado(antiga, "espia:rooms", false).0,
        valor_dado(r#"[{"code":"ABC234","name":"Sala do Zé ✓"}]"#, false),
    )?;
    p(chave_dado(antiga, "espia:owner:ABC234", false).0, valor_dado(DONO_VALIDO, false))?;
    p(chave_dado(antiga, "espia:golive", false).0, valor_dado("1", false))?;
    // Valores repetitivos para o Snappy comprimir de verdade no `.ldb`.
    for i in 0..200 {
        p(
            chave_dado(antiga, &format!("espia:volume:amigo{i:03}"), false).0,
            valor_dado(&"0.75 ".repeat(40), false),
        )?;
    }
    drop(p);
    // Empurra o que está na memória para um `.ldb`.
    db.compact_range(b"", b"\xff\xff\xff\xff")
        .map_err(|e| format!("erro ao compactar a fixture: {e}"))?;

    // O resto fica só no `.log` (registro não compactado).
    let mut p = |k: Vec<u8>, v: Vec<u8>| db.put(&k, &v).map_err(erro);
    p(chave_dado(oficial, "espia:nickname", true).0, valor_dado("Ωmega João", true))?;
    p(
        chave_dado(oficial, "espia:rooms", true).0,
        valor_dado(r#"[{"code":"XYZ789","name":"Sala Ωmega"}]"#, true),
    )?;
    p(chave_dado(oficial, "espia:owner:XYZ789", true).0, valor_dado(DONO_VALIDO, true))?;
    p(chave_dado(oficial, "espia:owner:QQQ222", true).0, valor_dado(DONO_INVALIDO, true))?;
    p(chave_dado(oficial, "espia:session:XYZ789", false).0, valor_dado(SESSAO_SECRETA, false))?;
    drop(p);
    db.close().map_err(|e| format!("erro ao fechar a fixture: {e}"))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Programa
// ---------------------------------------------------------------------------

const AJUDA: &str = "\
Sonda G3 (T013): lê o localStorage do app de antes a partir de uma CÓPIA da pasta leveldb.

Uso:
  espia-webview-data-probe <pasta-da-copia-do-leveldb>
  espia-webview-data-probe --criar-fixture <pasta-nova>

A pasta recebida é copiada para uma pasta temporária e só essa segunda cópia é aberta.
Caminhos dentro de %LOCALAPPDATA%\\dev.espia.app\\ são recusados.
Códigos de saída: 0 leu e achou dados do Espia; 1 não conseguiu ler; 2 uso errado ou caminho
recusado; 3 leu o banco mas não achou dados do Espia.";

fn sondar(pasta: &Path) -> Result<ExitCode, String> {
    recusar_pasta_viva(pasta)?;

    let tmp = pasta_temporaria()?;
    let n = copiar_pasta(pasta, &tmp.0)?;
    if n == 0 {
        return Err(format!("a pasta `{}` está vazia: não há banco para ler", pasta.display()));
    }
    if !tmp.0.join("CURRENT").is_file() {
        return Err(format!(
            "a pasta `{}` não tem o arquivo CURRENT: não parece uma pasta leveldb \
             (confira se é a pasta `Local Storage\\leveldb`)",
            pasta.display()
        ));
    }

    // A crate pode entrar em pânico com arquivo corrompido: transformar em erro claro.
    let antigo = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| ler_banco(&tmp.0)));
    std::panic::set_hook(antigo);
    let leitura = match res {
        Ok(r) => r?,
        Err(_) => return Err("a biblioteca leveldb falhou ao ler (arquivo corrompido ou formato inesperado)".into()),
    };

    let (texto, resumo) = relatorio(&leitura);
    println!("{texto}");
    println!(
        "RESULTADO: salas fixas = {}; chaves de dono = {} ({} com 64 hex); apelido = {}",
        if resumo.salas { "lidas" } else { "não achadas" },
        resumo.donos,
        resumo.donos_hex64,
        if resumo.apelido { "lido" } else { "não achado" }
    );
    if leitura.entradas.is_empty() {
        eprintln!(
            "Aviso: o banco abriu, mas nenhuma chave das origens do Espia foi encontrada. \
             Veja acima as origens que existem no banco."
        );
        return Ok(ExitCode::from(3));
    }
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let resultado = match args.as_slice() {
        [flag, pasta] if flag == "--criar-fixture" => {
            criar_fixture(Path::new(pasta)).map(|_| {
                println!("Fixture criada em {pasta}");
                ExitCode::SUCCESS
            })
        }
        [pasta] if !pasta.starts_with("--") => sondar(Path::new(pasta)),
        _ => {
            eprintln!("{AJUDA}");
            return ExitCode::from(2);
        }
    };
    match resultado {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Não li: {e}");
            ExitCode::from(if e.starts_with("recusado") { 2 } else { 1 })
        }
    }
}

// ---------------------------------------------------------------------------
// Testes
// ---------------------------------------------------------------------------

#[cfg(test)]
mod testes {
    use super::*;

    fn pasta_de_teste(nome: &str) -> Temporaria {
        let p = std::env::temp_dir().join(format!("espia-g3-teste-{nome}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&p);
        Temporaria(p)
    }

    #[test]
    fn decodifica_os_dois_formatos() {
        assert_eq!(decodificar(1, &[0x4a, 0x6f, 0xe3, 0x6f]).as_deref(), Some("João"));
        let omega: Vec<u8> = "Ωé".encode_utf16().flat_map(|u| u.to_le_bytes()).collect();
        assert_eq!(decodificar(0, &omega).as_deref(), Some("Ωé"));
        assert_eq!(decodificar(0, &[0x41]), None);
        assert_eq!(decodificar(7, b"x"), None);
    }

    #[test]
    fn classifica_chaves() {
        let (k, _) = chave_dado("https://a.b", "espia:x", false);
        assert_eq!(
            classificar_chave(&k).0,
            ChaveCrua::Dado { origem: "https://a.b".into(), nome: "espia:x".into() }
        );
        assert_eq!(classificar_chave(b"VERSION").0, ChaveCrua::Versao);
        assert_eq!(classificar_chave(b"META:https://a.b").0, ChaveCrua::Meta);
        assert_eq!(classificar_chave(b"lixo").0, ChaveCrua::Desconhecida);
        assert!(mesma_origem("https://a.b/", "https://a.b"));
    }

    #[test]
    fn le_a_fixture_e_nao_vaza_segredos() {
        let fx = pasta_de_teste("fixture");
        criar_fixture(&fx.0).expect("fixture");
        let leitura = ler_banco(&fx.0).expect("leitura");
        let (txt, r) = relatorio(&leitura);

        assert!(r.salas && r.apelido);
        assert_eq!(r.donos, 3);
        assert_eq!(r.donos_hex64, 2);
        assert!(txt.contains("João Ação"), "Latin-1 com acento: {txt}");
        assert!(txt.contains("Ωmega João"), "UTF-16LE: {txt}");
        assert!(txt.contains("Sala do Zé ✓") && txt.contains("Sala Ωmega"));
        assert!(txt.contains("dono ABC234: 64 hex minúsculos = sim"));
        assert!(txt.contains("dono QQQ222: 64 hex minúsculos = não"));
        assert!(txt.contains("espia:session:* = 1"));
        assert!(!txt.contains(DONO_VALIDO) && !txt.contains(DONO_INVALIDO));
        assert!(!txt.contains(SESSAO_SECRETA));
        assert!(!txt.contains("intruso"));
        assert!(txt.contains("https://outro.exemplo: 1"));
        assert!(txt.contains("espia:volume:* (200 chaves)"));
    }

    #[test]
    fn sondar_nao_altera_a_pasta_recebida() {
        let fx = pasta_de_teste("intacta");
        criar_fixture(&fx.0).expect("fixture");
        let antes = listar(&fx.0);
        let leitura = {
            let tmp = pasta_temporaria().unwrap();
            copiar_pasta(&fx.0, &tmp.0).unwrap();
            ler_banco(&tmp.0).unwrap()
        };
        assert!(!leitura.entradas.is_empty());
        assert_eq!(antes, listar(&fx.0));
    }

    fn listar(p: &Path) -> Vec<(String, u64)> {
        let mut v: Vec<_> = fs::read_dir(p)
            .unwrap()
            .map(|e| {
                let e = e.unwrap();
                (e.file_name().to_string_lossy().into_owned(), e.metadata().unwrap().len())
            })
            .collect();
        v.sort();
        v
    }

    #[test]
    fn pasta_vazia_ou_invalida_nao_entra_em_panico() {
        let vazia = pasta_de_teste("vazia");
        fs::create_dir_all(&vazia.0).unwrap();
        let e = sondar(&vazia.0).unwrap_err();
        assert!(e.contains("vazia"), "{e}");

        let lixo = pasta_de_teste("lixo");
        fs::create_dir_all(&lixo.0).unwrap();
        fs::write(lixo.0.join("qualquer.txt"), b"oi").unwrap();
        let e = sondar(&lixo.0).unwrap_err();
        assert!(e.contains("CURRENT"), "{e}");

        let corrompida = pasta_de_teste("corrompida");
        fs::create_dir_all(&corrompida.0).unwrap();
        fs::write(corrompida.0.join("CURRENT"), b"MANIFEST-000001\n").unwrap();
        fs::write(corrompida.0.join("MANIFEST-000001"), b"isto nao e um manifest").unwrap();
        assert!(sondar(&corrompida.0).is_err());

        let e = sondar(Path::new("C:\\caminho\\que\\nao\\existe-g3")).unwrap_err();
        assert!(!e.is_empty());
    }

    #[test]
    fn recusa_pasta_viva() {
        // Raiz falsa: nunca toca no perfil real da pessoa.
        let raiz = pasta_de_teste("local");
        let viva = raiz.0.join(PASTA_VIVA);
        let dentro = viva.join("EBWebView").join("Default");
        fs::create_dir_all(&dentro).unwrap();
        let fora = pasta_de_teste("fora");
        fs::create_dir_all(&fora.0).unwrap();

        let e = recusar_dentro_de(&dentro, Some(raiz.0.clone())).unwrap_err();
        assert!(e.starts_with("recusado"), "{e}");
        let e = recusar_dentro_de(&viva, Some(raiz.0.clone())).unwrap_err();
        assert!(e.starts_with("recusado"), "{e}");
        assert!(recusar_dentro_de(&fora.0, Some(raiz.0.clone())).is_ok());
    }
}
