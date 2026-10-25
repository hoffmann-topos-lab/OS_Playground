//! Demonstração passo a passo de duas linhas de execução concorrentes.
//!
//! Cada algoritmo de exclusão mútua é descrito como um **roteiro determinístico**
//! de quadros: em cada quadro sabemos em que linha cada ator está, quanto valem
//! as variáveis compartilhadas e o que está acontecendo. Isso mostra um
//! entrelaçamento específico — justamente o entrelaçamento que revela o
//! problema, ou que mostra por que a solução resiste a ele.
//!
//! A transição entre quadros é **interpolada**, não um corte seco: o destaque
//! desliza de uma linha para outra, as variáveis pulsam ao mudar e a legenda faz
//! dissolução. É isso que transforma a sequência de estados numa animação.

use eframe::egui;

use crate::ui::ACCENT;

const DANGER: egui::Color32 = egui::Color32::from_rgb(0xff, 0x6b, 0x5b);
const SAFE: egui::Color32 = egui::Color32::from_rgb(0x5b, 0xd6, 0x8a);

/// Altura de cada linha de pseudocódigo. Fixa, porque é ela que permite
/// calcular a posição do destaque por interpolação.
const ROW_H: f32 = 20.0;
/// No modo apresentação, os quadros marcados como `danger` seguram por mais
/// tempo — são eles que carregam a virada da história.
const DANGER_DWELL: f32 = 2.0;

/// Desfecho da demonstração, mostrado ao chegar no último quadro.
pub enum Outcome {
    /// O algoritmo se sustenta neste entrelaçamento.
    Holds(&'static str),
    /// O algoritmo falha — e a legenda diz por quê.
    Breaks(&'static str),
}

/// Um quadro do roteiro.
pub struct Frame {
    /// Linha destacada em cada ator; `None` = parado / fora do trecho mostrado.
    pub pc: [Option<usize>; 2],
    /// Estado das variáveis compartilhadas neste quadro.
    pub vars: &'static [(&'static str, &'static str)],
    /// O que está acontecendo, em uma frase.
    pub caption: &'static str,
    /// Quem está dentro da região crítica agora.
    pub in_cs: [bool; 2],
    /// Marca o quadro onde a coisa dá errado.
    pub danger: bool,
}

/// Uma demonstração completa.
pub struct Demo {
    /// Identificador estável — é a chave do estado de reprodução.
    pub id: &'static str,
    pub actors: [&'static str; 2],
    /// Pseudocódigo de cada ator, uma linha por índice de `Frame::pc`.
    pub code: [&'static [&'static str]; 2],
    pub frames: &'static [Frame],
    pub outcome: Outcome,
}

pub struct DemoState {
    /// Quadro em que estamos.
    idx: usize,
    /// Progresso da transição rumo a `idx + 1`, em [0, 1].
    t: f32,
    playing: bool,
    /// Passos por segundo.
    speed: f32,
    /// Modo apresentação: toca do início, mais devagar, sem os controles finos.
    presenting: bool,
}

impl Default for DemoState {
    fn default() -> Self {
        Self {
            idx: 0,
            t: 0.0,
            playing: false,
            speed: 1.3,
            presenting: false,
        }
    }
}

impl DemoState {
    fn go_to(&mut self, idx: usize) {
        self.idx = idx;
        self.t = 0.0;
    }
    fn stop(&mut self) {
        self.playing = false;
        self.presenting = false;
        self.t = 0.0;
    }
}

/// Suavização: acelera e desacelera, em vez de deslizar em velocidade constante.
fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Sino centrado em 0.5, usado para os pulsos e a dissolução.
fn bell(t: f32) -> f32 {
    let x = (t.clamp(0.0, 1.0) - 0.5) * 2.0;
    (1.0 - x * x).max(0.0)
}

pub fn show(ui: &mut egui::Ui, demo: &Demo, st: &mut DemoState) {
    if demo.frames.is_empty() {
        return;
    }
    let last = demo.frames.len() - 1;
    st.idx = st.idx.min(last);

    // ---- avanço no tempo ----
    if st.playing {
        // Quadros de virada seguram mais tempo no modo apresentação.
        let dwell = if st.presenting && demo.frames[st.idx].danger {
            DANGER_DWELL
        } else {
            1.0
        };
        st.t += ui.input(|i| i.stable_dt).min(0.1) * st.speed / dwell;
        while st.t >= 1.0 {
            if st.idx < last {
                st.idx += 1;
                st.t -= 1.0;
            } else {
                st.t = 0.0;
                st.playing = false;
                st.presenting = false;
                break;
            }
        }
        ui.ctx().request_repaint();
    } else {
        st.t = 0.0;
    }

    let from = &demo.frames[st.idx];
    let to = &demo.frames[(st.idx + 1).min(last)];
    let e = ease(st.t);

    egui::Frame::new()
        .inner_margin(egui::Margin::same(12))
        .corner_radius(9.0)
        .fill(ui.visuals().faint_bg_color)
        .stroke(egui::Stroke::new(
            1.0,
            ui.visuals().widgets.noninteractive.bg_stroke.color,
        ))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            controls(ui, st, last);
            ui.add_space(8.0);
            vars_row(ui, from, to, st.t);
            ui.add_space(10.0);
            code_columns(ui, demo, from, to, e);
            ui.add_space(10.0);
            caption(ui, demo, from, to, st, last);
        });
}

