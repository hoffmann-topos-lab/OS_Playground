//! Diagramas e simulador do capítulo 5 (entrada e saída).
//!
//! Traz três figuras estáticas (o dispositivo canônico da seção 5.1.2, o
//! acesso direto à memória da seção 5.1.4 e as camadas de software da seção
//! 5.3) e um simulador animado que compara as três formas de fazer E/S: por
//! programa, por interrupção e por DMA (seções 5.2.2 a 5.2.4). O simulador
//! mostra, lado a lado, o que a CPU e o dispositivo fazem no tempo.

use eframe::egui;

use crate::ui::{widget_frame, widget_header, ACCENT};

const CPU_BUSY: egui::Color32 = egui::Color32::from_rgb(0x6f, 0xb3, 0xff);
const OTHER_PROC: egui::Color32 = egui::Color32::from_rgb(0x7f, 0xd6, 0x9b);
const WASTED: egui::Color32 = egui::Color32::from_rgb(0xff, 0x6b, 0x5b);
const DEVICE: egui::Color32 = egui::Color32::from_rgb(0xff, 0xb4, 0x5b);
const DMA_COL: egui::Color32 = egui::Color32::from_rgb(0x9a, 0x8c, 0xff);

fn box_label(
    painter: &egui::Painter,
    rect: egui::Rect,
    fill: egui::Color32,
    stroke: egui::Color32,
    label: &str,
    font: f32,
    text: egui::Color32,
) {
    painter.rect(rect, 6.0, fill, egui::Stroke::new(1.3, stroke), egui::StrokeKind::Inside);
    if !label.is_empty() {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(font),
            text,
        );
    }
}

// ---------------------------------------------------------------------------
// O dispositivo canônico: interface (registradores) e interior (seção 5.1.2)
// ---------------------------------------------------------------------------

