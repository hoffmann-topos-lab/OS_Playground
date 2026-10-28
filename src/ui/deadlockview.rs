//! Diagramas e simuladores do capítulo 6 (impasses).
//!
//! Reúne:
//! - duas figuras estáticas: o grafo de alocação com um ciclo (6.3.8) e a
//!   trajetória de recursos (6.6.13);
//! - o grafo de alocação de recursos com detecção de ciclo (6.5.2 a 6.5.5),
//!   que escolhe entre cenários prontos e revela o ciclo passo a passo;
//! - a detecção com vários recursos de cada tipo (6.5.6 a 6.5.15), com as
//!   matrizes editáveis e a marcação dos processos que terminam;
//! - o algoritmo do banqueiro (6.6.7 a 6.6.11), que decide se um estado é
//!   seguro e, quando é, exibe a ordem de escalonamento que o conclui.
//!
//! A detecção com matrizes e o banqueiro compartilham a mesma busca: achar um
//! processo cuja demanda cabe no vetor de disponíveis, marcá-lo e devolver a
//! sua alocação ao vetor. Muda só a demanda (requisição atual na detecção,
//! necessidade que falta no banqueiro).

use eframe::egui;

use crate::ui::{ACCENT, widget_frame, widget_header};

const DANGER: egui::Color32 = egui::Color32::from_rgb(0xff, 0x6b, 0x5b);
const SAFE: egui::Color32 = egui::Color32::from_rgb(0x5b, 0xd6, 0x8a);

// ---------------------------------------------------------------------------
// Primitivas de desenho de grafos
// ---------------------------------------------------------------------------

const NODE_R: f32 = 20.0;

fn draw_node(
    painter: &egui::Painter,
    center: egui::Pos2,
    is_proc: bool,
    label: &str,
    fill: egui::Color32,
    stroke: egui::Color32,
    txt: egui::Color32,
) {
    if is_proc {
        painter.circle(center, NODE_R, fill, egui::Stroke::new(1.8, stroke));
    } else {
        let r = egui::Rect::from_center_size(center, egui::vec2(NODE_R * 2.0, NODE_R * 2.0));
        painter.rect(
            r,
            4.0,
            fill,
            egui::Stroke::new(1.8, stroke),
            egui::StrokeKind::Inside,
        );
    }
    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(13.0),
        txt,
    );
}

/// Desenha uma seta reta de `from` a `to` com a ponta de tamanho fixo.
///
/// O `painter.arrow` do egui calcula a ponta como `comprimento / 4`: numa
/// aresta longa ela vira um "V" enorme cujas barbas parecem outras arestas.
/// Aqui a ponta é curta e constante, encolhendo só em arestas bem curtas.
fn draw_arrow(painter: &egui::Painter, from: egui::Pos2, to: egui::Pos2, color: egui::Color32, w: f32) {
    let stroke = egui::Stroke::new(w, color);
    painter.line_segment([from, to], stroke);
    let len = (to - from).length();
    if len < 1.0 {
        return;
    }
    let dir = (to - from) / len;
    let tip = 9.0_f32.min(len * 0.4);
    let rot = egui::emath::Rot2::from_angle(std::f32::consts::TAU / 14.0);
    painter.line_segment([to, to - tip * (rot * dir)], stroke);
    painter.line_segment([to, to - tip * (rot.inverse() * dir)], stroke);
}

/// Ponto na borda de um nó, na direção `dir` a partir do centro. Um processo é
/// um círculo (raio `NODE_R`); um recurso é um quadrado de meio-lado `NODE_R`.
fn node_edge(center: egui::Pos2, is_proc: bool, dir: egui::Vec2) -> egui::Pos2 {
    if is_proc {
        center + dir * NODE_R
    } else {
        let d = dir.x.abs().max(dir.y.abs()).max(1e-3);
        center + dir * (NODE_R / d)
    }
}

/// Seta entre dois nós-círculo adjacentes (figuras estáticas do capítulo).
fn arrow_between(painter: &egui::Painter, a: egui::Pos2, b: egui::Pos2, color: egui::Color32, w: f32) {
    let dir = (b - a).normalized();
    draw_arrow(painter, a + dir * (NODE_R + 4.0), b - dir * (NODE_R + 5.0), color, w);
}

// ---------------------------------------------------------------------------
// Figura estática: grafo de alocação com um ciclo (seção 6.3.8)
// ---------------------------------------------------------------------------

