//! Endereçamento: diagramas e widgets das seções 3.1–3.3 e 3.5.4.
//!
//! - [`layouts_diagram`]: as três organizações da memória sem abstração;
//! - [`base_limit`]: registradores de base e limite, com o endereço editável;
//! - [`mmu_diagram`]: a MMU entre o processador e a memória;
//! - [`translator`]: o tradutor de endereços virtuais — decomposição em bits,
//!   tabela de páginas e quadros desenhados, falta de página e, quando o
//!   cenário pede, uma TLB acumulando acertos e faltas;
//! - [`pte_diagram`]: os campos de uma entrada da tabela de páginas;
//! - [`page_size_calc`]: a curva do desperdício em função do tamanho da página.
//!
//! O tradutor usa o exemplo das anotações (3.3.10): espaço virtual de 64 KB,
//! memória física de 32 KB e páginas de 4 KB — 16 páginas, 8 quadros.

use std::collections::VecDeque;

use eframe::egui;

use crate::ui::{ACCENT, param_row, params_grid, widget_frame, widget_header};

const DANGER: egui::Color32 = egui::Color32::from_rgb(0xff, 0x6b, 0x5b);
const SAFE: egui::Color32 = egui::Color32::from_rgb(0x5b, 0xd6, 0x8a);
/// Cor do deslocamento nos endereços decompostos (laranja da paleta).
const OFFSET: egui::Color32 = egui::Color32::from_rgb(0xeb, 0x68, 0x34);

// ---------------------------------------------------------------------------
// Figura 5 — três organizações sem abstração de memória
// ---------------------------------------------------------------------------

