// Som da transmissão (research R6–R8): só a árvore do programa da janela, ou o PC inteiro menos o
// Discord. Nunca cai para o loopback do dispositivo inteiro — sem plano seguro, vai sem som.

/// O que a página recebe como `audio` / `audioWarn` (contracts/native-capture.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Warn {
  Unsupported,
  ManyDiscords,
}

impl Warn {
  pub fn as_str(self) -> &'static str {
    match self {
      Warn::Unsupported => "unsupported",
      Warn::ManyDiscords => "many_discords",
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plan {
  None(Option<Warn>),
  /// Só o som dessa árvore de processos (janela escolhida).
  Include(u32),
  /// Todo o som menos essa árvore (tela inteira, menos o Discord).
  Exclude(u32),
}

impl Plan {
  /// `AudioKind` do protocolo.
  pub fn kind(self) -> &'static str {
    match self {
      Plan::None(_) => "none",
      Plan::Include(_) => "window",
      Plan::Exclude(_) => "pc",
    }
  }

  pub fn warn(self) -> Option<Warn> {
    match self {
      Plan::None(w) => w,
      _ => None,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
  /// Raiz da árvore do processo da janela.
  Window(u32),
  Monitor,
}

/// Tabela "AudioPlan" de data-model.md. A falha de ativação vira `None(Unsupported)` depois.
pub fn audio_plan(target: Target, wants: bool, discord_roots: &[u32], own_pid: u32) -> Plan {
  if !wants {
    return Plan::None(None);
  }
  match target {
    // Inclusive se a janela for do próprio Discord: foi escolhida de propósito.
    Target::Window(root) => Plan::Include(root),
    Target::Monitor => match discord_roots {
      // Sem Discord: excluir uma árvore que não toca nada (o próprio Espia) = som do PC inteiro.
      [] => Plan::Exclude(own_pid),
      [root] => Plan::Exclude(*root),
      _ => Plan::None(Some(Warn::ManyDiscords)),
    },
  }
}

#[cfg(windows)]
pub use imp::{Loopback, playing_roots};
#[cfg(all(test, windows))]
pub use imp::CHUNK_FRAMES;

#[cfg(windows)]
mod imp {
  use std::{
    collections::HashSet,
    mem::ManuallyDrop,
    sync::{
      Arc,
      atomic::{AtomicBool, Ordering},
      mpsc,
    },
    thread::{self, JoinHandle},
    time::Duration,
  };

  use windows::{
    Win32::{
      Foundation::{CloseHandle, WAIT_OBJECT_0},
      Media::Audio::{
        AUDCLNT_BUFFERFLAGS_SILENT, AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM,
        AUDCLNT_STREAMFLAGS_EVENTCALLBACK, AUDCLNT_STREAMFLAGS_LOOPBACK, AUDIOCLIENT_ACTIVATION_PARAMS,
        AUDIOCLIENT_ACTIVATION_PARAMS_0, AUDIOCLIENT_ACTIVATION_TYPE_PROCESS_LOOPBACK, AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS,
        ActivateAudioInterfaceAsync, AudioSessionStateActive, Endpoints::IAudioMeterInformation, IActivateAudioInterfaceAsyncOperation,
        IActivateAudioInterfaceCompletionHandler, IActivateAudioInterfaceCompletionHandler_Impl, IAudioCaptureClient, IAudioClient,
        IAudioSessionControl2, IAudioSessionManager2, IMMDeviceEnumerator, MMDeviceEnumerator, PROCESS_LOOPBACK_MODE_EXCLUDE_TARGET_PROCESS_TREE,
        PROCESS_LOOPBACK_MODE_INCLUDE_TARGET_PROCESS_TREE, VIRTUAL_AUDIO_DEVICE_PROCESS_LOOPBACK, WAVE_FORMAT_PCM, WAVEFORMATEX, eConsole,
        eRender,
      },
      System::{
        Com::{
          BLOB, CLSCTX_ALL, CoCreateInstance, IAgileObject,
          StructuredStorage::{PROPVARIANT, PROPVARIANT_0, PROPVARIANT_0_0, PROPVARIANT_0_0_0},
        },
        Threading::{CreateEventW, WaitForSingleObject},
        Variant::VT_BLOB,
      },
    },
    core::{Interface, Ref, implement},
  };

  use super::{super::com_init, Plan};
  use crate::capture::procs::{Table, tree_root};

  pub const RATE: u32 = 48_000;
  pub const CHANNELS: u16 = 2;
  /// 10 ms por pedaço (contracts/native-capture.md).
  pub const CHUNK_FRAMES: usize = 480;

  /// Raízes das árvores que estão tocando som agora no dispositivo de saída padrão (selo "tocando som").
  pub fn playing_roots(table: &Table) -> HashSet<u32> {
    let mut out = HashSet::new();
    let _com = com_init();
    unsafe {
      let Ok(enumerator) = CoCreateInstance::<_, IMMDeviceEnumerator>(&MMDeviceEnumerator, None, CLSCTX_ALL) else { return out };
      let Ok(device) = enumerator.GetDefaultAudioEndpoint(eRender, eConsole) else { return out };
      let Ok(manager) = device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None) else { return out };
      let Ok(sessions) = manager.GetSessionEnumerator() else { return out };
      let count = sessions.GetCount().unwrap_or(0);
      for i in 0..count {
        let Ok(control) = sessions.GetSession(i) else { continue };
        if control.GetState().ok() != Some(AudioSessionStateActive) {
          continue;
        }
        let Ok(control2) = control.cast::<IAudioSessionControl2>() else { continue };
        let Ok(pid) = control2.GetProcessId() else { continue };
        let peak = control.cast::<IAudioMeterInformation>().and_then(|m| m.GetPeakValue()).unwrap_or(0.0);
        if pid != 0 && peak > 0.001 {
          out.insert(tree_root(pid, table));
        }
      }
    }
    out
  }

  #[implement(IActivateAudioInterfaceCompletionHandler, IAgileObject)]
  struct Done(std::sync::Mutex<Option<mpsc::Sender<()>>>);

  // Ágil: o Windows chama o fim da ativação em qualquer thread.
  impl windows::Win32::System::Com::IAgileObject_Impl for Done_Impl {}

  impl IActivateAudioInterfaceCompletionHandler_Impl for Done_Impl {
    fn ActivateCompleted(&self, _op: Ref<IActivateAudioInterfaceAsyncOperation>) -> windows::core::Result<()> {
      if let Some(tx) = self.0.lock().unwrap().take() {
        let _ = tx.send(());
      }
      Ok(())
    }
  }

  fn activate(plan: Plan) -> windows::core::Result<IAudioClient> {
    let (pid, mode) = match plan {
      Plan::Include(pid) => (pid, PROCESS_LOOPBACK_MODE_INCLUDE_TARGET_PROCESS_TREE),
      Plan::Exclude(pid) => (pid, PROCESS_LOOPBACK_MODE_EXCLUDE_TARGET_PROCESS_TREE),
      Plan::None(_) => return Err(windows::core::Error::empty()),
    };
    let mut params = AUDIOCLIENT_ACTIVATION_PARAMS {
      ActivationType: AUDIOCLIENT_ACTIVATION_TYPE_PROCESS_LOOPBACK,
      Anonymous: AUDIOCLIENT_ACTIVATION_PARAMS_0 {
        ProcessLoopbackParams: AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS { TargetProcessId: pid, ProcessLoopbackMode: mode },
      },
    };
    // O PROPVARIANT só aponta para `params`: ManuallyDrop para ninguém tentar liberar o blob.
    let prop = ManuallyDrop::new(PROPVARIANT {
      Anonymous: PROPVARIANT_0 {
        Anonymous: ManuallyDrop::new(PROPVARIANT_0_0 {
          vt: VT_BLOB,
          wReserved1: 0,
          wReserved2: 0,
          wReserved3: 0,
          Anonymous: PROPVARIANT_0_0_0 {
            blob: BLOB { cbSize: size_of::<AUDIOCLIENT_ACTIVATION_PARAMS>() as u32, pBlobData: &mut params as *mut _ as *mut u8 },
          },
        }),
      },
    });
    let (tx, rx) = mpsc::channel();
    let handler: IActivateAudioInterfaceCompletionHandler = Done(std::sync::Mutex::new(Some(tx))).into();
    unsafe {
      let op = ActivateAudioInterfaceAsync(VIRTUAL_AUDIO_DEVICE_PROCESS_LOOPBACK, &IAudioClient::IID, Some(&*prop as *const _), &handler)?;
      rx.recv_timeout(Duration::from_secs(5)).map_err(|_| windows::core::Error::empty())?;
      let mut hr = Default::default();
      let mut unknown = None;
      op.GetActivateResult(&mut hr, &mut unknown)?;
      hr.ok()?;
      unknown.ok_or_else(windows::core::Error::empty)?.cast()
    }
  }

  /// Uma captura de som em andamento. Para quando é solta (Drop).
  pub struct Loopback {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
  }

  impl Loopback {
    /// Ativa e começa a mandar pedaços de 10 ms (f32 intercalado, 48 kHz, estéreo).
    pub fn start(plan: Plan, mut on_chunk: impl FnMut(Vec<u8>) + Send + 'static) -> windows::core::Result<Self> {
      let (ready_tx, ready_rx) = mpsc::channel::<windows::core::Result<()>>();
      let stop = Arc::new(AtomicBool::new(false));
      let stop2 = stop.clone();
      let thread = thread::Builder::new()
        .name("espia-audio".into())
        .spawn(move || {
          let _com = com_init();
          let started = (|| unsafe {
            let client = activate(plan)?;
            // 16-bit PCM como o exemplo oficial "Application Loopback"; convertemos para f32 aqui.
            let block = CHANNELS * 2;
            let format = WAVEFORMATEX {
              wFormatTag: WAVE_FORMAT_PCM as u16,
              nChannels: CHANNELS,
              nSamplesPerSec: RATE,
              nAvgBytesPerSec: RATE * block as u32,
              nBlockAlign: block,
              wBitsPerSample: 16,
              cbSize: 0,
            };
            client.Initialize(
              AUDCLNT_SHAREMODE_SHARED,
              AUDCLNT_STREAMFLAGS_LOOPBACK | AUDCLNT_STREAMFLAGS_EVENTCALLBACK | AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM,
              200_000, // 20 ms
              0,
              &format,
              None,
            )?;
            let event = CreateEventW(None, false, false, None)?;
            client.SetEventHandle(event)?;
            let capture: IAudioCaptureClient = client.GetService()?;
            client.Start()?;
            Ok::<_, windows::core::Error>((client, capture, event))
          })();
          let (client, capture, event) = match started {
            Ok(v) => {
              let _ = ready_tx.send(Ok(()));
              v
            }
            Err(e) => {
              let _ = ready_tx.send(Err(e));
              return;
            }
          };
          let mut pending: Vec<f32> = Vec::with_capacity(CHUNK_FRAMES * CHANNELS as usize * 2);
          let chunk = CHUNK_FRAMES * CHANNELS as usize;
          while !stop2.load(Ordering::Relaxed) {
            unsafe {
              if WaitForSingleObject(event, 100) != WAIT_OBJECT_0 {
                continue;
              }
              loop {
                let Ok(size) = capture.GetNextPacketSize() else { break };
                if size == 0 {
                  break;
                }
                let mut data = std::ptr::null_mut();
                let mut frames = 0u32;
                let mut flags = 0u32;
                if capture.GetBuffer(&mut data, &mut frames, &mut flags, None, None).is_err() {
                  break;
                }
                let samples = frames as usize * CHANNELS as usize;
                if flags & AUDCLNT_BUFFERFLAGS_SILENT.0 as u32 != 0 || data.is_null() {
                  pending.extend(std::iter::repeat_n(0.0, samples));
                } else {
                  let pcm = std::slice::from_raw_parts(data as *const i16, samples);
                  pending.extend(pcm.iter().map(|&s| s as f32 / 32768.0));
                }
                let _ = capture.ReleaseBuffer(frames);
              }
            }
            while pending.len() >= chunk {
              let bytes: Vec<u8> = pending.drain(..chunk).flat_map(f32::to_le_bytes).collect();
              on_chunk(bytes);
            }
          }
          unsafe {
            let _ = client.Stop();
            let _ = CloseHandle(event);
          }
        })
        .map_err(|_| windows::core::Error::empty())?;
      match ready_rx.recv_timeout(Duration::from_secs(6)) {
        Ok(Ok(())) => Ok(Self { stop, thread: Some(thread) }),
        Ok(Err(e)) => Err(e),
        Err(_) => {
          stop.store(true, Ordering::Relaxed);
          Err(windows::core::Error::empty())
        }
      }
    }
  }

  impl Drop for Loopback {
    fn drop(&mut self) {
      self.stop.store(true, Ordering::Relaxed);
      if let Some(t) = self.thread.take() {
        let _ = t.join();
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const ME: u32 = 4242;

  #[test]
  fn som_desligado_nunca_manda_nada() {
    assert_eq!(audio_plan(Target::Window(10), false, &[], ME), Plan::None(None));
    assert_eq!(audio_plan(Target::Monitor, false, &[20], ME), Plan::None(None));
  }

  #[test]
  fn janela_com_som_inclui_so_a_arvore_dela() {
    let p = audio_plan(Target::Window(10), true, &[20], ME);
    assert_eq!(p, Plan::Include(10));
    assert_eq!(p.kind(), "window");
  }

  #[test]
  fn janela_do_proprio_discord_vai_com_o_som_dela() {
    assert_eq!(audio_plan(Target::Window(20), true, &[20], ME), Plan::Include(20));
  }

  #[test]
  fn tela_sem_discord_exclui_o_espia() {
    let p = audio_plan(Target::Monitor, true, &[], ME);
    assert_eq!(p, Plan::Exclude(ME));
    assert_eq!(p.kind(), "pc");
  }

  #[test]
  fn tela_com_um_discord_exclui_ele() {
    assert_eq!(audio_plan(Target::Monitor, true, &[20], ME), Plan::Exclude(20));
  }

  #[test]
  fn tela_com_dois_discords_vai_sem_som_e_avisa() {
    let p = audio_plan(Target::Monitor, true, &[20, 30], ME);
    assert_eq!(p, Plan::None(Some(Warn::ManyDiscords)));
    assert_eq!(p.kind(), "none");
    assert_eq!(p.warn().map(Warn::as_str), Some("many_discords"));
  }
}
