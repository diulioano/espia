//! Memória do processo no Windows.
//! "Conjunto de trabalho privado" é o número da coluna "Memória" do Gerenciador de Tarefas: as páginas
//! do conjunto de trabalho que não são compartilhadas. Vem de `QueryWorkingSet` (cada página traz o bit
//! "compartilhada"). Também imprimimos a memória confirmada (`PrivateUsage`), que costuma ser maior.

#[cfg(windows)]
mod win {
    use windows_sys::Win32::System::ProcessStatus::{
        K32GetProcessMemoryInfo, K32QueryWorkingSet, PROCESS_MEMORY_COUNTERS_EX,
    };
    use windows_sys::Win32::System::Threading::GetCurrentProcess;

    const PAGINA: usize = 4096;
    /// PSAPI_WORKING_SET_BLOCK: bits 0-4 proteção, 5-7 contagem de compartilhamento, bit 8 "compartilhada".
    const BIT_COMPARTILHADA: usize = 1 << 8;

    /// Bytes do conjunto de trabalho privado, ou None se a consulta falhar.
    pub fn conjunto_privado() -> Option<usize> {
        unsafe {
            let proc = GetCurrentProcess();
            // buffer: [NumberOfEntries, entradas...] (tudo usize). Cresce até caber.
            let mut cap = 1 << 20;
            for _ in 0..8 {
                let mut buf = vec![0usize; cap];
                if K32QueryWorkingSet(proc, buf.as_mut_ptr().cast(), (cap * size_of::<usize>()) as u32) != 0 {
                    let n = buf[0];
                    let privadas = buf[1..=n].iter().filter(|e| **e & BIT_COMPARTILHADA == 0).count();
                    return Some(privadas * PAGINA);
                }
                cap *= 4;
            }
            None
        }
    }

    /// (confirmada `PrivateUsage`, conjunto de trabalho total) em bytes.
    pub fn contadores() -> Option<(usize, usize)> {
        unsafe {
            let mut c: PROCESS_MEMORY_COUNTERS_EX = std::mem::zeroed();
            c.cb = size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
            let ok = K32GetProcessMemoryInfo(GetCurrentProcess(), (&mut c as *mut PROCESS_MEMORY_COUNTERS_EX).cast(), c.cb);
            (ok != 0).then_some((c.PrivateUsage, c.WorkingSetSize))
        }
    }
}

/// Linha de texto com a memória do processo, em MB.
pub fn descricao() -> String {
    #[cfg(windows)]
    {
        let mb = |b: usize| b as f64 / (1024.0 * 1024.0);
        let priv_ws = win::conjunto_privado().map(|b| format!("{:.1}", mb(b))).unwrap_or_else(|| "?".into());
        let (commit, ws) = win::contadores().map(|(a, b)| (format!("{:.1}", mb(a)), format!("{:.1}", mb(b)))).unwrap_or(("?".into(), "?".into()));
        format!("memória privada (conjunto de trabalho privado): {priv_ws} MB [confirmada {commit} MB, conjunto total {ws} MB]")
    }
    #[cfg(not(windows))]
    {
        "memória: só medida no Windows".to_string()
    }
}
