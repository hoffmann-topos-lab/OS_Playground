//! Simulador de substituição de páginas (seção 3.4).
//!
//! Uma única simulação, referência a referência, atende a todos os algoritmos:
//! a cada referência sabemos se a página está residente (acerto) ou não (falta)
//! e, na falta sem quadro livre, o algoritmo escolhe a vítima. Os bits R e M são
//! ligados pelo "hardware" a cada acesso e desligados pelo "sistema" — na
//! interrupção de relógio (NRU, envelhecimento, conjunto de trabalho, WSClock)
//! ou pela própria varredura (segunda chance, relógio, WSClock).
//!
//! O resultado é uma grade — colunas são referências, linhas são quadros — e,
//! para cada passo, uma explicação de por que aquela página foi descartada. O
//! ponteiro do relógio é desenhado como um anel de verdade.
//!
//! O usuário pode editar a cadeia de referências, o número de quadros, o
//! intervalo do relógio e o limiar τ, e ver tudo recalcular na hora.

use eframe::egui;

use crate::ui::{ACCENT, param_row, params_grid, widget_frame, widget_header};

const DANGER: egui::Color32 = egui::Color32::from_rgb(0xff, 0x6b, 0x5b);
const SAFE: egui::Color32 = egui::Color32::from_rgb(0x5b, 0xd6, 0x8a);
/// Página descartada que estava modificada (custou uma escrita em disco).
const DIRTY: egui::Color32 = egui::Color32::from_rgb(0xeb, 0x68, 0x34);

/// Os algoritmos das anotações, na ordem em que aparecem.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Algo {
    Optimal,
    Nru,
    Fifo,
    SecondChance,
    Clock,
    Lru,
    Aging,
    WorkingSet,
    WsClock,
}

impl Algo {
    pub const ALL: [Algo; 9] = [
        Algo::Optimal,
        Algo::Nru,
        Algo::Fifo,
        Algo::SecondChance,
        Algo::Clock,
        Algo::Lru,
        Algo::Aging,
        Algo::WorkingSet,
        Algo::WsClock,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Algo::Optimal => "Ótimo",
            Algo::Nru => "NRU",
            Algo::Fifo => "FIFO",
            Algo::SecondChance => "Segunda chance",
            Algo::Clock => "Relógio",
            Algo::Lru => "LRU",
            Algo::Aging => "Envelhecimento",
            Algo::WorkingSet => "Conjunto de trabalho",
            Algo::WsClock => "WSClock",
        }
    }

    /// Precisa da interrupção de relógio (que desliga os bits R).
    pub fn uses_tick(self) -> bool {
        matches!(
            self,
            Algo::Nru | Algo::Aging | Algo::WorkingSet | Algo::WsClock
        )
    }
    /// Usa o limiar de idade τ.
    pub fn uses_tau(self) -> bool {
        matches!(self, Algo::WorkingSet | Algo::WsClock)
    }
    /// Mantém um ponteiro circular.
    pub fn has_hand(self) -> bool {
        matches!(self, Algo::Clock | Algo::WsClock)
    }
}

/// Uma referência à memória: a página e se é escrita.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Ref {
    pub page: u8,
    pub write: bool,
}

/// Lê "7 0 1w 2 …": números separados por espaço, sufixo `w` = escrita.
pub fn parse_refs(text: &str) -> Vec<Ref> {
    text.split_whitespace()
        .filter_map(|tok| {
            let (num, write) = match tok.strip_suffix(['w', 'W']) {
                Some(n) => (n, true),
                None => (tok, false),
            };
            num.parse::<u8>().ok().map(|page| Ref { page, write })
        })
        .collect()
}

/// Um cenário pronto, embutido num artigo.
pub struct Scenario {
    /// Identificador estável — é a chave do estado editável.
    pub id: &'static str,
    pub algo: Algo,
    /// Cadeia de referências, no formato de [`parse_refs`].
    pub refs: &'static str,
    pub frames: u32,
    /// Referências entre duas interrupções de relógio.
    pub tick: u32,
    /// Limiar de idade τ, em referências.
    pub tau: u32,
    /// Frase curta sobre o que observar neste cenário.
    pub hint: &'static str,
}

/// Estado editável de um cenário, mais a reprodução passo a passo.
pub struct ReplState {
    refs_text: String,
    frames: u32,
    tick: u32,
    tau: u32,
    /// Passo mostrado (0-based). `None` = tudo revelado, detalhe do último.
    cursor: Option<usize>,
    playing: bool,
    /// Passos por segundo.
    speed: f32,
    clock: f32,
}

impl ReplState {
    pub fn new(s: &Scenario) -> Self {
        Self {
            refs_text: s.refs.to_string(),
            frames: s.frames.max(1),
            tick: s.tick.max(1),
            tau: s.tau.max(1),
            cursor: None,
            playing: false,
            speed: 1.5,
            clock: 0.0,
        }
    }
    fn reset(&mut self, s: &Scenario) {
        let speed = self.speed;
        *self = Self::new(s);
        self.speed = speed;
    }
}

/// Um quadro de página e os metadados que os algoritmos consultam.
#[derive(Clone, Debug, Default)]
pub struct Slot {
    pub page: Option<u8>,
    pub r: bool,
    pub m: bool,
    /// Ordem de carga (FIFO / segunda chance); renovada ao ganhar a 2ª chance.
    pub seq: u32,
    /// Instante da última referência real (LRU).
    pub last_ref: u32,
    /// Instante de último uso registrado pelo algoritmo (conjunto de trabalho).
    pub ws_time: u32,
    /// Contador de envelhecimento (8 bits).
    pub age: u8,
}

/// Um passo da simulação: a referência e o estado dos quadros logo após ela.
#[derive(Clone, Debug)]
pub struct Step {
    pub r: Ref,
    pub fault: bool,
    /// Quadro tocado: o do acerto ou o que recebeu a página.
    pub frame: usize,
    pub evicted: Option<u8>,
    pub dirty_evicted: bool,
    pub note: String,
    pub slots: Vec<Slot>,
    pub hand: usize,
    /// Houve interrupção de relógio logo depois desta referência.
    pub ticked: bool,
}

pub struct Sim {
    pub steps: Vec<Step>,
    pub faults: u32,
}

