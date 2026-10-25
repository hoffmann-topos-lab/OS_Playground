//! Simulador de gerenciamento de memória livre (seção 3.2.3).
//!
//! A memória é uma fita de unidades de alocação. Um cenário parte de uma
//! configuração inicial (processos e blocos livres) e aplica uma sequência de
//! operações — alocar um processo ou liberá-lo — sob uma das quatro políticas
//! de encaixe. A cada passo o widget mostra a fita, a **lista encadeada** de
//! segmentos (tipo, início, comprimento, ponteiro) e o **mapa de bits** da
//! mesma memória, lado a lado, mais uma legenda dizendo por que o algoritmo
//! escolheu aquele bloco.
//!
//! A simulação é refeita do zero a cada quadro: as listas são minúsculas e
//! isso deixa os tamanhos das requisições editáveis sem estado extra.

use eframe::egui;

use crate::ui::gantt::series_color;
use crate::ui::{ACCENT, widget_frame, widget_header};

const DANGER: egui::Color32 = egui::Color32::from_rgb(0xff, 0x6b, 0x5b);

/// As quatro políticas de encaixe (3.2.26).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fit {
    First,
    Next,
    Best,
    Worst,
}

impl Fit {
    pub const ALL: [Fit; 4] = [Fit::First, Fit::Next, Fit::Best, Fit::Worst];

    pub fn label(self) -> &'static str {
        match self {
            Fit::First => "primeiro encaixe",
            Fit::Next => "próximo encaixe",
            Fit::Best => "melhor encaixe",
            Fit::Worst => "pior encaixe",
        }
    }
}