pub fn device_model(ui: &mut egui::Ui) {
    let width = ui.available_width().min(600.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 200.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let v = ui.visuals().clone();
    let txt = v.text_color();
    let line = txt.gamma_multiply(0.5);

    // CPU / SO à esquerda.
    let cpu = egui::Rect::from_min_size(egui::pos2(rect.left() + 4.0, rect.top() + 70.0), egui::vec2(96.0, 56.0));
    box_label(&painter, cpu, CPU_BUSY.gamma_multiply(0.22), CPU_BUSY, "CPU / SO", 12.0, txt);

    // Dispositivo à direita, com interface em cima e interior embaixo.
    let dev = egui::Rect::from_min_size(egui::pos2(rect.left() + 170.0, rect.top() + 14.0), egui::vec2(width - 180.0, 172.0));
    box_label(&painter, dev, v.faint_bg_color, line, "", 0.0, txt);
    painter.text(
        egui::pos2(dev.center().x, dev.top() + 12.0),
        egui::Align2::CENTER_CENTER,
        "Dispositivo",
        egui::FontId::proportional(12.5),
        txt,
    );

    // Interface: três registradores.
    let regs = ["Status", "Comando", "Dados"];
    let rw = (dev.width() - 40.0) / 3.0;
    for (i, name) in regs.iter().enumerate() {
        let r = egui::Rect::from_min_size(
            egui::pos2(dev.left() + 12.0 + i as f32 * (rw + 4.0), dev.top() + 26.0),
            egui::vec2(rw, 30.0),
        );
        box_label(&painter, r, ACCENT.gamma_multiply(0.22), ACCENT, name, 11.0, txt);
    }
    painter.text(
        egui::pos2(dev.left() + 12.0, dev.top() + 64.0),
        egui::Align2::LEFT_TOP,
        "interface (registradores)",
        egui::FontId::proportional(9.5),
        v.weak_text_color(),
    );

    // Interior.
    let inner = egui::Rect::from_min_max(
        egui::pos2(dev.left() + 12.0, dev.top() + 86.0),
        egui::pos2(dev.right() - 12.0, dev.bottom() - 12.0),
    );
    box_label(&painter, inner, v.extreme_bg_color, line, "", 0.0, txt);
    painter.text(
        inner.center() - egui::vec2(0.0, 10.0),
        egui::Align2::CENTER_CENTER,
        "micro-controlador · memória · chips",
        egui::FontId::proportional(10.5),
        v.weak_text_color(),
    );
    painter.text(
        egui::pos2(inner.left() + 4.0, inner.bottom() - 10.0),
        egui::Align2::LEFT_CENTER,
        "interior (implementação do dispositivo)",
        egui::FontId::proportional(9.5),
        v.weak_text_color(),
    );

    // Barramento entre CPU e os registradores.
    painter.arrow(
        egui::pos2(cpu.right(), cpu.center().y),
        egui::vec2(dev.left() - cpu.right(), -(cpu.center().y - (dev.top() + 41.0))),
        egui::Stroke::new(1.6, line),
    );
    painter.text(
        egui::pos2((cpu.right() + dev.left()) / 2.0, cpu.center().y + 12.0),
        egui::Align2::CENTER_CENTER,
        "lê/escreve",
        egui::FontId::proportional(9.5),
        v.weak_text_color(),
    );
}

// ---------------------------------------------------------------------------
// Acesso direto à memória (seção 5.1.4)
// ---------------------------------------------------------------------------

pub fn dma_diagram(ui: &mut egui::Ui) {
    let width = ui.available_width().min(600.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 190.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let v = ui.visuals().clone();
    let txt = v.text_color();

    let bw = 110.0;
    let bh = 48.0;
    let cpu = egui::Rect::from_min_size(egui::pos2(rect.left() + 10.0, rect.top() + 16.0), egui::vec2(bw, bh));
    let mem = egui::Rect::from_min_size(egui::pos2(rect.right() - bw - 10.0, rect.top() + 16.0), egui::vec2(bw, bh));
    let dma = egui::Rect::from_min_size(egui::pos2(rect.left() + 10.0, rect.bottom() - bh - 16.0), egui::vec2(bw, bh));
    let dev = egui::Rect::from_min_size(egui::pos2(rect.right() - bw - 10.0, rect.bottom() - bh - 16.0), egui::vec2(bw, bh));

    box_label(&painter, cpu, CPU_BUSY.gamma_multiply(0.22), CPU_BUSY, "CPU", 13.0, txt);
    box_label(&painter, mem, OTHER_PROC.gamma_multiply(0.22), OTHER_PROC, "Memória", 12.5, txt);
    box_label(&painter, dma, DMA_COL.gamma_multiply(0.22), DMA_COL, "Controlador\nDMA", 11.0, txt);
    box_label(&painter, dev, DEVICE.gamma_multiply(0.22), DEVICE, "Dispositivo", 11.5, txt);

    let num = |painter: &egui::Painter, c: egui::Pos2, n: &str| {
        painter.circle_filled(c, 9.0, v.extreme_bg_color);
        painter.circle_stroke(c, 9.0, egui::Stroke::new(1.1, txt.gamma_multiply(0.6)));
        painter.text(c, egui::Align2::CENTER_CENTER, n, egui::FontId::proportional(10.5), txt);
    };
    let arrow = |a: egui::Pos2, b: egui::Pos2, color: egui::Color32| {
        painter.arrow(a, b - a, egui::Stroke::new(1.6, color));
    };

    // 1. CPU programa o DMA.
    arrow(egui::pos2(cpu.center().x, cpu.bottom()), egui::pos2(dma.center().x, dma.top()), CPU_BUSY);
    num(&painter, egui::pos2(cpu.center().x, (cpu.bottom() + dma.top()) / 2.0), "1");
    // 2. DMA move dados entre dispositivo e memória.
    arrow(egui::pos2(dma.right(), dma.center().y), egui::pos2(dev.left(), dev.center().y), DMA_COL);
    arrow(egui::pos2(dma.center().x, dma.top()), egui::pos2(mem.center().x, mem.bottom()), DMA_COL);
    num(&painter, egui::pos2((dma.right() + dev.left()) / 2.0, dma.center().y - 10.0), "2");
    // 3. DMA avisa a CPU com uma interrupção.
    arrow(egui::pos2(mem.center().x, mem.bottom()), egui::pos2(cpu.right(), cpu.bottom() + 4.0), DEVICE);
    num(&painter, egui::pos2(rect.center().x, rect.top() + 40.0), "3");

    painter.text(
        egui::pos2(rect.center().x, rect.center().y + 4.0),
        egui::Align2::CENTER_CENTER,
        "1 programa · 2 transfere · 3 interrompe ao terminar",
        egui::FontId::proportional(10.0),
        v.weak_text_color(),
    );
}

// ---------------------------------------------------------------------------
// As quatro camadas do software de E/S (seção 5.3)
// ---------------------------------------------------------------------------

pub fn io_layers(ui: &mut egui::Ui) {
    let width = ui.available_width().min(560.0);
    let layers = [
        ("Software de E/S do nível do usuário", "bibliotecas, spooling", OTHER_PROC),
        ("Software do SO independente do dispositivo", "nomes, buffers, erros, alocação", ACCENT),
        ("Drivers de dispositivo", "código específico de cada dispositivo", DMA_COL),
        ("Tratadores de interrupção", "acordam o driver quando a E/S termina", DEVICE),
        ("Hardware", "controladores e dispositivos", egui::Color32::GRAY),
    ];
    let lh = 34.0;
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(width, lh * layers.len() as f32 + 8.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let v = ui.visuals().clone();
    let txt = v.text_color();

    let x0 = rect.left() + 28.0;
    let w = width - 56.0;
    for (i, (name, sub, color)) in layers.iter().enumerate() {
        let r = egui::Rect::from_min_size(
            egui::pos2(x0, rect.top() + 4.0 + i as f32 * lh),
            egui::vec2(w, lh - 5.0),
        );
        box_label(&painter, r, color.gamma_multiply(0.18), *color, "", 0.0, txt);
        painter.text(
            egui::pos2(r.left() + 12.0, r.center().y - 6.0),
            egui::Align2::LEFT_CENTER,
            *name,
            egui::FontId::proportional(11.5),
            txt,
        );
        painter.text(
            egui::pos2(r.left() + 12.0, r.center().y + 8.0),
            egui::Align2::LEFT_CENTER,
            *sub,
            egui::FontId::proportional(9.5),
            v.weak_text_color(),
        );
    }
    // Seta de descida (o pedido desce) e de subida (a resposta sobe).
    let top = rect.top() + 4.0;
    let bot = rect.top() + 4.0 + layers.len() as f32 * lh - 5.0;
    painter.arrow(egui::pos2(rect.left() + 12.0, top), egui::vec2(0.0, bot - top - 4.0), egui::Stroke::new(1.5, v.weak_text_color()));
    painter.text(
        egui::pos2(rect.left() + 12.0, (top + bot) / 2.0),
        egui::Align2::CENTER_CENTER,
        "pedido",
        egui::FontId::proportional(9.0),
        v.weak_text_color(),
    );
    painter.arrow(egui::pos2(rect.right() - 12.0, bot), egui::vec2(0.0, -(bot - top - 4.0)), egui::Stroke::new(1.5, v.weak_text_color()));
}

// ---------------------------------------------------------------------------
// Simulador: por programa × por interrupção × por DMA (seções 5.2.2 a 5.2.4)
// ---------------------------------------------------------------------------

const LEN: f32 = 20.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum IoMode {
    Programmed,
    Interrupt,
    Dma,
}

impl IoMode {
    const ALL: [IoMode; 3] = [IoMode::Programmed, IoMode::Interrupt, IoMode::Dma];
    fn label(self) -> &'static str {
        match self {
            IoMode::Programmed => "por programa",
            IoMode::Interrupt => "por interrupção",
            IoMode::Dma => "por DMA",
        }
    }
}

pub struct IoModesState {
    mode: IoMode,
    playing: bool,
    t: f32,
}

impl Default for IoModesState {
    fn default() -> Self {
        Self {
            mode: IoMode::Interrupt,
            playing: false,
            t: 0.0,
        }
    }
}

/// Um trecho de uma faixa: início e fim (em ticks), cor e rótulo.
struct Span {
    a: f32,
    b: f32,
    color: egui::Color32,
    label: &'static str,
}

/// As faixas (CPU, Dispositivo, DMA) de cada modo. A fase de E/S é sempre de 5
/// a 15; o que muda é o que a CPU faz nesse intervalo.
fn lanes(mode: IoMode) -> Vec<(&'static str, Vec<Span>)> {
    let p1 = |a, b| Span { a, b, color: CPU_BUSY, label: "P1" };
    let busy = |a, b| Span { a, b, color: DEVICE, label: "ocupado" };
    match mode {
        IoMode::Programmed => vec![
            (
                "CPU",
                vec![
                    p1(0.0, 5.0),
                    Span { a: 5.0, b: 15.0, color: WASTED, label: "espera ativa" },
                    p1(15.0, 20.0),
                ],
            ),
            ("Dispositivo", vec![busy(5.0, 15.0)]),
        ],
        IoMode::Interrupt => vec![
            (
                "CPU",
                vec![
                    p1(0.0, 5.0),
                    Span { a: 5.0, b: 15.0, color: OTHER_PROC, label: "P2" },
                    p1(15.0, 20.0),
                ],
            ),
            ("Dispositivo", vec![busy(5.0, 15.0)]),
        ],
        IoMode::Dma => vec![
            (
                "CPU",
                vec![
                    p1(0.0, 5.0),
                    Span { a: 5.0, b: 15.0, color: OTHER_PROC, label: "P2" },
                    p1(15.0, 20.0),
                ],
            ),
            ("DMA", vec![Span { a: 5.0, b: 15.0, color: DMA_COL, label: "transfere" }]),
            ("Dispositivo", vec![busy(5.0, 15.0)]),
        ],
    }
}

/// Aproveitamento útil da CPU: fração do tempo em trabalho de processo (P1/P2),
/// sem contar a espera ativa.
fn cpu_useful(mode: IoMode) -> f32 {
    match mode {
        IoMode::Programmed => 0.5,
        IoMode::Interrupt | IoMode::Dma => 1.0,
    }
}

fn caption(mode: IoMode, t: f32) -> String {
    let phase = if t < 5.0 {
        "P1 computa e então pede uma operação de E/S."
    } else if t < 15.0 {
        match mode {
            IoMode::Programmed => {
                "A CPU fica presa consultando o dispositivo em laço. Nenhum outro processo avança."
            }
            IoMode::Interrupt => {
                "A CPU troca para P2 e trabalha enquanto o dispositivo opera. No fim, uma \
                 interrupção acorda P1."
            }
            IoMode::Dma => {
                "O controlador DMA transfere os dados sozinho. A CPU trabalha em P2 o tempo todo."
            }
        }
    } else {
        match mode {
            IoMode::Programmed => "A E/S terminou e P1 volta. A CPU desperdiçou metade do tempo.",
            _ => "A interrupção avisou que a E/S terminou e P1 retoma de onde parou.",
        }
    };
    phase.to_string()
}

pub fn show(ui: &mut egui::Ui, st: &mut IoModesState) {
    if st.playing {
        st.t += ui.input(|i| i.stable_dt).min(0.1) * 4.0;
        if st.t >= LEN {
            st.t = LEN;
            st.playing = false;
        }
        ui.ctx().request_repaint();
    }

    widget_frame(ui).show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        widget_header(
            ui,
            "Simulador",
            "O que a CPU e o dispositivo fazem no tempo, nas três formas de E/S.",
        );

        ui.horizontal_wrapped(|ui| {
            for m in IoMode::ALL {
                if ui.selectable_label(st.mode == m, m.label()).clicked() {
                    st.mode = m;
                    st.t = 0.0;
                    st.playing = false;
                }
            }
        });
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            let label = if st.playing { "⏸" } else { "▶" };
            if ui.button(label).on_hover_text("Executar a linha do tempo").clicked() {
                if st.playing {
                    st.playing = false;
                } else {
                    if st.t >= LEN {
                        st.t = 0.0;
                    }
                    st.playing = true;
                }
            }
            if ui.button("↺").on_hover_text("Reiniciar").clicked() {
                st.t = 0.0;
                st.playing = false;
            }
            ui.add(egui::Slider::new(&mut st.t, 0.0..=LEN).show_value(false).text("tempo"));
        });
        ui.add_space(8.0);

        timeline(ui, st.mode, st.t);
        ui.add_space(8.0);

        // Legenda do aproveitamento da CPU.
        let useful = cpu_useful(st.mode) * 100.0;
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("Aproveitamento útil da CPU:").size(12.5));
            let color = if useful >= 100.0 { OTHER_PROC } else { WASTED };
            ui.label(
                egui::RichText::new(format!("{useful:.0}%"))
                    .size(13.0)
                    .strong()
                    .color(color),
            );
        });
        ui.add_space(6.0);

        // Caixa de explicação do passo atual.
        egui::Frame::new()
            .inner_margin(egui::Margin::symmetric(10, 8))
            .corner_radius(7.0)
            .fill(ACCENT.gamma_multiply(0.10))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal_top(|ui| {
                    ui.label(egui::RichText::new("➡").size(14.0));
                    ui.add(egui::Label::new(
                        egui::RichText::new(caption(st.mode, st.t)).size(13.0),
                    ).wrap());
                });
            });
    });
}