/// Gerador congruente linear: o "acaso" do NRU é reprodutível.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self, bound: usize) -> usize {
        if bound == 0 {
            return 0;
        }
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.0 >> 33) as usize) % bound
    }
}

struct Machine {
    slots: Vec<Slot>,
    hand: usize,
    seq: u32,
    rng: Lcg,
}

impl Machine {
    fn new(n: usize) -> Self {
        Self {
            slots: vec![Slot::default(); n],
            hand: 0,
            seq: 0,
            rng: Lcg(0x5EED_0BAD_F00D_0001),
        }
    }

    fn find(&self, page: u8) -> Option<usize> {
        self.slots.iter().position(|s| s.page == Some(page))
    }

    fn free(&self) -> Option<usize> {
        self.slots.iter().position(|s| s.page.is_none())
    }

    fn next_seq(&mut self) -> u32 {
        self.seq += 1;
        self.seq
    }

    fn load(&mut self, k: usize, r: Ref, t: u32) {
        let seq = self.next_seq();
        self.slots[k] = Slot {
            page: Some(r.page),
            r: true,
            m: r.write,
            seq,
            last_ref: t,
            ws_time: t,
            age: 0,
        };
    }

    fn page_name(&self, k: usize) -> String {
        self.slots[k]
            .page
            .map_or("—".to_string(), |p| p.to_string())
    }

    /// A interrupção de relógio, para os algoritmos que a usam.
    fn tick(&mut self, algo: Algo) {
        match algo {
            Algo::Nru | Algo::WorkingSet | Algo::WsClock => {
                for s in &mut self.slots {
                    s.r = false;
                }
            }
            Algo::Aging => {
                for s in &mut self.slots {
                    if s.page.is_some() {
                        s.age = (s.age >> 1) | (u8::from(s.r) << 7);
                    }
                    s.r = false;
                }
            }
            _ => {}
        }
    }

    /// Escolhe a vítima e explica a escolha.
    fn victim(&mut self, algo: Algo, refs: &[Ref], i: usize, t: u32, tau: u32) -> (usize, String) {
        let n = self.slots.len();
        match algo {
            Algo::Optimal => {
                let dist: Vec<Option<usize>> = (0..n)
                    .map(|k| next_use(refs, i, self.slots[k].page))
                    .collect();
                let k = (0..n)
                    .max_by_key(|&k| (dist[k].map_or(usize::MAX, |d| d), usize::MAX - k))
                    .unwrap_or(0);
                let labels: Vec<String> = (0..n)
                    .map(|j| {
                        format!(
                            "{}: {}",
                            self.page_name(j),
                            dist[j].map_or("nunca".to_string(), |d| format!("+{d}"))
                        )
                    })
                    .collect();
                let why = match dist[k] {
                    Some(d) => format!("só volta a ser referenciada daqui a {d} referências"),
                    None => "nunca mais é referenciada".to_string(),
                };
                (
                    k,
                    format!(
                        "Rótulos (próxima referência): {}. Descartou a página {}, que {why}.",
                        labels.join(", "),
                        self.page_name(k)
                    ),
                )
            }
            Algo::Nru => {
                let class = |s: &Slot| 2 * usize::from(s.r) + usize::from(s.m);
                let lowest = (0..n).map(|k| class(&self.slots[k])).min().unwrap_or(0);
                let pool: Vec<usize> = (0..n)
                    .filter(|&k| class(&self.slots[k]) == lowest)
                    .collect();
                let k = pool[self.rng.next(pool.len())];
                let mut groups = String::new();
                for c in 0..4 {
                    let pages: Vec<String> = (0..n)
                        .filter(|&j| class(&self.slots[j]) == c)
                        .map(|j| self.page_name(j))
                        .collect();
                    if !pages.is_empty() {
                        groups.push_str(&format!(" classe {c}: {{{}}}", pages.join(", ")));
                    }
                }
                (
                    k,
                    format!(
                        "Classes (R,M):{groups}. Descartou uma página ao acaso da menor classe \
                         não vazia: a página {} (classe {lowest}).",
                        self.page_name(k)
                    ),
                )
            }
            Algo::Fifo => {
                let k = (0..n).min_by_key(|&k| self.slots[k].seq).unwrap_or(0);
                (
                    k,
                    format!(
                        "Descartou a página {}, a mais antiga da fila — sem olhar se ainda é usada.",
                        self.page_name(k)
                    ),
                )
            }
            Algo::SecondChance => {
                let mut passed: Vec<String> = Vec::new();
                loop {
                    let k = (0..n).min_by_key(|&k| self.slots[k].seq).unwrap_or(0);
                    if self.slots[k].r {
                        self.slots[k].r = false;
                        self.slots[k].seq = self.next_seq();
                        passed.push(self.page_name(k));
                    } else {
                        let mut note = String::new();
                        if !passed.is_empty() {
                            note.push_str(&format!(
                                "Segunda chance para {} (tinha R=1: R zerado, vai para o fim da fila). ",
                                passed.join(", ")
                            ));
                        }
                        note.push_str(&format!(
                            "A página {} estava com R=0: descartada.",
                            self.page_name(k)
                        ));
                        return (k, note);
                    }
                }
            }
            Algo::Clock => {
                let mut passed: Vec<String> = Vec::new();
                loop {
                    let k = self.hand;
                    if self.slots[k].r {
                        self.slots[k].r = false;
                        passed.push(self.page_name(k));
                        self.hand = (self.hand + 1) % n;
                    } else {
                        let mut note = String::new();
                        if !passed.is_empty() {
                            note.push_str(&format!(
                                "O ponteiro passou por {} (tinha R=1: R zerado) e ",
                                passed.join(", ")
                            ));
                        } else {
                            note.push_str("O ponteiro ");
                        }
                        note.push_str(&format!(
                            "parou na página {} (R=0): descartada; o ponteiro avança.",
                            self.page_name(k)
                        ));
                        return (k, note);
                    }
                }
            }
            Algo::Lru => {
                let k = (0..n).min_by_key(|&k| self.slots[k].last_ref).unwrap_or(0);
                (
                    k,
                    format!(
                        "Descartou a página {}, a que passou mais tempo sem ser referenciada \
                         (última referência em t={}).",
                        self.page_name(k),
                        self.slots[k].last_ref
                    ),
                )
            }
            Algo::Aging => {
                let k = (0..n)
                    .min_by_key(|&k| (self.slots[k].age, self.slots[k].last_ref))
                    .unwrap_or(0);
                let counters: Vec<String> = (0..n)
                    .map(|j| {
                        format!(
                            "{}={:08b} ({})",
                            self.page_name(j),
                            self.slots[j].age,
                            self.slots[j].age
                        )
                    })
                    .collect();
                (
                    k,
                    format!(
                        "Contadores: {}. Descartou a página {}, de menor contador.",
                        counters.join(", "),
                        self.page_name(k)
                    ),
                )
            }
            Algo::WorkingSet => {
                let mut notes: Vec<String> = Vec::new();
                let mut candidate: Option<(usize, u32)> = None;
                for k in 0..n {
                    let name = self.page_name(k);
                    let s = &mut self.slots[k];
                    if s.r {
                        s.ws_time = t;
                        notes.push(format!(
                            "{name}: R=1, no conjunto de trabalho (tempo = {t})"
                        ));
                        continue;
                    }
                    let age = t.saturating_sub(s.ws_time);
                    if age > tau {
                        notes.push(format!(
                            "{name}: R=0 e idade {age} > τ={tau}, fora do conjunto de trabalho"
                        ));
                        return (
                            k,
                            format!(
                                "Varredura: {}. Descartou a página {name} e a varredura terminou.",
                                notes.join("; ")
                            ),
                        );
                    }
                    notes.push(format!(
                        "{name}: R=0, idade {age} ≤ τ={tau}, ainda no conjunto"
                    ));
                    if candidate.is_none_or(|(_, a)| age > a) {
                        candidate = Some((k, age));
                    }
                }
                match candidate {
                    Some((k, _)) => (
                        k,
                        format!(
                            "Varredura: {}. Ninguém está fora do conjunto de trabalho: descartou \
                             a candidata mais antiga, a página {}.",
                            notes.join("; "),
                            self.page_name(k)
                        ),
                    ),
                    None => {
                        let k = (0..n).find(|&k| !self.slots[k].m).unwrap_or(0);
                        (
                            k,
                            format!(
                                "Varredura: {}. Todas com R=1: descartou uma página limpa, a {}.",
                                notes.join("; "),
                                self.page_name(k)
                            ),
                        )
                    }
                }
            }
            Algo::WsClock => {
                let mut notes: Vec<String> = Vec::new();
                let mut scheduled = false;
                let mut visited = 0usize;
                let mut turns = 0usize;
                loop {
                    let k = self.hand;
                    let name = self.page_name(k);
                    let s = &mut self.slots[k];
                    if s.r {
                        s.r = false;
                        s.ws_time = t;
                        notes.push(format!("{name}: R=1, zerado; tempo = {t}"));
                    } else {
                        let age = t.saturating_sub(s.ws_time);
                        if age > tau {
                            if !s.m {
                                notes.push(format!(
                                    "{name}: R=0, idade {age} > τ={tau}, limpa: descartada"
                                ));
                                return (k, format!("Ponteiro: {}.", notes.join("; ")));
                            }
                            s.m = false;
                            scheduled = true;
                            notes.push(format!(
                                "{name}: R=0, idade {age} > τ={tau}, suja: escrita agendada"
                            ));
                        } else {
                            notes.push(format!("{name}: R=0, idade {age} ≤ τ={tau}, no conjunto"));
                        }
                    }
                    self.hand = (self.hand + 1) % n;
                    visited += 1;
                    if visited >= n {
                        turns += 1;
                        visited = 0;
                        if scheduled && turns < 3 {
                            notes.push(
                                "volta completa com escritas agendadas: segue girando".into(),
                            );
                            scheduled = false;
                            continue;
                        }
                        // Todas no conjunto de trabalho: qualquer página limpa.
                        let k = (0..n)
                            .map(|j| (self.hand + j) % n)
                            .find(|&j| !self.slots[j].m)
                            .unwrap_or(self.hand);
                        notes.push(format!(
                            "volta completa sem nada a descartar: todas no conjunto de \
                             trabalho; descartou uma página limpa, a {}",
                            self.page_name(k)
                        ));
                        return (k, format!("Ponteiro: {}.", notes.join("; ")));
                    }
                }
            }
        }
    }
}