/// Um segmento da configuração inicial.
pub enum Seg {
    Proc(&'static str, u32),
    Hole(u32),
}

/// Uma operação da sequência do cenário.
pub enum Op {
    /// Um processo novo pede `n` unidades (editável no widget).
    Alloc(&'static str, u32),
    /// O processo termina e devolve seu segmento.
    Free(&'static str),
}

/// Um cenário pronto, embutido num artigo.
pub struct Scenario {
    /// Identificador estável — é a chave do estado editável.
    pub id: &'static str,
    pub initial: &'static [Seg],
    pub ops: &'static [Op],
    pub fit: Fit,
    /// Frase curta sobre o que observar neste cenário.
    pub hint: &'static str,
}

/// Estado editável de um cenário.
pub struct FitState {
    fit: Fit,
    /// Tamanho pedido por cada operação (0 nas liberações, que não têm tamanho).
    sizes: Vec<u32>,
    /// Quantas operações já foram aplicadas.
    step: usize,
    playing: bool,
    /// Acumulador do avanço automático, em segundos.
    clock: f32,
}

impl FitState {
    pub fn new(s: &Scenario) -> Self {
        Self {
            fit: s.fit,
            sizes: s
                .ops
                .iter()
                .map(|op| match op {
                    Op::Alloc(_, n) => *n,
                    Op::Free(_) => 0,
                })
                .collect(),
            step: 0,
            playing: false,
            clock: 0.0,
        }
    }
    fn reset(&mut self, s: &Scenario) {
        *self = Self::new(s);
    }
}

/// Um segmento da memória simulada.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segment {
    /// `None` = bloco livre.
    pub owner: Option<&'static str>,
    pub start: u32,
    pub len: u32,
}

impl Segment {
    fn is_hole(&self) -> bool {
        self.owner.is_none()
    }
    fn end(&self) -> u32 {
        self.start + self.len
    }
}

/// O que aconteceu numa operação, para a legenda e para os destaques.
#[derive(Clone, Debug)]
pub struct Decision {
    pub caption: String,
    /// Segmento recém-alocado (ou o bloco que resultou da liberação).
    pub touched: Option<(u32, u32)>,
    /// Blocos livres examinados antes da escolha, como (início, comprimento).
    pub examined: Vec<(u32, u32)>,
    pub failed: bool,
}

/// A memória depois de `step` operações, mais a decisão de cada uma.
pub struct Sim {
    pub segments: Vec<Segment>,
    pub decisions: Vec<Decision>,
    pub total: u32,
    /// Onde o próximo encaixe recomeça a busca.
    pub rover: u32,
}

/// Aplica as `step` primeiras operações do cenário.
pub fn simulate(s: &Scenario, fit: Fit, sizes: &[u32], step: usize) -> Sim {
    let mut segments: Vec<Segment> = Vec::new();
    let mut cursor = 0;
    for seg in s.initial {
        let (owner, len) = match seg {
            Seg::Proc(name, n) => (Some(*name), *n),
            Seg::Hole(n) => (None, *n),
        };
        segments.push(Segment {
            owner,
            start: cursor,
            len,
        });
        cursor += len;
    }
    let total = cursor;
    let mut rover = 0u32;
    let mut decisions = Vec::new();

    for (i, op) in s.ops.iter().enumerate().take(step) {
        let d = match op {
            Op::Alloc(name, _) => {
                let n = sizes.get(i).copied().unwrap_or(1).max(1);
                allocate(&mut segments, fit, &mut rover, name, n)
            }
            Op::Free(name) => release(&mut segments, name),
        };
        decisions.push(d);
    }

    Sim {
        segments,
        decisions,
        total,
        rover,
    }
}

/// Escolhe um bloco pela política e o divide (3.2.26).
fn allocate(
    segments: &mut Vec<Segment>,
    fit: Fit,
    rover: &mut u32,
    name: &'static str,
    n: u32,
) -> Decision {
    let holes: Vec<usize> = (0..segments.len())
        .filter(|&i| segments[i].is_hole())
        .collect();
    let mut examined: Vec<(u32, u32)> = Vec::new();

    let chosen: Option<usize> = match fit {
        Fit::First => holes.iter().copied().find(|&i| {
            examined.push((segments[i].start, segments[i].len));
            segments[i].len >= n
        }),
        Fit::Next => {
            // Recomeça de onde a última busca parou e dá a volta.
            let (after, before): (Vec<usize>, Vec<usize>) =
                holes.iter().partition(|&&i| segments[i].start >= *rover);
            after.into_iter().chain(before).find(|&i| {
                examined.push((segments[i].start, segments[i].len));
                segments[i].len >= n
            })
        }
        Fit::Best => {
            examined = holes
                .iter()
                .map(|&i| (segments[i].start, segments[i].len))
                .collect();
            holes
                .iter()
                .copied()
                .filter(|&i| segments[i].len >= n)
                .min_by_key(|&i| (segments[i].len, segments[i].start))
        }
        Fit::Worst => {
            examined = holes
                .iter()
                .map(|&i| (segments[i].start, segments[i].len))
                .collect();
            holes
                .iter()
                .copied()
                .filter(|&i| segments[i].len >= n)
                .max_by_key(|&i| (segments[i].len, u32::MAX - segments[i].start))
        }
    };

    let Some(i) = chosen else {
        let free: u32 = holes.iter().map(|&i| segments[i].len).sum();
        let caption = if free >= n {
            format!(
                "{name} pede {n} unidades e nenhum bloco livre comporta o pedido. Há {free} \
                 unidades livres no total, mas espalhadas em {} blocos — é a fragmentação \
                 externa.",
                holes.len()
            )
        } else {
            format!("{name} pede {n} unidades, mas só restam {free} livres no total.")
        };
        return Decision {
            caption,
            touched: None,
            examined,
            failed: true,
        };
    };

    let hole = segments[i].clone();
    let leftover = hole.len - n;
    segments[i] = Segment {
        owner: Some(name),
        start: hole.start,
        len: n,
    };
    if leftover > 0 {
        segments.insert(
            i + 1,
            Segment {
                owner: None,
                start: hole.start + n,
                len: leftover,
            },
        );
    }
    *rover = hole.start + n;

    let how = match fit {
        Fit::First => format!(
            "percorreu a lista desde o início e parou no primeiro bloco grande o bastante \
             ({} unidades, no endereço {})",
            hole.len, hole.start
        ),
        Fit::Next => format!(
            "recomeçou a busca de onde a anterior parou e tomou o primeiro bloco que coube \
             ({} unidades, no endereço {})",
            hole.len, hole.start
        ),
        Fit::Best => format!(
            "examinou todos os {} blocos livres e escolheu o menor que cabe ({} unidades, \
             no endereço {})",
            examined.len(),
            hole.len,
            hole.start
        ),
        Fit::Worst => format!(
            "examinou todos os {} blocos livres e escolheu o maior ({} unidades, no \
             endereço {})",
            examined.len(),
            hole.len,
            hole.start
        ),
    };
    let rest = if leftover == 0 {
        "O bloco foi ocupado por inteiro.".to_string()
    } else {
        format!("Sobrou um bloco livre de {leftover}.")
    };
    Decision {
        caption: format!("{name} pede {n} unidades: {how}. {rest}"),
        touched: Some((hole.start, n)),
        examined,
        failed: false,
    }
}

/// Libera o segmento do processo e funde com os vizinhos livres (3.2.25).
fn release(segments: &mut Vec<Segment>, name: &str) -> Decision {
    let Some(i) = segments.iter().position(|s| s.owner == Some(name)) else {
        return Decision {
            caption: format!("{name} não está na memória."),
            touched: None,
            examined: Vec::new(),
            failed: true,
        };
    };
    let left_hole = i > 0 && segments[i - 1].is_hole();
    let right_hole = i + 1 < segments.len() && segments[i + 1].is_hole();

    let mut start = segments[i].start;
    let mut end = segments[i].end();
    let mut lo = i;
    let mut hi = i;
    if left_hole {
        start = segments[i - 1].start;
        lo = i - 1;
    }
    if right_hole {
        end = segments[i + 1].end();
        hi = i + 1;
    }
    segments.splice(
        lo..=hi,
        [Segment {
            owner: None,
            start,
            len: end - start,
        }],
    );

    let caption = match (left_hole, right_hole) {
        (false, false) => format!(
            "{name} terminou. Os dois vizinhos são processos, então a entrada apenas muda \
             de tipo: vira um bloco livre de {} unidades.",
            end - start
        ),
        (true, false) | (false, true) => format!(
            "{name} terminou. Um dos vizinhos era um bloco livre, então as duas entradas se \
             fundiram num bloco só de {} unidades.",
            end - start
        ),
        (true, true) => format!(
            "{name} terminou. Os dois vizinhos eram blocos livres, então as três entradas se \
             fundiram num bloco só de {} unidades — a lista encolheu.",
            end - start
        ),
    };
    Decision {
        caption,
        touched: Some((start, end - start)),
        examined: Vec::new(),
        failed: false,
    }
}

// ---------------------------------------------------------------------------
// Interface
// ---------------------------------------------------------------------------

pub fn show(ui: &mut egui::Ui, scenario: &Scenario, st: &mut FitState) {
    let n_ops = scenario.ops.len();
    st.step = st.step.min(n_ops);

    // Avanço automático: uma operação a cada 1,6 s.
    if st.playing {
        st.clock += ui.input(|i| i.stable_dt).min(0.1);
        if st.clock >= 1.6 {
            st.clock = 0.0;
            st.step += 1;
            if st.step >= n_ops {
                st.step = n_ops;
                st.playing = false;
            }
        }
        ui.ctx().request_repaint();
    }

    let sim = simulate(scenario, st.fit, &st.sizes, st.step);
    let current = st.step.checked_sub(1).and_then(|i| sim.decisions.get(i));
    let names = scenario_names(scenario);

    widget_frame(ui).show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        widget_header(ui, "Simulador", scenario.hint);

        if n_ops > 0 {
            ui.horizontal_wrapped(|ui| {
                for f in Fit::ALL {
                    if ui.selectable_label(st.fit == f, f.label()).clicked() {
                        st.fit = f;
                    }
                }
            });
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                if ui
                    .add_enabled(st.step > 0, egui::Button::new("◀"))
                    .on_hover_text("Desfazer a última operação")
                    .clicked()
                {
                    st.step -= 1;
                    st.playing = false;
                }
                if ui
                    .add_enabled(st.step < n_ops, egui::Button::new("▶|"))
                    .on_hover_text("Aplicar a próxima operação")
                    .clicked()
                {
                    st.step += 1;
                    st.playing = false;
                }
                let label = if st.playing { "⏸" } else { "▶" };
                if ui
                    .button(label)
                    .on_hover_text("Aplicar as operações uma a uma")
                    .clicked()
                {
                    if st.playing {
                        st.playing = false;
                    } else {
                        if st.step >= n_ops {
                            st.step = 0;
                        }
                        st.clock = 0.0;
                        st.playing = true;
                    }
                }
                if ui
                    .button("⏭")
                    .on_hover_text("Aplicar todas as operações")
                    .clicked()
                {
                    st.step = n_ops;
                    st.playing = false;
                }
                ui.separator();
                ui.label(
                    egui::RichText::new(format!("operação {} de {n_ops}", st.step))
                        .size(12.0)
                        .color(egui::Color32::GRAY),
                );
                ui.separator();
                if ui
                    .small_button("↺")
                    .on_hover_text("Restaurar cenário")
                    .clicked()
                {
                    st.reset(scenario);
                }
            });
            ui.add_space(8.0);
        }

