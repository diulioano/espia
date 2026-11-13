//! Leitor dos tokens de design (R18, constituição IV): lê `src/ui/tokens.css` e gera o `tokens.slint`.
//!
//! Este arquivo é usado por dois lados: pelo `build.rs` (via `#[path]`, para gerar `ui/tokens.slint`)
//! e pela crate (para os testes, que rodam sem abrir janela). Não depende de nada fora da `std`.

use std::collections::HashMap;
use std::fmt;

/// Fonte monoespaçada no Windows: o CSS pede `ui-monospace, Menlo, monospace`, e o Chrome cai em Consolas.
pub const MONO_WINDOWS: &str = "Consolas";

/// Tokens que as telas usam. Se algum faltar no CSS, o build falha dizendo qual.
pub const REQUIRED: &[&str] = &[
    "color-accent",
    "color-accent-200",
    "color-accent-300",
    "color-accent-400",
    "color-accent-600",
    "color-accent-700",
    "color-accent-800",
    "color-accent-900",
    "color-accent-2",
    "color-accent-2-200",
    "color-accent-2-300",
    "color-accent-2-400",
    "color-accent-2-800",
    "color-accent-2-900",
    "color-neutral-300",
    "color-neutral-800",
    "color-stage",
    "color-bg",
    "color-surface",
    "color-raised",
    "color-text",
    "color-text-muted",
    "color-divider",
    "color-overlay",
    "color-ctl",
    "color-backdrop",
    "font-heading",
    "font-body",
    "font-mono",
    "space-1",
    "space-2",
    "space-3",
    "space-4",
    "space-6",
    "space-8",
    "radius-tile-sm",
    "radius-tile",
    "radius-card",
    "radius-stage",
    "radius-pill",
    "shadow-md",
    "shadow-lg",
    "ring-live",
    "ring-focus",
];

/// Erro ao ler um token: sempre diz o nome do token (sem o `--`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenError {
    pub token: String,
    pub motivo: String,
}

impl fmt::Display for TokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "token --{}: {}", self.token, self.motivo)
    }
}

impl std::error::Error for TokenError {}

fn erro(token: &str, motivo: impl Into<String>) -> TokenError {
    TokenError { token: token.to_string(), motivo: motivo.into() }
}

/// Cor em canais 0..255 e alfa 0..1.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Rgba {
    r: u8,
    g: u8,
    b: u8,
    a: f64,
}

impl Rgba {
    /// Literal da Slint: `#rrggbb` quando opaca, `rgba(r, g, b, a)` quando não.
    fn slint(self) -> String {
        if (self.a - 1.0).abs() < 1e-9 {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            let a = (self.a * 10000.0).round() / 10000.0;
            format!("rgba({}, {}, {}, {})", self.r, self.g, self.b, a)
        }
    }
}

/// Tira os comentários `/* … */` do CSS.
fn sem_comentarios(css: &str) -> String {
    let mut saida = String::with_capacity(css.len());
    let mut resto = css;
    while let Some(i) = resto.find("/*") {
        saida.push_str(&resto[..i]);
        match resto[i + 2..].find("*/") {
            Some(j) => resto = &resto[i + 2 + j + 2..],
            None => return saida,
        }
    }
    saida.push_str(resto);
    saida
}