/// Distância até a próxima referência à página, a partir de `i` (exclusivo).
fn next_use(refs: &[Ref], i: usize, page: Option<u8>) -> Option<usize> {
    let page = page?;
    refs[i + 1..]
        .iter()
        .position(|r| r.page == page)
        .map(|d| d + 1)
}

/// Roda a simulação, referência a referência.
pub fn simulate(refs: &[Ref], algo: Algo, n_frames: usize, tick: u32, tau: u32) -> Sim {
    let n = n_frames.max(1);
    let mut m = Machine::new(n);
    let mut steps = Vec::with_capacity(refs.len());
    let mut faults = 0;

    for (i, r) in refs.iter().enumerate() {
        let t = i as u32 + 1;
        let (fault, frame, evicted, dirty, note) = if let Some(k) = m.find(r.page) {
            let s = &mut m.slots[k];
            s.r = true;
            if r.write {
                s.m = true;
            }
            s.last_ref = t;
            let bits = if r.write { "R e M ligados" } else { "R ligado" };
            (
                false,
                k,
                None,
                false,
                format!(
                    "Acerto: a página {} já está no quadro {k} ({bits}).",
                    r.page
                ),
            )
        } else if let Some(k) = m.free() {
            m.load(k, *r, t);
            (
                true,
                k,
                None,
                false,
                format!(
                    "Falta: a página {} foi carregada no quadro livre {k}.",
                    r.page
                ),
            )
        } else {
            let (k, why) = m.victim(algo, refs, i, t, tau);
            let old = m.slots[k].page;
            let dirty = m.slots[k].m;
            m.load(k, *r, t);
            if algo.has_hand() {
                m.hand = (k + 1) % n;
            }
            let disk = if dirty {
                " A vítima estava modificada: foi escrita em disco antes."
            } else {
                " A vítima estava limpa: apenas sobrescrita."
            };
            (
                true,
                k,
                old,
                dirty,
                format!(
                    "Falta, sem quadro livre. {why}{disk} A página {} entrou no quadro {k}.",
                    r.page
                ),
            )
        };
        if fault {
            faults += 1;
        }

        let ticked = algo.uses_tick() && tick > 0 && t % tick == 0;
        if ticked {
            m.tick(algo);
        }

        steps.push(Step {
            r: *r,
            fault,
            frame,
            evicted,
            dirty_evicted: dirty,
            note,
            slots: m.slots.clone(),
            hand: m.hand,
            ticked,
        });
    }

    Sim { steps, faults }
}