pub fn rag_diagram(ui: &mut egui::Ui) {
    let width = ui.available_width().min(460.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 220.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let v = ui.visuals().clone();
    let txt = v.text_color();

    let a = egui::pos2(rect.left() + width * 0.3, rect.top() + 46.0);
    let r2 = egui::pos2(rect.right() - width * 0.3, rect.top() + 46.0);
    let b = egui::pos2(rect.right() - width * 0.3, rect.bottom() - 54.0);
    let r1 = egui::pos2(rect.left() + width * 0.3, rect.bottom() - 54.0);

    // O ciclo R1 -> A -> R2 -> B -> R1.
    arrow_between(&painter, r1, a, DANGER, 1.8);
    arrow_between(&painter, a, r2, DANGER, 1.8);
    arrow_between(&painter, r2, b, DANGER, 1.8);
    arrow_between(&painter, b, r1, DANGER, 1.8);

    draw_node(&painter, a, true, "A", ACCENT.gamma_multiply(0.22), ACCENT, txt);
    draw_node(&painter, b, true, "B", ACCENT.gamma_multiply(0.22), ACCENT, txt);
    draw_node(&painter, r1, false, "R1", SAFE.gamma_multiply(0.18), SAFE, txt);
    draw_node(&painter, r2, false, "R2", SAFE.gamma_multiply(0.18), SAFE, txt);

    painter.text(
        egui::pos2(rect.center().x, rect.bottom() - 12.0),
        egui::Align2::CENTER_CENTER,
        "R1 está com A, que pede R2; R2 está com B, que pede R1. O ciclo confirma o impasse.",
        egui::FontId::proportional(10.5),
        v.weak_text_color(),
    );
}

// ---------------------------------------------------------------------------
// Figura estática: trajetória de recursos (seção 6.6.13)
// ---------------------------------------------------------------------------

pub fn trajectory(ui: &mut egui::Ui) {
    let width = ui.available_width().min(460.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 300.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let v = ui.visuals().clone();
    let txt = v.text_color();
    let axis = txt.gamma_multiply(0.6);

    let pad = 42.0;
    let plot = egui::Rect::from_min_max(
        egui::pos2(rect.left() + pad, rect.top() + 14.0),
        egui::pos2(rect.right() - 14.0, rect.bottom() - pad),
    );
    // Mapa de coordenadas 0..8 para o quadro, com origem embaixo à esquerda.
    let span = 8.0;
    let px = |x: f32| plot.left() + x / span * plot.width();
    let py = |y: f32| plot.bottom() - y / span * plot.height();

    // Eixos.
    painter.arrow(
        egui::pos2(plot.left(), plot.bottom()),
        egui::vec2(plot.width() + 6.0, 0.0),
        egui::Stroke::new(1.4, axis),
    );
    painter.arrow(
        egui::pos2(plot.left(), plot.bottom()),
        egui::vec2(0.0, -(plot.height() + 6.0)),
        egui::Stroke::new(1.4, axis),
    );
    painter.text(
        egui::pos2(plot.right(), plot.bottom() + 14.0),
        egui::Align2::RIGHT_CENTER,
        "progresso de A",
        egui::FontId::proportional(10.5),
        axis,
    );
    painter.text(
        egui::pos2(plot.left() - 30.0, plot.top()),
        egui::Align2::LEFT_CENTER,
        "progresso de B",
        egui::FontId::proportional(10.5),
        axis,
    );

    // Regiões proibidas (cada recurso é único): dois retângulos cruzados.
    let gray = txt.gamma_multiply(0.18);
    painter.rect_filled(
        egui::Rect::from_min_max(egui::pos2(px(2.0), py(6.0)), egui::pos2(px(5.0), py(3.0))),
        0.0,
        gray,
    );
    painter.rect_filled(
        egui::Rect::from_min_max(egui::pos2(px(3.0), py(5.0)), egui::pos2(px(6.0), py(2.0))),
        0.0,
        gray,
    );
    painter.text(
        egui::pos2(px(4.0), py(4.0)),
        egui::Align2::CENTER_CENTER,
        "região\nproibida",
        egui::FontId::proportional(10.0),
        txt,
    );

    // Marcas de instrução nos eixos.
    for (x, l) in [(2.0, "I1"), (3.0, "I2"), (5.0, "I3"), (6.0, "I4")] {
        painter.line_segment(
            [egui::pos2(px(x), plot.bottom() - 3.0), egui::pos2(px(x), plot.bottom() + 3.0)],
            egui::Stroke::new(1.0, axis),
        );
        painter.text(
            egui::pos2(px(x), plot.bottom() + 12.0),
            egui::Align2::CENTER_CENTER,
            l,
            egui::FontId::proportional(9.5),
            v.weak_text_color(),
        );
    }
    for (y, l) in [(2.0, "I5"), (3.0, "I6"), (5.0, "I7"), (6.0, "I8")] {
        painter.line_segment(
            [egui::pos2(plot.left() - 3.0, py(y)), egui::pos2(plot.left() + 3.0, py(y))],
            egui::Stroke::new(1.0, axis),
        );
        painter.text(
            egui::pos2(plot.left() - 12.0, py(y)),
            egui::Align2::CENTER_CENTER,
            l,
            egui::FontId::proportional(9.5),
            v.weak_text_color(),
        );
    }

    // Trajetória segura: anda em A, sobe em B e volta a andar em A, por fora.
    let path = [
        egui::pos2(px(0.0), py(0.0)),
        egui::pos2(px(1.5), py(0.0)),
        egui::pos2(px(1.5), py(7.0)),
        egui::pos2(px(8.0), py(7.0)),
    ];
    for w in path.windows(2) {
        painter.line_segment([w[0], w[1]], egui::Stroke::new(2.2, ACCENT));
    }
    painter.text(
        egui::pos2(px(5.5), py(7.4)),
        egui::Align2::CENTER_CENTER,
        "trajetória segura",
        egui::FontId::proportional(10.0),
        ACCENT,
    );
}

// ---------------------------------------------------------------------------
// Grafo de alocação de recursos: detecção de ciclo (seções 6.5.2 a 6.5.5)
//
// O grafo é montado pelo usuário. Cada cruzamento processo × recurso guarda a
// relação entre os dois: o processo detém o recurso (arco recurso ➡ processo),
// pede o recurso (arco processo ➡ recurso) ou nada. Como há um único recurso
// de cada tipo, um recurso tem no máximo um detentor.
// ---------------------------------------------------------------------------

/// Estados de uma relação processo × recurso.
const NADA: u8 = 0;
const DETEM: u8 = 1;
const PEDE: u8 = 2;

/// O maior número de processos ou de recursos que o grafo aceita.
const RAG_MAX: usize = 8;

/// Um cenário pronto: as dimensões e as relações não vazias, cada uma um trio
/// (processo, recurso, estado). Processos e recursos são numerados a partir de
/// 1 ao exibir (P1, R1, ...).
struct RagPreset {
    name: &'static str,
    n_proc: usize,
    n_res: usize,
    rels: &'static [(usize, usize, u8)],
}

const RAG_PRESETS: &[RagPreset] = &[
    // Sem impasse: P1 detém R1 e pede R2, que P2 detém sem esperar por nada.
    // P2 termina, devolve R2 e P1 segue. Nenhum ciclo.
    RagPreset {
        name: "Sem impasse",
        n_proc: 2,
        n_res: 2,
        rels: &[(0, 0, DETEM), (0, 1, PEDE), (1, 1, DETEM)],
    },
    // Impasse a dois: cada processo detém um recurso e pede o do outro.
    RagPreset {
        name: "Impasse a dois",
        n_proc: 2,
        n_res: 2,
        rels: &[(0, 0, DETEM), (0, 1, PEDE), (1, 1, DETEM), (1, 0, PEDE)],
    },
    // Espera circular a três: P1 ➡ R2 ➡ P2 ➡ R3 ➡ P3 ➡ R1 ➡ P1.
    RagPreset {
        name: "Espera circular a três",
        n_proc: 3,
        n_res: 3,
        rels: &[
            (0, 0, DETEM), (0, 1, PEDE),
            (1, 1, DETEM), (1, 2, PEDE),
            (2, 2, DETEM), (2, 0, PEDE),
        ],
    },
    // Exemplo do Tanenbaum (6.5.4): sete processos e seis recursos, com um
    // ciclo escondido (P4 ➡ R3 ➡ P5 ➡ R5 ➡ P7 ➡ R4 ➡ P4) entre processos que
    // não estão em impasse. Só P4, P5 e P7 travam.
    RagPreset {
        name: "Exemplo do livro",
        n_proc: 7,
        n_res: 6,
        rels: &[
            (0, 0, DETEM), (0, 1, PEDE),               // P1 detém R1, pede R2
            (1, 2, PEDE),                              // P2 pede R3
            (2, 1, PEDE),                              // P3 pede R2
            (3, 3, DETEM), (3, 1, PEDE), (3, 2, PEDE), // P4 detém R4, pede R2 e R3
            (4, 2, DETEM), (4, 4, PEDE),               // P5 detém R3, pede R5
            (5, 5, DETEM), (5, 1, PEDE),               // P6 detém R6, pede R2
            (6, 4, DETEM), (6, 3, PEDE),               // P7 detém R5, pede R4
        ],
    },
];

/// Acha um ciclo dirigido e devolve a sequência de nós que o forma.
fn find_cycle(n: usize, edges: &[(usize, usize)]) -> Option<Vec<usize>> {
    let mut adj = vec![Vec::new(); n];
    for &(from, to) in edges {
        adj[from].push(to);
    }
    let mut color = vec![0u8; n]; // 0 branco, 1 cinza, 2 preto
    let mut parent = vec![usize::MAX; n];
    for s in 0..n {
        if color[s] == 0 {
            if let Some((anc, u)) = dfs_cycle(s, &adj, &mut color, &mut parent) {
                let mut path = vec![u];
                let mut x = u;
                while x != anc {
                    x = parent[x];
                    path.push(x);
                }
                path.reverse();
                return Some(path);
            }
        }
    }
    None
}

fn dfs_cycle(
    u: usize,
    adj: &[Vec<usize>],
    color: &mut [u8],
    parent: &mut [usize],
) -> Option<(usize, usize)> {
    color[u] = 1;
    for &w in &adj[u] {
        if color[w] == 0 {
            parent[w] = u;
            if let Some(e) = dfs_cycle(w, adj, color, parent) {
                return Some(e);
            }
        } else if color[w] == 1 {
            // Arco de retorno: w é ancestral na pilha, u o fecha.
            return Some((w, u));
        }
    }
    color[u] = 2;
    None
}

pub struct RagState {
    n_proc: usize,
    n_res: usize,
    /// `cells[p][r]` guarda a relação do processo p com o recurso r
    /// (`NADA`, `DETEM` ou `PEDE`).
    cells: Vec<Vec<u8>>,
    /// O cenário pronto em vigor, enquanto o usuário não edita nada.
    preset: Option<usize>,
    /// Quantos nós do ciclo já foram revelados (a lista L crescendo).
    reveal: usize,
    playing: bool,
    speed: f32,
    clock: f32,
}

impl Default for RagState {
    fn default() -> Self {
        let mut s = Self {
            n_proc: 0,
            n_res: 0,
            cells: Vec::new(),
            preset: None,
            reveal: 0,
            playing: false,
            speed: 1.3,
            clock: 0.0,
        };
        s.load_preset(1); // abre no impasse a dois, que já mostra um ciclo
        s
    }
}

impl RagState {
    /// Ajusta a matriz às dimensões atuais, preenchendo com `NADA` o que cresce
    /// e descartando o que encolhe.
    fn resize(&mut self) {
        self.cells.resize(self.n_proc, Vec::new());
        for row in &mut self.cells {
            row.resize(self.n_res, NADA);
        }
    }

    fn load_preset(&mut self, i: usize) {
        let p = &RAG_PRESETS[i];
        self.n_proc = p.n_proc;
        self.n_res = p.n_res;
        self.cells = vec![vec![NADA; p.n_res]; p.n_proc];
        for &(pr, rs, state) in p.rels {
            self.cells[pr][rs] = state;
        }
        self.preset = Some(i);
        self.rewind();
    }

    fn clear(&mut self) {
        for row in &mut self.cells {
            for c in row.iter_mut() {
                *c = NADA;
            }
        }
        self.preset = None;
        self.rewind();
    }

    fn rewind(&mut self) {
        self.reveal = 0;
        self.playing = false;
        self.clock = 0.0;
    }

    /// Alterna a célula entre `NADA`, `DETEM` e `PEDE`. Ao passar a `DETEM`,
    /// tira a posse do recurso de qualquer outro processo: há um recurso só.
    fn cycle_cell(&mut self, p: usize, r: usize) {
        let next = (self.cells[p][r] + 1) % 3;
        self.cells[p][r] = next;
        if next == DETEM {
            for q in 0..self.n_proc {
                if q != p && self.cells[q][r] == DETEM {
                    self.cells[q][r] = NADA;
                }
            }
        }
        self.preset = None;
        self.rewind();
    }

    /// Arcos dirigidos (origem, destino) por índice de nó: os processos ocupam
    /// 0..n_proc e os recursos n_proc..n_proc+n_res.
    fn edges(&self) -> Vec<(usize, usize)> {
        let mut e = Vec::new();
        for p in 0..self.n_proc {
            for r in 0..self.n_res {
                match self.cells[p][r] {
                    DETEM => e.push((self.n_proc + r, p)),
                    PEDE => e.push((p, self.n_proc + r)),
                    _ => {}
                }
            }
        }
        e
    }

    fn is_proc(&self, i: usize) -> bool {
        i < self.n_proc
    }

    fn node_label(&self, i: usize) -> String {
        if i < self.n_proc {
            format!("P{}", i + 1)
        } else {
            format!("R{}", i - self.n_proc + 1)
        }
    }
}

pub fn resource_graph(ui: &mut egui::Ui, st: &mut RagState) {
    widget_frame(ui).show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        widget_header(
            ui,
            "Grafo de alocação",
            "Monte o cenário e procure um ciclo. Círculos são processos, quadrados são recursos. \
             Recurso ➡ processo é posse, processo ➡ recurso é espera.",
        );

        // Cenários prontos e o botão de esvaziar.
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("Exemplos:").size(12.5).color(egui::Color32::GRAY));
            for (i, p) in RAG_PRESETS.iter().enumerate() {
                if ui.selectable_label(st.preset == Some(i), p.name).clicked() {
                    st.load_preset(i);
                }
            }
            if ui.button("limpar").on_hover_text("Esvaziar o grafo").clicked() {
                st.clear();
            }
        });
        ui.add_space(8.0);

        // Quantos processos e recursos existem.
        let mut changed = false;
        ui.horizontal_wrapped(|ui| {
            changed |= count_stepper(ui, "Processos", &mut st.n_proc);
            ui.add_space(18.0);
            changed |= count_stepper(ui, "Recursos", &mut st.n_res);
        });
        if changed {
            st.resize();
            st.preset = None;
            st.rewind();
        }
        ui.add_space(8.0);

        // A matriz de relações: um clique percorre os três estados da célula.
        relation_grid(ui, st);
        ui.add_space(8.0);

        // Só agora, com as dimensões e a matriz definitivas, procuramos o ciclo.
        let n = st.n_proc + st.n_res;
        let cycle = find_cycle(n, &st.edges());
        let total = cycle.as_ref().map_or(0, |c| c.len());
        st.reveal = st.reveal.min(total);

        if st.playing {
            if cycle.is_some() {
                st.clock += ui.input(|i| i.stable_dt).min(0.1) * st.speed;
                while st.clock >= 1.0 {
                    st.clock -= 1.0;
                    if st.reveal < total {
                        st.reveal += 1;
                    }
                    if st.reveal >= total {
                        st.playing = false;
                    }
                }
                ui.ctx().request_repaint();
            } else {
                st.playing = false;
            }
        }

        ui.horizontal_wrapped(|ui| {
            let enabled = cycle.is_some();
            let label = if st.playing { "⏸" } else { "▶" };
            if ui
                .add_enabled(enabled, egui::Button::new(label))
                .on_hover_text("Percorrer o grafo, construindo a lista L")
                .clicked()
            {
                if st.playing {
                    st.playing = false;
                } else {
                    if st.reveal >= total {
                        st.reveal = 0;
                    }
                    st.clock = 0.0;
                    st.playing = true;
                }
            }
            if ui
                .add_enabled(enabled && st.reveal < total, egui::Button::new("▶|"))
                .on_hover_text("Próximo nó")
                .clicked()
            {
                st.reveal += 1;
                st.playing = false;
            }
            if ui
                .add_enabled(enabled, egui::Button::new("⏭"))
                .on_hover_text("Fechar o ciclo")
                .clicked()
            {
                st.reveal = total;
                st.playing = false;
            }
            if ui.button("↺").on_hover_text("Reiniciar a lista L").clicked() {
                st.reveal = 0;
                st.playing = false;
            }
            ui.add(
                egui::Slider::new(&mut st.speed, 0.4..=4.0)
                    .show_value(false)
                    .text("velocidade"),
            );
        });
        ui.add_space(8.0);

        rag_canvas(ui, st, cycle.as_deref());
        ui.add_space(8.0);

        rag_result(ui, st, cycle.as_deref());
    });
}

