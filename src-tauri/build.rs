fn main() {
  // APIs nativas expostas à página (constituição 1.4.0): a ponte de captura (specs/005,
  // contracts/native-capture.md) e o aviso de ao vivo da bandeja (specs/007, contracts/tray.md).
  // Liberada só para a origem do site por capabilities/capture.json.
  tauri_build::try_build(tauri_build::Attributes::new().app_manifest(tauri_build::AppManifest::new().commands(&[
    "capture_supported",
    "capture_list",
    "capture_start",
    "capture_set_quality",
    "capture_stop",
    "app_set_live",
  ])))
  .expect("falha no tauri-build");
}