fn controls(ui: &mut egui::Ui, st: &mut DemoState, last: usize) {
    ui.horizontal_wrapped(|ui| {
        if ui.button("⏮").on_hover_text("Voltar ao início").clicked() {
            st.go_to(0);
            st.stop();
        }
        if ui.button("◀").on_hover_text("Passo anterior").clicked() {
            st.go_to(st.idx.saturating_sub(1));
            st.stop();
        }
        let label = if st.playing { "⏸" } else { "▶" };
        if ui
            .button(label)
            .on_hover_text(if st.playing { "Pausar" } else { "Reproduzir" })
            .clicked()
        {
            if st.playing {
                st.stop();
            } else {
                // Dar play no fim recomeça, senão nada acontece de visível.
                if st.idx == last {
                    st.go_to(0);
                }
                st.playing = true;
                st.t = 0.0;
            }
        }
        if ui.button("▶|").on_hover_text("Próximo passo").clicked() {
            st.go_to((st.idx + 1).min(last));
            st.stop();
        }

        ui.add_space(4.0);
        if ui
            .button("🎬 Apresentar")
            .on_hover_text("Toca do início, mais devagar, segurando nos pontos de virada")
            .clicked()
        {
            st.go_to(0);
            st.playing = true;
            st.presenting = true;
            st.speed = 0.75;
        }

        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(format!("passo {} / {}", st.idx + 1, last + 1))
                .size(12.0)
                .color(egui::Color32::GRAY),
        );

        ui.add_space(6.0);
        let mut idx = st.idx;
        if ui
            .add(
                egui::Slider::new(&mut idx, 0..=last)
                    .show_value(false)
                    .trailing_fill(true),
            )
            .changed()
        {
            st.go_to(idx);
            st.stop();
        }

        ui.add(
            egui::Slider::new(&mut st.speed, 0.4..=4.0)
                .show_value(false)
                .text("velocidade"),
        );
    });
}

/// Faixa das variáveis compartilhadas. O valor troca no meio da transição, e a
/// que mudou pulsa — para o olho saber onde olhar.
fn vars_row(ui: &mut egui::Ui, from: &Frame, to: &Frame, t: f32) {
    let vars = if t < 0.5 { from.vars } else { to.vars };
    if vars.is_empty() {
        return;
    }
    ui.horizontal_wrapped(|ui| {
        ui.label(
            egui::RichText::new("compartilhado:")
                .size(12.0)
                .color(egui::Color32::GRAY),
        );
        for (name, value) in vars {
            // Vai mudar (ou acabou de mudar) nesta transição?
            let muda = from
                .vars
                .iter()
                .find(|(n, _)| n == name)
                .zip(to.vars.iter().find(|(n, _)| n == name))
                .is_some_and(|((_, a), (_, b))| a != b);
            let pulso = if muda { bell(t) } else { 0.0 };
            let fill = if pulso > 0.01 {
                ACCENT.gamma_multiply(0.12 + 0.45 * pulso)
            } else {
                ui.visuals().extreme_bg_color
            };
            egui::Frame::new()
                .inner_margin(egui::Margin::symmetric(8, 4))
                .corner_radius(6.0)
                .fill(fill)
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(format!("{name} = {value}"))
                            .monospace()
                            .size(12.5),
                    );
                });
        }
    });
}