/// Um botão de menos, a contagem e um botão de mais. Devolve `true` se mudou.
fn count_stepper(ui: &mut egui::Ui, label: &str, n: &mut usize) -> bool {
    let mut changed = false;
    ui.label(egui::RichText::new(label).size(12.5).strong().color(ACCENT));
    if ui
        .add_enabled(*n > 1, egui::Button::new("−").min_size(egui::vec2(24.0, 0.0)))
        .clicked()
    {
        *n -= 1;
        changed = true;
    }
    ui.label(egui::RichText::new(n.to_string()).size(14.0).monospace());
    if ui
        .add_enabled(*n < RAG_MAX, egui::Button::new("+").min_size(egui::vec2(24.0, 0.0)))
        .clicked()
    {
        *n += 1;
        changed = true;
    }
    changed
}

/// A matriz processo × recurso de botões que alternam a relação.
fn relation_grid(ui: &mut egui::Ui, st: &mut RagState) {
    let weak = ui.visuals().weak_text_color();
    // Guardamos o clique e só mexemos na matriz depois do laço, para não a
    // alterar enquanto a percorremos.
    let mut toggled: Option<(usize, usize)> = None;
    egui::Grid::new("rag_cells")
        .num_columns(st.n_res + 1)
        .spacing([6.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label("");
            for r in 0..st.n_res {
                ui.label(
                    egui::RichText::new(format!("R{}", r + 1))
                        .size(11.5)
                        .strong()
                        .color(SAFE),
                );
            }
            ui.end_row();
            for p in 0..st.n_proc {
                ui.label(
                    egui::RichText::new(format!("P{}", p + 1))
                        .size(11.5)
                        .strong()
                        .color(ACCENT),
                );
                for r in 0..st.n_res {
                    let (txt, color, fill) = match st.cells[p][r] {
                        DETEM => ("detém", SAFE, SAFE.gamma_multiply(0.16)),
                        PEDE => ("pede", ACCENT, ACCENT.gamma_multiply(0.16)),
                        _ => ("·", weak, egui::Color32::TRANSPARENT),
                    };
                    let btn = egui::Button::new(egui::RichText::new(txt).size(11.0).color(color))
                        .min_size(egui::vec2(50.0, 20.0))
                        .fill(fill);
                    if ui.add(btn).clicked() {
                        toggled = Some((p, r));
                    }
                }
                ui.end_row();
            }
        });
    if let Some((p, r)) = toggled {
        st.cycle_cell(p, r);
    }

    ui.add_space(4.0);
    ui.horizontal_wrapped(|ui| {
        let gray = egui::Color32::GRAY;
        ui.label(egui::RichText::new("Em cada cruzamento, um clique percorre").size(11.5).color(gray));
        ui.label(egui::RichText::new("detém").size(11.5).strong().color(SAFE));
        ui.label(egui::RichText::new("(recurso ➡ processo),").size(11.5).color(gray));
        ui.label(egui::RichText::new("pede").size(11.5).strong().color(ACCENT));
        ui.label(egui::RichText::new("(processo ➡ recurso) e vazio.").size(11.5).color(gray));
    });
}

