// Sem janela de console no Windows em release. NÃO REMOVER.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  espia_lib::run();
}
