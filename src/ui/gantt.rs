//! Simulador de escalonamento com diagrama de Gantt.
//!
//! Uma única simulação, discreta em unidades de tempo, atende a todas as
//! políticas da seção 2.5: a cada instante decidimos quem roda e registramos a
//! unidade. As fatias do Gantt saem da fusão de unidades consecutivas do mesmo
//! processo, e as métricas (tempo de espera e de retorno médios) saem dos
//! instantes de término.
//!
//! O usuário pode editar chegada, duração, prioridade e bilhetes e ver o
//! resultado recalcular na hora.

use std::collections::VecDeque;

use eframe::egui;

use crate::ui::ACCENT;

/// Níveis de prioridade das múltiplas filas.
const MLFQ_LEVELS: usize = 4;

/// Cores por processo.
///
/// Ordem categórica documentada da paleta de referência (azul, laranja, aqua,
/// magenta, verde, violeta), com um par claro/escuro por posição. O amarelo foi
/// deliberadamente pulado: a paleta o marca como indistinguível do laranja
/// quando os dois aparecem juntos. Cada linha do Gantt também leva o nome do
/// processo à esquerda, então a identidade nunca depende só da cor.
const SERIES: [(egui::Color32, egui::Color32); 6] = [
    (rgb(0x2a, 0x78, 0xd6), rgb(0x39, 0x87, 0xe5)),
    (rgb(0xeb, 0x68, 0x34), rgb(0xd9, 0x59, 0x26)),
    (rgb(0x1b, 0xaf, 0x7a), rgb(0x19, 0x9e, 0x70)),
    (rgb(0xe8, 0x7b, 0xa4), rgb(0xd5, 0x51, 0x81)),
    (rgb(0x00, 0x83, 0x00), rgb(0x00, 0x83, 0x00)),
    (rgb(0x4a, 0x3a, 0xa7), rgb(0x90, 0x85, 0xe9)),
];

const fn rgb(r: u8, g: u8, b: u8) -> egui::Color32 {
    egui::Color32::from_rgb(r, g, b)
}

pub fn series_color(i: usize, dark: bool) -> egui::Color32 {
    let (light, dk) = SERIES[i % SERIES.len()];
    if dark { dk } else { light }
}

/// As políticas de escalonamento cobertas pelas anotações.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Policy {
    Fcfs,
    Sjf,
    Srtf,
    RoundRobin,
    Priority,
    MultiQueue,
    Lottery,
}

impl Policy {
    pub fn uses_quantum(self) -> bool {
        matches!(self, Policy::RoundRobin | Policy::MultiQueue)
    }
    pub fn uses_priority(self) -> bool {
        matches!(self, Policy::Priority)
    }
    pub fn uses_tickets(self) -> bool {
        matches!(self, Policy::Lottery)
    }
    /// Preempta um processo em execução?
    pub fn preemptive(self) -> bool {
        !matches!(self, Policy::Fcfs | Policy::Sjf)
    }
}

/// Um processo do cenário.
#[derive(Clone, Copy)]
pub struct Job {
    pub name: &'static str,
    pub arrival: u32,
    pub burst: u32,
    /// Maior valor = mais prioritário.
    pub priority: u32,
    pub tickets: u32,
}

/// Um cenário pronto, embutido num artigo.
pub struct Scenario {
    /// Identificador estável — é a chave do estado editável.
    pub id: &'static str,
    pub policy: Policy,
    pub jobs: &'static [Job],
    pub quantum: u32,
    /// Frase curta sobre o que observar neste cenário.
    pub hint: &'static str,
}

/// Estado editável de um cenário, mais a reprodução da linha do tempo.
pub struct SchedState {
    jobs: Vec<Job>,
    quantum: u32,
    /// Instante até onde o Gantt está revelado. `None` = mostra tudo.
    cursor: Option<f32>,
    playing: bool,
    /// Unidades de tempo por segundo.
    speed: f32,
}

impl SchedState {
    pub fn new(s: &Scenario) -> Self {
        Self {
            jobs: s.jobs.to_vec(),
            quantum: s.quantum.max(1),
            cursor: None,
            playing: false,
            speed: 2.5,
        }
    }
    fn reset(&mut self, s: &Scenario) {
        self.jobs = s.jobs.to_vec();
        self.quantum = s.quantum.max(1);
        self.cursor = None;
        self.playing = false;
    }
}

/// Uma faixa contínua de execução de um processo.
pub struct Slice {
    pub job: usize,
    pub start: u32,
    pub end: u32,
}