fn rag_canvas(ui: &mut egui::Ui, st: &RagState, cycle: Option<&[usize]>) {
    let n = st.n_proc + st.n_res;
    let rows = st.n_proc.max(st.n_res).max(1);
    let width = ui.available_width().min(520.0);
    let height = (rows as f32 * 60.0 + 36.0).clamp(150.0, 560.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let v = ui.visuals().clone();
    let txt = v.text_color();
    let reveal = st.reveal;

    // Processos na coluna da esquerda, recursos na da direita.
    let vpad = NODE_R + 18.0;
    let top = rect.top() + vpad;
    let bottom = rect.bottom() - vpad;
    let left_x = rect.left() + NODE_R + 38.0;
    let right_x = rect.right() - NODE_R - 38.0;
    let col_y = |i: usize, count: usize| -> f32 {
        if count <= 1 {
            (top + bottom) * 0.5
        } else {
            top + i as f32 / (count - 1) as f32 * (bottom - top)
        }
    };
    let pos = |node: usize| -> egui::Pos2 {
        if node < st.n_proc {
            egui::pos2(left_x, col_y(node, st.n_proc))
        } else {
            egui::pos2(right_x, col_y(node - st.n_proc, st.n_res))
        }
    };

    painter.text(
        egui::pos2(left_x, rect.top() + 9.0),
        egui::Align2::CENTER_CENTER,
        "processos",
        egui::FontId::proportional(10.5),
        v.weak_text_color(),
    );
    painter.text(
        egui::pos2(right_x, rect.top() + 9.0),
        egui::Align2::CENTER_CENTER,
        "recursos",
        egui::FontId::proportional(10.5),
        v.weak_text_color(),
    );

    // Um arco pertence ao ciclo revelado se liga dois nós consecutivos já
    // mostrados, ou é o arco que fecha o ciclo quando tudo foi revelado.
    let in_cycle_edge = |from: usize, to: usize| -> bool {
        let Some(cy) = cycle else { return false };
        let full = reveal >= cy.len();
        for w in 0..cy.len() {
            let a = cy[w];
            let b = cy[(w + 1) % cy.len()];
            if a == from && b == to {
                let closing = w + 1 == cy.len();
                return if closing { full } else { w + 1 < reveal };
            }
        }
        false
    };

    let base_line = txt.gamma_multiply(0.55);
    for (from, to) in st.edges() {
        let hot = in_cycle_edge(from, to);
        let color = if hot { DANGER } else { base_line };
        let (ca, cb) = (pos(from), pos(to));
        let dir = (cb - ca).normalized();
        // Da borda do nó de origem à borda do de destino, com uma folga curta
        // para a ponta não invadir o quadrado ou o círculo.
        let start = node_edge(ca, st.is_proc(from), dir) + dir * 2.0;
        let end = node_edge(cb, st.is_proc(to), -dir) - dir * 3.0;
        draw_arrow(&painter, start, end, color, if hot { 2.2 } else { 1.4 });
    }

    for i in 0..n {
        let in_cycle = cycle
            .map(|cy| cy.iter().take(reveal).any(|&c| c == i))
            .unwrap_or(false);
        let (fill, stroke) = if in_cycle {
            (DANGER.gamma_multiply(0.25), DANGER)
        } else if st.is_proc(i) {
            (ACCENT.gamma_multiply(0.20), ACCENT)
        } else {
            (SAFE.gamma_multiply(0.16), SAFE)
        };
        draw_node(&painter, pos(i), st.is_proc(i), &st.node_label(i), fill, stroke, txt);
    }
}

fn rag_result(ui: &mut egui::Ui, st: &RagState, cycle: Option<&[usize]>) {
    match cycle {
        None => callout(
            ui,
            SAFE,
            "✔",
            "O grafo não tem ciclo: nenhum processo espera em círculo, logo não há impasse."
                .to_string(),
        ),
        Some(cy) => {
            let reveal = st.reveal;
            let full = reveal >= cy.len();
            let (color, icon, msg) = if full {
                let loop_labels: Vec<String> = cy.iter().map(|&i| st.node_label(i)).collect();
                (
                    DANGER,
                    "✖",
                    format!(
                        "L volta a {}: um nó reapareceu na lista. O ciclo {} ➡ {} fecha sobre si \
                         mesmo e denuncia o impasse.",
                        st.node_label(cy[0]),
                        loop_labels.join(" ➡ "),
                        st.node_label(cy[0])
                    ),
                )
            } else if reveal == 0 {
                (
                    ACCENT,
                    "➡",
                    "O grafo tem um ciclo. Toque ▶ para percorrer a lista L nó a nó e revelá-lo."
                        .to_string(),
                )
            } else {
                let labels: Vec<String> =
                    cy.iter().take(reveal).map(|&i| st.node_label(i)).collect();
                (
                    ACCENT,
                    "➡",
                    format!(
                        "Construindo a lista L nó a nó, seguindo os arcos de saída. L = {}.",
                        labels.join(", ")
                    ),
                )
            };
            callout(ui, color, icon, msg);
        }
    }
}

// ---------------------------------------------------------------------------
// Busca comum: marcar os processos que conseguem terminar
// ---------------------------------------------------------------------------

fn le(a: &[u32], b: &[u32]) -> bool {
    a.iter().zip(b).all(|(x, y)| x <= y)
}

fn colsum(m: &[Vec<u32>], cols: usize) -> Vec<u32> {
    let mut s = vec![0u32; cols];
    for row in m {
        for j in 0..cols {
            s[j] += row.get(j).copied().unwrap_or(0);
        }
    }
    s
}

/// Um passo da marcação: o vetor de trabalho antes, o processo escolhido (se
/// houver) e o vetor depois de devolver a sua alocação.
#[derive(Clone)]
struct MarkStep {
    work_before: Vec<u32>,
    picked: Option<usize>,
    work_after: Vec<u32>,
}

/// Roda a busca e devolve os passos e o vetor final de processos marcados.
fn safety_trace(demand: &[Vec<u32>], alloc: &[Vec<u32>], work0: &[u32]) -> (Vec<MarkStep>, Vec<bool>) {
    let n = demand.len();
    let mut work = work0.to_vec();
    let mut marked = vec![false; n];
    let mut steps = Vec::new();
    loop {
        let pick = (0..n).find(|&i| !marked[i] && le(&demand[i], &work));
        match pick {
            Some(i) => {
                let before = work.clone();
                for (j, w) in work.iter_mut().enumerate() {
                    *w += alloc[i].get(j).copied().unwrap_or(0);
                }
                marked[i] = true;
                steps.push(MarkStep {
                    work_before: before,
                    picked: Some(i),
                    work_after: work.clone(),
                });
                if marked.iter().all(|&m| m) {
                    break;
                }
            }
            None => {
                steps.push(MarkStep {
                    work_before: work.clone(),
                    picked: None,
                    work_after: work.clone(),
                });
                break;
            }
        }
    }
    (steps, marked)
}

fn vec_str(v: &[u32]) -> String {
    let inner: Vec<String> = v.iter().map(|x| x.to_string()).collect();
    format!("({})", inner.join(" "))
}

fn callout(ui: &mut egui::Ui, color: egui::Color32, icon: &str, msg: String) {
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(10, 8))
        .corner_radius(7.0)
        .fill(color.gamma_multiply(0.12))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal_top(|ui| {
                ui.label(egui::RichText::new(icon).size(14.0).color(color));
                ui.add(egui::Label::new(egui::RichText::new(msg).size(12.5)).wrap());
            });
        });
}

