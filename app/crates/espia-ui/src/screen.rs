//! A API de dados da tela: o que ela recebe (`Screen`) e o que devolve (`UiAction`).

use espia_core::view_model::{HomeAction, HomeView, JoinAction, JoinView, NotFoundAction, NotFoundView, RoomAction, RoomView};

/// Cena da vitrine dos componentes base (`--mock components`): `all`, `logo`, `mascot`, `buttons`,
/// `avatars`, `spinner`, `fields` ou `toasts`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentsScene(pub String);

/// O que a janela mostra agora.
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    /// Índice do mock; `erro` é a linha de aviso no alto ("a tela x não existe").
    MockIndex { erro: Option<String> },
    Home(HomeView),
    Join(JoinView),
    Room(RoomView),
    NotFound(NotFoundView),
    /// Sem internet (porte de `src-tauri/www/index.html`, T030).
    Offline,
    /// Vitrine dos componentes base. Só existe no mock (desenvolvimento).
    Components(ComponentsScene),
}

/// Ações da tela de sem internet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflineAction {
    /// "Tentar de novo".
    Retry,
}

/// Navegação do mock (índice e seletor de cenas).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MockAction {
    /// Abrir uma tela; `scene` ausente = a cena padrão dela.
    Open { screen: String, scene: Option<String> },
    /// "todas as telas".
    Index,
}

/// Tudo que a tela pode devolver a quem a controla.
#[derive(Debug, Clone, PartialEq)]
pub enum UiAction {
    Home(HomeAction),
    Join(JoinAction),
    Room(RoomAction),
    NotFound(NotFoundAction),
    Offline(OfflineAction),
    Mock(MockAction),
    /// "Atualizar" na tela "Atualize o Espia para continuar" (o site recarrega a página; o app abre a sala de novo).
    Reload,
}