        memory_bar(ui, &sim, &names, current, st.fit);
        ui.add_space(8.0);

        if let Some(d) = current {
            let (icon, fill) = if d.failed {
                ("✖", DANGER.gamma_multiply(0.14))
            } else {
                ("➡", ACCENT.gamma_multiply(0.10))
            };
            egui::Frame::new()
                .inner_margin(egui::Margin::symmetric(10, 8))
                .corner_radius(7.0)
                .fill(fill)
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.horizontal_top(|ui| {
                        ui.label(egui::RichText::new(icon).size(14.0));
                        ui.add(egui::Label::new(egui::RichText::new(&d.caption).size(13.0)).wrap());
                    });
                });
            ui.add_space(8.0);
        }

        ui.label(
            egui::RichText::new("Lista encadeada")
                .size(12.5)
                .strong()
                .color(ACCENT),
        );
        ui.add_space(4.0);
        linked_list(ui, &sim, &names, current);
        ui.add_space(8.0);

        ui.label(
            egui::RichText::new("Mapa de bits")
                .size(12.5)
                .strong()
                .color(ACCENT),
        );
        ui.add_space(4.0);
        bitmap(ui, &sim);

        if n_ops > 0 {
            ui.add_space(10.0);
            ops_table(ui, scenario, st, &sim);
        }
    });
}

