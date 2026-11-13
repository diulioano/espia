//! Rotas do app como dados puros. Porte de `src/lib/route.ts`.
use crate::room_id::is_valid_room_id;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    Home,
    Room { code: String },
}

/// `/s/<código>` (com uma barra final opcional) vira a sala; qualquer outra coisa, o início.
/// Como `/^\/s\/([^/]+)\/?$/` seguido do teste do formato do código.
pub fn parse_route(pathname: &str) -> Route {
    if let Some(rest) = pathname.strip_prefix("/s/") {
        let code = rest.strip_suffix('/').unwrap_or(rest);
        if !code.contains('/') && is_valid_room_id(code) {
            return Route::Room { code: code.to_string() };
        }
    }
    Route::Home
}

/// Mesma tela: um "voltar" que só fecha a tela cheia não pode remontar nada.
pub fn same_route(a: &Route, b: &Route) -> bool {
    a == b
}

#[cfg(test)]
mod tests {
    //! Testes portados de `test/lib/router.test.ts`: o arquivo TS tem 3 casos (`it`) e os 3 foram
    //! portados. Mais casos novos para `parse_route` com sala (o TS não os cobre no router.test.ts).
    use super::*;

    fn sala(code: &str) -> Route {
        Route::Room { code: code.to_string() }
    }

    #[test]
    fn sala_e_inicio() {
        assert_eq!(parse_route("/"), Route::Home);
        assert_eq!(parse_route("/coisa"), Route::Home);
    }

    #[test]
    fn voltar_no_mesmo_endereco_e_a_mesma_rota() {
        let a = parse_route("/");
        assert!(same_route(&a, &parse_route("/")));
        assert!(same_route(&sala("k7f-q2x"), &sala("k7f-q2x")));
    }

    #[test]
    fn outro_endereco_muda_a_rota() {
        assert!(!same_route(&sala("k7f-q2x"), &sala("abc-def")));
        assert!(!same_route(&sala("k7f-q2x"), &Route::Home));
        assert!(!same_route(&Route::Home, &sala("k7f-q2x")));
    }

    #[test]
    fn caminho_de_sala_valido() {
        assert_eq!(parse_route("/s/k7f-q2x"), sala("k7f-q2x"));
        assert_eq!(parse_route("/s/k7f-q2x/"), sala("k7f-q2x"));
    }

    #[test]
    fn caminho_de_sala_invalido_vira_inicio() {
        assert_eq!(parse_route("/s/"), Route::Home);
        assert_eq!(parse_route("/s/K7F-Q2X"), Route::Home);
        assert_eq!(parse_route("/s/k7f-q2o"), Route::Home);
        assert_eq!(parse_route("/s/k7f-q2x//"), Route::Home);
        assert_eq!(parse_route("/s/k7f-q2x/mais"), Route::Home);
        assert_eq!(parse_route("/x/s/k7f-q2x"), Route::Home);
        assert_eq!(parse_route("s/k7f-q2x"), Route::Home);
        assert_eq!(parse_route("/s/k7f-q2x\n"), Route::Home);
        assert_eq!(parse_route(""), Route::Home);
    }
}