/// Editor de uma matriz processo × classe. Devolve `true` se algo mudou.
fn matrix_editor(
    ui: &mut egui::Ui,
    id: &str,
    title: &str,
    procs: &[&str],
    classes: &[&str],
    data: &mut [Vec<u32>],
    max: u32,
) -> bool {
    let mut changed = false;
    ui.label(egui::RichText::new(title).size(12.0).strong().color(ACCENT));
    egui::Grid::new(id)
        .num_columns(classes.len() + 1)
        .spacing([8.0, 3.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label("");
            for c in classes {
                ui.label(egui::RichText::new(*c).size(11.0).strong());
            }
            ui.end_row();
            for (i, row) in data.iter_mut().enumerate() {
                ui.label(egui::RichText::new(procs[i]).size(11.5).monospace());
                for cell in row.iter_mut().take(classes.len()) {
                    let r = ui.add(egui::DragValue::new(cell).range(0..=max).speed(0.08));
                    changed |= r.changed();
                }
                ui.end_row();
            }
        });
    changed
}

fn vector_editor(
    ui: &mut egui::Ui,
    id: &str,
    title: &str,
    classes: &[&str],
    data: &mut [u32],
    max: u32,
) -> bool {
    let mut changed = false;
    egui::Grid::new(id)
        .num_columns(classes.len() + 1)
        .spacing([8.0, 3.0])
        .show(ui, |ui| {
            ui.label(egui::RichText::new(title).size(12.0).strong().color(ACCENT));
            for cell in data.iter_mut().take(classes.len()) {
                let r = ui.add(egui::DragValue::new(cell).range(0..=max).speed(0.08));
                changed |= r.changed();
            }
            ui.end_row();
        });
    changed
}