/// Índice de cor por processo: ordem de aparição no cenário, estável entre
/// os passos (liberar um processo não muda a cor dos outros).
fn color_index(name: &str, scenario_names: &[&str]) -> usize {
    scenario_names.iter().position(|n| *n == name).unwrap_or(0)
}

fn scenario_names(s: &Scenario) -> Vec<&'static str> {
    let mut names: Vec<&'static str> = Vec::new();
    let initial = s.initial.iter().filter_map(|seg| match seg {
        Seg::Proc(n, _) => Some(*n),
        Seg::Hole(_) => None,
    });
    let allocs = s.ops.iter().filter_map(|op| match op {
        Op::Alloc(n, _) => Some(*n),
        Op::Free(_) => None,
    });
    for n in initial.chain(allocs) {
        if n != "SO" && !names.contains(&n) {
            names.push(n);
        }
    }
    names
}

/// A fita de memória, com o segmento tocado pela última operação em destaque.
fn memory_bar(ui: &mut egui::Ui, sim: &Sim, names: &[&str], current: Option<&Decision>, fit: Fit) {
    let dark = ui.visuals().dark_mode;
    let visuals = ui.visuals().clone();

    const BAR_H: f32 = 44.0;
    const AXIS_H: f32 = 18.0;
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(width, BAR_H + AXIS_H + 6.0),
        egui::Sense::hover(),
    );
    let painter = ui.painter_at(rect);
    let bar = egui::Rect::from_min_size(
        egui::pos2(rect.left() + 1.0, rect.top() + 2.0),
        egui::vec2(width - 2.0, BAR_H),
    );
    let unit = bar.width() / sim.total.max(1) as f32;
    let line = visuals.text_color().gamma_multiply(0.45);

    painter.rect_filled(bar, 4.0, visuals.extreme_bg_color);

    for seg in &sim.segments {
        let x0 = bar.left() + seg.start as f32 * unit;
        let x1 = bar.left() + seg.end() as f32 * unit;
        let r = egui::Rect::from_min_max(egui::pos2(x0, bar.top()), egui::pos2(x1, bar.bottom()));
        match seg.owner {
            Some(name) => {
                let idx = color_index(name, names);
                let fill = if name == "SO" {
                    visuals.widgets.inactive.bg_fill
                } else {
                    series_color(idx, dark)
                };
                painter.rect_filled(r.shrink(1.0), 3.0, fill);
                let text_color = if name == "SO" {
                    visuals.text_color()
                } else {
                    egui::Color32::WHITE
                };
                if r.width() >= 26.0 {
                    painter.text(
                        r.center() - egui::vec2(0.0, 6.0),
                        egui::Align2::CENTER_CENTER,
                        name,
                        egui::FontId::monospace(12.0),
                        text_color,
                    );
                    painter.text(
                        r.center() + egui::vec2(0.0, 8.0),
                        egui::Align2::CENTER_CENTER,
                        seg.len.to_string(),
                        egui::FontId::proportional(10.0),
                        text_color.gamma_multiply(0.85),
                    );
                }
            }
            None => {
                // Bloco livre: só a borda tracejada.
                painter.rect_stroke(
                    r.shrink(1.5),
                    3.0,
                    egui::Stroke::new(1.0, line.gamma_multiply(0.6)),
                    egui::StrokeKind::Inside,
                );
                if r.width() >= 22.0 {
                    painter.text(
                        r.center(),
                        egui::Align2::CENTER_CENTER,
                        format!("{}", seg.len),
                        egui::FontId::proportional(11.0),
                        visuals.weak_text_color(),
                    );
                }
            }
        }
    }

    // Blocos examinados pela busca e o segmento tocado.
    if let Some(d) = current {
        for (start, len) in &d.examined {
            let x0 = bar.left() + *start as f32 * unit;
            let x1 = bar.left() + (*start + *len) as f32 * unit;
            painter.line_segment(
                [
                    egui::pos2(x0 + 2.0, bar.bottom() + 2.0),
                    egui::pos2(x1 - 2.0, bar.bottom() + 2.0),
                ],
                egui::Stroke::new(2.0, ACCENT.gamma_multiply(0.55)),
            );
        }
        if let Some((start, len)) = d.touched {
            let x0 = bar.left() + start as f32 * unit;
            let x1 = bar.left() + (start + len) as f32 * unit;
            let r =
                egui::Rect::from_min_max(egui::pos2(x0, bar.top()), egui::pos2(x1, bar.bottom()));
            painter.rect_stroke(
                r.shrink(0.5),
                3.0,
                egui::Stroke::new(2.2, ACCENT),
                egui::StrokeKind::Inside,
            );
        }
    }

    // Ponteiro do próximo encaixe.
    if fit == Fit::Next {
        let x = bar.left() + sim.rover as f32 * unit;
        painter.arrow(
            egui::pos2(x, bar.bottom() + 16.0),
            egui::vec2(0.0, -12.0),
            egui::Stroke::new(1.6, ACCENT),
        );
    }

    // Eixo de endereços.
    let step = if sim.total > 48 { 8 } else { 4 };
    let mut a = 0;
    while a <= sim.total {
        let x = bar.left() + a as f32 * unit;
        painter.line_segment(
            [
                egui::pos2(x, bar.bottom()),
                egui::pos2(x, bar.bottom() + 3.0),
            ],
            egui::Stroke::new(1.0, line),
        );
        // Os extremos ficam para dentro, para não serem cortados pela borda.
        let align = if a == 0 {
            egui::Align2::LEFT_TOP
        } else if a + step > sim.total {
            egui::Align2::RIGHT_TOP
        } else {
            egui::Align2::CENTER_TOP
        };
        painter.text(
            egui::pos2(x, bar.bottom() + 5.0),
            align,
            a.to_string(),
            egui::FontId::proportional(9.5),
            visuals.weak_text_color(),
        );
        a += step;
    }
}

