//! T033: paridade de cenas com o site. Todo `id` de cena de `src/mocks/{home,join,room,notfound}.ts`
//! que não é `device: 'mobile'` precisa existir no registro de cenas do app, na tela de mesmo nome.
//! O teste lê os `.ts` como texto (a partir da raiz do repositório). Cenas só do app são permitidas.

use espia_ui::mocks::registry;
use std::path::PathBuf;

/// Raiz do repositório: `app/crates/espia-ui` sobe três níveis.
fn raiz() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn ler(arquivo: &str) -> String {
    let caminho = raiz().join("src/mocks").join(arquivo);
    std::fs::read_to_string(&caminho).unwrap_or_else(|e| panic!("não deu para ler {}: {e}", caminho.display()))
}

/// O texto entre aspas simples que vem logo depois de `chave` (`id: 'abc'` ou `id:'abc'`).
fn valor_apos<'a>(linha: &'a str, chave: &str) -> Option<&'a str> {
    let resto = &linha[linha.find(chave)? + chave.len()..];
    let resto = resto.trim_start().strip_prefix('\'')?;
    resto.split('\'').next()
}

/// Ids de cena do site em `texto`: um por entrada `{ id: '...' }` do bloco `*_SCENES`, sem as marcadas
/// `device: 'mobile'`. A entrada gerada por `.map` (`watch-${state}`) é expandida com a lista de estados.
fn ids_do_site(texto: &str) -> Vec<String> {
    let inicio = texto.find("_SCENES = [").expect("bloco *_SCENES não encontrado");
    let bloco = &texto[inicio..];
    let bloco = &bloco[..bloco.find("] as const;").expect("fim do bloco *_SCENES não encontrado")];
    let mut ids = Vec::new();
    for linha in bloco.lines() {
        let linha = linha.trim();
        if linha.starts_with("//") {
            continue;
        }
        // O `.map` de room.ts: `...(['a','b'] as const).map(state=>({id:\`watch-${state}\` ...`.
        if linha.contains("watch-${") {
            let a = linha.find("(['").map(|i| i + 1).expect("lista de estados do .map");
            let lista = &linha[a + 1..linha[a..].find(']').map(|i| a + i).expect("fim da lista de estados")];
            for estado in lista.split(',') {
                ids.push(format!("watch-{}", estado.trim().trim_matches('\'')));
            }
            continue;
        }
        if linha.contains("device: 'mobile'") {
            continue;
        }
        if let Some(id) = valor_apos(linha, "id:") {
            ids.push(id.to_string());
        }
    }
    ids
}

/// (tela do app, arquivo do site, quantos ids o site tem sem as de celular)
const TELAS: [(&str, &str, usize); 4] = [("home", "home.ts", 13), ("join", "join.ts", 5), ("room", "room.ts", 47), ("notfound", "notfound.ts", 4)];

#[test]
fn contas_do_site_batem_com_o_contrato() {
    let mut total = 0;
    for (tela, arquivo, esperado) in TELAS {
        let ids = ids_do_site(&ler(arquivo));
        assert_eq!(ids.len(), esperado, "{tela}: o site mudou? ids lidos: {ids:?}");
        total += ids.len();
    }
    assert_eq!(total, 69);
}

#[test]
fn toda_cena_do_site_existe_no_app() {
    let app = registry();
    let mut faltam = Vec::new();
    for (tela, arquivo, _) in TELAS {
        let do_app = app.iter().find(|t| t.id == tela);
        for id in ids_do_site(&ler(arquivo)) {
            if !do_app.is_some_and(|t| t.scenes.iter().any(|c| c.id == id)) {
                faltam.push(format!("{tela}/{id}"));
            }
        }
    }
    assert!(faltam.is_empty(), "cenas do site que faltam no app ({}): {}", faltam.len(), faltam.join(", "));
}

#[test]
fn cenas_de_celular_ficam_so_no_site() {
    let todos: Vec<String> = TELAS.iter().flat_map(|(t, a, _)| ids_do_site(&ler(a)).into_iter().map(move |i| format!("{t}/{i}"))).collect();
    for fora in ["home/mobile", "join/mobile", "join/fixedmobile", "notfound/mobile"] {
        assert!(!todos.iter().any(|i| i == fora), "{fora} devia ser pulada");
    }
    for expandida in ["watch-available", "watch-stopped", "watch-opening", "watch-playing", "watch-unavailable", "watch-reconnecting"] {
        assert!(todos.iter().any(|i| i == &format!("room/{expandida}")), "{expandida} devia vir do .map");
    }
}