fn derived_vector(ui: &mut egui::Ui, title: &str, classes: &[&str], data: &[u32]) {
    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new(title).size(12.0).strong().color(egui::Color32::GRAY));
        egui::Grid::new(("deriv", title))
            .num_columns(classes.len())
            .spacing([8.0, 3.0])
            .show(ui, |ui| {
                for v in data.iter().take(classes.len()) {
                    ui.label(egui::RichText::new(v.to_string()).size(11.5).monospace());
                }
                ui.end_row();
            });
    });
}

// ---------------------------------------------------------------------------
// Detecção com vários recursos de cada tipo (seções 6.5.6 a 6.5.15)
// ---------------------------------------------------------------------------

const MAT_PROCS: &[&str] = &["P1", "P2", "P3"];
const MAT_CLASSES: &[&str] = &["gravador", "scanner", "fita", "disco"];

pub struct MatrixState {
    alloc: Vec<Vec<u32>>,
    request: Vec<Vec<u32>>,
    avail: Vec<u32>,
    cursor: Option<usize>,
    playing: bool,
    speed: f32,
    clock: f32,
}

impl Default for MatrixState {
    fn default() -> Self {
        Self {
            // Exemplo de 6.5.13.
            alloc: vec![
                vec![0, 0, 1, 0],
                vec![2, 0, 0, 1],
                vec![0, 1, 2, 0],
            ],
            request: vec![
                vec![2, 0, 0, 1],
                vec![1, 0, 1, 0],
                vec![2, 1, 0, 0],
            ],
            avail: vec![2, 1, 0, 0],
            cursor: None,
            playing: false,
            speed: 1.2,
            clock: 0.0,
        }
    }
}

impl MatrixState {
    fn reset_example(&mut self) {
        let def = MatrixState::default();
        self.alloc = def.alloc;
        self.request = def.request;
        self.avail = def.avail;
        self.cursor = None;
        self.playing = false;
    }
}

pub fn matrix_detect(ui: &mut egui::Ui, st: &mut MatrixState) {
    let (steps, marked) = safety_trace(&st.request, &st.alloc, &st.avail);
    let n_steps = steps.len();
    if let Some(c) = st.cursor {
        if c + 1 >= n_steps {
            st.cursor = None;
        }
    }

    if st.playing {
        st.clock += ui.input(|i| i.stable_dt).min(0.1) * st.speed;
        while st.clock >= 1.0 {
            st.clock -= 1.0;
            let next = st.cursor.map_or(0, |c| c + 1);
            if next + 1 >= n_steps {
                st.cursor = None;
                st.playing = false;
            } else {
                st.cursor = Some(next);
            }
        }
        ui.ctx().request_repaint();
    }
    let shown = st.cursor.map_or(n_steps, |c| c + 1);

    widget_frame(ui).show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        widget_header(
            ui,
            "Detecção por matrizes",
            "Edite a alocação C, a requisição R e o disponível A. O total E é a soma por coluna de \
             C com A.",
        );

        let e: Vec<u32> = colsum(&st.alloc, MAT_CLASSES.len())
            .iter()
            .zip(&st.avail)
            .map(|(c, a)| c + a)
            .collect();

        let mut changed = false;
        ui.columns(2, |cols| {
            changed |= matrix_editor(
                &mut cols[0],
                "mat_c",
                "Alocação C",
                MAT_PROCS,
                MAT_CLASSES,
                &mut st.alloc,
                9,
            );
            changed |= matrix_editor(
                &mut cols[1],
                "mat_r",
                "Requisição R",
                MAT_PROCS,
                MAT_CLASSES,
                &mut st.request,
                9,
            );
        });
        ui.add_space(6.0);
        changed |= vector_editor(ui, "mat_a", "Disponível A", MAT_CLASSES, &mut st.avail, 9);
        derived_vector(ui, "Existente E =", MAT_CLASSES, &e);
        if changed {
            st.cursor = None;
            st.playing = false;
        }
        ui.add_space(6.0);

        step_controls(ui, &mut st.cursor, &mut st.playing, &mut st.clock, &mut st.speed, n_steps);
        if ui.small_button("↺ exemplo").on_hover_text("Restaurar o exemplo das anotações").clicked() {
            st.reset_example();
        }
        ui.add_space(8.0);

        trace_lines(ui, &steps, MAT_PROCS, shown, "R");

        ui.add_space(6.0);
        let deadlocked: Vec<&str> = (0..MAT_PROCS.len())
            .filter(|&i| !marked[i])
            .map(|i| MAT_PROCS[i])
            .collect();
        if deadlocked.is_empty() {
            callout(
                ui,
                SAFE,
                "✔",
                "Todos os processos ficaram marcados: cada um consegue terminar e devolver os \
                 recursos. Não há impasse."
                    .to_string(),
            );
        } else {
            callout(
                ui,
                DANGER,
                "✖",
                format!(
                    "O algoritmo parou com processos não marcados: {} está em impasse.",
                    deadlocked.join(", ")
                ),
            );
        }
    });
}

