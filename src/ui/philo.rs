//! Simulador do jantar dos filósofos (seção 6.2).
//!
//! Cinco filósofos (de 3 a 7, ajustável) em torno de uma mesa redonda, com um
//! hashi entre cada par de vizinhos. Cada filósofo alterna entre pensar, ficar
//! com fome e comer, e para comer precisa dos dois hashis vizinhos. A simulação
//! é discreta: a cada passo o escalonador percorre os filósofos numa ordem que
//! gira, e cada um executa no máximo uma ação (pegar um hashi, soltar, comer).
//! É esse "uma ação por passo" que deixa o impasse acontecer de verdade na
//! estratégia ingênua, quando todos pegam o hashi da esquerda ao mesmo tempo.
//!
//! Três estratégias, exatamente as de 6.2.5 a 6.2.8:
//! - Ingênua: pega a esquerda e bloqueia esperando a direita, sem soltar.
//!   Leva ao impasse.
//! - Com sorteio: pega a esquerda, tenta a direita e, se estiver ocupada, solta
//!   a esquerda e pensa por um tempo sorteado antes de recomeçar.
//! - Definitiva: só começa a comer quando nenhum dos dois vizinhos come, e aí
//!   toma os dois hashis de uma vez. Nunca bloqueia, e até dois comem juntos.

use eframe::egui;

use crate::ui::{param_row, params_grid, widget_frame, widget_header, ACCENT};

const THINK: egui::Color32 = egui::Color32::from_rgb(0x5b, 0xd6, 0x8a);
const HUNGRY: egui::Color32 = egui::Color32::from_rgb(0xff, 0xb4, 0x5b);
const EATING: egui::Color32 = ACCENT;
const BLOCKED: egui::Color32 = egui::Color32::from_rgb(0xff, 0x6b, 0x5b);

/// As três estratégias das anotações.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    Naive,
    Random,
    Monitor,
}

impl Strategy {
    const ALL: [Strategy; 3] = [Strategy::Naive, Strategy::Random, Strategy::Monitor];