// ---------------------------------------------------------------------------
// Interface
// ---------------------------------------------------------------------------

pub fn show(ui: &mut egui::Ui, scenario: &Scenario, st: &mut ReplState) {
    let refs = parse_refs(&st.refs_text);
    let algo = scenario.algo;
    let sim = simulate(&refs, algo, st.frames as usize, st.tick, st.tau);
    let n_steps = sim.steps.len();

    // Reprodução: um passo a cada 1/speed segundos.
    if st.playing {
        st.clock += ui.input(|i| i.stable_dt).min(0.1) * st.speed;
        if st.clock >= 1.0 {
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
    if let Some(c) = st.cursor {
        if c + 1 >= n_steps {
            st.cursor = None;
        }
    }
    let shown = st.cursor.map_or(n_steps, |c| c + 1);

    widget_frame(ui).show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        widget_header(ui, &format!("Simulador — {}", algo.label()), scenario.hint);

        controls(ui, scenario, st, n_steps);
        ui.add_space(8.0);

        if refs.is_empty() {
            ui.label(
                egui::RichText::new(
                    "Digite uma cadeia de referências: números separados por espaço.",
                )
                .color(DANGER),
            );
            return;
        }

        if let Some(clicked) = grid(ui, &sim, st.frames as usize, shown) {
            st.cursor = Some(clicked);
            st.playing = false;
        }
        ui.label(
            egui::RichText::new(
                "✔ acerto · ✖ falta, seguida da página descartada (laranja: estava modificada e \
                 foi escrita em disco) · ⏰ interrupção de relógio · clique numa coluna para \
                 ver o passo",
            )
            .size(10.5)
            .color(egui::Color32::GRAY),
        );
        ui.add_space(6.0);

        let faults_shown = sim.steps[..shown].iter().filter(|s| s.fault).count();
        ui.label(
            egui::RichText::new(format!(
                "faltas de página: {faults_shown} em {shown} referências{}",
                if shown == n_steps {
                    format!("  ({:.0}%)", 100.0 * sim.faults as f32 / n_steps as f32)
                } else {
                    String::new()
                }
            ))
            .size(13.0)
            .strong()
            .color(ACCENT),
        );
        ui.add_space(8.0);

        let step = &sim.steps[shown - 1];
        step_detail(ui, algo, step, &refs, shown - 1, st.tau);
    });
}

fn controls(ui: &mut egui::Ui, scenario: &Scenario, st: &mut ReplState, n_steps: usize) {
    let algo = scenario.algo;
    ui.horizontal_wrapped(|ui| {
        let label = if st.playing { "⏸" } else { "▶" };
        if ui
            .button(label)
            .on_hover_text("Revelar as referências uma a uma")
            .clicked()
        {
            if st.playing {
                st.playing = false;
            } else {
                if st.cursor.is_none() {
                    st.cursor = Some(0);
                }
                st.clock = 0.0;
                st.playing = true;
            }
        }
        let cur = st.cursor.unwrap_or(n_steps.saturating_sub(1));
        if ui
            .add_enabled(cur > 0, egui::Button::new("◀"))
            .on_hover_text("Passo anterior")
            .clicked()
        {
            st.cursor = Some(cur - 1);
            st.playing = false;
        }
        if ui
            .add_enabled(cur + 1 < n_steps, egui::Button::new("▶|"))
            .on_hover_text("Próximo passo")
            .clicked()
        {
            st.cursor = Some(cur + 1);
            st.playing = false;
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
            egui::Slider::new(&mut st.speed, 0.5..=6.0)
                .show_value(false)
                .text("velocidade"),
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
    // Os parâmetros ficam numa grade rótulo | controle: sliders com rótulo não
    // quebram bem dentro de `horizontal_wrapped` e transbordariam a moldura.
    ui.add_space(4.0);
    params_grid(ui, ("params", scenario.id), |ui| {
        param_row(ui, "quadros", egui::Slider::new(&mut st.frames, 1..=7).integer());
        if algo.uses_tick() {
            param_row(
                ui,
                "relógio a cada",
                egui::Slider::new(&mut st.tick, 1..=8)
                    .suffix(" ref.")
                    .integer(),
            )
            .on_hover_text("Referências entre duas interrupções de relógio (que zeram os bits R)");
        }
        if algo.uses_tau() {
            param_row(
                ui,
                "τ",
                egui::Slider::new(&mut st.tau, 1..=12)
                    .suffix(" ref.")
                    .integer(),
            )
            .on_hover_text("Limiar de idade do conjunto de trabalho, em referências");
        }
    });
    ui.add_space(4.0);
    refs_field(ui, &mut st.refs_text, |st_changed| {
        if st_changed {
            st.cursor = None;
            st.playing = false;
        }
    });
}

/// O campo da cadeia de referências, elástico para caber na largura da tela.
fn refs_field(ui: &mut egui::Ui, text: &mut String, on_change: impl FnOnce(bool)) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("referências").size(12.0));
        let w = (ui.available_width() - 120.0).clamp(120.0, 560.0);
        let resp = ui.add(
            egui::TextEdit::singleline(text)
                .desired_width(w)
                .font(egui::TextStyle::Monospace),
        );
        ui.label(
            egui::RichText::new("(w = escrita)")
                .size(11.0)
                .color(egui::Color32::GRAY),
        );
        on_change(resp.changed());
    });
}