/// Declarações `--nome: valor;` do primeiro bloco `:root { … }`, na ordem do arquivo.
pub fn parse_root(css: &str) -> Result<Vec<(String, String)>, TokenError> {
    let css = sem_comentarios(css);
    let ini = css.find(":root").ok_or_else(|| erro("(arquivo)", "bloco :root não encontrado"))?;
    let abre = css[ini..].find('{').ok_or_else(|| erro("(arquivo)", "bloco :root sem '{'"))? + ini + 1;
    // Fecha na primeira '}' fora de parênteses (os valores não têm chaves, então basta achar a primeira).
    let fecha = css[abre..].find('}').ok_or_else(|| erro("(arquivo)", "bloco :root sem '}'"))? + abre;
    let corpo = &css[abre..fecha];

    let mut saida = Vec::new();
    // Divide em ';' fora de parênteses.
    let mut prof = 0i32;
    let mut atual = String::new();
    let mut pedacos = Vec::new();
    for c in corpo.chars() {
        match c {
            '(' => prof += 1,
            ')' => prof -= 1,
            _ => {}
        }
        if c == ';' && prof == 0 {
            pedacos.push(std::mem::take(&mut atual));
        } else {
            atual.push(c);
        }
    }
    pedacos.push(atual);

    for p in pedacos {
        let p = p.trim();
        if p.is_empty() {
            continue;
        }
        let (nome, valor) = p.split_once(':').ok_or_else(|| erro(p, "declaração sem ':'"))?;
        let nome = nome.trim();
        let nome = nome
            .strip_prefix("--")
            .ok_or_else(|| erro(nome, "nome de token deve começar com '--'"))?;
        let valor = valor.split_whitespace().collect::<Vec<_>>().join(" ");
        if valor.is_empty() {
            return Err(erro(nome, "valor vazio"));
        }
        saida.push((nome.to_string(), valor));
    }
    Ok(saida)
}

fn parse_hex(token: &str, s: &str) -> Result<Rgba, TokenError> {
    let h = s.strip_prefix('#').ok_or_else(|| erro(token, format!("cor inválida: {s}")))?;
    let hex2 = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).map_err(|_| erro(token, format!("cor inválida: {s}")));
    if !h.is_ascii() {
        return Err(erro(token, format!("cor inválida: {s}")));
    }
    match h.len() {
        6 => Ok(Rgba { r: hex2(0)?, g: hex2(2)?, b: hex2(4)?, a: 1.0 }),
        8 => Ok(Rgba { r: hex2(0)?, g: hex2(2)?, b: hex2(4)?, a: f64::from(hex2(6)?) / 255.0 }),
        _ => Err(erro(token, format!("cor inválida: {s} (use #rrggbb)"))),
    }
}

/// Lê uma cor: `#rrggbb`, `rgba(r, g, b, a)`, `rgb(r, g, b)`, `var(--outra)` ou
/// `color-mix(in srgb, <cor> P%, transparent)`.
fn parse_cor(token: &str, s: &str, todos: &HashMap<String, String>, fundo: u8) -> Result<Rgba, TokenError> {
    let s = s.trim();
    if fundo > 8 {
        return Err(erro(token, "referências circulares ou fundas demais"));
    }
    if s.starts_with('#') {
        return parse_hex(token, s);
    }
    if let Some(interno) = s.strip_prefix("var(").and_then(|r| r.strip_suffix(')')) {
        let ref_nome = interno.trim().trim_start_matches("--");
        let valor = todos.get(ref_nome).ok_or_else(|| erro(token, format!("var(--{ref_nome}) não existe")))?;
        return parse_cor(token, valor, todos, fundo + 1);
    }
    if let Some(interno) = s.strip_prefix("rgba(").or_else(|| s.strip_prefix("rgb(")).and_then(|r| r.strip_suffix(')')) {
        let partes: Vec<&str> = interno.split(',').map(str::trim).collect();
        if partes.len() != 3 && partes.len() != 4 {
            return Err(erro(token, format!("rgba precisa de 3 ou 4 números: {s}")));
        }
        let canal = |p: &str| p.parse::<u8>().map_err(|_| erro(token, format!("canal inválido '{p}' em {s}")));
        let a = match partes.get(3) {
            Some(p) => p.parse::<f64>().map_err(|_| erro(token, format!("alfa inválido '{p}' em {s}")))?,
            None => 1.0,
        };
        if !(0.0..=1.0).contains(&a) {
            return Err(erro(token, format!("alfa fora de 0..1 em {s}")));
        }
        return Ok(Rgba { r: canal(partes[0])?, g: canal(partes[1])?, b: canal(partes[2])?, a });
    }
    if let Some(interno) = s.strip_prefix("color-mix(").and_then(|r| r.strip_suffix(')')) {
        // color-mix(in srgb, <cor> P%, transparent): a mesma cor com P% de opacidade.
        let partes = divide_virgulas(interno);
        if partes.len() != 3 || partes[0].trim() != "in srgb" || partes[2].trim() != "transparent" {
            return Err(erro(token, format!("só entendo color-mix(in srgb, <cor> P%, transparent): {s}")));
        }
        let (cor, pct) = partes[1]
            .trim()
            .rsplit_once(' ')
            .ok_or_else(|| erro(token, format!("color-mix sem porcentagem: {s}")))?;
        let pct: f64 = pct
            .strip_suffix('%')
            .and_then(|p| p.parse().ok())
            .ok_or_else(|| erro(token, format!("porcentagem inválida em {s}")))?;
        let base = parse_cor(token, cor, todos, fundo + 1)?;
        return Ok(Rgba { a: base.a * pct / 100.0, ..base });
    }
    Err(erro(token, format!("não sei ler a cor: {s}")))
}

