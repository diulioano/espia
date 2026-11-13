//! Sonda G2 (019 T011): a tela da sala do Espia em Slint, com dados falsos e vídeo gerado em código.
//! Sem rede e sem captura. Uso: `ui-probe [--exit-after <segundos>]`.

mod mem;
mod video;

use slint::{ComponentHandle, ModelRc, VecModel};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

slint::include_modules!();

/// Contadores lidos pela linha de relatório a cada 5 s.
#[derive(Default)]
struct Contadores {
    /// Quadros que a Slint pintou de verdade (fim de cada renderização).
    pintados: AtomicU64,
    /// Quadros gerados pelo thread de vídeo e entregues à tela.
    entregues: AtomicU64,
    /// Quadros gerados que foram jogados fora porque a tela ainda não tinha consumido o anterior.
    descartados: AtomicU64,
}

fn pessoa(nome: &str, voce: bool, ao_vivo: bool, sub: &str, assistindo: bool, tom: i32) -> Person {
    Person {
        name: nome.into(),
        initial: nome.chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_default().into(),
        you: voce,
        live: ao_vivo,
        sub: sub.into(),
        watching: assistindo,
        tone: tom,
    }
}

fn segundos_de_saida() -> Option<u64> {
    let args: Vec<String> = std::env::args().collect();
    let i = args.iter().position(|a| a == "--exit-after")?;
    args.get(i + 1)?.parse().ok()
}

fn main() -> Result<(), slint::PlatformError> {
    let sair_em = segundos_de_saida();
    let ui = RoomWindow::new()?;

    // Cena `focus` de src/mocks/room.ts: Bia em foco, Lu e Tomás nas miniaturas, convite de teste local.
    ui.set_people(ModelRc::new(VecModel::from(vec![
        pessoa("Rafa", true, false, "", false, 3),
        pessoa("Lu", false, true, "Hollow Knight", false, 0),
        pessoa("Bia", false, true, "você está assistindo", true, 1),
        pessoa("Tomás", false, true, "Visual Studio Code", false, 2),
    ])));
    ui.set_capacity(10);
    ui.set_room_title("Vendo Bia".into());
    ui.set_room_subtitle("3 telas ao vivo".into());
    ui.set_invite_label("localhost:5173/s/k7f-q2x".into());
    ui.set_toast_text("Caio saiu da sala".into());
    ui.set_footer_hint("Tela, janela ou aba — você escolhe.".into());
    ui.set_strip_hint("Clique pra trocar · Esc volta pra grade".into());
    ui.set_stage_who("Bia".into());
    ui.set_thumb_a_who("Lu".into());
    ui.set_thumb_a_what("Hollow Knight".into());
    ui.set_thumb_b_who("Tomás".into());
    ui.set_thumb_b_what("Visual Studio Code".into());

    let c = Arc::new(Contadores::default());

    // Quadros pintados: o aviso de renderização da Slint (só existe com renderizador de placa de vídeo).
    let aviso_ok = {
        let c = c.clone();
        ui.window()
            .set_rendering_notifier(move |estado, _| {
                if matches!(estado, slint::RenderingState::AfterRendering) {
                    c.pintados.fetch_add(1, Ordering::Relaxed);
                }
            })
            .is_ok()
    };
    if !aviso_ok {
        eprintln!("aviso: este renderizador não permite contar quadros pintados; o FPS abaixo será o dos entregues");
    }

    // Pisca do castor do logo (o site pisca a cada 4,2 s).
    let pisca = slint::Timer::default();
    {
        let fraco = ui.as_weak();
        pisca.start(slint::TimerMode::Repeated, Duration::from_millis(4200), move || {
            if let Some(ui) = fraco.upgrade() {
                ui.set_eyes_open(false);
                let f2 = ui.as_weak();
                slint::Timer::single_shot(Duration::from_millis(140), move || {
                    if let Some(ui) = f2.upgrade() {
                        ui.set_eyes_open(true);
                    }
                });
            }
        });
    }

    // Vídeo gerado: palco 1920x1080 e duas miniaturas, 60 quadros por segundo.
    let parar = Arc::new(AtomicBool::new(false));
    let thread_video = video::iniciar(ui.as_weak(), c.clone(), parar.clone());

    // Relatório a cada 5 s no terminal.
    let inicio = Instant::now();
    let thread_relatorio = {
        let c = c.clone();
        let parar = parar.clone();
        std::thread::spawn(move || {
            let mut ultimo = Instant::now();
            let (mut p0, mut e0, mut d0) = (0u64, 0u64, 0u64);
            while !parar.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(100));
                if ultimo.elapsed() < Duration::from_secs(5) {
                    continue;
                }
                let dt = ultimo.elapsed().as_secs_f64();
                ultimo = Instant::now();
                let (p, e, d) = (
                    c.pintados.load(Ordering::Relaxed),
                    c.entregues.load(Ordering::Relaxed),
                    c.descartados.load(Ordering::Relaxed),
                );
                let fps_pintados = if aviso_ok { (p - p0) as f64 / dt } else { (e - e0) as f64 / dt };
                let linha = format!(
                    "[{:>4.0}s] {} | FPS pintados: {:.1} | entregues: {:.1}/s | descartados: {}",
                    inicio.elapsed().as_secs_f64(),
                    mem::descricao(),
                    fps_pintados,
                    (e - e0) as f64 / dt,
                    d - d0,
                );
                println!("{linha}");
                (p0, e0, d0) = (p, e, d);
            }
        })
    };

    if let Some(s) = sair_em {
        println!("fecha sozinha em {s} s (--exit-after)");
        slint::Timer::single_shot(Duration::from_secs(s), || {
            let _ = slint::quit_event_loop();
        });
    }

    println!("sonda G2 aberta; {}", mem::descricao());
    ui.run()?;

    parar.store(true, Ordering::Relaxed);
    let _ = thread_video.join();
    let _ = thread_relatorio.join();
    println!(
        "fechou sem erro após {:.1} s | total pintados: {} | entregues: {} | descartados: {} | {}",
        inicio.elapsed().as_secs_f64(),
        c.pintados.load(Ordering::Relaxed),
        c.entregues.load(Ordering::Relaxed),
        c.descartados.load(Ordering::Relaxed),
        mem::descricao(),
    );
    Ok(())
}