/// A grade referências × quadros. Devolve a coluna clicada, se houver.
fn grid(ui: &mut egui::Ui, sim: &Sim, n_frames: usize, shown: usize) -> Option<usize> {
    let n = sim.steps.len();
    if n == 0 {
        return None;
    }
    let visuals = ui.visuals().clone();
    const LABEL_W: f32 = 62.0;
    const HEAD_H: f32 = 22.0;
    const ROW_H: f32 = 24.0;
    const FOOT_H: f32 = 18.0;

    let width = ui.available_width();
    let cell_w = ((width - LABEL_W - 4.0) / n as f32).clamp(14.0, 34.0);
    let height = HEAD_H + n_frames as f32 * ROW_H + FOOT_H + 4.0;
    let (rect, resp) = ui.allocate_exact_size(
        egui::vec2(LABEL_W + cell_w * n as f32 + 4.0, height),
        egui::Sense::click(),
    );
    let painter = ui.painter_at(rect);
    let grid_line = visuals.text_color().gamma_multiply(0.18);
    let x0 = rect.left() + LABEL_W;
    let rows_top = rect.top() + HEAD_H;

    // Rótulos das linhas.
    painter.text(
        egui::pos2(rect.left() + 2.0, rect.top() + HEAD_H / 2.0),
        egui::Align2::LEFT_CENTER,
        "página",
        egui::FontId::proportional(10.5),
        visuals.weak_text_color(),
    );
    for f in 0..n_frames {
        painter.text(
            egui::pos2(rect.left() + 2.0, rows_top + f as f32 * ROW_H + ROW_H / 2.0),
            egui::Align2::LEFT_CENTER,
            format!("quadro {f}"),
            egui::FontId::proportional(10.5),
            visuals.weak_text_color(),
        );
    }

    let mut clicked = None;
    let hover = resp.hover_pos();

    for (i, step) in sim.steps.iter().enumerate() {
        let x = x0 + i as f32 * cell_w;
        let col =
            egui::Rect::from_min_size(egui::pos2(x, rect.top()), egui::vec2(cell_w, height - 4.0));
        let revealed = i < shown;
        let is_current = i + 1 == shown;

        if is_current {
            painter.rect_filled(col, 3.0, ACCENT.gamma_multiply(0.12));
        }
        if let Some(h) = hover {
            if col.contains(h) && ui.input(|inp| inp.pointer.primary_clicked()) {
                clicked = Some(i);
            }
        }

        // Cabeçalho: a página referenciada.
        let head_color = if !revealed {
            visuals.weak_text_color().gamma_multiply(0.5)
        } else if step.fault {
            DANGER
        } else {
            visuals.text_color()
        };
        painter.text(
            egui::pos2(x + cell_w / 2.0, rect.top() + HEAD_H / 2.0),
            egui::Align2::CENTER_CENTER,
            format!("{}{}", step.r.page, if step.r.write { "w" } else { "" }),
            egui::FontId::monospace(if step.r.write { 10.5 } else { 12.0 }),
            head_color,
        );

        if !revealed {
            continue;
        }

        // Células: o conteúdo de cada quadro depois da referência.
        for f in 0..n_frames {
            let cell = egui::Rect::from_min_size(
                egui::pos2(x, rows_top + f as f32 * ROW_H),
                egui::vec2(cell_w, ROW_H),
            );
            let slot = step.slots.get(f);
            let page = slot.and_then(|s| s.page);
            let touched = f == step.frame;
            let fill = if touched && step.fault {
                DANGER.gamma_multiply(0.30)
            } else if touched {
                SAFE.gamma_multiply(0.25)
            } else if page.is_some() {
                visuals.extreme_bg_color
            } else {
                egui::Color32::TRANSPARENT
            };
            painter.rect(
                cell.shrink(1.0),
                3.0,
                fill,
                egui::Stroke::new(0.8, grid_line),
                egui::StrokeKind::Inside,
            );
            if let Some(p) = page {
                painter.text(
                    cell.center(),
                    egui::Align2::CENTER_CENTER,
                    p.to_string(),
                    egui::FontId::monospace(12.0),
                    visuals.text_color(),
                );
                // Bits R e M, discretos, no canto.
                if let Some(s) = slot {
                    if cell_w >= 24.0 {
                        let mut tag = String::new();
                        if s.r {
                            tag.push('R');
                        }
                        if s.m {
                            tag.push('M');
                        }
                        painter.text(
                            egui::pos2(cell.right() - 2.0, cell.bottom() - 1.0),
                            egui::Align2::RIGHT_BOTTOM,
                            tag,
                            egui::FontId::proportional(7.5),
                            ACCENT,
                        );
                    }
                }
            }
        }

        // Rodapé: falta ou acerto e, na falta, a página descartada (em laranja
        // quando estava modificada e precisou ser escrita em disco).
        let (glyph, color) = if step.fault {
            ("✖", DANGER)
        } else {
            ("✔", SAFE)
        };
        let foot_y = rect.bottom() - 4.0 - FOOT_H / 2.0;
        match step.evicted {
            Some(p) if cell_w >= 22.0 => {
                painter.text(
                    egui::pos2(x + cell_w / 2.0 - 5.0, foot_y),
                    egui::Align2::CENTER_CENTER,
                    glyph,
                    egui::FontId::proportional(12.0),
                    color,
                );
                painter.text(
                    egui::pos2(x + cell_w / 2.0 + 5.0, foot_y),
                    egui::Align2::CENTER_CENTER,
                    p.to_string(),
                    egui::FontId::monospace(9.5),
                    if step.dirty_evicted {
                        DIRTY
                    } else {
                        visuals.weak_text_color()
                    },
                );
            }
            _ => {
                painter.text(
                    egui::pos2(x + cell_w / 2.0, foot_y),
                    egui::Align2::CENTER_CENTER,
                    glyph,
                    egui::FontId::proportional(12.0),
                    color,
                );
            }
        }

        // Interrupção de relógio: um traço entre as colunas.
        if step.ticked && i + 1 < n {
            let tx = x + cell_w;
            painter.line_segment(
                [
                    egui::pos2(tx, rect.top() + 2.0),
                    egui::pos2(tx, rect.bottom() - 4.0),
                ],
                egui::Stroke::new(1.5, ACCENT.gamma_multiply(0.6)),
            );
            painter.text(
                egui::pos2(tx, rect.top() + 1.0),
                egui::Align2::CENTER_TOP,
                "⏰",
                egui::FontId::proportional(8.0),
                ACCENT,
            );
        }
    }

    if hover.is_some() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    clicked
}