    fn label(self) -> &'static str {
        match self {
            Strategy::Naive => "Ingênua",
            Strategy::Random => "Com sorteio",
            Strategy::Monitor => "Definitiva",
        }
    }

    fn caption(self) -> &'static str {
        match self {
            Strategy::Naive => {
                "Cada filósofo pega o hashi da esquerda e espera o da direita sem soltá-lo. Quando \
                 todos pegam a esquerda ao mesmo tempo, ninguém consegue a direita e o jantar \
                 trava para sempre."
            }
            Strategy::Random => {
                "Como a ingênua, mas um filósofo que não consegue o hashi da direita devolve o da \
                 esquerda e pensa por um tempo sorteado antes de tentar de novo. Costuma avançar, \
                 embora dois vizinhos ainda possam se sincronizar por um tempo."
            }
            Strategy::Monitor => {
                "Um filósofo só começa a comer quando nenhum dos dois vizinhos está comendo, e aí \
                 toma os dois hashis de uma vez. Ninguém fica bloqueado, e até dois filósofos \
                 comem ao mesmo tempo."
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Act {
    Thinking,
    Hungry,
    Eating,
}

#[derive(Clone, Copy)]
struct Phil {
    act: Act,
    /// Segura o hashi da esquerda (índice do próprio filósofo).
    left: bool,
    /// Segura o hashi da direita (índice do vizinho seguinte).
    right: bool,
    /// Passos restantes de pensar ou comer.
    timer: i32,
    meals: u32,
}

/// Gerador congruente linear: os tempos sorteados são reprodutíveis.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self, bound: u32) -> u32 {
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

pub struct PhiloState {
    strategy: Strategy,
    n: usize,
    phils: Vec<Phil>,
    /// Hashi j: `None` se livre, `Some(i)` se o filósofo i o segura.
    forks: Vec<Option<usize>>,
    playing: bool,
    /// Passos por segundo.
    speed: f32,
    clock: f32,
    steps: u64,
    rng: Lcg,
    offset: usize,
    deadlocked: bool,
}

impl Default for PhiloState {
    fn default() -> Self {
        let mut st = Self {
            strategy: Strategy::Naive,
            n: 5,
            phils: Vec::new(),
            forks: Vec::new(),
            playing: false,
            speed: 2.5,
            clock: 0.0,
            steps: 0,
            rng: Lcg(0x5EED_D1_11_F00D_0001),
            offset: 0,
            deadlocked: false,
        };
        st.rebuild();
        st
    }
}

impl PhiloState {
    fn rebuild(&mut self) {
        self.forks = vec![None; self.n];
        self.phils = Vec::with_capacity(self.n);
        self.deadlocked = false;
        self.steps = 0;
        self.offset = 0;
        self.clock = 0.0;
        for _ in 0..self.n {
            // Na ingênua todos pensam o mesmo tempo, então ficam com fome
            // juntos e caem no impasse; nas outras, um escalonamento sorteado
            // os desencontra.
            let timer = match self.strategy {
                Strategy::Naive => 2,
                _ => 1 + self.rng.next(4) as i32,
            };
            self.phils.push(Phil {
                act: Act::Thinking,
                left: false,
                right: false,
                timer,
                meals: 0,
            });
        }
    }

    fn lf(&self, i: usize) -> usize {
        i
    }
    fn rf(&self, i: usize) -> usize {
        (i + 1) % self.n
    }
    fn left_neighbor(&self, i: usize) -> usize {
        (i + self.n - 1) % self.n
    }
    fn right_neighbor(&self, i: usize) -> usize {
        (i + 1) % self.n
    }

    fn think_time(&mut self) -> i32 {
        match self.strategy {
            Strategy::Naive => 3,
            _ => 2 + self.rng.next(5) as i32,
        }
    }
    fn eat_time(&mut self) -> i32 {
        match self.strategy {
            Strategy::Naive => 3,
            _ => 2 + self.rng.next(3) as i32,
        }
    }

    fn start_eating(&mut self, i: usize) {
        let t = self.eat_time();
        self.phils[i].act = Act::Eating;
        self.phils[i].timer = t;
    }

    fn step(&mut self) {
        if self.deadlocked || self.n == 0 {
            return;
        }
        for k in 0..self.n {
            let i = (self.offset + k) % self.n;
            match self.phils[i].act {
                Act::Thinking => {
                    self.phils[i].timer -= 1;
                    if self.phils[i].timer <= 0 {
                        self.phils[i].act = Act::Hungry;
                    }
                }
                Act::Eating => {
                    self.phils[i].timer -= 1;
                    if self.phils[i].timer <= 0 {
                        let (l, r) = (self.lf(i), self.rf(i));
                        self.forks[l] = None;
                        self.forks[r] = None;
                        self.phils[i].left = false;
                        self.phils[i].right = false;
                        self.phils[i].meals += 1;
                        self.phils[i].act = Act::Thinking;
                        self.phils[i].timer = self.think_time();
                    }
                }
                Act::Hungry => self.hungry_action(i),
            }
        }
        self.offset = (self.offset + 1) % self.n;
        self.steps += 1;
        self.detect_deadlock();
    }

    /// Uma única ação de um filósofo com fome, conforme a estratégia.
    fn hungry_action(&mut self, i: usize) {
        let (l, r) = (self.lf(i), self.rf(i));
        match self.strategy {
            Strategy::Naive => {
                if !self.phils[i].left {
                    if self.forks[l].is_none() {
                        self.forks[l] = Some(i);
                        self.phils[i].left = true;
                    }
                } else if !self.phils[i].right && self.forks[r].is_none() {
                    self.forks[r] = Some(i);
                    self.phils[i].right = true;
                    self.start_eating(i);
                }
                // Tem a esquerda e a direita está ocupada: bloqueado, sem soltar.
            }
            Strategy::Random => {
                if !self.phils[i].left {
                    if self.forks[l].is_none() {
                        self.forks[l] = Some(i);
                        self.phils[i].left = true;
                    }
                } else if !self.phils[i].right {
                    if self.forks[r].is_none() {
                        self.forks[r] = Some(i);
                        self.phils[i].right = true;
                        self.start_eating(i);
                    } else {
                        // Recuo: solta a esquerda e pensa por um tempo sorteado.
                        self.forks[l] = None;
                        self.phils[i].left = false;
                        self.phils[i].act = Act::Thinking;
                        self.phils[i].timer = 1 + self.rng.next(4) as i32;
                    }
                }
            }
            Strategy::Monitor => {
                let ln = self.left_neighbor(i);
                let rn = self.right_neighbor(i);
                let livre = self.phils[ln].act != Act::Eating && self.phils[rn].act != Act::Eating;
                if livre {
                    // Os dois hashis estão livres, pois só quem come os segura.
                    self.forks[l] = Some(i);
                    self.forks[r] = Some(i);
                    self.phils[i].left = true;
                    self.phils[i].right = true;
                    self.start_eating(i);
                }
            }
        }
    }

    /// Impasse: todos com fome, cada um segurando só o hashi da esquerda.
    fn detect_deadlock(&mut self) {
        if self.n == 0 {
            return;
        }
        let travado = self
            .phils
            .iter()
            .all(|p| p.act == Act::Hungry && p.left && !p.right);
        if travado {
            self.deadlocked = true;
        }
    }

    fn eating_now(&self) -> usize {
        self.phils.iter().filter(|p| p.act == Act::Eating).count()
    }
    fn total_meals(&self) -> u32 {
        self.phils.iter().map(|p| p.meals).sum()
    }
}

pub fn show(ui: &mut egui::Ui, st: &mut PhiloState) {
    if st.playing && !st.deadlocked {
        st.clock += ui.input(|i| i.stable_dt).min(0.1) * st.speed;
        let mut guard = 0;
        while st.clock >= 1.0 && guard < 8 {
            st.clock -= 1.0;
            st.step();
            guard += 1;
            if st.deadlocked {
                break;
            }
        }
        ui.ctx().request_repaint();
    }
    if st.deadlocked {
        st.playing = false;
    }

    widget_frame(ui).show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        widget_header(
            ui,
            "Simulador",
            "Os filósofos em torno da mesa, com um hashi entre cada par de vizinhos.",
        );

        ui.horizontal_wrapped(|ui| {
            for s in Strategy::ALL {
                if ui.selectable_label(st.strategy == s, s.label()).clicked() && st.strategy != s {
                    st.strategy = s;
                    st.playing = false;
                    st.rebuild();
                }
            }
        });
        ui.add_space(6.0);

        ui.horizontal_wrapped(|ui| {
            let label = if st.playing { "⏸" } else { "▶" };
            if ui
                .add_enabled(!st.deadlocked, egui::Button::new(label))
                .on_hover_text("Executar o jantar")
                .clicked()
            {
                st.playing = !st.playing;
            }
            if ui.button("↺").on_hover_text("Reiniciar").clicked() {
                st.playing = false;
                st.rebuild();
            }
            ui.add(
                egui::Slider::new(&mut st.speed, 0.5..=8.0)
                    .show_value(false)
                    .text("velocidade"),
            );
        });
        ui.add_space(4.0);
        params_grid(ui, "philo_params", |ui| {
            let mut n = st.n as u32;
            if param_row(ui, "filósofos", egui::Slider::new(&mut n, 3..=7).integer()).changed() {
                st.n = n as usize;
                st.playing = false;
                st.rebuild();
            }
        });
        ui.add_space(8.0);

        table(ui, st);
        ui.add_space(6.0);
        meals_row(ui, st);
        ui.add_space(6.0);
        status(ui, st);
        ui.add_space(6.0);
        legend(ui);
    });
}