/// A lista de segmentos: cada entrada com tipo, início, comprimento e o
/// ponteiro para a seguinte (3.2.23). Desenhada com o painter, em linhas de
/// entradas de tamanho fixo, para nenhuma entrada ser espremida na borda.
fn linked_list(ui: &mut egui::Ui, sim: &Sim, names: &[&str], current: Option<&Decision>) {
    let dark = ui.visuals().dark_mode;
    let visuals = ui.visuals().clone();
    let touched = current.and_then(|d| d.touched);

    const ENTRY_W: f32 = 64.0;
    const ENTRY_H: f32 = 34.0;
    const GAP: f32 = 22.0;
    const ROW_GAP: f32 = 10.0;

    let width = ui.available_width();
    let per_row = (((width + GAP) / (ENTRY_W + GAP)).floor() as usize).max(1);
    let n = sim.segments.len();
    let rows = n.div_ceil(per_row).max(1);
    let height = rows as f32 * (ENTRY_H + ROW_GAP) - ROW_GAP + 2.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let line = visuals.widgets.noninteractive.bg_stroke.color;

    for (i, seg) in sim.segments.iter().enumerate() {
        let col = i % per_row;
        let row = i / per_row;
        let r = egui::Rect::from_min_size(
            egui::pos2(
                rect.left() + col as f32 * (ENTRY_W + GAP),
                rect.top() + 1.0 + row as f32 * (ENTRY_H + ROW_GAP),
            ),
            egui::vec2(ENTRY_W, ENTRY_H),
        );
        let is_touched = touched == Some((seg.start, seg.len));
        let (tag, color) = match seg.owner {
            Some("SO") => ("P", visuals.text_color()),
            Some(name) => ("P", series_color(color_index(name, names), dark)),
            None => ("H", visuals.weak_text_color()),
        };
        painter.rect(
            r,
            5.0,
            visuals.extreme_bg_color,
            if is_touched {
                egui::Stroke::new(1.8, ACCENT)
            } else {
                egui::Stroke::new(1.0, line)
            },
            egui::StrokeKind::Inside,
        );
        painter.text(
            egui::pos2(r.center().x, r.top() + 9.0),
            egui::Align2::CENTER_CENTER,
            format!("{tag} {}", seg.owner.unwrap_or("")),
            egui::FontId::monospace(11.5),
            color,
        );
        painter.text(
            egui::pos2(r.center().x, r.bottom() - 9.0),
            egui::Align2::CENTER_CENTER,
            format!("{} · {}", seg.start, seg.len),
            egui::FontId::monospace(10.5),
            visuals.text_color(),
        );
        // O ponteiro para a entrada seguinte.
        if i + 1 < n {
            let a = egui::pos2(r.right() + 3.0, r.center().y);
            let b = if col + 1 < per_row {
                egui::pos2(r.right() + GAP - 3.0, r.center().y)
            } else {
                // Fim da linha: o ponteiro desce para a linha de baixo.
                egui::pos2(r.right() + 3.0, r.bottom() + ROW_GAP - 2.0)
            };
            painter.arrow(a, b - a, egui::Stroke::new(1.3, visuals.weak_text_color()));
        }
    }
    ui.add_space(2.0);
    ui.label(
        egui::RichText::new("P = processo, H = bloco livre; abaixo, início · comprimento.")
            .size(10.5)
            .color(egui::Color32::GRAY),
    );
}