pub fn layouts_diagram(ui: &mut egui::Ui) {
    let width = ui.available_width().min(620.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 230.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let visuals = ui.visuals().clone();
    let line = visuals.text_color().gamma_multiply(0.55);
    let stroke = egui::Stroke::new(1.3, line);

    // Três colunas iguais; cada uma é uma "memória" de 160 px de altura.
    let col_w = (width - 40.0) / 3.0;
    let box_w = col_w.min(150.0);
    let box_h = 160.0;
    let top = rect.top() + 6.0;

    // Cada organização: lista de (fração do topo, fração de baixo, rótulo, é_ROM).
    let layouts: [(&str, &[(f32, f32, &str, bool)]); 3] = [
        (
            "(a) tudo em RAM",
            &[
                (0.0, 0.72, "Programa\ndo usuário", false),
                (0.72, 1.0, "Sistema\noperacional", false),
            ],
        ),
        (
            "(b) sistema em ROM",
            &[
                (0.0, 0.28, "Sistema\noperacional", true),
                (0.28, 1.0, "Programa\ndo usuário", false),
            ],
        ),
        (
            "(c) drivers em ROM",
            &[
                (0.0, 0.2, "Drivers", true),
                (0.2, 0.72, "Programa\ndo usuário", false),
                (0.72, 1.0, "Sistema\noperacional", false),
            ],
        ),
    ];

    for (i, (caption, parts)) in layouts.iter().enumerate() {
        let cx = rect.left() + 20.0 + col_w * (i as f32 + 0.5);
        let r = egui::Rect::from_center_size(
            egui::pos2(cx, top + box_h / 2.0),
            egui::vec2(box_w, box_h),
        );
        for (f0, f1, label, rom) in parts.iter() {
            let part = egui::Rect::from_min_max(
                egui::pos2(r.left(), r.top() + box_h * f0),
                egui::pos2(r.right(), r.top() + box_h * f1),
            );
            let fill = if *rom {
                ACCENT.gamma_multiply(0.22)
            } else if label.starts_with("Sistema") {
                visuals.widgets.inactive.bg_fill
            } else {
                visuals.extreme_bg_color
            };
            painter.rect(part, 0.0, fill, stroke, egui::StrokeKind::Inside);
            painter.text(
                part.center(),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(12.0),
                visuals.text_color(),
            );
            if *rom {
                painter.text(
                    egui::pos2(part.right() - 4.0, part.top() + 3.0),
                    egui::Align2::RIGHT_TOP,
                    "ROM",
                    egui::FontId::proportional(9.0),
                    ACCENT,
                );
            }
        }
        painter.text(
            egui::pos2(cx, r.bottom() + 10.0),
            egui::Align2::CENTER_TOP,
            caption,
            egui::FontId::proportional(11.5),
            visuals.weak_text_color(),
        );
    }

    // Eixo de endereços à esquerda.
    painter.text(
        egui::pos2(rect.left() + 2.0, top),
        egui::Align2::LEFT_TOP,
        "M−1",
        egui::FontId::proportional(9.5),
        visuals.weak_text_color(),
    );
    painter.text(
        egui::pos2(rect.left() + 2.0, top + box_h),
        egui::Align2::LEFT_BOTTOM,
        "0",
        egui::FontId::proportional(9.5),
        visuals.weak_text_color(),
    );
    painter.text(
        egui::pos2(rect.center().x, rect.bottom() - 4.0),
        egui::Align2::CENTER_BOTTOM,
        "Em (a) e (c) o sistema está em RAM: um ponteiro inválido do usuário pode escrever sobre ele.",
        egui::FontId::proportional(11.0),
        visuals.weak_text_color(),
    );
}

// ---------------------------------------------------------------------------
// Registradores de base e limite (3.2.6–3.2.10)
// ---------------------------------------------------------------------------

/// Memória física do exemplo: 64 KB.
const BL_MEMORY: u32 = 65_536;

pub struct BaseLimitState {
    base: u32,
    limit: u32,
    v: u32,
}

impl Default for BaseLimitState {
    fn default() -> Self {
        Self {
            base: 16_384,
            limit: 20_480,
            v: 1000,
        }
    }
}

pub fn base_limit(ui: &mut egui::Ui, st: &mut BaseLimitState) {
    widget_frame(ui).show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        widget_header(
            ui,
            "Base e limite",
            "Mova a base: o programa muda de lugar sem mudar uma palavra. Passe do limite e veja o desvio.",
        );

        params_grid(ui, "base_limit", |ui| {
            param_row(
                ui,
                "base (b)",
                egui::Slider::new(&mut st.base, 0..=(BL_MEMORY - 4096)).step_by(1024.0),
            );
            param_row(
                ui,
                "limite",
                egui::Slider::new(&mut st.limit, 4096..=32_768).step_by(1024.0),
            );
            param_row(
                ui,
                "endereço virtual (v)",
                egui::Slider::new(&mut st.v, 0..=40_000),
            );
        });
        st.base = st.base.min(BL_MEMORY - st.limit);
        ui.add_space(8.0);

        let ok = st.v < st.limit;
        let f = st.v + st.base;

        // ---- desenho: a memória física com a região do processo ----
        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 92.0), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let visuals = ui.visuals().clone();
        let bar = egui::Rect::from_min_size(
            egui::pos2(rect.left() + 40.0, rect.top() + 26.0),
            egui::vec2(width - 80.0, 30.0),
        );
        let unit = bar.width() / BL_MEMORY as f32;
        painter.rect(
            bar,
            4.0,
            visuals.extreme_bg_color,
            egui::Stroke::new(1.0, visuals.text_color().gamma_multiply(0.4)),
            egui::StrokeKind::Inside,
        );
        let region = egui::Rect::from_min_max(
            egui::pos2(bar.left() + st.base as f32 * unit, bar.top()),
            egui::pos2(bar.left() + (st.base + st.limit) as f32 * unit, bar.bottom()),
        );
        painter.rect_filled(region.shrink(1.0), 3.0, ACCENT.gamma_multiply(0.35));
        painter.text(
            region.center(),
            egui::Align2::CENTER_CENTER,
            "processo",
            egui::FontId::proportional(11.5),
            visuals.text_color(),
        );
        // Marcas de base e base+limite.
        for (x, label) in [
            (region.left(), format!("b = {}", st.base)),
            (region.right(), format!("b + limite = {}", st.base + st.limit)),
        ] {
            painter.line_segment(
                [egui::pos2(x, bar.bottom()), egui::pos2(x, bar.bottom() + 6.0)],
                egui::Stroke::new(1.0, visuals.text_color().gamma_multiply(0.6)),
            );
            painter.text(
                egui::pos2(x, bar.bottom() + 8.0),
                egui::Align2::CENTER_TOP,
                label,
                egui::FontId::proportional(10.0),
                visuals.weak_text_color(),
            );
        }
        painter.text(
            egui::pos2(bar.left(), rect.top() + 2.0),
            egui::Align2::LEFT_TOP,
            "0",
            egui::FontId::proportional(10.0),
            visuals.weak_text_color(),
        );
        painter.text(
            egui::pos2(bar.right(), rect.top() + 2.0),
            egui::Align2::RIGHT_TOP,
            format!("{BL_MEMORY} (64 KB)"),
            egui::FontId::proportional(10.0),
            visuals.weak_text_color(),
        );
        // O endereço físico resultante.
        let fx = bar.left() + (f.min(BL_MEMORY) as f32) * unit;
        let color = if ok { SAFE } else { DANGER };
        painter.arrow(
            egui::pos2(fx, bar.top() - 2.0),
            egui::vec2(0.0, 8.0),
            egui::Stroke::new(2.0, color),
        );
        painter.text(
            egui::pos2(fx, rect.top() + 2.0),
            egui::Align2::CENTER_TOP,
            format!("f = {f}"),
            egui::FontId::proportional(10.5),
            color,
        );

        ui.add_space(4.0);
        if ok {
            ui.label(
                egui::RichText::new(format!(
                    "v = {} < limite = {}: acesso autorizado; f = v + b = {} + {} = {}",
                    st.v, st.limit, st.v, st.base, f
                ))
                .size(13.0)
                .color(SAFE),
            );
        } else {
            ui.label(
                egui::RichText::new(format!(
                    "v = {} ≥ limite = {}: desvio forçado para o sistema operacional, que \
                     encerra o processo infrator.",
                    st.v, st.limit
                ))
                .size(13.0)
                .color(DANGER),
            );
        }
    });
}

// ---------------------------------------------------------------------------
// Figura 6 — a MMU entre o processador e a memória
// ---------------------------------------------------------------------------

