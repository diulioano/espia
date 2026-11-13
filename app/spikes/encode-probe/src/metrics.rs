//! Uso de processador e memória privada do próprio processo (G1, itens f e i).
//! No Windows usa GetProcessTimes e K32GetProcessMemoryInfo (PrivateUsage = "bytes privados").
//! Em outros sistemas devolve zeros: a sonda é do Windows.

use std::time::Instant;

pub struct ProcSampler {
    last_wall: Instant,
    last_cpu_100ns: u64,
    cores: f64,
}

pub struct ProcSample {
    /// Uso de processador em % de UM núcleo (200 = dois núcleos cheios).
    pub cpu_one_core_pct: f64,
    /// Uso em % da máquina inteira (todos os núcleos lógicos).
    pub cpu_machine_pct: f64,
    pub private_mb: f64,
    pub working_set_mb: f64,
}

impl ProcSampler {
    pub fn new() -> Self {
        let cores = std::thread::available_parallelism().map(|n| n.get() as f64).unwrap_or(1.0);
        ProcSampler { last_wall: Instant::now(), last_cpu_100ns: cpu_time_100ns(), cores }
    }

    pub fn sample(&mut self) -> ProcSample {
        let now = Instant::now();
        let cpu = cpu_time_100ns();
        let wall = now.duration_since(self.last_wall).as_secs_f64().max(0.001);
        let used = cpu.saturating_sub(self.last_cpu_100ns) as f64 / 1e7;
        self.last_wall = now;
        self.last_cpu_100ns = cpu;
        let one = used / wall * 100.0;
        let (private, ws) = memory_bytes();
        ProcSample {
            cpu_one_core_pct: one,
            cpu_machine_pct: one / self.cores,
            private_mb: private as f64 / 1_048_576.0,
            working_set_mb: ws as f64 / 1_048_576.0,
        }
    }

    pub fn cores(&self) -> usize {
        self.cores as usize
    }
}

#[cfg(windows)]
fn cpu_time_100ns() -> u64 {
    use windows_sys::Win32::Foundation::FILETIME;
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, GetProcessTimes};
    let zero = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
    let (mut c, mut e, mut k, mut u) = (zero, zero, zero, zero);
    // SAFETY: ponteiros para variáveis locais válidas; GetCurrentProcess devolve um pseudo-handle.
    let ok = unsafe { GetProcessTimes(GetCurrentProcess(), &mut c, &mut e, &mut k, &mut u) };
    if ok == 0 {
        return 0;
    }
    let ft = |f: FILETIME| ((f.dwHighDateTime as u64) << 32) | f.dwLowDateTime as u64;
    ft(k) + ft(u)
}

#[cfg(windows)]
fn memory_bytes() -> (u64, u64) {
    use windows_sys::Win32::System::ProcessStatus::{K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX};
    use windows_sys::Win32::System::Threading::GetCurrentProcess;
    // SAFETY: estrutura zerada com `cb` preenchido; o Windows escreve só até `cb` bytes.
    unsafe {
        let mut c: PROCESS_MEMORY_COUNTERS_EX = std::mem::zeroed();
        c.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
        let ok = K32GetProcessMemoryInfo(GetCurrentProcess(), &mut c as *mut _ as *mut PROCESS_MEMORY_COUNTERS, c.cb);
        if ok == 0 {
            return (0, 0);
        }
        (c.PrivateUsage as u64, c.WorkingSetSize as u64)
    }
}

#[cfg(not(windows))]
fn cpu_time_100ns() -> u64 {
    0
}

#[cfg(not(windows))]
fn memory_bytes() -> (u64, u64) {
    (0, 0)
}
