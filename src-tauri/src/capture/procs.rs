// Tabela de processos e "raiz da árvore" de um programa (research R6/R7).
// Um navegador ou o Discord tocam som por processos filhos com o mesmo executável; o loopback de
// processo do Windows age sobre uma árvore inteira, então mira a raiz.
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proc {
  pub pid: u32,
  pub ppid: u32,
  /// Nome do executável, ex.: "Discord.exe".
  pub exe: String,
}

pub type Table = HashMap<u32, Proc>;

/// Sobe pelos pais enquanto o pai tiver o mesmo executável (sem diferenciar maiúsculas).
pub fn tree_root(pid: u32, table: &Table) -> u32 {
  let mut cur = pid;
  // Limite: a tabela pode ter ciclos (PIDs reaproveitados).
  for _ in 0..64 {
    let Some(p) = table.get(&cur) else { break };
    match table.get(&p.ppid) {
      Some(parent) if parent.pid != cur && parent.exe.eq_ignore_ascii_case(&p.exe) => cur = parent.pid,
      _ => break,
    }
  }
  cur
}

#[cfg(windows)]
pub fn snapshot() -> Table {
  use windows::Win32::{
    Foundation::CloseHandle,
    System::Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS},
  };
  let mut table = Table::new();
  unsafe {
    let Ok(snap) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else { return table };
    let mut entry = PROCESSENTRY32W { dwSize: size_of::<PROCESSENTRY32W>() as u32, ..Default::default() };
    let mut ok = Process32FirstW(snap, &mut entry).is_ok();
    while ok {
      let len = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len());
      let exe = String::from_utf16_lossy(&entry.szExeFile[..len]);
      table.insert(entry.th32ProcessID, Proc { pid: entry.th32ProcessID, ppid: entry.th32ParentProcessID, exe });
      ok = Process32NextW(snap, &mut entry).is_ok();
    }
    let _ = CloseHandle(snap);
  }
  table
}

#[cfg(test)]
pub fn table(rows: &[(u32, u32, &str)]) -> Table {
  rows.iter().map(|&(pid, ppid, exe)| (pid, Proc { pid, ppid, exe: exe.into() })).collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn navegador_com_processo_de_audio_filho() {
    let t = table(&[(1, 0, "explorer.exe"), (10, 1, "firefox.exe"), (11, 10, "firefox.exe"), (12, 11, "firefox.exe")]);
    assert_eq!(tree_root(12, &t), 10);
    assert_eq!(tree_root(10, &t), 10);
  }

  #[test]
  fn pai_de_outro_programa_para_a_subida() {
    let t = table(&[(1, 0, "explorer.exe"), (5, 1, "Game.exe")]);
    assert_eq!(tree_root(5, &t), 5);
  }

  #[test]
  fn processo_orfao_ou_desconhecido() {
    let t = table(&[(7, 999, "app.exe")]);
    assert_eq!(tree_root(7, &t), 7);
    assert_eq!(tree_root(42, &t), 42);
  }

  #[test]
  fn ciclo_nao_trava() {
    let t = table(&[(2, 3, "a.exe"), (3, 2, "a.exe")]);
    let r = tree_root(2, &t);
    assert!(r == 2 || r == 3);
  }

  #[test]
  fn maiusculas_nao_importam() {
    let t = table(&[(10, 1, "Discord.exe"), (11, 10, "discord.EXE")]);
    assert_eq!(tree_root(11, &t), 10);
  }
}
