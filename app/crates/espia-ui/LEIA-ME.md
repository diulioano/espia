# espia-ui: telas em Slint

Depende só de `espia-core`. A tela recebe view-models (`Screen`) e devolve ações (`UiAction`); nunca fala com rede, captura ou mídia.

## Conferir uma tela (a partir de `app/`, um build por vez, sempre `-j 6`)

```text
cargo run -p espia -j 6 -- --mock                       # índice
cargo run -p espia -j 6 -- --mock home --scene empty    # uma cena
cargo run -p espia -j 6 -- --mock home --screenshot ..\home.png   # salva a janela após 1 s e sai (abra o PNG com Read)
... --exit-after 5                                      # fecha sozinho
```

## Acrescentar uma tela (home, join, notfound, offline, room): só estes arquivos

1. `ui/pages/<tela>.slint`: troque o conteúdo de `<Tela>Page`; ponha dados e callbacks no global `<Tela>State` (o `app.slint` já o importa e reexporta: NÃO mexa nele). Tipos próprios (struct/enum) da tela ficam no mesmo arquivo e precisam de `export`; para o Rust enxergá-los, use-os em propriedade do global.
2. `src/bind/<tela>.rs`: `apply(app, &View)` copia o view-model para `<Tela>State` (`app.global::<HomeState>().set_x(..)`); `connect(app, &emit)` liga cada callback a `emit(UiAction::Home(HomeAction::..))`.
3. Componentes novos da tela: `ui/components/<nome>.slint`, importados pela página.
4. Cenas: `src/mocks/<tela>.rs`, `pub fn scenes() -> Vec<Scene<View>>` com `Scene::new(id, code, label, view)`, mesmos `id`, `code` e `label` de `src/mocks/<tela>.ts`. O registro (`mocks/registry.rs`) já junta tudo; a cena padrão de cada tela é `default` (room: `single`, notfound: `quick`).
5. Rodar: `--mock <tela> --scene <cena>`; a barra "MOCK" no canto troca de cena.

Com `--screenshot` a barra "MOCK" começa recolhida (sem ele, aberta como no site). A captura é lida dentro do `AfterRendering`: ler o buffer de trás depois do swap saía tudo branco no Windows.

Receitas Slint: não há "primeira letra" (use `espia_ui::initial(nome)`), não há `max-length` no `TextInput` (corte o texto em `edited`), e `border-radius: Tokens.radius-pill` (999px) NÃO dá cápsula: use `self.height / 2`. Elemento dentro de `if` não pode ser citado fora dele. O método `focus` já existe em todo elemento (nomeie outro, como `focus-input`).

## Tokens (`ui/tokens.slint`, GERADO, fora do git)

`build.rs` lê `src/ui/tokens.css` (raiz do repositório) e gera `Tokens` (`Tokens.color-accent`, `Tokens.space-2`, `Tokens.radius-card`, `Tokens.font-heading`, `Tokens.shadow-lg-*`, `Tokens.ring-focus-*`). Nunca escreva cor, raio ou fonte solta numa tela: use token (`Tokens.color-text.with-alpha(0.14)` vale). Token ilegível quebra o build dizendo qual. Ícones: `Icon { source: Icons.user-plus; size: 17px; tint: ...; }` (SVG Lucide em `assets/icons`, traço 2.75; acrescente o SVG e uma linha em `components/icon.slint`).

## Componentes base (`ui/components/`; vitrine: `--mock components [--scene all|logo|mascot|buttons|avatars|spinner|fields|toasts]`)

- `buttons.slint`: `Button { label, icon: image, kind: ButtonKind (primary|secondary|stop|invite|ghost|plain), copied, enabled, font-size, icon-size, pad-v, pad-h; callback clicked() }`; `IconButton { icon, icon-size, size, label, enabled; clicked() }` (`.btn-icon`).
- `logo.slint`: `Logo { size; callback home() }`. `mascot.slint`: `Mascot { mood: MascotMood, size, tail, animate: MascotAnimate (off|blink|full) }`.
- `avatar.slint`: `Avatar { letter, tone: Tone (accent|sage|sand|deep), live, size, ring-gap }`, struct `AvatarData { letter, tone }`. `avatar-stack.slint`: `AvatarStack { people: [AvatarData], size, ring }`.
- `spinner.slint`: `Spinner { size }`. `text-field.slint`: `TextField { text (in-out), label, hint, error, placeholder, enabled; accepted(string), edited(string); focus-input() }`.
- `toast.slint`: `Toast { kind: ToastKind (join|leave|live|copied|error), text, action; action-clicked() }`, struct `ToastData { id, kind, text, action }`. `toast-stack.slint`: `ToastStack { toasts: [ToastData], placement: ToastPlacement (top-right|bottom-center|inline); action-clicked(id), dismissed(id) }` (só os 3 mais recentes).
- `scroll.slint`: `Scroll { viewport-y }` com a barra do site; os filhos entram numa coluna. `icon.slint`: `Icon`, global `Icons`.
- Telas de apoio: `mock-index.slint` e `mock-switcher.slint` (índice e barra do mock), `pages/components.slint` (vitrine).