/// As duas colunas de pseudocódigo, com o destaque deslizando entre as linhas.
fn code_columns(ui: &mut egui::Ui, demo: &Demo, from: &Frame, to: &Frame, e: f32) {
    ui.columns(2, |cols| {
        for (i, ui) in cols.iter_mut().enumerate() {
            // A borda da região crítica também transiciona.
            let cs = lerp(f32::from(from.in_cs[i]), f32::from(to.in_cs[i]), e);
            let perigo = if from.danger || to.danger { DANGER } else { SAFE };
            let neutra = ui.visuals().widgets.noninteractive.bg_stroke.color;
            let border = mix(neutra, perigo, cs);

            egui::Frame::new()
                .inner_margin(egui::Margin::same(9))
                .corner_radius(8.0)
                .fill(ui.visuals().extreme_bg_color)
                .stroke(egui::Stroke::new(1.0 + cs, border))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(demo.actors[i])
                                .size(13.0)
                                .strong()
                                .color(mix(ACCENT, perigo, cs)),
                        );
                        if cs > 0.5 {
                            ui.label(
                                egui::RichText::new("· região crítica")
                                    .size(11.0)
                                    .color(perigo.gamma_multiply(cs)),
                            );
                        }
                    });
                    ui.add_space(5.0);
                    code_block(ui, demo.code[i], from.pc[i], to.pc[i], e);
                });
        }
    });
}

/// Desenha as linhas e o destaque interpolado entre `pc_from` e `pc_to`.
fn code_block(
    ui: &mut egui::Ui,
    lines: &[&str],
    pc_from: Option<usize>,
    pc_to: Option<usize>,
    e: f32,
) {
    let height = lines.len() as f32 * ROW_H;
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), height), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let visuals = ui.visuals().clone();

    // Onde o destaque está, e o quanto ele está presente. Quando um ator sai de
    // cena (`None`), a posição é mantida e só a opacidade cai — assim o
    // destaque some no lugar, em vez de voar para a linha 0.
    let pos_from = pc_from.or(pc_to).unwrap_or(0) as f32;
    let pos_to = pc_to.or(pc_from).unwrap_or(0) as f32;
    let pos = lerp(pos_from, pos_to, e);
    let alpha = lerp(f32::from(pc_from.is_some()), f32::from(pc_to.is_some()), e);

    if alpha > 0.01 {
        let y = rect.top() + pos * ROW_H;
        let hl = egui::Rect::from_min_size(
            egui::pos2(rect.left(), y + 1.0),
            egui::vec2(rect.width(), ROW_H - 2.0),
        );
        painter.rect_filled(hl, 4.0, ACCENT.gamma_multiply(0.30 * alpha));
    }

    for (i, line) in lines.iter().enumerate() {
        // Quão perto o destaque está desta linha (0..1), para acender o texto.
        let peso = if alpha > 0.01 {
            (1.0 - (pos - i as f32).abs()).clamp(0.0, 1.0) * alpha
        } else {
            0.0
        };
        let y = rect.top() + i as f32 * ROW_H + ROW_H / 2.0;

        painter.text(
            egui::pos2(rect.left() + 6.0, y),
            egui::Align2::LEFT_CENTER,
            "▸",
            egui::FontId::monospace(12.0),
            ACCENT.gamma_multiply(peso),
        );
        painter.text(
            egui::pos2(rect.left() + 20.0, y),
            egui::Align2::LEFT_CENTER,
            *line,
            egui::FontId::monospace(12.0),
            mix(visuals.weak_text_color(), visuals.strong_text_color(), peso),
        );
    }
}

