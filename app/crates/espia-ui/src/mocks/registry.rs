//! Registro das telas e cenas do mock. Porte de `src/mocks/registry.ts` (sem a tela `mobile`).
//! Para acrescentar uma cena basta pô-la na lista de `mocks/<tela>.rs`: o registro já a junta.

use super::{components, home, join, notfound, offline, room, Scene};
use crate::{MockSceneInfo, MockScreenInfo, Screen};

/// Uma cena pronta para abrir: o `Screen` que a janela recebe.
#[derive(Debug, Clone, PartialEq)]
pub struct MockScene {
    pub id: String,
    pub code: String,
    pub label: String,
    pub screen: Screen,
}

/// Uma tela do mock com as cenas dela.
#[derive(Debug, Clone, PartialEq)]
pub struct MockScreen {
    pub id: &'static str,
    pub label: &'static str,
    pub default_scene: &'static str,
    pub scenes: Vec<MockScene>,
}

fn juntar<V>(lista: Vec<Scene<V>>, para_tela: impl Fn(V) -> Screen) -> Vec<MockScene> {
    lista
        .into_iter()
        .map(|s| MockScene { id: s.id, code: s.code, label: s.label, screen: para_tela(s.view) })
        .collect()
}

/// Telas na ordem do índice: as quatro do site, `offline` e a vitrine `components`.
pub fn registry() -> Vec<MockScreen> {
    vec![
        MockScreen { id: "home", label: "Página inicial", default_scene: "default", scenes: juntar(home::scenes(), Screen::Home) },
        MockScreen { id: "join", label: "Entrar por link", default_scene: "default", scenes: juntar(join::scenes(), Screen::Join) },
        MockScreen { id: "room", label: "Sala", default_scene: "single", scenes: juntar(room::scenes(), Screen::Room) },
        MockScreen { id: "notfound", label: "Sala não encontrada", default_scene: "quick", scenes: juntar(notfound::scenes(), Screen::NotFound) },
        MockScreen { id: "offline", label: "Sem internet", default_scene: "default", scenes: juntar(offline::scenes(), offline::tela) },
        MockScreen { id: "components", label: "Componentes base", default_scene: "all", scenes: juntar(components::scenes(), Screen::Components) },
    ]
}

/// Procura `tela` (e `cena`, ou a padrão da tela). O erro é a linha de aviso do índice.
pub fn resolve<'a>(
    registro: &'a [MockScreen],
    tela: &str,
    cena: Option<&str>,
) -> Result<(&'a MockScreen, &'a MockScene), String> {
    let t = registro
        .iter()
        .find(|t| t.id == tela)
        .ok_or_else(|| format!("A tela “{tela}” não existe."))?;
    let id = cena.filter(|c| !c.is_empty()).unwrap_or(t.default_scene);
    let c = t
        .scenes
        .iter()
        .find(|c| c.id == id)
        .ok_or_else(|| format!("A cena “{id}” não existe na tela “{tela}”."))?;
    Ok((t, c))
}

/// O registro como dados simples para o índice e o seletor.
pub fn catalogo(registro: &[MockScreen]) -> Vec<MockScreenInfo> {
    registro
        .iter()
        .map(|t| MockScreenInfo {
            id: t.id.to_string(),
            label: t.label.to_string(),
            scenes: t
                .scenes
                .iter()
                .map(|c| MockSceneInfo { id: c.id.clone(), code: c.code.clone(), label: c.label.clone() })
                .collect(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn telas_na_ordem_do_contrato() {
        let ids: Vec<_> = registry().iter().map(|t| t.id).collect();
        assert_eq!(ids, ["home", "join", "room", "notfound", "offline", "components"]);
    }

    #[test]
    fn ids_de_cena_nao_se_repetem_na_tela() {
        for t in registry() {
            let mut ids: Vec<_> = t.scenes.iter().map(|c| c.id.clone()).collect();
            let n = ids.len();
            ids.sort();
            ids.dedup();
            assert_eq!(ids.len(), n, "ids repetidos em {}", t.id);
        }
    }

    #[test]
    fn offline_e_componentes_tem_a_cena_padrao() {
        let r = registry();
        assert!(resolve(&r, "offline", None).is_ok());
        assert!(resolve(&r, "components", None).is_ok());
        assert!(resolve(&r, "components", Some("buttons")).is_ok());
    }

    #[test]
    fn tela_que_nao_existe_diz_o_nome() {
        let r = registry();
        let e = resolve(&r, "naoexiste", None).unwrap_err();
        assert!(e.contains("naoexiste"), "{e}");
    }

    #[test]
    fn cena_que_nao_existe_diz_o_nome_e_a_tela() {
        let r = registry();
        let e = resolve(&r, "offline", Some("zzz")).unwrap_err();
        assert!(e.contains("zzz") && e.contains("offline"), "{e}");
    }

    #[test]
    fn cena_vazia_vale_como_a_padrao() {
        let r = registry();
        let (_, c) = resolve(&r, "offline", Some("")).unwrap();
        assert_eq!(c.id, "default");
    }

    #[test]
    fn catalogo_acompanha_o_registro() {
        let r = registry();
        let c = catalogo(&r);
        assert_eq!(c.len(), r.len());
        assert_eq!(c[5].scenes.len(), r[5].scenes.len());
    }
}