fn color_of(p: &Phil, deadlocked: bool) -> egui::Color32 {
    match p.act {
        Act::Thinking => THINK,
        Act::Eating => EATING,
        Act::Hungry => {
            if deadlocked {
                BLOCKED
            } else {
                HUNGRY
            }
        }
    }
}

/// A mesa redonda: nós dos filósofos e os hashis entre eles.
fn table(ui: &mut egui::Ui, st: &PhiloState) {
    let n = st.n;
    let side = ui.available_width().min(340.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(side, side), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let v = ui.visuals().clone();
    let center = rect.center();
    let node_r = (side * 0.5 - 52.0).clamp(80.0, 150.0);
    let line = v.text_color().gamma_multiply(0.5);

    let node_pos = |i: usize| -> egui::Pos2 {
        let ang = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::TAU / n as f32;
        center + egui::vec2(ang.cos(), ang.sin()) * node_r
    };

    // Mesa.
    painter.circle(
        center,
        node_r - 30.0,
        v.faint_bg_color,
        egui::Stroke::new(1.0, line.gamma_multiply(0.6)),
    );
    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        "mesa",
        egui::FontId::proportional(12.0),
        v.weak_text_color(),
    );

    // Hashis: o hashi j fica entre o filósofo j e o filósofo j-1.
    for j in 0..n {
        let a = node_pos(j);
        let b = node_pos((j + n - 1) % n);
        let mid = a + (b - a) * 0.5;
        let held = st.forks[j];
        let base = egui::vec2((mid - center).x, (mid - center).y).normalized();
        // Posição do hashi: puxado para perto de quem o segura.
        let anchor = match held {
            Some(h) => {
                let hp = node_pos(h);
                mid + (hp - mid) * 0.33
            }
            None => center + base * (node_r - 30.0) * 0.82,
        };
        let perp = egui::vec2(-base.y, base.x) * 9.0;
        let color = if held.is_some() {
            ACCENT
        } else {
            line.gamma_multiply(0.8)
        };
        painter.line_segment(
            [anchor - perp, anchor + perp],
            egui::Stroke::new(3.0, color),
        );
    }

    // Filósofos.
    for i in 0..n {
        let pos = node_pos(i);
        let p = &st.phils[i];
        let color = color_of(p, st.deadlocked);
        painter.circle(
            pos,
            19.0,
            color.gamma_multiply(0.30),
            egui::Stroke::new(1.8, color),
        );
        painter.text(
            pos,
            egui::Align2::CENTER_CENTER,
            format!("F{i}"),
            egui::FontId::proportional(13.0),
            v.text_color(),
        );
        // Rótulo do estado, ancorado para fora do círculo, crescendo para longe
        // do nó, de modo a nunca cobri-lo.
        let dir = (pos - center).normalized();
        let out = pos + dir * 24.0;
        let align = if dir.x > 0.35 {
            egui::Align2::LEFT_CENTER
        } else if dir.x < -0.35 {
            egui::Align2::RIGHT_CENTER
        } else if dir.y < 0.0 {
            egui::Align2::CENTER_BOTTOM
        } else {
            egui::Align2::CENTER_TOP
        };
        let estado = match p.act {
            Act::Thinking => "pensa",
            Act::Hungry => {
                if st.deadlocked {
                    "travado"
                } else {
                    "com fome"
                }
            }
            Act::Eating => "come",
        };
        painter.text(out, align, estado, egui::FontId::proportional(10.5), color);
    }
}