fn timeline(ui: &mut egui::Ui, mode: IoMode, t: f32) {
    let lanes = lanes(mode);
    let label_w = 90.0;
    let width = ui.available_width();
    let lane_h = 30.0;
    let gap = 8.0;
    let height = lanes.len() as f32 * (lane_h + gap) + 20.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let v = ui.visuals().clone();
    let txt = v.text_color();

    let x0 = rect.left() + label_w;
    let track_w = width - label_w - 6.0;
    let unit = track_w / LEN;

    for (i, (name, spans)) in lanes.iter().enumerate() {
        let y = rect.top() + i as f32 * (lane_h + gap);
        painter.text(
            egui::pos2(rect.left(), y + lane_h / 2.0),
            egui::Align2::LEFT_CENTER,
            *name,
            egui::FontId::proportional(12.0),
            txt,
        );
        // Trilho vazio.
        let track = egui::Rect::from_min_size(egui::pos2(x0, y), egui::vec2(track_w, lane_h));
        painter.rect_filled(track, 4.0, v.extreme_bg_color);
        // Trechos preenchidos até onde o tempo já passou.
        for s in spans {
            let fill_to = t.clamp(s.a, s.b);
            if fill_to <= s.a {
                continue;
            }
            let r = egui::Rect::from_min_max(
                egui::pos2(x0 + s.a * unit, y + 2.0),
                egui::pos2(x0 + fill_to * unit, y + lane_h - 2.0),
            );
            painter.rect(r, 3.0, s.color.gamma_multiply(0.85), egui::Stroke::NONE, egui::StrokeKind::Inside);
            if r.width() >= 34.0 {
                painter.text(
                    r.center(),
                    egui::Align2::CENTER_CENTER,
                    s.label,
                    egui::FontId::proportional(10.5),
                    egui::Color32::WHITE,
                );
            }
        }
        // Contorno dos trechos ainda não alcançados, para antecipar o plano.
        for s in spans {
            let r = egui::Rect::from_min_max(
                egui::pos2(x0 + s.a * unit, y + 2.0),
                egui::pos2(x0 + s.b * unit, y + lane_h - 2.0),
            );
            painter.rect_stroke(r, 3.0, egui::Stroke::new(1.0, s.color.gamma_multiply(0.5)), egui::StrokeKind::Inside);
        }
    }

    // Marca da interrupção, nos modos que a usam.
    if mode != IoMode::Programmed {
        let x = x0 + 15.0 * unit;
        painter.line_segment(
            [egui::pos2(x, rect.top() - 2.0), egui::pos2(x, rect.top() + lanes.len() as f32 * (lane_h + gap) - gap)],
            egui::Stroke::new(1.4, DEVICE),
        );
        painter.text(
            egui::pos2(x, rect.bottom() - 6.0),
            egui::Align2::CENTER_CENTER,
            "interrupção",
            egui::FontId::proportional(9.5),
            DEVICE,
        );
    }

    // Cursor de tempo (playhead).
    let xt = x0 + t * unit;
    painter.line_segment(
        [egui::pos2(xt, rect.top() - 2.0), egui::pos2(xt, rect.top() + lanes.len() as f32 * (lane_h + gap) - gap)],
        egui::Stroke::new(1.6, ACCENT),
    );
}