/// Divide em vírgulas fora de parênteses.
fn divide_virgulas(s: &str) -> Vec<String> {
    let mut prof = 0i32;
    let mut atual = String::new();
    let mut saida = Vec::new();
    for c in s.chars() {
        match c {
            '(' => prof += 1,
            ')' => prof -= 1,
            _ => {}
        }
        if c == ',' && prof == 0 {
            saida.push(std::mem::take(&mut atual));
        } else {
            atual.push(c);
        }
    }
    saida.push(atual);
    saida
}

/// `4.4px` ou `0` → número em px.
fn parse_px(token: &str, s: &str) -> Result<f64, TokenError> {
    let s = s.trim();
    if s == "0" {
        return Ok(0.0);
    }
    s.strip_suffix("px")
        .and_then(|n| n.trim().parse::<f64>().ok())
        .ok_or_else(|| erro(token, format!("esperava um valor em px, veio '{s}'")))
}

fn fmt_px(n: f64) -> String {
    let n = (n * 1000.0).round() / 1000.0;
    if n.fract() == 0.0 { format!("{n:.0}px") } else { format!("{n}px") }
}

/// Uma camada de sombra: `[inset] x y blur [spread] cor`.
struct Camada {
    y: f64,
    blur: f64,
    spread: f64,
    cor: Rgba,
    inset: bool,
}

fn parse_camada(token: &str, s: &str, todos: &HashMap<String, String>) -> Result<Camada, TokenError> {
    let mut s = s.trim();
    let mut inset = false;
    if let Some(r) = s.strip_prefix("inset ") {
        inset = true;
        s = r.trim();
    }
    // A cor é o que começa em 'rgba(', 'rgb(', 'var(', 'color-mix(' ou '#': separa os números do início.
    let corte = ["rgba(", "rgb(", "var(", "color-mix(", "#"]
        .iter()
        .filter_map(|p| s.find(p))
        .min()
        .ok_or_else(|| erro(token, format!("sombra sem cor: {s}")))?;
    let nums: Vec<&str> = s[..corte].split_whitespace().collect();
    if nums.len() < 3 || nums.len() > 4 {
        return Err(erro(token, format!("sombra deve ter 3 ou 4 medidas: {s}")));
    }
    let _x = parse_px(token, nums[0])?;
    let y = parse_px(token, nums[1])?;
    let blur = parse_px(token, nums[2])?;
    let spread = match nums.get(3) {
        Some(n) => parse_px(token, n)?,
        None => 0.0,
    };
    let cor = parse_cor(token, &s[corte..], todos, 0)?;
    Ok(Camada { y, blur, spread, cor, inset })
}

fn parse_fonte(token: &str, s: &str) -> Result<String, TokenError> {
    let primeira = divide_virgulas(s).into_iter().next().unwrap_or_default();
    let nome = primeira.trim().trim_matches('"').trim_matches('\'').trim();
    if nome.is_empty() {
        return Err(erro(token, "lista de fontes vazia"));
    }
    // Famílias genéricas do CSS não existem na Slint.
    if nome == "ui-monospace" || nome == "monospace" {
        return Ok(MONO_WINDOWS.to_string());
    }
    Ok(nome.to_string())
}