pub fn mmu_diagram(ui: &mut egui::Ui) {
    let width = ui.available_width().min(560.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 86.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let visuals = ui.visuals().clone();
    let line = visuals.text_color().gamma_multiply(0.55);

    let box_w = (width - 2.0 * 70.0) / 3.0;
    let box_h = 44.0;
    let y = rect.top() + 12.0;
    let boxes = ["Processador", "MMU", "Memória"];
    let mut centers = Vec::new();
    for (i, label) in boxes.iter().enumerate() {
        let x0 = rect.left() + i as f32 * (box_w + 70.0);
        let r = egui::Rect::from_min_size(egui::pos2(x0, y), egui::vec2(box_w, box_h));
        let accent = i == 1;
        painter.rect(
            r,
            8.0,
            if accent {
                ACCENT.gamma_multiply(0.22)
            } else {
                visuals.widgets.inactive.bg_fill
            },
            egui::Stroke::new(1.4, if accent { ACCENT } else { line }),
            egui::StrokeKind::Inside,
        );
        painter.text(
            r.center(),
            egui::Align2::CENTER_CENTER,
            *label,
            egui::FontId::proportional(14.0),
            visuals.text_color(),
        );
        centers.push(r);
    }
    for (i, (top, bottom)) in [("endereço", "virtual"), ("endereço", "físico")]
        .iter()
        .enumerate()
    {
        let a = egui::pos2(centers[i].right() + 4.0, y + box_h / 2.0);
        let b = egui::pos2(centers[i + 1].left() - 4.0, y + box_h / 2.0);
        painter.arrow(a, b - a, egui::Stroke::new(1.6, line));
        let mid = egui::pos2((a.x + b.x) / 2.0, a.y);
        painter.text(
            mid - egui::vec2(0.0, 6.0),
            egui::Align2::CENTER_BOTTOM,
            *top,
            egui::FontId::proportional(10.5),
            visuals.weak_text_color(),
        );
        painter.text(
            mid + egui::vec2(0.0, 6.0),
            egui::Align2::CENTER_TOP,
            *bottom,
            egui::FontId::proportional(10.5),
            if i == 0 { ACCENT } else { OFFSET },
        );
    }
}

// ---------------------------------------------------------------------------
// Figura 8 — campos de uma entrada da tabela de páginas
// ---------------------------------------------------------------------------

pub fn pte_diagram(ui: &mut egui::Ui) {
    let width = ui.available_width().min(640.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 96.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let visuals = ui.visuals().clone();
    let line = visuals.text_color().gamma_multiply(0.55);
    let stroke = egui::Stroke::new(1.2, line);

    let small = ((width - 8.0) / 12.0).clamp(30.0, 46.0);
    let n_small = 6;
    let big_w = width - small * n_small as f32 - 4.0;
    let h = 36.0;
    let top = rect.top() + 4.0;

    let big = egui::Rect::from_min_size(egui::pos2(rect.left() + 2.0, top), egui::vec2(big_w, h));
    painter.rect(
        big,
        0.0,
        ACCENT.gamma_multiply(0.18),
        stroke,
        egui::StrokeKind::Inside,
    );
    painter.text(
        big.center(),
        egui::Align2::CENTER_CENTER,
        "Número do quadro de página (52 bits)",
        egui::FontId::proportional(12.5),
        visuals.text_color(),
    );

    let fields: [(&str, &str); 6] = [
        ("C", "cache\ndesabilitado"),
        ("R", "referen-\nciada"),
        ("M", "modifi-\ncada"),
        ("S", "super-\nvisor"),
        ("Prot", "proteção\n(r w x)"),
        ("P", "presen-\nte"),
    ];
    for (i, (sigla, label)) in fields.iter().enumerate() {
        let r = egui::Rect::from_min_size(
            egui::pos2(big.right() + i as f32 * small, top),
            egui::vec2(small, h),
        );
        painter.rect(
            r,
            0.0,
            visuals.extreme_bg_color,
            stroke,
            egui::StrokeKind::Inside,
        );
        painter.text(
            r.center(),
            egui::Align2::CENTER_CENTER,
            *sigla,
            egui::FontId::proportional(if sigla.len() > 1 { 10.5 } else { 13.0 }),
            visuals.text_color(),
        );
        painter.text(
            egui::pos2(r.center().x, r.bottom() + 6.0),
            egui::Align2::CENTER_TOP,
            *label,
            egui::FontId::proportional(9.0),
            visuals.weak_text_color(),
        );
    }
}

// ---------------------------------------------------------------------------
// Tradutor de endereços (3.3.10–3.3.19) + TLB (3.3.30–3.3.33)
// ---------------------------------------------------------------------------

const PAGES: usize = 16;
const FRAMES: usize = 8;
const OFFSET_BITS: u32 = 12;
const PAGE_SIZE: u32 = 1 << OFFSET_BITS;
const VMAX: u32 = (PAGES as u32) * PAGE_SIZE - 1;
const TLB_SIZE: usize = 4;

/// Mapeamento da Figura 7: página → quadro.
const INITIAL_MAP: [Option<u8>; PAGES] = [
    None,
    None,
    None,
    None,
    Some(7),
    None,
    Some(5),
    None,
    None,
    None,
    Some(3),
    Some(4),
    Some(0),
    None,
    Some(6),
    Some(1),
];

pub struct TranslateScenario {
    pub id: &'static str,
    /// Mostra a TLB e o histórico de acessos.
    pub tlb: bool,
    pub hint: &'static str,
}

pub struct TranslateState {
    v: u32,
    /// Página → quadro (`None` = ausente).
    map: [Option<u8>; PAGES],
    /// Ordem de carga das páginas residentes, para a substituição FIFO.
    loaded: VecDeque<u8>,
    /// Entradas da TLB: (página, quadro), a mais antiga primeiro.
    tlb: VecDeque<(u8, u8)>,
    hits: u32,
    misses: u32,
    /// Últimos acessos: (página, acerto na TLB).
    history: VecDeque<(u8, bool)>,
    /// Última mensagem de evento (falta tratada, vetor percorrido, …).
    message: Option<(String, bool)>,
}

impl Default for TranslateState {
    fn default() -> Self {
        let loaded: VecDeque<u8> = [12, 15, 10, 11, 6, 14, 4].into_iter().collect();
        Self {
            v: 16_388,
            map: INITIAL_MAP,
            loaded,
            tlb: VecDeque::new(),
            hits: 0,
            misses: 0,
            history: VecDeque::new(),
            message: None,
        }
    }
}

/// Resultado de um acesso resolvido pela MMU (com TLB).
struct Access {
    tlb_hit: bool,
    page_fault: bool,
}

impl TranslateState {
    fn page_of(v: u32) -> u8 {
        (v >> OFFSET_BITS) as u8
    }

    fn free_frame(&self) -> Option<u8> {
        (0..FRAMES as u8).find(|f| !self.map.contains(&Some(*f)))
    }

    /// Trata a falta: quadro livre se houver, senão descarta a página mais
    /// antiga (FIFO). Devolve (quadro usado, página descartada).
    fn handle_fault(&mut self, page: u8) -> (u8, Option<u8>) {
        let (frame, evicted) = match self.free_frame() {
            Some(f) => (f, None),
            None => {
                let old = self.loaded.pop_front().unwrap_or(0);
                let f = self.map[old as usize].unwrap_or(0);
                self.map[old as usize] = None;
                self.tlb.retain(|(p, _)| *p != old);
                (f, Some(old))
            }
        };
        self.map[page as usize] = Some(frame);
        self.loaded.push_back(page);
        (frame, evicted)
    }

    /// Um acesso completo: TLB → tabela de páginas → (falta) → TLB atualizada.
    fn access(&mut self, v: u32) -> Access {
        let page = Self::page_of(v);
        let tlb_hit = self.tlb.iter().any(|(p, _)| *p == page);
        let mut page_fault = false;
        if tlb_hit {
            self.hits += 1;
        } else {
            self.misses += 1;
            if self.map[page as usize].is_none() {
                page_fault = true;
                self.handle_fault(page);
            }
            let frame = self.map[page as usize].unwrap_or(0);
            if self.tlb.len() >= TLB_SIZE {
                self.tlb.pop_front();
            }
            self.tlb.push_back((page, frame));
        }
        self.history.push_back((page, tlb_hit));
        while self.history.len() > 14 {
            self.history.pop_front();
        }
        Access {
            tlb_hit,
            page_fault,
        }
    }
}

pub fn translator(ui: &mut egui::Ui, scenario: &TranslateScenario, st: &mut TranslateState) {
    widget_frame(ui).show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        widget_header(ui, "Tradutor de endereços", scenario.hint);

        // ---- entrada ----
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("endereço virtual").size(12.5));
            ui.add(egui::DragValue::new(&mut st.v).range(0..=VMAX).speed(16.0));
            ui.label(
                egui::RichText::new(format!("= 0x{:04X}", st.v))
                    .monospace()
                    .size(12.5)
                    .color(egui::Color32::GRAY),
            );
            ui.separator();
            ui.label(
                egui::RichText::new("exemplos:")
                    .size(11.5)
                    .color(egui::Color32::GRAY),
            );
            for ex in [8196u32, 0, 20_500, 49_155, 65_535] {
                if ui.small_button(ex.to_string()).clicked() {
                    st.v = ex;
                    st.message = None;
                }
            }
            ui.separator();
            if ui
                .small_button("↺")
                .on_hover_text("Restaurar o mapeamento da figura e zerar a TLB")
                .clicked()
            {
                *st = TranslateState::default();
            }
        });
        ui.add_space(8.0);

        let page = TranslateState::page_of(st.v);
        let offset = st.v & (PAGE_SIZE - 1);
        let frame = st.map[page as usize];

        // ---- decomposição em bits ----
        bits_row(
            ui,
            "virtual",
            st.v,
            16,
            4,
            &format!("página = {page}"),
            &format!("deslocamento = {offset}"),
        );
        ui.add_space(4.0);
        match frame {
            Some(f) => {
                let phys = (u32::from(f) << OFFSET_BITS) | offset;
                bits_row(
                    ui,
                    "físico",
                    phys,
                    15,
                    3,
                    &format!("quadro = {f}"),
                    &format!("deslocamento = {offset}"),
                );
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(format!(
                        "{} = {page} × {PAGE_SIZE} + {offset}; a página {page} está no quadro {f}; \
                         endereço físico = {f} × {PAGE_SIZE} + {offset} = {phys}",
                        st.v
                    ))
                    .size(13.0)
                    .color(SAFE),
                );
            }
            None => {
                ui.label(
                    egui::RichText::new(format!(
                        "{} = {page} × {PAGE_SIZE} + {offset}; a página {page} tem bit de \
                         presença 0: falta de página. A MMU desvia para o sistema operacional.",
                        st.v
                    ))
                    .size(13.0)
                    .color(DANGER),
                );
                ui.add_space(4.0);
                if ui.button("Tratar a falta de página").clicked() {
                    let (f, evicted) = st.handle_fault(page);
                    let text = match evicted {
                        None => format!(
                            "O quadro {f} estava livre: a página {page} foi carregada nele, a \
                             tabela foi atualizada e a instrução repetida."
                        ),
                        Some(old) => format!(
                            "Nenhum quadro livre: o sistema escolheu a página {old} (a mais \
                             antiga), gravou-a em disco se estava modificada, carregou a página \
                             {page} no quadro {f}, atualizou a tabela e repetiu a instrução."
                        ),
                    };
                    st.message = Some((text, true));
                }
            }
        }

        if let Some((text, ok)) = &st.message {
            ui.add_space(6.0);
            egui::Frame::new()
                .inner_margin(egui::Margin::symmetric(10, 8))
                .corner_radius(7.0)
                .fill(if *ok {
                    ACCENT.gamma_multiply(0.10)
                } else {
                    DANGER.gamma_multiply(0.14)
                })
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.label(egui::RichText::new(text).size(12.5));
                });
        }

        ui.add_space(10.0);
        page_table_drawing(ui, st, page);

        if scenario.tlb {
            ui.add_space(10.0);
            tlb_panel(ui, st);
        }
    });
}