/// O mapa de bits da mesma memória: um bit por unidade de alocação (3.2.20).
fn bitmap(ui: &mut egui::Ui, sim: &Sim) {
    let visuals = ui.visuals().clone();
    let total = sim.total.max(1) as usize;
    let per_row = 32.min(total);
    let rows = total.div_ceil(per_row);

    let width = ui.available_width();
    let cell = ((width - 2.0) / per_row as f32).clamp(6.0, 16.0);
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(cell * per_row as f32 + 2.0, cell * rows as f32 + 2.0),
        egui::Sense::hover(),
    );
    let painter = ui.painter_at(rect);

    let mut bits = vec![false; total];
    for seg in &sim.segments {
        if seg.owner.is_some() {
            for u in seg.start..seg.end() {
                if let Some(b) = bits.get_mut(u as usize) {
                    *b = true;
                }
            }
        }
    }

    for (i, &set) in bits.iter().enumerate() {
        let col = i % per_row;
        let row = i / per_row;
        let r = egui::Rect::from_min_size(
            egui::pos2(
                rect.left() + 1.0 + col as f32 * cell,
                rect.top() + 1.0 + row as f32 * cell,
            ),
            egui::vec2(cell, cell),
        );
        let fill = if set {
            ACCENT.gamma_multiply(0.75)
        } else {
            visuals.extreme_bg_color
        };
        painter.rect(
            r.shrink(0.5),
            1.5,
            fill,
            egui::Stroke::new(0.6, visuals.text_color().gamma_multiply(0.25)),
            egui::StrokeKind::Inside,
        );
        if cell >= 11.0 {
            painter.text(
                r.center(),
                egui::Align2::CENTER_CENTER,
                if set { "1" } else { "0" },
                egui::FontId::monospace(cell * 0.62),
                if set {
                    egui::Color32::WHITE
                } else {
                    visuals.weak_text_color()
                },
            );
        }
    }
    ui.add_space(2.0);
    ui.label(
        egui::RichText::new(format!(
            "{total} unidades, {total} bits: 1 = ocupada, 0 = livre."
        ))
        .size(10.5)
        .color(egui::Color32::GRAY),
    );
}