#[derive(Default)]
pub struct Sim {
    pub slices: Vec<Slice>,
    pub finish: Vec<u32>,
    pub avg_wait: f32,
    pub avg_turnaround: f32,
    pub end: u32,
}

/// Gerador congruente linear: sorteios da loteria reprodutíveis entre execuções.
struct Lcg(u64);

impl Lcg {
    fn next_u32(&mut self, bound: u32) -> u32 {
        if bound == 0 {
            return 0;
        }
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.0 >> 33) % u64::from(bound)) as u32
    }
}

/// Roda a simulação, unidade de tempo a unidade de tempo.
pub fn simulate(jobs: &[Job], policy: Policy, quantum: u32) -> Sim {
    let n = jobs.len();
    let mut sim = Sim::default();
    if n == 0 {
        return sim;
    }
    let quantum = quantum.max(1);

    let mut remaining: Vec<u32> = jobs.iter().map(|j| j.burst.max(1)).collect();
    let mut finish = vec![0u32; n];
    let mut level = vec![0usize; n];
    let mut queues: Vec<VecDeque<usize>> = vec![VecDeque::new(); MLFQ_LEVELS];
    let mut current: Option<usize> = None;
    let mut quantum_left = 0u32;
    let mut rng = Lcg(0x5EED_1234_ABCD_0001);

    let total: u32 = remaining.iter().sum();
    let horizon = jobs.iter().map(|j| j.arrival).max().unwrap_or(0) + total + 1;

    let mut units: Vec<Option<usize>> = Vec::new();
    let mut done = 0usize;
    let mut t = 0u32;
    let queued = policy.uses_quantum();

    while done < n && t < horizon {
        // Chegadas deste instante entram na fila de nível mais alto.
        for (i, j) in jobs.iter().enumerate() {
            if j.arrival == t {
                level[i] = 0;
                queues[0].push_back(i);
            }
        }

        let ready: Vec<usize> = (0..n)
            .filter(|&i| jobs[i].arrival <= t && remaining[i] > 0)
            .collect();
        if ready.is_empty() {
            units.push(None);
            current = None;
            t += 1;
            continue;
        }

        // ---- quem roda nesta unidade ----
        let pick = if queued {
            let levels = if policy == Policy::MultiQueue {
                MLFQ_LEVELS
            } else {
                1
            };
            let needs_new = match current {
                Some(c) => remaining[c] == 0 || quantum_left == 0,
                None => true,
            };
            if needs_new {
                let lvl = (0..levels).find(|&l| !queues[l].is_empty());
                match lvl.and_then(|l| queues[l].front().copied()) {
                    Some(j) => {
                        current = Some(j);
                        // Cada nível abaixo dobra o quantum (2.5.3, múltiplas filas).
                        quantum_left = quantum << level[j].min(6);
                    }
                    // Não deveria ocorrer: `ready` não está vazio.
                    None => current = Some(ready[0]),
                }
            }
            current.unwrap_or(ready[0])
        } else {
            let keep = (!policy.preemptive())
                .then(|| current.filter(|&c| remaining[c] > 0))
                .flatten();
            match keep {
                Some(c) => c,
                None => choose(policy, &ready, jobs, &remaining, &mut rng),
            }
        };
        current = Some(pick);

        // ---- roda uma unidade ----
        remaining[pick] -= 1;
        units.push(Some(pick));
        if queued {
            quantum_left = quantum_left.saturating_sub(1);
        }

        if remaining[pick] == 0 {
            done += 1;
            finish[pick] = t + 1;
            if queued {
                remove_from_queues(&mut queues, pick);
            }
            current = None;
        } else if queued && quantum_left == 0 {
            // Consumiu o quantum: vai para o fim da fila; nas múltiplas filas,
            // ainda desce um nível.
            remove_from_queues(&mut queues, pick);
            if policy == Policy::MultiQueue {
                level[pick] = (level[pick] + 1).min(MLFQ_LEVELS - 1);
            }
            queues[level[pick]].push_back(pick);
            current = None;
        }

        t += 1;
    }

    // ---- fatias e métricas ----
    sim.slices = merge(&units);
    sim.end = units.len() as u32;
    let mut wait = 0f32;
    let mut turn = 0f32;
    for (i, j) in jobs.iter().enumerate() {
        let turnaround = finish[i].saturating_sub(j.arrival);
        turn += turnaround as f32;
        wait += turnaround.saturating_sub(j.burst.max(1)) as f32;
    }
    sim.finish = finish;
    sim.avg_wait = wait / n as f32;
    sim.avg_turnaround = turn / n as f32;
    sim
}