/// Uma fileira de bits com a parte alta (página/quadro) e a baixa (deslocamento)
/// em cores distintas.
fn bits_row(
    ui: &mut egui::Ui,
    label: &str,
    value: u32,
    n_bits: u32,
    high_bits: u32,
    high_label: &str,
    low_label: &str,
) {
    let visuals = ui.visuals().clone();
    let cell = 17.0;
    let label_w = 52.0;
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, cell + 16.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);

    painter.text(
        egui::pos2(rect.left(), rect.top() + cell / 2.0),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(11.5),
        visuals.weak_text_color(),
    );
    let x0 = rect.left() + label_w;
    for i in 0..n_bits {
        let bit = (value >> (n_bits - 1 - i)) & 1;
        let is_high = i < high_bits;
        let color = if is_high { ACCENT } else { OFFSET };
        let r = egui::Rect::from_min_size(
            egui::pos2(x0 + i as f32 * cell, rect.top()),
            egui::vec2(cell, cell),
        );
        painter.rect(
            r.shrink(0.5),
            2.0,
            color.gamma_multiply(if bit == 1 { 0.55 } else { 0.12 }),
            egui::Stroke::new(0.8, color.gamma_multiply(0.7)),
            egui::StrokeKind::Inside,
        );
        painter.text(
            r.center(),
            egui::Align2::CENTER_CENTER,
            bit.to_string(),
            egui::FontId::monospace(11.0),
            visuals.text_color(),
        );
    }
    // Chaves com os rótulos.
    let high_w = high_bits as f32 * cell;
    let low_w = (n_bits - high_bits) as f32 * cell;
    painter.text(
        egui::pos2(x0 + high_w / 2.0, rect.top() + cell + 2.0),
        egui::Align2::CENTER_TOP,
        high_label,
        egui::FontId::proportional(10.0),
        ACCENT,
    );
    painter.text(
        egui::pos2(x0 + high_w + low_w / 2.0, rect.top() + cell + 2.0),
        egui::Align2::CENTER_TOP,
        low_label,
        egui::FontId::proportional(10.0),
        OFFSET,
    );
}

