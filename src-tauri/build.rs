fn main() {
  // Única API nativa exposta à página: a ponte de captura (specs/005, contracts/native-capture.md).
  // Liberada só para a origem do site por capabilities/capture.json.
  tauri_build::try_build(tauri_build::Attributes::new().app_manifest(tauri_build::AppManifest::new().commands(&[
    "capture_supported",
    "capture_list",
    "capture_start",
    "capture_set_quality",
    "capture_stop",
  ])))
  .expect("falha no tauri-build");
}