/// Gera o texto do `tokens.slint` a partir do CSS. Falha dizendo o token que não pôde ser lido.
pub fn generate_slint(css: &str) -> Result<String, TokenError> {
    let decls = parse_root(css)?;
    let todos: HashMap<String, String> = decls.iter().cloned().collect();

    for req in REQUIRED {
        if !todos.contains_key(*req) {
            return Err(erro(req, "token obrigatório não existe em tokens.css"));
        }
    }

    let mut s = String::new();
    s.push_str("// GERADO pelo build.rs a partir de src/ui/tokens.css (R18). NÃO edite: mude o CSS e rode o build.\n");
    s.push_str("// Fora do git (.gitignore).\n\n");
    s.push_str("export global Tokens {\n");
    for (nome, valor) in &decls {
        if nome.starts_with("color-") {
            let c = parse_cor(nome, valor, &todos, 0)?;
            s.push_str(&format!("    out property <color> {nome}: {};\n", c.slint()));
        } else if nome.starts_with("font-") {
            s.push_str(&format!("    out property <string> {nome}: \"{}\";\n", parse_fonte(nome, valor)?));
        } else if nome.starts_with("space-") || nome.starts_with("radius-") {
            s.push_str(&format!("    out property <length> {nome}: {};\n", fmt_px(parse_px(nome, valor)?)));
        } else if nome.starts_with("shadow-") {
            // Duas camadas: anel (sem desfoque, com espalhamento) e sombra caída.
            let camadas: Vec<Camada> = divide_virgulas(valor)
                .iter()
                .map(|c| parse_camada(nome, c, &todos))
                .collect::<Result<_, _>>()?;
            if camadas.len() != 2 || camadas[0].blur != 0.0 || camadas[0].spread == 0.0 || camadas[1].blur == 0.0 {
                return Err(erro(nome, format!("esperava anel + sombra caída, veio '{valor}'")));
            }
            s.push_str(&format!("    out property <color> {nome}-ring: {};\n", camadas[0].cor.slint()));
            s.push_str(&format!("    out property <length> {nome}-ring-width: {};\n", fmt_px(camadas[0].spread)));
            s.push_str(&format!("    out property <color> {nome}-color: {};\n", camadas[1].cor.slint()));
            s.push_str(&format!("    out property <length> {nome}-offset-y: {};\n", fmt_px(camadas[1].y)));
            s.push_str(&format!("    out property <length> {nome}-blur: {};\n", fmt_px(camadas[1].blur)));
        } else if nome.starts_with("ring-") {
            let c = parse_camada(nome, valor, &todos)?;
            if c.blur != 0.0 || c.spread == 0.0 {
                return Err(erro(nome, format!("esperava anel sem desfoque, veio '{valor}'")));
            }
            s.push_str(&format!("    out property <length> {nome}-width: {};\n", fmt_px(c.spread)));
            s.push_str(&format!("    out property <color> {nome}-color: {};\n", c.cor.slint()));
            s.push_str(&format!("    out property <bool> {nome}-inset: {};\n", c.inset));
        } else {
            return Err(erro(nome, "prefixo sem tipo conhecido (use color-, font-, space-, radius-, shadow-, ring-)"));
        }
    }
    s.push_str("}\n");
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CSS_REAL: &str = include_str!("../../../../src/ui/tokens.css");

    fn minimo(extra: &str) -> String {
        // Um CSS com todos os tokens obrigatórios, para testar um token isolado.
        let mut css = String::from(":root {\n");
        for r in REQUIRED {
            let v = if r.starts_with("color-text-muted") {
                "color-mix(in srgb, var(--color-text) 60%, transparent)"
            } else if r.starts_with("color-") {
                "#112233"
            } else if r.starts_with("font-") {
                "\"X\", sans-serif"
            } else if r.starts_with("space-") || r.starts_with("radius-") {
                "4px"
            } else if r.starts_with("shadow-") {
                "0 0 0 1px rgba(0,0,0,0.1), 0 6px 18px rgba(0,0,0,0.4)"
            } else {
                "0 0 0 2px #112233"
            };
            css.push_str(&format!("--{r}: {v};\n"));
        }
        css.push_str(extra);
        css.push_str("}\n");
        css
    }

    #[test]
    fn le_o_tokens_css_real() {
        let s = generate_slint(CSS_REAL).expect("tokens.css real deve ser legível");
        assert!(s.contains("export global Tokens"));
        assert!(s.contains("color-accent: #c67139;"));
        assert!(s.contains("space-1: 4.4px;"));
        assert!(s.contains("radius-pill: 999px;"));
        assert!(s.contains("font-heading: \"Caprasimo\";"));
    }

    #[test]
    fn mesmo_numero_de_cores_do_css() {
        let decls = parse_root(CSS_REAL).unwrap();
        let no_css = decls.iter().filter(|(n, _)| n.starts_with("color-")).count();
        let gerado = generate_slint(CSS_REAL).unwrap();
        let no_slint = gerado.lines().filter(|l| l.contains("<color> color-")).count();
        assert_eq!(no_css, 27, "tokens.css mudou: confira as telas");
        assert_eq!(no_slint, no_css);
    }

    #[test]
    fn arquivo_gerado_pelo_build_confere_com_o_css() {
        // O build.rs já gerou ui/tokens.slint; a contagem tem de bater com o CSS de hoje.
        let gerado = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/tokens.slint"))
            .expect("ui/tokens.slint é gerado pelo build.rs");
        assert_eq!(gerado, generate_slint(CSS_REAL).unwrap());
    }

    #[test]
    fn color_mix_vira_a_mesma_cor_com_alfa() {
        let s = generate_slint(CSS_REAL).unwrap();
        assert!(s.contains("color-text-muted: rgba(245, 234, 216, 0.6);"), "{s}");
    }

    #[test]
    fn rgba_passa_com_alfa() {
        let s = generate_slint(CSS_REAL).unwrap();
        assert!(s.contains("color-divider: rgba(245, 234, 216, 0.13);"));
    }

    #[test]
    fn sombras_viram_anel_e_sombra_caida() {
        let s = generate_slint(CSS_REAL).unwrap();
        assert!(s.contains("shadow-lg-ring: rgba(245, 234, 216, 0.08);"));
        assert!(s.contains("shadow-lg-ring-width: 1px;"));
        assert!(s.contains("shadow-lg-offset-y: 18px;"));
        assert!(s.contains("shadow-lg-blur: 48px;"));
        assert!(s.contains("shadow-lg-color: rgba(0, 0, 0, 0.55);"));
        assert!(s.contains("ring-focus-width: 3px;"));
        assert!(s.contains("ring-live-inset: true;"));
    }

    #[test]
    fn fonte_monoespacada_vira_a_do_windows() {
        let s = generate_slint(CSS_REAL).unwrap();
        assert!(s.contains("font-mono: \"Consolas\";"));
    }

    #[test]
    fn comentarios_no_fim_da_linha_sao_ignorados() {
        let decls = parse_root(":root { --a: 1px; /* x: y; */ --b: 2px; /* z */ }").unwrap();
        assert_eq!(decls, vec![("a".into(), "1px".into()), ("b".into(), "2px".into())]);
    }

    #[test]
    fn cor_invalida_diz_qual_token() {
        let css = minimo("--color-quebrado: #12;");
        let e = generate_slint(&css).unwrap_err();
        assert_eq!(e.token, "color-quebrado");
        assert!(e.to_string().contains("--color-quebrado"));
    }

    #[test]
    fn token_obrigatorio_ausente_diz_qual() {
        let css = CSS_REAL.replace("--color-raised:", "--color-raisedx:");
        let e = generate_slint(&css).unwrap_err();
        assert_eq!(e.token, "color-raised");
    }

    #[test]
    fn var_inexistente_diz_qual_token() {
        let css = minimo("--color-x: color-mix(in srgb, var(--nao-existe) 50%, transparent);");
        let e = generate_slint(&css).unwrap_err();
        assert_eq!(e.token, "color-x");
        assert!(e.motivo.contains("nao-existe"));
    }

    #[test]
    fn medida_invalida_diz_qual_token() {
        let css = minimo("--space-9: 3em;");
        assert_eq!(generate_slint(&css).unwrap_err().token, "space-9");
    }

    #[test]
    fn prefixo_desconhecido_falha() {
        let css = minimo("--zzz-1: 3px;");
        assert_eq!(generate_slint(&css).unwrap_err().token, "zzz-1");
    }

    #[test]
    fn sem_root_falha() {
        assert!(generate_slint("body { color: red; }").is_err());
    }
}