/// Uma linha compacta com as refeições de cada filósofo, para enxergar a
/// distribuição e a inanição.
fn meals_row(ui: &mut egui::Ui, st: &PhiloState) {
    ui.horizontal_wrapped(|ui| {
        ui.label(
            egui::RichText::new("refeições:")
                .size(11.0)
                .color(egui::Color32::GRAY),
        );
        for (i, p) in st.phils.iter().enumerate() {
            ui.label(
                egui::RichText::new(format!("F{i}: {}", p.meals))
                    .size(11.0)
                    .monospace(),
            );
            ui.add_space(4.0);
        }
    });
}

fn status(ui: &mut egui::Ui, st: &PhiloState) {
    let (fill, icon, msg) = if st.deadlocked {
        (
            BLOCKED.gamma_multiply(0.16),
            "✖",
            "Impasse. Todos seguram o hashi da esquerda e esperam o da direita, que nunca fica \
             livre. Nenhum filósofo volta a progredir."
                .to_string(),
        )
    } else {
        (
            ACCENT.gamma_multiply(0.10),
            "➡",
            format!(
                "{}  ·  comendo agora: {}  ·  refeições servidas: {}  ·  passo {}",
                st.strategy.caption(),
                st.eating_now(),
                st.total_meals(),
                st.steps
            ),
        )
    };
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(10, 8))
        .corner_radius(7.0)
        .fill(fill)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal_top(|ui| {
                ui.label(egui::RichText::new(icon).size(14.0));
                ui.add(egui::Label::new(egui::RichText::new(msg).size(12.5)).wrap());
            });
        });
}