fn remove_from_queues(queues: &mut [VecDeque<usize>], job: usize) {
    for q in queues.iter_mut() {
        q.retain(|&x| x != job);
    }
}

/// Escolha para as políticas sem fila explícita.
fn choose(
    policy: Policy,
    ready: &[usize],
    jobs: &[Job],
    remaining: &[u32],
    rng: &mut Lcg,
) -> usize {
    match policy {
        // Ordem de chegada; empate pelo índice, para ser determinístico.
        Policy::Fcfs => *ready
            .iter()
            .min_by_key(|&&i| (jobs[i].arrival, i))
            .unwrap_or(&ready[0]),
        Policy::Sjf => *ready
            .iter()
            .min_by_key(|&&i| (jobs[i].burst, i))
            .unwrap_or(&ready[0]),
        Policy::Srtf => *ready
            .iter()
            .min_by_key(|&&i| (remaining[i], i))
            .unwrap_or(&ready[0]),
        // Maior prioridade primeiro.
        Policy::Priority => *ready
            .iter()
            .max_by_key(|&&i| (jobs[i].priority, usize::MAX - i))
            .unwrap_or(&ready[0]),
        Policy::Lottery => {
            let total: u32 = ready.iter().map(|&i| jobs[i].tickets.max(1)).sum();
            let mut draw = rng.next_u32(total);
            for &i in ready {
                let t = jobs[i].tickets.max(1);
                if draw < t {
                    return i;
                }
                draw -= t;
            }
            ready[ready.len() - 1]
        }
        // Tratadas pelo caminho das filas.
        Policy::RoundRobin | Policy::MultiQueue => ready[0],
    }
}

/// Funde unidades consecutivas do mesmo processo numa fatia só.
fn merge(units: &[Option<usize>]) -> Vec<Slice> {
    let mut out: Vec<Slice> = Vec::new();
    for (t, u) in units.iter().enumerate() {
        let Some(job) = *u else { continue };
        let t = t as u32;
        match out.last_mut() {
            Some(s) if s.job == job && s.end == t => s.end = t + 1,
            _ => out.push(Slice {
                job,
                start: t,
                end: t + 1,
            }),
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Interface
// ---------------------------------------------------------------------------

pub fn show(ui: &mut egui::Ui, scenario: &Scenario, st: &mut SchedState) {
    let sim = simulate(&st.jobs, scenario.policy, st.quantum);

    // Avanço do cursor: o Gantt se desenha da esquerda para a direita, como se
    // o tempo estivesse passando.
    if st.playing {
        let t = st.cursor.unwrap_or(0.0) + ui.input(|i| i.stable_dt).min(0.1) * st.speed;
        if t >= sim.end as f32 {
            st.cursor = None; // chegou ao fim: revela tudo
            st.playing = false;
        } else {
            st.cursor = Some(t);
        }
        ui.ctx().request_repaint();
    }

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

            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new("Simulador")
                        .size(14.0)
                        .strong()
                        .color(ACCENT),
                );
                ui.label(
                    egui::RichText::new(scenario.hint)
                        .size(12.0)
                        .color(egui::Color32::GRAY),
                );
            });
            ui.add_space(8.0);

            ui.horizontal_wrapped(|ui| {
                // Reprodução da linha do tempo.
                let label = if st.playing { "⏸" } else { "▶" };
                if ui
                    .button(label)
                    .on_hover_text("Desenhar o Gantt conforme o tempo passa")
                    .clicked()
                {
                    if st.playing {
                        st.playing = false;
                    } else {
                        // Dar play com tudo revelado recomeça do zero.
                        if st.cursor.is_none() {
                            st.cursor = Some(0.0);
                        }
                        st.playing = true;
                    }
                }
                if ui
                    .button("⏭")
                    .on_hover_text("Mostrar o resultado completo")
                    .clicked()
                {
                    st.cursor = None;
                    st.playing = false;
                }
                ui.add(
                    egui::Slider::new(&mut st.speed, 0.5..=12.0)
                        .show_value(false)
                        .text("velocidade"),
                );

                if scenario.policy.uses_quantum() {
                    ui.separator();
                    ui.add(
                        egui::Slider::new(&mut st.quantum, 1..=8)
                            .text("quantum")
                            .integer(),
                    );
                }
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
            job_table(ui, scenario, st);
            ui.add_space(12.0);
            chart(ui, &st.jobs, &sim, st.cursor);
            ui.add_space(10.0);
            metrics(ui, &st.jobs, &sim);
        });
}