/// Legenda do passo, com dissolução na troca, e o desfecho no último quadro.
fn caption(ui: &mut egui::Ui, demo: &Demo, from: &Frame, to: &Frame, st: &DemoState, last: usize) {
    let f = if st.t < 0.5 { from } else { to };
    // Escurece no meio da transição e volta: uma dissolução curta.
    let fade = 1.0 - 0.55 * bell(st.t);

    let (icon, color) = if f.danger {
        ("⚠", DANGER)
    } else {
        ("▸", ACCENT)
    };
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(10, 8))
        .corner_radius(7.0)
        .fill(color.gamma_multiply(0.12 * fade))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal_top(|ui| {
                ui.label(
                    egui::RichText::new(icon)
                        .size(14.0)
                        .color(color.gamma_multiply(fade)),
                );
                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new(f.caption)
                        .size(13.5)
                        .color(ui.visuals().text_color().gamma_multiply(fade)),
                );
            });
        });

    // O desfecho só aparece parado no fim, e entra com um fade próprio.
    if st.idx == last && !st.playing {
        let (icon, color, text) = match &demo.outcome {
            Outcome::Holds(t) => ("✔", SAFE, *t),
            Outcome::Breaks(t) => ("✖", DANGER, *t),
        };
        ui.add_space(6.0);
        egui::Frame::new()
            .inner_margin(egui::Margin::symmetric(10, 8))
            .corner_radius(7.0)
            .fill(color.gamma_multiply(0.16))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal_top(|ui| {
                    ui.label(egui::RichText::new(icon).size(14.0).color(color));
                    ui.add_space(2.0);
                    ui.label(egui::RichText::new(text).size(13.5).strong());
                });
            });
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

/// Mistura duas cores, componente a componente.
fn mix(a: egui::Color32, b: egui::Color32, t: f32) -> egui::Color32 {
    let t = t.clamp(0.0, 1.0);
    let c = |x: u8, y: u8| (lerp(f32::from(x), f32::from(y), t)).round() as u8;
    egui::Color32::from_rgba_unmultiplied(
        c(a.r(), b.r()),
        c(a.g(), b.g()),
        c(a.b(), b.b()),
        c(a.a(), b.a()),
    )
}

/// Valida um roteiro: índices de linha existentes e quadros com legenda.
///
/// Chamado pelo validador de conteúdo em [`crate::ui::reader::check`].
#[cfg(test)]
pub fn check(demo: &Demo) {
    assert!(!demo.frames.is_empty(), "demo '{}' sem quadros", demo.id);
    for (i, f) in demo.frames.iter().enumerate() {
        assert!(
            !f.caption.trim().is_empty(),
            "demo '{}': quadro {i} sem legenda",
            demo.id
        );
        for actor in 0..2 {
            if let Some(pc) = f.pc[actor] {
                assert!(
                    pc < demo.code[actor].len(),
                    "demo '{}': quadro {i} aponta para a linha {pc} do ator {actor}, \
                     que só tem {} linhas",
                    demo.id,
                    demo.code[actor].len()
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estado_inicial_e_coerente() {
        let st = DemoState::default();
        assert_eq!(st.idx, 0);
        assert!(!st.playing);
        assert!(!st.presenting);
        assert!(st.speed > 0.0);
    }

    #[test]
    fn suavizacao_e_bem_comportada() {
        assert_eq!(ease(0.0), 0.0);
        assert_eq!(ease(1.0), 1.0);
        // Monotônica e sempre dentro de [0, 1], inclusive fora do domínio.
        let mut anterior = -1.0;
        for i in 0..=20 {
            let v = ease(i as f32 / 20.0);
            assert!((0.0..=1.0).contains(&v));
            assert!(v >= anterior, "ease deveria ser monotônica");
            anterior = v;
        }
        assert_eq!(ease(-3.0), 0.0);
        assert_eq!(ease(7.0), 1.0);
    }

    #[test]
    fn sino_tem_pico_no_meio_e_zera_nas_pontas() {
        assert_eq!(bell(0.0), 0.0);
        assert_eq!(bell(1.0), 0.0);
        assert!((bell(0.5) - 1.0).abs() < 1e-6);
        assert!(bell(0.25) > 0.0 && bell(0.25) < 1.0);
    }

    #[test]
    fn mistura_de_cores_respeita_as_pontas() {
        let a = egui::Color32::from_rgb(0, 0, 0);
        let b = egui::Color32::from_rgb(255, 255, 255);
        assert_eq!(mix(a, b, 0.0).r(), 0);
        assert_eq!(mix(a, b, 1.0).r(), 255);
        assert_eq!(mix(a, b, 0.5).r(), 128);
        // Fora do domínio, satura em vez de estourar.
        assert_eq!(mix(a, b, 5.0).r(), 255);
    }
}