/// A explicação do passo e a tabela dos quadros (mais o anel, se houver ponteiro).
fn step_detail(ui: &mut egui::Ui, algo: Algo, step: &Step, refs: &[Ref], i: usize, tau: u32) {
    let t = i as u32 + 1;
    let (icon, fill) = if step.fault {
        ("✖", DANGER.gamma_multiply(0.12))
    } else {
        ("✔", SAFE.gamma_multiply(0.12))
    };
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(10, 8))
        .corner_radius(7.0)
        .fill(fill)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal_top(|ui| {
                ui.label(egui::RichText::new(icon).size(14.0));
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(format!(
                            "t = {t}, referência à página {}{}: {}",
                            step.r.page,
                            if step.r.write { " (escrita)" } else { "" },
                            step.note
                        ))
                        .size(12.5),
                    )
                    .wrap(),
                );
            });
        });
    ui.add_space(8.0);

    let n = step.slots.len();
    let extra: &[&str] = match algo {
        Algo::Optimal => &["próx. uso"],
        Algo::Nru => &["classe"],
        Algo::Fifo | Algo::SecondChance => &["carga"],
        Algo::Clock => &[],
        Algo::Lru => &["último uso"],
        Algo::Aging => &["contador"],
        Algo::WorkingSet | Algo::WsClock => &["último uso", "idade"],
    };

    let table = |ui: &mut egui::Ui| {
        egui::Grid::new(("repl_slots", algo.label()))
            .num_columns(4 + extra.len())
            .spacing([12.0, 3.0])
            .striped(true)
            .show(ui, |ui| {
                for h in ["quadro", "página", "R", "M"] {
                    ui.label(egui::RichText::new(h).size(11.0).strong());
                }
                for h in extra {
                    ui.label(egui::RichText::new(*h).size(11.0).strong());
                }
                ui.end_row();
                for (k, s) in step.slots.iter().enumerate() {
                    let mono = |txt: String| egui::RichText::new(txt).size(11.5).monospace();
                    let mut cell = mono(k.to_string());
                    if algo.has_hand() && step.hand == k {
                        cell = mono(format!("{k} ◀")).color(ACCENT);
                    }
                    ui.label(cell);
                    ui.label(mono(s.page.map_or("—".into(), |p| p.to_string())));
                    ui.label(mono(u8::from(s.r).to_string()));
                    ui.label(mono(u8::from(s.m).to_string()));
                    if s.page.is_some() {
                        match algo {
                            Algo::Optimal => {
                                let d = next_use(refs, i, s.page);
                                ui.label(mono(d.map_or("nunca".into(), |d| format!("+{d}"))));
                            }
                            Algo::Nru => {
                                ui.label(mono((2 * u8::from(s.r) + u8::from(s.m)).to_string()));
                            }
                            Algo::Fifo | Algo::SecondChance => {
                                ui.label(mono(format!("#{}", s.seq)));
                            }
                            Algo::Lru => {
                                ui.label(mono(format!("t={}", s.last_ref)));
                            }
                            Algo::Aging => {
                                ui.label(mono(format!("{:08b} = {}", s.age, s.age)));
                            }
                            Algo::WorkingSet | Algo::WsClock => {
                                ui.label(mono(format!("t={}", s.ws_time)));
                                let age = t.saturating_sub(s.ws_time);
                                let txt = mono(age.to_string());
                                ui.label(if age > tau { txt.color(DANGER) } else { txt });
                            }
                            Algo::Clock => {}
                        }
                    } else {
                        for _ in extra {
                            ui.label(mono("—".into()));
                        }
                    }
                    ui.end_row();
                }
            });
    };

    if algo.has_hand() && n >= 2 {
        // Anel ao lado da tabela quando há largura; abaixo dela quando não há
        // (a tabela do WSClock tem seis colunas e não pode ser espremida).
        if ui.available_width() >= 620.0 {
            ui.columns(2, |cols| {
                table(&mut cols[0]);
                clock_ring(&mut cols[1], step, algo);
            });
        } else {
            table(ui);
            ui.add_space(6.0);
            clock_ring(ui, step, algo);
        }
    } else {
        table(ui);
    }
}