// ---------------------------------------------------------------------------
// Algoritmo do banqueiro (seções 6.6.7 a 6.6.11)
// ---------------------------------------------------------------------------

struct BankerPreset {
    name: &'static str,
    procs: &'static [&'static str],
    classes: &'static [&'static str],
    total: &'static [u32],
    max: &'static [&'static [u32]],
    alloc: &'static [&'static [u32]],
}

const BANKER_PRESETS: &[BankerPreset] = &[
    BankerPreset {
        name: "Um recurso",
        procs: &["A", "B", "C", "D"],
        classes: &["crédito"],
        total: &[10],
        max: &[&[6], &[5], &[4], &[7]],
        alloc: &[&[1], &[1], &[2], &[4]],
    },
    BankerPreset {
        name: "Vários recursos",
        procs: &["P0", "P1", "P2"],
        classes: &["A", "B", "C"],
        total: &[5, 4, 5],
        max: &[&[3, 3, 2], &[1, 2, 3], &[3, 2, 2]],
        alloc: &[&[1, 1, 2], &[1, 0, 0], &[1, 2, 1]],
    },
];

pub struct BankerState {
    preset: usize,
    total: Vec<u32>,
    max: Vec<Vec<u32>>,
    alloc: Vec<Vec<u32>>,
    cursor: Option<usize>,
    playing: bool,
    speed: f32,
    clock: f32,
}

impl Default for BankerState {
    fn default() -> Self {
        let mut s = Self {
            preset: 0,
            total: Vec::new(),
            max: Vec::new(),
            alloc: Vec::new(),
            cursor: None,
            playing: false,
            speed: 1.2,
            clock: 0.0,
        };
        s.load(0);
        s
    }
}

impl BankerState {
    fn load(&mut self, preset: usize) {
        let p = &BANKER_PRESETS[preset];
        self.preset = preset;
        self.total = p.total.to_vec();
        self.max = p.max.iter().map(|r| r.to_vec()).collect();
        self.alloc = p.alloc.iter().map(|r| r.to_vec()).collect();
        self.cursor = None;
        self.playing = false;
    }
}

pub fn banker(ui: &mut egui::Ui, st: &mut BankerState) {
    widget_frame(ui).show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        widget_header(
            ui,
            "Algoritmo do banqueiro",
            "Edite o crédito máximo e o já alocado. A necessidade é o máximo menos o alocado; o \
             disponível é o total menos o que está alocado.",
        );

        // O preset é escolhido antes de qualquer cálculo: trocá-lo recarrega as
        // matrizes com novas dimensões, e o resto do quadro já usa as novas.
        ui.horizontal_wrapped(|ui| {
            for (i, pr) in BANKER_PRESETS.iter().enumerate() {
                if ui.selectable_label(st.preset == i, pr.name).clicked() && st.preset != i {
                    st.load(i);
                }
            }
        });
        ui.add_space(6.0);

        let p = &BANKER_PRESETS[st.preset];
        let m = p.classes.len();
        let avail: Vec<u32> = st
            .total
            .iter()
            .zip(colsum(&st.alloc, m))
            .map(|(t, c)| t.saturating_sub(c))
            .collect();
        let need: Vec<Vec<u32>> = st
            .max
            .iter()
            .zip(&st.alloc)
            .map(|(mx, al)| mx.iter().zip(al).map(|(x, a)| x.saturating_sub(*a)).collect())
            .collect();

        let (steps, marked) = safety_trace(&need, &st.alloc, &avail);
        let n_steps = steps.len();
        if let Some(c) = st.cursor {
            if c + 1 >= n_steps {
                st.cursor = None;
            }
        }
        if st.playing {
            st.clock += ui.input(|i| i.stable_dt).min(0.1) * st.speed;
            while st.clock >= 1.0 {
                st.clock -= 1.0;
                let next = st.cursor.map_or(0, |c| c + 1);
                if next + 1 >= n_steps {
                    st.cursor = None;
                    st.playing = false;
                } else {
                    st.cursor = Some(next);
                }
            }
            ui.ctx().request_repaint();
        }
        let shown = st.cursor.map_or(n_steps, |c| c + 1);

        let mut changed = false;
        ui.columns(2, |cols| {
            changed |= matrix_editor(
                &mut cols[0],
                "bank_max",
                "Máximo",
                p.procs,
                p.classes,
                &mut st.max,
                20,
            );
            changed |= matrix_editor(
                &mut cols[1],
                "bank_alloc",
                "Alocado",
                p.procs,
                p.classes,
                &mut st.alloc,
                20,
            );
        });
        ui.add_space(6.0);
        changed |= vector_editor(ui, "bank_total", "Total", p.classes, &mut st.total, 40);
        derived_vector(ui, "Disponível =", p.classes, &avail);
        if changed {
            st.cursor = None;
            st.playing = false;
        }
        ui.add_space(6.0);

        step_controls(ui, &mut st.cursor, &mut st.playing, &mut st.clock, &mut st.speed, n_steps);
        ui.add_space(8.0);

        trace_lines(ui, &steps, p.procs, shown, "necessidade");

        ui.add_space(6.0);
        if marked.iter().all(|&x| x) {
            let order: Vec<&str> = steps
                .iter()
                .filter_map(|s| s.picked)
                .map(|i| p.procs[i])
                .collect();
            callout(
                ui,
                SAFE,
                "✔",
                format!(
                    "Estado seguro: existe uma ordem que leva todos ao fim. Sequência: {}.",
                    order.join(" ➡ ")
                ),
            );
        } else {
            let left: Vec<&str> = (0..p.procs.len())
                .filter(|&i| !marked[i])
                .map(|i| p.procs[i])
                .collect();
            callout(
                ui,
                DANGER,
                "✖",
                format!(
                    "Estado inseguro: nenhuma ordem conclui {}. O banqueiro recusaria um pedido que \
                     levasse a este estado.",
                    left.join(", ")
                ),
            );
        }
    });
}