/// As páginas virtuais à esquerda, os quadros à direita e as setas entre eles.
fn page_table_drawing(ui: &mut egui::Ui, st: &TranslateState, current: u8) {
    let visuals = ui.visuals().clone();
    let row_h = 17.0;
    let box_w = 92.0;
    let width = ui.available_width().min(520.0);
    let height = PAGES as f32 * row_h + 24.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let line = visuals.text_color().gamma_multiply(0.5);

    let left_x = rect.left() + 34.0;
    let right_x = rect.right() - box_w - 34.0;
    let top = rect.top() + 20.0;

    painter.text(
        egui::pos2(left_x + box_w / 2.0, rect.top() + 2.0),
        egui::Align2::CENTER_TOP,
        "páginas virtuais",
        egui::FontId::proportional(10.5),
        visuals.weak_text_color(),
    );
    painter.text(
        egui::pos2(right_x + box_w / 2.0, rect.top() + 2.0),
        egui::Align2::CENTER_TOP,
        "quadros de página",
        egui::FontId::proportional(10.5),
        visuals.weak_text_color(),
    );

    // As páginas crescem de baixo para cima, como na figura.
    let page_rect = |p: usize| {
        let y = top + (PAGES - 1 - p) as f32 * row_h;
        egui::Rect::from_min_size(egui::pos2(left_x, y), egui::vec2(box_w, row_h))
    };
    let frame_rect = |f: usize| {
        let y = top + (PAGES - 1 - f) as f32 * row_h;
        egui::Rect::from_min_size(egui::pos2(right_x, y), egui::vec2(box_w, row_h))
    };

    for p in 0..PAGES {
        let r = page_rect(p);
        let is_current = p == current as usize;
        let present = st.map[p].is_some();
        let fill = if is_current {
            ACCENT.gamma_multiply(0.35)
        } else if present {
            ACCENT.gamma_multiply(0.10)
        } else {
            visuals.extreme_bg_color
        };
        painter.rect(
            r.shrink(0.5),
            0.0,
            fill,
            egui::Stroke::new(
                if is_current { 1.6 } else { 0.8 },
                if is_current { ACCENT } else { line },
            ),
            egui::StrokeKind::Inside,
        );
        painter.text(
            egui::pos2(r.left() - 5.0, r.center().y),
            egui::Align2::RIGHT_CENTER,
            p.to_string(),
            egui::FontId::monospace(9.5),
            visuals.weak_text_color(),
        );
        let text = match st.map[p] {
            Some(f) => format!("{f}   P=1"),
            None => "—   P=0".to_string(),
        };
        painter.text(
            r.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::monospace(10.5),
            visuals.text_color(),
        );
    }

    for f in 0..FRAMES {
        let r = frame_rect(f);
        let resident = (0..PAGES).find(|&p| st.map[p] == Some(f as u8));
        let is_current = resident == Some(current as usize);
        let fill = if is_current {
            OFFSET.gamma_multiply(0.35)
        } else if resident.is_some() {
            OFFSET.gamma_multiply(0.10)
        } else {
            visuals.extreme_bg_color
        };
        painter.rect(
            r.shrink(0.5),
            0.0,
            fill,
            egui::Stroke::new(
                if is_current { 1.6 } else { 0.8 },
                if is_current { OFFSET } else { line },
            ),
            egui::StrokeKind::Inside,
        );
        painter.text(
            egui::pos2(r.right() + 5.0, r.center().y),
            egui::Align2::LEFT_CENTER,
            f.to_string(),
            egui::FontId::monospace(9.5),
            visuals.weak_text_color(),
        );
        let text = match resident {
            Some(p) => format!("página {p}"),
            None => "livre".to_string(),
        };
        painter.text(
            r.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::monospace(10.5),
            visuals.text_color(),
        );
    }

    // Setas: as demais em cinza, a página atual em destaque.
    for p in 0..PAGES {
        if let Some(f) = st.map[p] {
            let a = page_rect(p).right_center() + egui::vec2(2.0, 0.0);
            let b = frame_rect(f as usize).left_center() - egui::vec2(2.0, 0.0);
            let is_current = p == current as usize;
            painter.arrow(
                a,
                b - a,
                egui::Stroke::new(
                    if is_current { 2.0 } else { 1.0 },
                    if is_current {
                        ACCENT
                    } else {
                        line.gamma_multiply(0.5)
                    },
                ),
            );
        }
    }
}