/// O anel do relógio: os quadros em círculo e o ponteiro na página mais antiga.
fn clock_ring(ui: &mut egui::Ui, step: &Step, algo: Algo) {
    let visuals = ui.visuals().clone();
    let n = step.slots.len();
    let size = ui.available_width().min(190.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let center = rect.center();
    let radius = size / 2.0 - 34.0;
    let line = visuals.text_color().gamma_multiply(0.45);

    painter.circle_stroke(
        center,
        radius,
        egui::Stroke::new(1.0, line.gamma_multiply(0.6)),
    );

    for (k, s) in step.slots.iter().enumerate() {
        // Sentido horário a partir do topo.
        let ang = std::f32::consts::FRAC_PI_2 - k as f32 * std::f32::consts::TAU / n as f32;
        let pos = center + egui::vec2(ang.cos(), -ang.sin()) * radius;
        let is_hand = step.hand == k;
        let fill = if is_hand {
            ACCENT.gamma_multiply(0.35)
        } else {
            visuals.extreme_bg_color
        };
        painter.circle(
            pos,
            13.0,
            fill,
            egui::Stroke::new(
                if is_hand { 1.8 } else { 1.0 },
                if is_hand { ACCENT } else { line },
            ),
        );
        painter.text(
            pos,
            egui::Align2::CENTER_CENTER,
            s.page.map_or("—".to_string(), |p| p.to_string()),
            egui::FontId::monospace(12.0),
            visuals.text_color(),
        );
        let bits = if algo == Algo::WsClock {
            format!("R={} M={}", u8::from(s.r), u8::from(s.m))
        } else {
            format!("R={}", u8::from(s.r))
        };
        let out = center + egui::vec2(ang.cos(), -ang.sin()) * (radius + 25.0);
        painter.text(
            out,
            egui::Align2::CENTER_CENTER,
            bits,
            egui::FontId::proportional(9.0),
            visuals.weak_text_color(),
        );
    }

    // O ponteiro.
    let ang = std::f32::consts::FRAC_PI_2 - step.hand as f32 * std::f32::consts::TAU / n as f32;
    let tip = center + egui::vec2(ang.cos(), -ang.sin()) * (radius - 16.0);
    painter.arrow(center, tip - center, egui::Stroke::new(2.2, ACCENT));
    painter.circle_filled(center, 3.0, ACCENT);
}

// ---------------------------------------------------------------------------
// Comparação entre os algoritmos
// ---------------------------------------------------------------------------

/// Faltas de página de cada algoritmo na mesma cadeia, como barras.
pub fn compare(ui: &mut egui::Ui, scenario: &Scenario, st: &mut ReplState) {
    let refs = parse_refs(&st.refs_text);
    widget_frame(ui).show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        widget_header(ui, "Comparação", scenario.hint);

        params_grid(ui, ("params", scenario.id), |ui| {
            param_row(ui, "quadros", egui::Slider::new(&mut st.frames, 1..=7).integer());
            param_row(
                ui,
                "relógio a cada",
                egui::Slider::new(&mut st.tick, 1..=8)
                    .suffix(" ref.")
                    .integer(),
            );
            param_row(
                ui,
                "τ",
                egui::Slider::new(&mut st.tau, 1..=12)
                    .suffix(" ref.")
                    .integer(),
            );
        });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if ui
                .small_button("↺")
                .on_hover_text("Restaurar cenário")
                .clicked()
            {
                st.reset(scenario);
            }
        });
        refs_field(ui, &mut st.refs_text, |_| {});
        ui.add_space(8.0);

        if refs.is_empty() {
            return;
        }
        let results: Vec<(Algo, u32)> = Algo::ALL
            .iter()
            .map(|&a| {
                (
                    a,
                    simulate(&refs, a, st.frames as usize, st.tick, st.tau).faults,
                )
            })
            .collect();
        let max = results.iter().map(|r| r.1).max().unwrap_or(1).max(1);
        let best = results.iter().map(|r| r.1).min().unwrap_or(0);
        let total = refs.len() as u32;

        let visuals = ui.visuals().clone();
        const ROW_H: f32 = 22.0;
        const LABEL_W: f32 = 150.0;
        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(width, ROW_H * results.len() as f32 + 4.0),
            egui::Sense::hover(),
        );
        let painter = ui.painter_at(rect);
        let bar_w = (width - LABEL_W - 60.0).max(40.0);

        for (row, (algo, faults)) in results.iter().enumerate() {
            let y = rect.top() + row as f32 * ROW_H;
            painter.text(
                egui::pos2(rect.left() + LABEL_W - 8.0, y + ROW_H / 2.0),
                egui::Align2::RIGHT_CENTER,
                algo.label(),
                egui::FontId::proportional(12.0),
                visuals.text_color(),
            );
            let w = bar_w * *faults as f32 / max as f32;
            let bar = egui::Rect::from_min_size(
                egui::pos2(rect.left() + LABEL_W, y + 3.0),
                egui::vec2(w.max(2.0), ROW_H - 6.0),
            );
            let color = if *algo == Algo::Optimal {
                SAFE
            } else if *faults == best {
                ACCENT
            } else {
                ACCENT.gamma_multiply(0.55)
            };
            painter.rect_filled(bar, 3.0, color);
            painter.text(
                egui::pos2(bar.right() + 6.0, y + ROW_H / 2.0),
                egui::Align2::LEFT_CENTER,
                format!("{faults} ({:.0}%)", 100.0 * *faults as f32 / total as f32),
                egui::FontId::proportional(11.0),
                visuals.weak_text_color(),
            );
        }
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(
                "O ótimo é a referência: nenhum algoritmo implementável faz menos faltas que ele \
                 na mesma cadeia.",
            )
            .size(11.5)
            .color(egui::Color32::GRAY),
        );
    });
}