// ---------------------------------------------------------------------------
// Controles e rastro compartilhados pelos dois simuladores de matriz
// ---------------------------------------------------------------------------

fn step_controls(
    ui: &mut egui::Ui,
    cursor: &mut Option<usize>,
    playing: &mut bool,
    clock: &mut f32,
    speed: &mut f32,
    n_steps: usize,
) {
    ui.horizontal_wrapped(|ui| {
        let label = if *playing { "⏸" } else { "▶" };
        if ui
            .button(label)
            .on_hover_text("Revelar a marcação passo a passo")
            .clicked()
        {
            if *playing {
                *playing = false;
            } else {
                if cursor.is_none() {
                    *cursor = Some(0);
                }
                *clock = 0.0;
                *playing = true;
            }
        }
        let cur = cursor.unwrap_or(n_steps.saturating_sub(1));
        if ui
            .add_enabled(cur > 0, egui::Button::new("◀"))
            .on_hover_text("Passo anterior")
            .clicked()
        {
            *cursor = Some(cur - 1);
            *playing = false;
        }
        if ui
            .add_enabled(cur + 1 < n_steps, egui::Button::new("▶|"))
            .on_hover_text("Próximo passo")
            .clicked()
        {
            *cursor = Some(cur + 1);
            *playing = false;
        }
        if ui.button("⏭").on_hover_text("Mostrar tudo").clicked() {
            *cursor = None;
            *playing = false;
        }
        ui.add(
            egui::Slider::new(speed, 0.4..=4.0)
                .show_value(false)
                .text("velocidade"),
        );
    });
}

fn trace_lines(ui: &mut egui::Ui, steps: &[MarkStep], procs: &[&str], shown: usize, demand_name: &str) {
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(10, 8))
        .corner_radius(7.0)
        .fill(ui.visuals().extreme_bg_color)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 4.0;
            for step in steps.iter().take(shown) {
                let line = match step.picked {
                    Some(i) => format!(
                        "trabalho = {}. A {} de {} cabe: marca {} e soma a alocação, trabalho passa a {}.",
                        vec_str(&step.work_before),
                        demand_name,
                        procs[i],
                        procs[i],
                        vec_str(&step.work_after),
                    ),
                    None => format!(
                        "trabalho = {}. Nenhum processo não marcado tem {} menor ou igual ao trabalho: o algoritmo para.",
                        vec_str(&step.work_before),
                        demand_name,
                    ),
                };
                ui.horizontal_top(|ui| {
                    let icon = if step.picked.is_some() { "✔" } else { "✖" };
                    let color = if step.picked.is_some() { SAFE } else { DANGER };
                    ui.label(egui::RichText::new(icon).size(12.5).color(color));
                    ui.add(egui::Label::new(egui::RichText::new(line).size(12.0)).wrap());
                });
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exemplo_sem_impasse() {
        let st = MatrixState::default();
        let (_steps, marked) = safety_trace(&st.request, &st.alloc, &st.avail);
        assert!(marked.iter().all(|&m| m), "o exemplo de 6.5.13 não tem impasse");
    }

    #[test]
    fn impasse_quando_ninguem_cabe() {
        // Dois processos, um recurso, cada um pede mais do que há livre.
        let alloc = vec![vec![1], vec![1]];
        let request = vec![vec![1], vec![1]];
        let avail = vec![0];
        let (_s, marked) = safety_trace(&request, &alloc, &avail);
        assert!(marked.iter().all(|&m| !m), "ambos deveriam ficar em impasse");
    }

    #[test]
    fn banqueiro_um_recurso_e_seguro() {
        let mut st = BankerState::default();
        st.load(0);
        let m = 1;
        let avail: Vec<u32> = st
            .total
            .iter()
            .zip(colsum(&st.alloc, m))
            .map(|(t, c)| t - c)
            .collect();
        let need: Vec<Vec<u32>> = st
            .max
            .iter()
            .zip(&st.alloc)
            .map(|(mx, al)| mx.iter().zip(al).map(|(x, a)| x - a).collect())
            .collect();
        let (_s, marked) = safety_trace(&need, &st.alloc, &avail);
        assert!(marked.iter().all(|&x| x), "o estado de 6.6.9 é seguro");
    }

    #[test]
    fn presets_rag_tem_ciclo_quando_deve() {
        for (i, p) in RAG_PRESETS.iter().enumerate() {
            let mut st = RagState::default();
            st.load_preset(i);
            let cycle = find_cycle(st.n_proc + st.n_res, &st.edges());
            if p.name == "Sem impasse" {
                assert!(cycle.is_none(), "'{}' não deveria ter ciclo", p.name);
            } else {
                assert!(cycle.is_some(), "'{}' deveria ter um ciclo", p.name);
            }
        }
    }

    #[test]
    fn exemplo_do_livro_tem_ciclo_de_seis() {
        let mut st = RagState::default();
        let idx = RAG_PRESETS
            .iter()
            .position(|p| p.name == "Exemplo do livro")
            .unwrap();
        st.load_preset(idx);
        let cy = find_cycle(st.n_proc + st.n_res, &st.edges()).expect("o exemplo tem impasse");
        assert_eq!(cy.len(), 6, "o ciclo do exemplo do livro passa por seis nós");
    }

    #[test]
    fn rag_um_detentor_por_recurso() {
        let mut st = RagState::default();
        st.load_preset(1); // impasse a dois: P1 detém R1
        assert_eq!(st.cells[0][0], DETEM);
        // P2 passa a deter R1: P1 perde a posse, pois há um recurso só.
        st.cells[1][0] = NADA;
        st.cycle_cell(1, 0);
        assert_eq!(st.cells[1][0], DETEM);
        assert_eq!(st.cells[0][0], NADA, "um recurso tem um único detentor");
    }
}
