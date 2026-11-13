//! Opções de linha de comando (sem dependência externa, para a sonda continuar pequena).

use crate::proto::ViewMode;

pub const AJUDA: &str = "\
Sonda G1 (019, T009): prova da libwebrtc (LiveKit) para o app nativo do Espia.

USO
  webrtc-probe --room <CODIGO> [opções]
  webrtc-probe <CODIGO> [opções]

LIGAÇÃO
  --base <URL>          endereço do site/servidor (padrão http://127.0.0.1:8787). O cabeçalho Origin
                        enviado no WebSocket é a origem desse endereço.
  --room <CODIGO>       código da sala (obrigatório; também pode ser o 1º argumento sem nome)
  --nick <APELIDO>      apelido na sala (padrão \"Sonda G1\")
  --view <modo>         como a sonda pede para ver quem transmite: full | thumb | hidden | off
                        (padrão full; é o ViewMessage versão 2 que ela manda pelo canal ctl)
  --no-live             não transmite (só recebe e troca ViewMessage)
  --low-latency         cria a fábrica com PeerConnectionFactory::with_zero_playout_delay()

VÍDEO E SOM GERADOS (nunca captura real)
  --size <LxA>          tamanho do vídeo gerado (padrão 1920x1080)
  --gen-fps <N>         quadros por segundo gerados (padrão 60)
  --backend <b>         codificador preferido: auto | software | hardware | nvenc  (padrão auto)
  --no-h264             não pede H.264 primeiro (deixa a ordem de codecs padrão)

PARÂMETROS POR LIGAÇÃO (valem para todas as ligações que estão recebendo a nossa imagem)
  --bitrate <bps>       maxBitrate do vídeo
  --fps <N>             maxFramerate do vídeo
  --scale <X>           scaleResolutionDownBy do vídeo (1 = sem redução)
  --active <true|false> active do vídeo
  --cycle <segundos>    troca os parâmetros em ciclos (cheio, miniatura, fps 30, 1 Mbps, escala 2, inativo)
  Ao vivo, pelo teclado (uma linha + Enter): bitrate N | fps N | scale X | active on|off | reset |
  view full|thumb|hidden|off | stats | quit

MEDIÇÃO
  --stats-secs <N>      intervalo do log de estatísticas e de processador/memória (padrão 5)
  --minutes <N>         quantos minutos de medição de som x imagem, um relatório por minuto (padrão 10)
  --detect-pulse        detecta o pulso gerado pela própria sonda (clarão no vídeo + bip no som) nos quadros
                        recebidos; só serve quando a outra ponta é outra sonda
  --exit-after          encerra sozinha depois dos --minutes minutos

  -h, --help            esta ajuda
";

#[derive(Debug, Clone)]
pub struct Args {
    pub base: String,
    pub room: String,
    pub nick: String,
    pub view: ViewMode,
    pub no_live: bool,
    pub low_latency: bool,
    pub width: u32,
    pub height: u32,
    pub gen_fps: u32,
    pub backend: String,
    pub no_h264: bool,
    pub bitrate: Option<u64>,
    pub fps: Option<f64>,
    pub scale: Option<f64>,
    pub active: Option<bool>,
    pub cycle_secs: Option<u64>,
    pub stats_secs: u64,
    pub minutes: u64,
    pub detect_pulse: bool,
    pub exit_after: bool,
}

pub fn parse() -> Result<Args, String> {
    let mut it = std::env::args().skip(1).peekable();
    let mut a = Args {
        base: "http://127.0.0.1:8787".into(),
        room: String::new(),
        nick: "Sonda G1".into(),
        view: ViewMode::Full,
        no_live: false,
        low_latency: false,
        width: 1920,
        height: 1080,
        gen_fps: 60,
        backend: "auto".into(),
        no_h264: false,
        bitrate: None,
        fps: None,
        scale: None,
        active: None,
        cycle_secs: None,
        stats_secs: 5,
        minutes: 10,
        detect_pulse: false,
        exit_after: false,
    };
    fn val(it: &mut impl Iterator<Item = String>, name: &str) -> Result<String, String> {
        it.next().ok_or_else(|| format!("falta o valor de {name}"))
    }
    fn num<T: std::str::FromStr>(s: String, name: &str) -> Result<T, String> {
        s.parse::<T>().map_err(|_| format!("valor inválido em {name}: {s}"))
    }
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print!("{AJUDA}");
                std::process::exit(0);
            }
            "--base" => a.base = val(&mut it, "--base")?,
            "--room" => a.room = val(&mut it, "--room")?,
            "--nick" => a.nick = val(&mut it, "--nick")?,
            "--view" => {
                let v = val(&mut it, "--view")?;
                a.view = ViewMode::parse(&v).ok_or_else(|| format!("--view inválido: {v}"))?;
            }
            "--no-live" => a.no_live = true,
            "--low-latency" => a.low_latency = true,
            "--size" => {
                let v = val(&mut it, "--size")?;
                let (w, h) = v.split_once('x').ok_or("--size precisa ser LARGURAxALTURA")?;
                a.width = num(w.to_string(), "--size")?;
                a.height = num(h.to_string(), "--size")?;
            }
            "--gen-fps" => a.gen_fps = num(val(&mut it, "--gen-fps")?, "--gen-fps")?,
            "--backend" => a.backend = val(&mut it, "--backend")?,
            "--no-h264" => a.no_h264 = true,
            "--bitrate" => a.bitrate = Some(num(val(&mut it, "--bitrate")?, "--bitrate")?),
            "--fps" => a.fps = Some(num(val(&mut it, "--fps")?, "--fps")?),
            "--scale" => a.scale = Some(num(val(&mut it, "--scale")?, "--scale")?),
            "--active" => {
                let v = val(&mut it, "--active")?;
                a.active = Some(matches!(v.as_str(), "true" | "on" | "1" | "sim"));
            }
            "--cycle" => a.cycle_secs = Some(num(val(&mut it, "--cycle")?, "--cycle")?),
            "--stats-secs" => a.stats_secs = num(val(&mut it, "--stats-secs")?, "--stats-secs")?,
            "--minutes" => a.minutes = num(val(&mut it, "--minutes")?, "--minutes")?,
            "--detect-pulse" => a.detect_pulse = true,
            "--exit-after" => a.exit_after = true,
            other if !other.starts_with('-') && a.room.is_empty() => a.room = other.to_string(),
            other => return Err(format!("opção desconhecida: {other} (veja --help)")),
        }
    }
    if a.room.is_empty() {
        return Err("falta o código da sala (--room CODIGO). Veja --help".into());
    }
    if a.width < 16 || a.height < 16 || a.width % 2 != 0 || a.height % 2 != 0 {
        return Err("--size precisa ter largura e altura pares e maiores que 16".into());
    }
    if a.gen_fps == 0 || a.gen_fps > 120 {
        return Err("--gen-fps precisa estar entre 1 e 120".into());
    }
    Ok(a)
}