/// A TLB: entradas, contadores e o histórico de acessos.
fn tlb_panel(ui: &mut egui::Ui, st: &mut TranslateState) {
    ui.label(egui::RichText::new("TLB").size(12.5).strong().color(ACCENT));
    ui.add_space(2.0);
    ui.horizontal_wrapped(|ui| {
        if ui
            .button("Acessar")
            .on_hover_text("Faz um acesso ao endereço acima passando pela TLB")
            .clicked()
        {
            let v = st.v;
            let a = st.access(v);
            let page = TranslateState::page_of(v);
            let text = match (a.tlb_hit, a.page_fault) {
                (true, _) => format!(
                    "Página {page} estava na TLB: quadro obtido sem consultar a tabela de páginas."
                ),
                (false, false) => format!(
                    "Página {page} não estava na TLB: a MMU consultou a tabela de páginas e \
                     colocou a tradução na TLB."
                ),
                (false, true) => format!(
                    "Página {page} não estava na TLB nem na memória: falta de página tratada, \
                     tabela atualizada e tradução colocada na TLB."
                ),
            };
            st.message = Some((text, a.tlb_hit || !a.page_fault));
        }
        if ui
            .button("Percorrer um vetor de 16 KB a partir daqui")
            .on_hover_text("4096 leituras de 4 bytes, uma atrás da outra (3.3.33)")
            .clicked()
        {
            let start = st.v;
            let before = (st.hits, st.misses);
            let mut faults = 0;
            for i in 0..4096u32 {
                let v = (start + 4 * i).min(VMAX);
                if st.access(v).page_fault {
                    faults += 1;
                }
            }
            let hits = st.hits - before.0;
            let misses = st.misses - before.1;
            let rate = 100.0 * hits as f32 / (hits + misses) as f32;
            st.message = Some((
                format!(
                    "4096 leituras: {misses} faltas na TLB (uma por página nova), {hits} acertos \
                     — taxa de acerto de {rate:.1}%. Faltas de página tratadas no caminho: {faults}."
                ),
                true,
            ));
        }
        if ui.small_button("zerar contadores").clicked() {
            st.hits = 0;
            st.misses = 0;
            st.history.clear();
            st.tlb.clear();
            st.message = None;
        }
    });
    ui.add_space(6.0);

    ui.columns(2, |cols| {
        egui::Grid::new("tlb_entries")
            .num_columns(3)
            .spacing([14.0, 3.0])
            .striped(true)
            .show(&mut cols[0], |ui| {
                for h in ["entrada", "página", "quadro"] {
                    ui.label(egui::RichText::new(h).size(11.0).strong());
                }
                ui.end_row();
                for i in 0..TLB_SIZE {
                    ui.label(egui::RichText::new(i.to_string()).size(11.5).monospace());
                    match st.tlb.get(i) {
                        Some((p, f)) => {
                            ui.label(egui::RichText::new(p.to_string()).size(11.5).monospace());
                            ui.label(egui::RichText::new(f.to_string()).size(11.5).monospace());
                        }
                        None => {
                            ui.label(
                                egui::RichText::new("—")
                                    .size(11.5)
                                    .color(egui::Color32::GRAY),
                            );
                            ui.label(
                                egui::RichText::new("—")
                                    .size(11.5)
                                    .color(egui::Color32::GRAY),
                            );
                        }
                    }
                    ui.end_row();
                }
            });

        let ui = &mut cols[1];
        let total = st.hits + st.misses;
        let rate = if total == 0 {
            "—".to_string()
        } else {
            format!("{:.1}%", 100.0 * st.hits as f32 / total as f32)
        };
        ui.label(
            egui::RichText::new(format!(
                "acertos: {}   ·   faltas: {}   ·   taxa de acerto: {rate}",
                st.hits, st.misses
            ))
            .size(12.5)
            .strong(),
        );
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("últimos acessos (página, ✔ acerto / ✖ falta na TLB):")
                .size(11.0)
                .color(egui::Color32::GRAY),
        );
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
            if st.history.is_empty() {
                ui.label(
                    egui::RichText::new("nenhum ainda")
                        .size(11.0)
                        .color(egui::Color32::GRAY),
                );
            }
            for (p, hit) in &st.history {
                let (glyph, color) = if *hit { ("✔", SAFE) } else { ("✖", DANGER) };
                egui::Frame::new()
                    .inner_margin(egui::Margin::symmetric(5, 2))
                    .corner_radius(4.0)
                    .fill(color.gamma_multiply(0.14))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(format!("{p} {glyph}"))
                                .monospace()
                                .size(11.0),
                        );
                    });
            }
        });
    });
}