/// Tabela editável dos processos.
fn job_table(ui: &mut egui::Ui, scenario: &Scenario, st: &mut SchedState) {
    let dark = ui.visuals().dark_mode;
    let show_prio = scenario.policy.uses_priority();
    let show_tickets = scenario.policy.uses_tickets();
    let cols = 3 + usize::from(show_prio) + usize::from(show_tickets);

    egui::Grid::new(("jobs", scenario.id))
        .num_columns(cols)
        .spacing([14.0, 5.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label(egui::RichText::new("processo").size(11.5).strong());
            ui.label(egui::RichText::new("chegada").size(11.5).strong());
            ui.label(egui::RichText::new("duração").size(11.5).strong());
            if show_prio {
                ui.label(egui::RichText::new("prioridade").size(11.5).strong());
            }
            if show_tickets {
                ui.label(egui::RichText::new("bilhetes").size(11.5).strong());
            }
            ui.end_row();

            for (i, job) in st.jobs.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(
                        egui::vec2(10.0, 10.0),
                        egui::Sense::hover(),
                    );
                    ui.painter()
                        .rect_filled(r, 2.0, series_color(i, dark));
                    ui.label(egui::RichText::new(job.name).size(12.5).monospace());
                });
                ui.add(egui::DragValue::new(&mut job.arrival).range(0..=20).speed(0.15));
                ui.add(egui::DragValue::new(&mut job.burst).range(1..=20).speed(0.15));
                if show_prio {
                    ui.add(egui::DragValue::new(&mut job.priority).range(0..=9).speed(0.1));
                }
                if show_tickets {
                    ui.add(egui::DragValue::new(&mut job.tickets).range(1..=100).speed(0.5));
                }
                ui.end_row();
            }
        });
}

/// O diagrama de Gantt: uma linha por processo, rotulada à esquerda.
///
/// `cursor` limita até onde o desenho é revelado; `None` mostra tudo.
fn chart(ui: &mut egui::Ui, jobs: &[Job], sim: &Sim, cursor: Option<f32>) {
    let n = jobs.len();
    if n == 0 || sim.end == 0 {
        return;
    }
    let dark = ui.visuals().dark_mode;
    let visuals = ui.visuals().clone();

    const ROW_H: f32 = 24.0;
    const ROW_GAP: f32 = 5.0;
    const LABEL_W: f32 = 74.0;
    const AXIS_H: f32 = 22.0;

    let width = ui.available_width();
    let height = n as f32 * (ROW_H + ROW_GAP) + AXIS_H;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let painter = ui.painter_at(rect);

    let plot_left = rect.left() + LABEL_W;
    let plot_w = (rect.width() - LABEL_W - 8.0).max(10.0);
    let unit = plot_w / sim.end as f32;

    let grid = visuals.text_color().gamma_multiply(0.16);
    let axis = visuals.text_color().gamma_multiply(0.45);

    // Grade vertical + rótulos do eixo do tempo.
    let tick_step = (sim.end as f32 / 16.0).ceil().max(1.0) as u32;
    let mut t = 0;
    while t <= sim.end {
        let x = plot_left + t as f32 * unit;
        painter.line_segment(
            [
                egui::pos2(x, rect.top()),
                egui::pos2(x, rect.bottom() - AXIS_H),
            ],
            egui::Stroke::new(1.0, grid),
        );
        painter.text(
            egui::pos2(x, rect.bottom() - AXIS_H + 8.0),
            egui::Align2::CENTER_TOP,
            t.to_string(),
            egui::FontId::proportional(10.5),
            axis,
        );
        t += tick_step;
    }

    for (i, job) in jobs.iter().enumerate() {
        let top = rect.top() + i as f32 * (ROW_H + ROW_GAP);
        let row = egui::Rect::from_min_size(
            egui::pos2(plot_left, top),
            egui::vec2(plot_w, ROW_H),
        );

        // Rótulo direto: a identidade não depende só da cor.
        painter.text(
            egui::pos2(rect.left() + 2.0, top + ROW_H / 2.0),
            egui::Align2::LEFT_CENTER,
            job.name,
            egui::FontId::monospace(12.0),
            visuals.text_color(),
        );

        // Trilho de fundo.
        painter.rect_filled(row, 3.0, visuals.extreme_bg_color);

        // Marca de chegada.
        let ax = plot_left + job.arrival as f32 * unit;
        painter.line_segment(
            [egui::pos2(ax, top), egui::pos2(ax, top + ROW_H)],
            egui::Stroke::new(1.5, axis),
        );

        for s in sim.slices.iter().filter(|s| s.job == i) {
            // Com o cursor ativo, a fatia só aparece até o instante atual.
            let fim = match cursor {
                Some(c) => (s.end as f32).min(c),
                None => s.end as f32,
            };
            if fim <= s.start as f32 {
                continue;
            }
            // 2px de respiro entre blocos vizinhos, para não colarem.
            let x0 = plot_left + s.start as f32 * unit + 1.0;
            let x1 = plot_left + fim * unit - 1.0;
            if x1 <= x0 {
                continue;
            }
            let block = egui::Rect::from_min_max(
                egui::pos2(x0, top + 2.0),
                egui::pos2(x1, top + ROW_H - 2.0),
            );
            painter.rect_filled(block, 3.0, series_color(i, dark));
        }
    }

    // Cursor do tempo, atravessando todas as linhas.
    if let Some(c) = cursor {
        let x = plot_left + c * unit;
        painter.line_segment(
            [
                egui::pos2(x, rect.top()),
                egui::pos2(x, rect.bottom() - AXIS_H),
            ],
            egui::Stroke::new(1.8, ACCENT),
        );
        painter.text(
            egui::pos2(x, rect.bottom() - AXIS_H + 8.0),
            egui::Align2::CENTER_TOP,
            format!("t = {:.0}", c),
            egui::FontId::proportional(11.0),
            ACCENT,
        );
    }

    // Linha do eixo.
    let y = rect.bottom() - AXIS_H;
    painter.line_segment(
        [
            egui::pos2(plot_left, y),
            egui::pos2(plot_left + plot_w, y),
        ],
        egui::Stroke::new(1.0, axis),
    );
}