/// A sequência de operações, com os tamanhos das alocações editáveis.
fn ops_table(ui: &mut egui::Ui, scenario: &Scenario, st: &mut FitState, sim: &Sim) {
    ui.label(
        egui::RichText::new("Operações")
            .size(12.5)
            .strong()
            .color(ACCENT),
    );
    ui.add_space(2.0);
    egui::Grid::new(("ops", scenario.id))
        .num_columns(4)
        .spacing([12.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            for (i, op) in scenario.ops.iter().enumerate() {
                let is_current = st.step == i + 1;
                let done = i < st.step;
                let marker = if is_current {
                    "▶"
                } else if done {
                    "✔"
                } else {
                    ""
                };
                ui.label(egui::RichText::new(marker).size(11.0).color(if is_current {
                    ACCENT
                } else {
                    egui::Color32::GRAY
                }));
                match op {
                    Op::Alloc(name, _) => {
                        ui.label(
                            egui::RichText::new(format!("alocar {name}"))
                                .size(12.0)
                                .monospace(),
                        );
                        ui.add(
                            egui::DragValue::new(&mut st.sizes[i])
                                .range(1..=32)
                                .speed(0.15)
                                .suffix(" un."),
                        );
                    }
                    Op::Free(name) => {
                        ui.label(
                            egui::RichText::new(format!("liberar {name}"))
                                .size(12.0)
                                .monospace(),
                        );
                        ui.label("");
                    }
                }
                let status = match sim.decisions.get(i) {
                    Some(d) if d.failed => egui::RichText::new("não coube").color(DANGER),
                    Some(_) => egui::RichText::new("ok").color(egui::Color32::GRAY),
                    None => egui::RichText::new(""),
                };
                ui.label(status.size(11.0));
                ui.end_row();
            }
        });
}