// ---------------------------------------------------------------------------
// Tamanho da página: o(p) = s·e/p + p/2 (3.5.22–3.5.24)
// ---------------------------------------------------------------------------

pub struct PageSizeState {
    /// Tamanho médio do processo, em KB.
    s_kb: f32,
    /// Tamanho da entrada da tabela, em bytes.
    e: u32,
}

impl Default for PageSizeState {
    fn default() -> Self {
        Self { s_kb: 1024.0, e: 8 }
    }
}

/// O desperdício por processo para páginas de `p` bytes.
pub fn overhead(s: f64, e: f64, p: f64) -> f64 {
    s * e / p + p / 2.0
}

/// O tamanho de página que minimiza o desperdício.
pub fn optimal_page(s: f64, e: f64) -> f64 {
    (2.0 * s * e).sqrt()
}

pub fn page_size_calc(ui: &mut egui::Ui, st: &mut PageSizeState) {
    widget_frame(ui).show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        widget_header(
            ui,
            "Tamanho da página",
            "Mexa no tamanho do processo e da entrada: o mínimo da curva é p = √(2se).",
        );
        params_grid(ui, "page_size", |ui| {
            param_row(
                ui,
                "tamanho do processo (s)",
                egui::Slider::new(&mut st.s_kb, 64.0..=65_536.0)
                    .logarithmic(true)
                    .suffix(" KB"),
            );
            param_row(
                ui,
                "entrada da tabela (e)",
                egui::Slider::new(&mut st.e, 2..=32).suffix(" B"),
            );
        });
        ui.add_space(6.0);

        let s = f64::from(st.s_kb) * 1024.0;
        let e = f64::from(st.e);
        let p_opt = optimal_page(s, e);
        let pow2 = 2f64.powf(p_opt.log2().round());

        // ---- gráfico ----
        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 190.0), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let visuals = ui.visuals().clone();
        let plot = egui::Rect::from_min_max(
            rect.min + egui::vec2(52.0, 10.0),
            rect.max - egui::vec2(12.0, 26.0),
        );
        let grid = visuals.text_color().gamma_multiply(0.16);
        let axis = visuals.text_color().gamma_multiply(0.45);

        // Eixo x: log2(p) de 7 (128 B) a 17 (128 KB). Eixo y: até 3× o mínimo.
        let (lx0, lx1) = (7.0f64, 17.0f64);
        let y_max = overhead(s, e, p_opt) * 3.0;
        let to_x = |p: f64| plot.left() + ((p.log2() - lx0) / (lx1 - lx0)) as f32 * plot.width();
        let to_y = |o: f64| plot.bottom() - (o / y_max).min(1.0) as f32 * plot.height();

        // Grade vertical nas potências de dois.
        for k in (lx0 as i32)..=(lx1 as i32) {
            let p = 2f64.powi(k);
            let x = to_x(p);
            painter.line_segment(
                [egui::pos2(x, plot.top()), egui::pos2(x, plot.bottom())],
                egui::Stroke::new(1.0, grid),
            );
            if k % 2 == 1 {
                // O último rótulo alinha para dentro, para não ser cortado.
                let align = if k == lx1 as i32 {
                    egui::Align2::RIGHT_TOP
                } else {
                    egui::Align2::CENTER_TOP
                };
                painter.text(
                    egui::pos2(x, plot.bottom() + 4.0),
                    align,
                    human(p),
                    egui::FontId::proportional(9.5),
                    axis,
                );
            }
        }
        painter.line_segment(
            [plot.left_bottom(), plot.right_bottom()],
            egui::Stroke::new(1.0, axis),
        );
        painter.line_segment(
            [plot.left_top(), plot.left_bottom()],
            egui::Stroke::new(1.0, axis),
        );
        painter.text(
            egui::pos2(plot.left() - 4.0, plot.top()),
            egui::Align2::RIGHT_TOP,
            human(y_max),
            egui::FontId::proportional(9.5),
            axis,
        );
        painter.text(
            egui::pos2(plot.left() - 4.0, plot.bottom()),
            egui::Align2::RIGHT_BOTTOM,
            "0",
            egui::FontId::proportional(9.5),
            axis,
        );

        // As duas parcelas (finas) e a soma (grossa).
        let n = 120;
        let sample = |f: &dyn Fn(f64) -> f64| -> Vec<egui::Pos2> {
            (0..=n)
                .map(|i| {
                    let lp = lx0 + (lx1 - lx0) * i as f64 / n as f64;
                    let p = 2f64.powf(lp);
                    egui::pos2(to_x(p), to_y(f(p)))
                })
                .collect()
        };
        painter.add(egui::Shape::line(
            sample(&|p| s * e / p),
            egui::Stroke::new(1.0, ACCENT.gamma_multiply(0.6)),
        ));
        painter.add(egui::Shape::line(
            sample(&|p| p / 2.0),
            egui::Stroke::new(1.0, OFFSET.gamma_multiply(0.7)),
        ));
        painter.add(egui::Shape::line(
            sample(&|p| overhead(s, e, p)),
            egui::Stroke::new(2.2, visuals.text_color()),
        ));

        // O mínimo.
        let mx = to_x(p_opt);
        let my = to_y(overhead(s, e, p_opt));
        painter.line_segment(
            [egui::pos2(mx, my), egui::pos2(mx, plot.bottom())],
            egui::Stroke::new(1.0, SAFE),
        );
        painter.circle_filled(egui::pos2(mx, my), 4.5, SAFE);
        painter.text(
            egui::pos2(mx + 6.0, my - 6.0),
            egui::Align2::LEFT_BOTTOM,
            format!("p = {}", human(p_opt)),
            egui::FontId::proportional(11.0),
            SAFE,
        );

        // Legenda, abaixo do gráfico (dentro dele, sobreporia os eixos nas
        // larguras menores).
        ui.horizontal_wrapped(|ui| {
            for (label, color) in [
                ("— tabela de páginas: s·e/p", ACCENT),
                ("— última página: p/2", OFFSET),
                ("— desperdício total o(p)", visuals.text_color()),
            ] {
                ui.label(egui::RichText::new(label).size(10.5).color(color));
            }
        });

        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(format!(
                "p = √(2 × {} × {}) = {:.0} bytes ≈ {}; desperdício mínimo por processo: {}.",
                human(s),
                st.e,
                p_opt,
                human(pow2),
                human(overhead(s, e, p_opt))
            ))
            .size(13.0)
            .strong()
            .color(ACCENT),
        );
    });
}