/// Tempo de espera e de retorno, por processo e na média.
fn metrics(ui: &mut egui::Ui, jobs: &[Job], sim: &Sim) {
    if jobs.is_empty() || sim.finish.len() != jobs.len() {
        return;
    }
    egui::Grid::new("metrics")
        .num_columns(4)
        .spacing([14.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            for h in ["processo", "término", "retorno", "espera"] {
                ui.label(egui::RichText::new(h).size(11.5).strong());
            }
            ui.end_row();
            for (i, job) in jobs.iter().enumerate() {
                let turnaround = sim.finish[i].saturating_sub(job.arrival);
                let wait = turnaround.saturating_sub(job.burst.max(1));
                ui.label(egui::RichText::new(job.name).size(12.0).monospace());
                ui.label(egui::RichText::new(sim.finish[i].to_string()).size(12.0));
                ui.label(egui::RichText::new(turnaround.to_string()).size(12.0));
                ui.label(egui::RichText::new(wait.to_string()).size(12.0));
                ui.end_row();
            }
        });

    ui.add_space(6.0);
    ui.label(
        egui::RichText::new(format!(
            "tempo de espera médio: {:.2}     ·     tempo de retorno médio: {:.2}",
            sim.avg_wait, sim.avg_turnaround
        ))
        .size(13.0)
        .strong()
        .color(ACCENT),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job(name: &'static str, arrival: u32, burst: u32) -> Job {
        Job {
            name,
            arrival,
            burst,
            priority: 1,
            tickets: 10,
        }
    }

    /// Toda política precisa executar exatamente `burst` unidades de cada processo.
    #[test]
    fn conserva_o_trabalho_total() {
        let jobs = [job("P1", 0, 5), job("P2", 1, 3), job("P3", 3, 4)];
        for policy in [
            Policy::Fcfs,
            Policy::Sjf,
            Policy::Srtf,
            Policy::RoundRobin,
            Policy::Priority,
            Policy::MultiQueue,
            Policy::Lottery,
        ] {
            let sim = simulate(&jobs, policy, 2);
            for (i, j) in jobs.iter().enumerate() {
                let ran: u32 = sim
                    .slices
                    .iter()
                    .filter(|s| s.job == i)
                    .map(|s| s.end - s.start)
                    .sum();
                assert_eq!(ran, j.burst, "{} rodou {ran} unidades, esperado {}", j.name, j.burst);
                assert!(sim.finish[i] > 0, "{} nunca terminou", j.name);
                assert!(
                    sim.finish[i] >= j.arrival + j.burst,
                    "{} terminou antes do possível",
                    j.name
                );
            }
        }
    }

    /// Nenhuma fatia pode se sobrepor: a CPU é uma só.
    #[test]
    fn fatias_nao_se_sobrepoem() {
        let jobs = [job("P1", 0, 4), job("P2", 0, 4), job("P3", 2, 2)];
        for policy in [Policy::Srtf, Policy::RoundRobin, Policy::MultiQueue] {
            let sim = simulate(&jobs, policy, 2);
            let mut sorted: Vec<_> = sim.slices.iter().map(|s| (s.start, s.end)).collect();
            sorted.sort();
            for w in sorted.windows(2) {
                assert!(w[0].1 <= w[1].0, "fatias sobrepostas: {:?} e {:?}", w[0], w[1]);
            }
        }
    }

    /// Nenhum processo pode rodar antes de chegar.
    #[test]
    fn ninguem_roda_antes_de_chegar() {
        let jobs = [job("P1", 3, 2), job("P2", 0, 3), job("P3", 7, 1)];
        for policy in [Policy::Fcfs, Policy::Sjf, Policy::Srtf, Policy::RoundRobin] {
            let sim = simulate(&jobs, policy, 2);
            for s in &sim.slices {
                assert!(
                    s.start >= jobs[s.job].arrival,
                    "{} rodou em {} mas chega em {}",
                    jobs[s.job].name,
                    s.start,
                    jobs[s.job].arrival
                );
            }
        }
    }

    /// FCFS com chegadas em ordem produz exatamente a ordem de chegada.
    #[test]
    fn fcfs_respeita_a_ordem_de_chegada() {
        let jobs = [job("P1", 0, 3), job("P2", 1, 2), job("P3", 2, 4)];
        let sim = simulate(&jobs, Policy::Fcfs, 1);
        let ordem: Vec<usize> = sim.slices.iter().map(|s| s.job).collect();
        assert_eq!(ordem, vec![0, 1, 2], "FCFS deveria rodar P1, P2, P3 sem fatiar");
        assert_eq!(sim.finish, vec![3, 5, 9]);
    }

    /// SJF, com todos disponíveis em t=0, roda do mais curto ao mais longo.
    #[test]
    fn sjf_roda_do_mais_curto_ao_mais_longo() {
        let jobs = [job("P1", 0, 6), job("P2", 0, 2), job("P3", 0, 4)];
        let sim = simulate(&jobs, Policy::Sjf, 1);
        let ordem: Vec<usize> = sim.slices.iter().map(|s| s.job).collect();
        assert_eq!(ordem, vec![1, 2, 0]);
    }

    /// Round-robin fatia: com quantum 1 e dois processos, alterna a cada unidade.
    #[test]
    fn round_robin_alterna_por_quantum() {
        let jobs = [job("P1", 0, 3), job("P2", 0, 3)];
        let sim = simulate(&jobs, Policy::RoundRobin, 1);
        let ordem: Vec<usize> = sim.slices.iter().map(|s| s.job).collect();
        assert_eq!(ordem, vec![0, 1, 0, 1, 0, 1]);
    }

    /// A loteria é determinística: o mesmo cenário dá sempre o mesmo resultado.
    #[test]
    fn loteria_e_reprodutivel() {
        let jobs = [job("P1", 0, 5), job("P2", 0, 5)];
        let a = simulate(&jobs, Policy::Lottery, 1);
        let b = simulate(&jobs, Policy::Lottery, 1);
        let fa: Vec<_> = a.slices.iter().map(|s| (s.job, s.start, s.end)).collect();
        let fb: Vec<_> = b.slices.iter().map(|s| (s.job, s.start, s.end)).collect();
        assert_eq!(fa, fb);
    }

    /// Com a CPU ociosa no começo, ninguém é escalado antes da primeira chegada.
    #[test]
    fn cpu_ociosa_ate_a_primeira_chegada() {
        let jobs = [job("P1", 4, 2)];
        let sim = simulate(&jobs, Policy::Fcfs, 1);
        assert_eq!(sim.slices.len(), 1);
        assert_eq!(sim.slices[0].start, 4);
        assert_eq!(sim.finish[0], 6);
        // Esperou 0: começou assim que chegou.
        assert_eq!(sim.avg_wait, 0.0);
    }
}