fn legend(ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        let chip = |ui: &mut egui::Ui, c: egui::Color32, t: &str| {
            let (r, _) = ui.allocate_exact_size(egui::vec2(11.0, 11.0), egui::Sense::hover());
            ui.painter().rect_filled(r, 2.0, c);
            ui.label(egui::RichText::new(t).size(11.0).color(egui::Color32::GRAY));
            ui.add_space(6.0);
        };
        chip(ui, THINK, "pensa");
        chip(ui, HUNGRY, "com fome");
        chip(ui, EATING, "come");
        chip(ui, BLOCKED, "travado");
        chip(ui, ACCENT, "hashi em uso");
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(strategy: Strategy, n: usize, steps: usize) -> PhiloState {
        let mut st = PhiloState {
            strategy,
            n,
            ..PhiloState::default()
        };
        st.rebuild();
        for _ in 0..steps {
            st.step();
        }
        st
    }

    /// A ingênua trava: todos pegam a esquerda juntos e esperam a direita.
    #[test]
    fn ingenua_cai_no_impasse() {
        for n in 3..=7 {
            let st = run(Strategy::Naive, n, 30);
            assert!(st.deadlocked, "n={n} deveria travar na estratégia ingênua");
            assert_eq!(st.total_meals(), 0, "ninguém come antes do impasse");
        }
    }

    /// A definitiva nunca trava e serve refeições.
    #[test]
    fn definitiva_progride_sem_travar() {
        let st = run(Strategy::Monitor, 5, 400);
        assert!(!st.deadlocked, "a definitiva não pode travar");
        assert!(st.total_meals() > 0, "alguém deveria ter comido");
    }

    /// Na definitiva dois vizinhos nunca comem ao mesmo tempo (exclusão do hashi).
    #[test]
    fn vizinhos_nunca_comem_juntos() {
        let mut st = PhiloState {
            strategy: Strategy::Monitor,
            n: 5,
            ..PhiloState::default()
        };
        st.rebuild();
        for _ in 0..400 {
            st.step();
            for i in 0..st.n {
                if st.phils[i].act == Act::Eating {
                    let rn = st.right_neighbor(i);
                    assert!(
                        st.phils[rn].act != Act::Eating,
                        "vizinhos {i} e {rn} comendo juntos"
                    );
                }
            }
        }
    }

    /// Um hashi nunca está com dois filósofos ao mesmo tempo.
    #[test]
    fn hashi_e_exclusivo() {
        for strategy in [Strategy::Naive, Strategy::Random, Strategy::Monitor] {
            let mut st = PhiloState {
                strategy,
                n: 5,
                ..PhiloState::default()
            };
            st.rebuild();
            for _ in 0..200 {
                st.step();
                for i in 0..st.n {
                    if st.phils[i].left {
                        assert_eq!(st.forks[st.lf(i)], Some(i));
                    }
                    if st.phils[i].right {
                        assert_eq!(st.forks[st.rf(i)], Some(i));
                    }
                }
            }
        }
    }
}