/// Formata bytes de forma legível (1024 → "1 KB").
fn human(bytes: f64) -> String {
    if bytes >= 1024.0 * 1024.0 {
        let v = bytes / (1024.0 * 1024.0);
        if (v - v.round()).abs() < 0.05 {
            format!("{} MB", v.round())
        } else {
            format!("{v:.1} MB")
        }
    } else if bytes >= 1024.0 {
        let v = bytes / 1024.0;
        if (v - v.round()).abs() < 0.05 {
            format!("{} KB", v.round())
        } else {
            format!("{v:.1} KB")
        }
    } else {
        format!("{} B", bytes.round())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O exemplo das anotações: 1 MB e entradas de 8 bytes dão páginas de 4 KB.
    #[test]
    fn pagina_otima_do_exemplo() {
        let p = optimal_page(1024.0 * 1024.0, 8.0);
        assert!((p - 4096.0).abs() < 1e-6, "esperado 4096, veio {p}");
    }

    /// O mínimo de o(p) está de fato em √(2se).
    #[test]
    fn minimo_da_curva() {
        let (s, e) = (4.0 * 1024.0 * 1024.0, 8.0);
        let p = optimal_page(s, e);
        let o = overhead(s, e, p);
        assert!(overhead(s, e, p * 0.8) > o);
        assert!(overhead(s, e, p * 1.25) > o);
    }

    /// O endereço 8196 do exemplo cai na página 2, deslocamento 4.
    #[test]
    fn decomposicao_do_exemplo() {
        assert_eq!(TranslateState::page_of(8196), 2);
        assert_eq!(8196 & (PAGE_SIZE - 1), 4);
        assert_eq!((6 << OFFSET_BITS) | 4, 24_580);
    }

    /// A falta de página usa o quadro livre; sem quadro livre, descarta o mais
    /// antigo e invalida a entrada dele na TLB.
    #[test]
    fn falta_de_pagina_usa_quadro_livre_depois_substitui() {
        let mut st = TranslateState::default();
        assert_eq!(st.free_frame(), Some(2));
        let (f, evicted) = st.handle_fault(3);
        assert_eq!((f, evicted), (2, None));
        assert_eq!(st.free_frame(), None);
        // Página 12 foi a primeira carregada: sai ela.
        st.tlb.push_back((12, 0));
        let (f, evicted) = st.handle_fault(9);
        assert_eq!((f, evicted), (0, Some(12)));
        assert!(st.map[12].is_none());
        assert!(st.tlb.iter().all(|(p, _)| *p != 12));
    }

    /// Percorrer 16 KB a partir de uma página residente dá 4 faltas na TLB.
    #[test]
    fn vetor_de_16kb_da_quatro_faltas_na_tlb() {
        let mut st = TranslateState::default();
        let start = 4 * PAGE_SIZE; // página 4, residente
        for i in 0..4096u32 {
            st.access(start + 4 * i);
        }
        assert_eq!(st.misses, 4);
        assert_eq!(st.hits, 4092);
    }

    #[test]
    fn formatacao_legivel() {
        assert_eq!(human(4096.0), "4 KB");
        assert_eq!(human(1024.0 * 1024.0), "1 MB");
        assert_eq!(human(512.0), "512 B");
    }
}