/// Validação de cenário, usada pelos testes de conteúdo.
#[cfg(test)]
pub fn check(s: &Scenario) {
    assert!(!s.hint.trim().is_empty(), "cenário '{}' sem dica", s.id);
    assert!(
        !parse_refs(s.refs).is_empty(),
        "cenário '{}' sem referências válidas",
        s.id
    );
    assert!(s.frames >= 1, "cenário '{}' sem quadros", s.id);
    assert!(
        s.tick >= 1 && s.tau >= 1,
        "cenário '{}' com relógio/τ zero",
        s.id
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn refs(s: &str) -> Vec<Ref> {
        parse_refs(s)
    }

    fn faults(s: &str, algo: Algo, frames: usize) -> u32 {
        simulate(&refs(s), algo, frames, 4, 6).faults
    }

    #[test]
    fn leitura_da_cadeia() {
        let r = parse_refs("7 0w 1 x 2W");
        assert_eq!(r.len(), 4);
        assert!(r[1].write && r[3].write && !r[0].write);
        assert_eq!(r[3].page, 2);
    }

    /// Todo algoritmo carrega a página referenciada: depois de cada passo ela
    /// está residente, e nunca há duas cópias da mesma página.
    #[test]
    fn a_pagina_referenciada_fica_residente() {
        let s = "7 0 1 2 0 3w 0 4 2 3 0 3w 2 1 2 0 1 7 0 1";
        for algo in Algo::ALL {
            for frames in 1..=5 {
                let sim = simulate(&refs(s), algo, frames, 3, 5);
                for step in &sim.steps {
                    let residents: Vec<u8> = step.slots.iter().filter_map(|x| x.page).collect();
                    assert!(
                        residents.contains(&step.r.page),
                        "{algo:?}/{frames}: {step:?}"
                    );
                    let mut uniq = residents.clone();
                    uniq.sort_unstable();
                    uniq.dedup();
                    assert_eq!(uniq.len(), residents.len(), "{algo:?}: página duplicada");
                }
            }
        }
    }

    /// Com quadros suficientes para tudo, só há faltas compulsórias.
    #[test]
    fn sem_substituicao_so_ha_faltas_compulsorias() {
        let s = "1 2 3 1 2 3 4 1 2";
        for algo in Algo::ALL {
            assert_eq!(faults(s, algo, 4), 4, "{algo:?}");
        }
    }

    /// FIFO sofre da anomalia de Belady: mais quadros, mais faltas.
    #[test]
    fn belady_no_fifo() {
        let s = "1 2 3 4 1 2 5 1 2 3 4 5";
        assert_eq!(faults(s, Algo::Fifo, 3), 9);
        assert_eq!(faults(s, Algo::Fifo, 4), 10);
    }

    /// O ótimo nunca perde para os outros na mesma cadeia.
    #[test]
    fn otimo_e_o_menor() {
        let cadeias = [
            "7 0 1 2 0 3 0 4 2 3 0 3 2 1 2 0 1 7 0 1",
            "1 2 3 4 1 2 5 1 2 3 4 5",
            "0 1 2 3 0 1 4 0 1 2 3 4",
            "3 1w 4 1 5 9w 2 6 5 3 5 8 9 7 9 3 2 3 8 4",
        ];
        for s in cadeias {
            for frames in 1..=5 {
                let best = faults(s, Algo::Optimal, frames);
                for algo in Algo::ALL {
                    assert!(
                        faults(s, algo, frames) >= best,
                        "{algo:?} com {frames} quadros bateu o ótimo em '{s}'"
                    );
                }
            }
        }
    }

    /// Exemplo clássico: 20 referências, 3 quadros.
    #[test]
    fn valores_conhecidos() {
        let s = "7 0 1 2 0 3 0 4 2 3 0 3 2 1 2 0 1 7 0 1";
        assert_eq!(faults(s, Algo::Optimal, 3), 9);
        assert_eq!(faults(s, Algo::Fifo, 3), 15);
        assert_eq!(faults(s, Algo::Lru, 3), 12);
    }

    /// Segunda chance e relógio tomam as mesmas decisões.
    #[test]
    fn relogio_equivale_a_segunda_chance() {
        let cadeias = [
            "7 0 1 2 0 3 0 4 2 3 0 3 2 1 2 0 1 7 0 1",
            "1 2 3 4 1 2 5 1 2 3 4 5",
            "2 3 2 1 5 2 4 5 3 2 5 2",
        ];
        for s in cadeias {
            for frames in 1..=5 {
                let a = simulate(&refs(s), Algo::SecondChance, frames, 4, 6);
                let b = simulate(&refs(s), Algo::Clock, frames, 4, 6);
                let ea: Vec<_> = a.steps.iter().map(|x| (x.fault, x.evicted)).collect();
                let eb: Vec<_> = b.steps.iter().map(|x| (x.fault, x.evicted)).collect();
                assert_eq!(ea, eb, "'{s}' com {frames} quadros");
            }
        }
    }

    /// O relógio dá segunda chance: uma página com R=1 não é descartada.
    #[test]
    fn relogio_poupa_quem_tem_r_ligado() {
        // 1 2 3 enchem (todas com R=1). 4 falta: o ponteiro dá a volta desligando
        // os R e tira a 1, a mais antiga. 2 é referenciada (R=1). 5 falta: o
        // ponteiro pula a 2 e tira a 3.
        let sim = simulate(&refs("1 2 3 4 2 5"), Algo::Clock, 3, 4, 6);
        assert_eq!(sim.steps[3].evicted, Some(1));
        assert_eq!(sim.steps[3].hand, 1);
        let last = sim.steps.last().unwrap();
        assert!(last.fault);
        assert_eq!(last.evicted, Some(3));
        assert_eq!(
            last.hand, 0,
            "o ponteiro avança para depois do quadro usado"
        );
    }

    /// O envelhecimento reproduz a decisão do LRU: a página usada mais
    /// recentemente tem o bit mais alto ligado e vence, como em 3.4.38.
    #[test]
    fn envelhecimento_do_exemplo() {
        // Relógio a cada referência. As duas páginas alternam; no fim só a 2 é
        // usada. Com 2 quadros, 3 falta e deve tirar a 1.
        let s = "1 2 1 2 1 2 1 2 1 2 1 2 1 2 2 2 3";
        let sim = simulate(&refs(s), Algo::Aging, 2, 1, 6);
        let last = sim.steps.last().unwrap();
        assert!(last.fault);
        // Depois da 16ª referência (t=16), 2 foi referenciada por último.
        let before = &sim.steps[15];
        let age_of = |p: u8| before.slots.iter().find(|x| x.page == Some(p)).unwrap().age;
        assert!(
            age_of(2) > age_of(1),
            "quem foi usada por último tem contador maior"
        );
        assert_eq!(last.evicted, Some(1));
    }

    /// NRU descarta da menor classe: uma página suja e não referenciada sai
    /// antes de uma limpa e referenciada.
    #[test]
    fn nru_prefere_a_classe_mais_baixa() {
        // 1w 2 enchem (M de 1 ligado); relógio a cada 2 refs desliga R de ambas;
        // 2 é referenciada de novo (R=1); 3 falta → classe 1 (0,1) é a menor: sai 1.
        let sim = simulate(&refs("1w 2 2 3"), Algo::Nru, 2, 2, 6);
        let last = sim.steps.last().unwrap();
        assert_eq!(last.evicted, Some(1));
        assert!(last.dirty_evicted);
    }

    /// Conjunto de trabalho: a página fora do conjunto (idade > τ) é a descartada.
    #[test]
    fn conjunto_de_trabalho_descarta_quem_envelheceu() {
        // 1 2 3 enchem; 2 e 3 seguem em uso; 1 fica parada além de τ=3.
        let sim = simulate(&refs("1 2 3 2 3 2 3 4"), Algo::WorkingSet, 3, 4, 3);
        let last = sim.steps.last().unwrap();
        assert!(last.fault);
        assert_eq!(last.evicted, Some(1));
    }

    /// WSClock: a página suja fora do conjunto tem a escrita agendada e só sai
    /// na volta seguinte; as páginas em uso têm o tempo atualizado.
    #[test]
    fn wsclock_agenda_a_escrita_da_suja() {
        let sim = simulate(&refs("1w 2 3 2 3 2 3 4"), Algo::WsClock, 3, 4, 3);
        let last = sim.steps.last().unwrap();
        assert!(last.fault);
        assert_eq!(last.evicted, Some(1));
        assert!(last.note.contains("escrita agendada"));
        assert!(last.note.contains("segue girando"));
        let ws_of = |p: u8| {
            last.slots
                .iter()
                .find(|x| x.page == Some(p))
                .unwrap()
                .ws_time
        };
        assert_eq!(ws_of(2), 8);
        assert_eq!(ws_of(3), 8);
    }

    /// A interrupção de relógio desliga os bits R de quem a usa.
    #[test]
    fn relogio_desliga_r() {
        let sim = simulate(&refs("1 2 3 4"), Algo::Nru, 4, 2, 6);
        assert!(sim.steps[1].ticked);
        assert!(sim.steps[1].slots.iter().all(|s| !s.r));
        assert!(!sim.steps[2].ticked);
        assert!(sim.steps[2].slots.iter().any(|s| s.r));
    }
}
