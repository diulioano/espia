//! Liga a tela de sem internet (`ui/pages/offline.slint`). Não há view-model: só o botão.

use crate::{AppWindow, Emit, OfflineAction, OfflineState, UiAction};
use slint::ComponentHandle;

pub(crate) fn connect(app: &AppWindow, emit: &Emit) {
    let emit = emit.clone();
    app.global::<OfflineState>().on_retry(move || emit(UiAction::Offline(OfflineAction::Retry)));
}