/// Validação de cenário, usada pelos testes de conteúdo.
#[cfg(test)]
pub fn check(s: &Scenario) {
    assert!(!s.hint.trim().is_empty(), "cenário '{}' sem dica", s.id);
    assert!(
        !s.initial.is_empty(),
        "cenário '{}' sem memória inicial",
        s.id
    );
    for seg in s.initial {
        let len = match seg {
            Seg::Proc(_, n) | Seg::Hole(n) => *n,
        };
        assert!(len >= 1, "cenário '{}' com segmento vazio", s.id);
    }
    for op in s.ops {
        if let Op::Alloc(_, n) = op {
            assert!(*n >= 1, "cenário '{}' com alocação de 0 unidades", s.id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INITIAL: &[Seg] = &[
        Seg::Proc("SO", 8),
        Seg::Proc("A", 10),
        Seg::Hole(6),
        Seg::Proc("B", 8),
        Seg::Hole(12),
        Seg::Proc("C", 6),
        Seg::Hole(4),
        Seg::Proc("D", 5),
        Seg::Hole(5),
    ];

    fn scenario(ops: &'static [Op], fit: Fit) -> Scenario {
        Scenario {
            id: "t",
            initial: INITIAL,
            ops,
            fit,
            hint: "teste",
        }
    }

    fn sizes(s: &Scenario) -> Vec<u32> {
        FitState::new(s).sizes
    }

    fn holes(sim: &Sim) -> Vec<(u32, u32)> {
        sim.segments
            .iter()
            .filter(|s| s.is_hole())
            .map(|s| (s.start, s.len))
            .collect()
    }

    /// Os segmentos cobrem a memória inteira, sem buracos nem sobreposição.
    fn assert_contiguous(sim: &Sim) {
        let mut cursor = 0;
        for s in &sim.segments {
            assert_eq!(s.start, cursor, "segmento fora de lugar: {s:?}");
            assert!(s.len > 0, "segmento vazio: {s:?}");
            cursor = s.end();
        }
        assert_eq!(cursor, sim.total);
        // Nunca dois blocos livres vizinhos: a fusão garante isso.
        for w in sim.segments.windows(2) {
            assert!(
                !(w[0].is_hole() && w[1].is_hole()),
                "blocos livres vizinhos não fundidos: {:?} {:?}",
                w[0],
                w[1]
            );
        }
    }

    #[test]
    fn primeiro_encaixe_para_no_primeiro_que_cabe() {
        static OPS: &[Op] = &[Op::Alloc("E", 5)];
        let s = scenario(OPS, Fit::First);
        let sim = simulate(&s, Fit::First, &sizes(&s), 1);
        assert_contiguous(&sim);
        let e = sim.segments.iter().find(|s| s.owner == Some("E")).unwrap();
        assert_eq!((e.start, e.len), (18, 5));
        assert_eq!(holes(&sim), vec![(23, 1), (32, 12), (50, 4), (59, 5)]);
    }

    #[test]
    fn melhor_encaixe_escolhe_o_menor_que_cabe() {
        static OPS: &[Op] = &[Op::Alloc("E", 5)];
        let s = scenario(OPS, Fit::Best);
        let sim = simulate(&s, Fit::Best, &sizes(&s), 1);
        let e = sim.segments.iter().find(|s| s.owner == Some("E")).unwrap();
        assert_eq!((e.start, e.len), (59, 5), "o bloco de 5 encaixa exatamente");
        assert_contiguous(&sim);
    }

    #[test]
    fn pior_encaixe_escolhe_o_maior() {
        static OPS: &[Op] = &[Op::Alloc("E", 5)];
        let s = scenario(OPS, Fit::Worst);
        let sim = simulate(&s, Fit::Worst, &sizes(&s), 1);
        let e = sim.segments.iter().find(|s| s.owner == Some("E")).unwrap();
        assert_eq!((e.start, e.len), (32, 5));
        assert!(holes(&sim).contains(&(37, 7)));
    }

    #[test]
    fn proximo_encaixe_recomeca_de_onde_parou_e_da_a_volta() {
        static OPS: &[Op] = &[Op::Alloc("E", 5), Op::Alloc("F", 4), Op::Alloc("G", 4)];
        let s = scenario(OPS, Fit::Next);
        let sim = simulate(&s, Fit::Next, &sizes(&s), 3);
        let at = |n| {
            sim.segments
                .iter()
                .find(|s| s.owner == Some(n))
                .unwrap()
                .start
        };
        assert_eq!(at("E"), 18);
        // F não volta para o bloco de 1 que sobrou em 23: segue adiante.
        assert_eq!(at("F"), 32);
        assert_eq!(at("G"), 36);
        assert_contiguous(&sim);
    }

    #[test]
    fn liberar_funde_com_os_vizinhos_livres() {
        static OPS: &[Op] = &[Op::Free("B")];
        let s = scenario(OPS, Fit::First);
        let sim = simulate(&s, Fit::First, &sizes(&s), 1);
        // B (24..32) tinha blocos livres dos dois lados: 18..24 e 32..44.
        assert!(holes(&sim).contains(&(18, 26)));
        assert_contiguous(&sim);
        assert!(sim.decisions[0].caption.contains("três entradas"));
    }

    #[test]
    fn liberar_entre_processos_so_muda_o_tipo() {
        static TIGHT: &[Seg] = &[Seg::Proc("A", 8), Seg::Proc("B", 8), Seg::Proc("C", 8)];
        static OPS: &[Op] = &[Op::Free("B")];
        let s = Scenario {
            id: "t",
            initial: TIGHT,
            ops: OPS,
            fit: Fit::First,
            hint: "teste",
        };
        let sim = simulate(&s, Fit::First, &sizes(&s), 1);
        assert_eq!(holes(&sim), vec![(8, 8)]);
        assert_eq!(sim.segments.len(), TIGHT.len());
        assert!(sim.decisions[0].caption.contains("muda de tipo"));
        assert_contiguous(&sim);
    }

    #[test]
    fn liberar_com_um_vizinho_livre_funde_duas_entradas() {
        // A (8..18) tem o SO à esquerda e um bloco livre à direita (18..24).
        static OPS: &[Op] = &[Op::Free("A")];
        let s = scenario(OPS, Fit::First);
        let sim = simulate(&s, Fit::First, &sizes(&s), 1);
        assert!(holes(&sim).contains(&(8, 16)));
        assert_eq!(sim.segments.len(), INITIAL.len() - 1);
        assert!(sim.decisions[0].caption.contains("duas entradas"));
        assert_contiguous(&sim);
    }

    #[test]
    fn pedido_grande_demais_falha_por_fragmentacao() {
        // 27 livres no total, mas o maior bloco tem 12.
        static OPS: &[Op] = &[Op::Alloc("E", 13)];
        let s = scenario(OPS, Fit::First);
        for fit in Fit::ALL {
            let sim = simulate(&s, fit, &sizes(&s), 1);
            assert!(sim.decisions[0].failed, "{fit:?} deveria falhar");
            assert!(sim.decisions[0].caption.contains("fragmentação externa"));
            assert_contiguous(&sim);
        }
    }

    #[test]
    fn passos_parciais_respeitam_a_ordem() {
        static OPS: &[Op] = &[Op::Alloc("E", 5), Op::Free("A")];
        let s = scenario(OPS, Fit::First);
        let zero = simulate(&s, Fit::First, &sizes(&s), 0);
        assert!(zero.decisions.is_empty());
        assert_eq!(zero.segments.len(), INITIAL.len());
        let one = simulate(&s, Fit::First, &sizes(&s), 1);
        assert_eq!(one.decisions.len(), 1);
        assert!(one.segments.iter().any(|s| s.owner == Some("A")));
        let two = simulate(&s, Fit::First, &sizes(&s), 2);
        assert!(two.segments.iter().all(|s| s.owner != Some("A")));
    }
}
