// Onde está o Discord (research R7): as raízes das árvores de processo dele. A tela inteira com som
// exclui essa árvore do loopback; com mais de uma raiz não dá (o Windows aceita uma exclusão só).
use super::procs::Table;

const EXES: [&str; 3] = ["discord.exe", "discordptb.exe", "discordcanary.exe"];

fn is_discord(exe: &str) -> bool {
  EXES.iter().any(|d| exe.eq_ignore_ascii_case(d))
}

/// Processos do Discord cujo pai não é o mesmo executável. Ordenado (para comparar mudanças).
pub fn discord_roots(table: &Table) -> Vec<u32> {
  let mut roots: Vec<u32> = table
    .values()
    .filter(|p| is_discord(&p.exe))
    .filter(|p| table.get(&p.ppid).is_none_or(|parent| !parent.exe.eq_ignore_ascii_case(&p.exe)))
    .map(|p| p.pid)
    .collect();
  roots.sort_unstable();
  roots
}

#[cfg(test)]
mod tests {
  use super::super::procs::table;
  use super::*;

  #[test]
  fn sem_discord() {
    assert!(discord_roots(&table(&[(1, 0, "explorer.exe"), (5, 1, "Game.exe")])).is_empty());
  }

  #[test]
  fn um_discord_com_filhos() {
    let t = table(&[(1, 0, "explorer.exe"), (20, 1, "Discord.exe"), (21, 20, "Discord.exe"), (22, 20, "Discord.exe")]);
    assert_eq!(discord_roots(&t), vec![20]);
  }

  #[test]
  fn discord_e_ptb_juntos() {
    let t = table(&[(20, 1, "Discord.exe"), (21, 20, "Discord.exe"), (30, 1, "DiscordPTB.exe"), (31, 30, "DiscordPTB.exe")]);
    assert_eq!(discord_roots(&t), vec![20, 30]);
  }

  #[test]
  fn reinicio_muda_o_pid() {
    let antes = discord_roots(&table(&[(20, 1, "Discord.exe")]));
    let depois = discord_roots(&table(&[(77, 1, "Discord.exe"), (78, 77, "Discord.exe")]));
    assert_ne!(antes, depois);
    assert_eq!(depois, vec![77]);
  }

  #[test]
  fn atualizador_iniciado_pelo_proprio_discord_nao_conta_como_raiz() {
    // Update.exe → Discord.exe (raiz) → Discord.exe (filhos)
    let t = table(&[(10, 1, "Update.exe"), (20, 10, "Discord.exe"), (21, 20, "Discord.exe")]);
    assert_eq!(discord_roots(&t), vec![20]);
  }
}
