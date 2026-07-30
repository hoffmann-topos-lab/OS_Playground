//! Leitor de artigos reutilizado pelas três telas de conteúdo (Conceitos, Race
//! Conditions e Escalonamento).
//!
//! Aqui ficam o *modelo* do conteúdo ([`Article`], [`Block`]) e a renderização.
//! Os artigos em si moram em `content*.rs`. O rodapé de referências é montado
//! automaticamente ao final de cada artigo.

use std::collections::HashMap;

use eframe::egui;

use crate::app::Screen;
use crate::ui::demo::{self, Demo, DemoState};
use crate::ui::gantt::{self, SchedState};
use crate::ui::ACCENT;

const TANENBAUM: &str =
    "TANENBAUM, A. S.; BOS, H. Sistemas Operacionais Modernos — capítulo 2, seção";
const OSTEP: &str = "ARPACI-DUSSEAU, R. H.; ARPACI-DUSSEAU, A. C. \
                     Operating Systems: Three Easy Pieces";
const OSTEP_URL: &str = "https://pages.cs.wisc.edu/~remzi/OSTEP/";

/// Um capítulo do OSTEP citado por um artigo: rótulo + link direto do PDF.
///
/// Os capítulos e as URLs vêm do índice em <https://pages.cs.wisc.edu/~remzi/OSTEP/>.
pub struct OstepRef {
    pub label: &'static str,
    pub url: &'static str,
}

/// Capítulos do OSTEP usados pelos artigos.
///
/// Números, títulos e URLs conferidos no índice do livro. Ao acrescentar um
/// capítulo, confira o link em vez de deduzir o nome do arquivo.
pub mod ostep {
    use super::OstepRef;

    macro_rules! chapter {
        ($name:ident, $label:literal, $file:literal) => {
            pub const $name: OstepRef = OstepRef {
                label: $label,
                url: concat!("https://pages.cs.wisc.edu/~remzi/OSTEP/", $file),
            };
        };
    }

    chapter!(PROCESSES, "capítulo 4 — Processes", "cpu-intro.pdf");
    chapter!(PROCESS_API, "capítulo 5 — Process API", "cpu-api.pdf");
    chapter!(
        DIRECT_EXECUTION,
        "capítulo 6 — Direct Execution",
        "cpu-mechanisms.pdf"
    );
    chapter!(SCHEDULING, "capítulo 7 — CPU Scheduling", "cpu-sched.pdf");
    chapter!(
        MLFQ,
        "capítulo 8 — Multi-level Feedback",
        "cpu-sched-mlfq.pdf"
    );
    chapter!(
        LOTTERY,
        "capítulo 9 — Lottery Scheduling",
        "cpu-sched-lottery.pdf"
    );
    chapter!(
        THREADS,
        "capítulo 26 — Concurrency and Threads",
        "threads-intro.pdf"
    );
    chapter!(THREAD_API, "capítulo 27 — Thread API", "threads-api.pdf");
    chapter!(LOCKS, "capítulo 28 — Locks", "threads-locks.pdf");
    chapter!(
        LOCKED_DATA,
        "capítulo 29 — Locked Data Structures",
        "threads-locks-usage.pdf"
    );
    chapter!(COND_VARS, "capítulo 30 — Condition Variables", "threads-cv.pdf");
    chapter!(SEMAPHORES, "capítulo 31 — Semaphores", "threads-sema.pdf");
    chapter!(
        CONC_BUGS,
        "capítulo 32 — Concurrency Bugs",
        "threads-bugs.pdf"
    );
}

/// Um bloco de conteúdo dentro de um artigo.
pub enum Block {
    /// Subtítulo dentro do artigo.
    H(&'static str),
    /// Parágrafo corrido.
    P(&'static str),
    /// Lista com marcadores.
    L(&'static [&'static str]),
    /// Lista numerada.
    N(&'static [&'static str]),
    /// Pares "termo — definição".
    Defs(&'static [(&'static str, &'static str)]),
    /// Caixa de destaque.
    Note(&'static str),
    /// Caixa de alerta (armadilha, condição violada).
    Warn(&'static str),
    /// Trecho de pseudocódigo.
    Code(&'static [&'static str]),
    /// Tabela de duas colunas: (cabeçalho A, cabeçalho B, linhas).
    Table2(
        &'static str,
        &'static str,
        &'static [(&'static str, &'static str)],
    ),
    /// Diagrama desenhado dos três estados de um processo.
    StateDiagram,
    /// Demonstração passo a passo de duas linhas de execução concorrentes.
    Demo(&'static Demo),
    /// Simulador de escalonamento com diagrama de Gantt.
    Scheduler(&'static gantt::Scenario),
    /// Convite para abrir outra tela do app.
    SeeAlso(&'static str, Screen),
}

/// Um artigo.
pub struct Article {
    pub icon: &'static str,
    pub title: &'static str,
    /// Seção correspondente no Tanenbaum, usada também nas referências.
    pub section: &'static str,
    /// Capítulos do OSTEP que tratam do mesmo assunto. Pode ficar vazio: nem
    /// todo recorte das anotações tem equivalente no OSTEP, e nesse caso o
    /// rodapé cita apenas o Tanenbaum em vez de forçar uma remissão frouxa.
    pub ostep: &'static [OstepRef],
    pub summary: &'static str,
    pub blocks: &'static [Block],
}

/// Estado de uma tela-leitor: artigo aberto e o estado dos widgets embutidos.
#[derive(Default)]
pub struct ReaderState {
    pub selected: usize,
    demos: HashMap<&'static str, DemoState>,
    scheds: HashMap<&'static str, SchedState>,
}

/// Desenha índice + artigo. Devolve a tela para onde navegar, se houver clique.
pub fn show(
    ui: &mut egui::Ui,
    articles: &'static [Article],
    subtitle: &str,
    st: &mut ReaderState,
) -> Option<Screen> {
    if articles.is_empty() {
        return None;
    }
    let mut selected = st.selected.min(articles.len() - 1);
    let mut goto: Option<Screen> = None;

    egui::Panel::left("reader_nav")
        .resizable(true)
        .default_size(280.0)
        .min_size(210.0)
        .max_size(380.0)
        .show(ui, |ui| {
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Índice").size(16.0).strong());
            ui.label(
                egui::RichText::new(subtitle)
                    .size(11.5)
                    .color(egui::Color32::GRAY),
            );
            ui.add_space(6.0);
            ui.separator();

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for (i, art) in articles.iter().enumerate() {
                        if index_entry(ui, art, i == selected).clicked() {
                            selected = i;
                        }
                    }
                });
        });

    egui::CentralPanel::default().show(ui, |ui| {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            // Um estado de rolagem por artigo: trocar de artigo começa do topo.
            .id_salt(selected)
            .show(ui, |ui| {
                ui.add_space(10.0);
                goto = article_ui(ui, &articles[selected], st);
                ui.add_space(18.0);
                if let Some(next) = pager(ui, articles, selected) {
                    selected = next;
                }
                ui.add_space(16.0);
            });
    });

    st.selected = selected;
    goto
}

fn index_entry(ui: &mut egui::Ui, art: &Article, is_sel: bool) -> egui::Response {
    let frame = egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(8, 7))
        .corner_radius(7.0)
        .fill(if is_sel {
            ui.visuals().selection.bg_fill.gamma_multiply(0.5)
        } else {
            egui::Color32::TRANSPARENT
        });

    let resp = frame
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(art.icon).size(17.0));
                ui.add_space(2.0);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    ui.label(egui::RichText::new(art.title).size(13.5).strong());
                    ui.label(
                        egui::RichText::new(format!("§ {}", art.section))
                            .size(10.5)
                            .color(egui::Color32::GRAY),
                    );
                });
            });
        })
        .response
        .interact(egui::Sense::click());

    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

fn article_ui(ui: &mut egui::Ui, art: &Article, st: &mut ReaderState) -> Option<Screen> {
    let mut goto = None;

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(art.icon).size(30.0));
        ui.add_space(4.0);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.label(egui::RichText::new(art.title).size(25.0).strong());
            ui.label(
                egui::RichText::new(format!("§ {}  ·  {}", art.section, art.summary))
                    .size(12.5)
                    .color(egui::Color32::GRAY),
            );
        });
    });
    ui.add_space(10.0);
    ui.separator();
    ui.add_space(6.0);

    for block in art.blocks {
        if let Some(s) = block_ui(ui, block, st) {
            goto = Some(s);
        }
    }

    ui.add_space(20.0);
    references(ui, art);
    goto
}

fn block_ui(ui: &mut egui::Ui, block: &Block, st: &mut ReaderState) -> Option<Screen> {
    let mut goto = None;
    match block {
        Block::H(t) => {
            ui.add_space(12.0);
            ui.label(egui::RichText::new(*t).size(17.0).strong().color(ACCENT));
            ui.add_space(2.0);
        }
        Block::P(t) => {
            ui.add_space(4.0);
            ui.label(egui::RichText::new(*t).size(14.5));
        }
        Block::L(items) => {
            ui.add_space(4.0);
            for it in *items {
                bullet_row(ui, "•", it);
            }
        }
        Block::N(items) => {
            ui.add_space(4.0);
            for (i, it) in items.iter().enumerate() {
                bullet_row(ui, &format!("{}.", i + 1), it);
            }
        }
        Block::Defs(pairs) => {
            ui.add_space(6.0);
            for (term, def) in *pairs {
                ui.horizontal_top(|ui| {
                    ui.add_space(10.0);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        ui.label(
                            egui::RichText::new(*term)
                                .size(14.0)
                                .strong()
                                .monospace()
                                .color(ACCENT),
                        );
                        ui.label(egui::RichText::new(*def).size(14.0));
                    });
                });
                ui.add_space(8.0);
            }
        }
        Block::Note(t) => callout(ui, "💡", t, ACCENT.gamma_multiply(0.10)),
        Block::Warn(t) => callout(
            ui,
            "⚠",
            t,
            egui::Color32::from_rgb(0xff, 0x8a, 0x5b).gamma_multiply(0.14),
        ),
        Block::Code(lines) => {
            ui.add_space(8.0);
            egui::Frame::new()
                .inner_margin(egui::Margin::symmetric(12, 10))
                .corner_radius(7.0)
                .fill(ui.visuals().extreme_bg_color)
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        for l in *lines {
                            ui.label(egui::RichText::new(*l).monospace().size(12.5));
                        }
                    });
                });
            ui.add_space(4.0);
        }
        Block::Table2(head_a, head_b, rows) => {
            ui.add_space(10.0);
            egui::Grid::new(("tbl", *head_a, *head_b, rows.len()))
                .num_columns(2)
                .spacing([16.0, 9.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new(*head_a).strong().color(ACCENT));
                    ui.label(egui::RichText::new(*head_b).strong().color(ACCENT));
                    ui.end_row();
                    for (a, b) in *rows {
                        ui.label(egui::RichText::new(*a).size(13.5));
                        ui.label(egui::RichText::new(*b).size(13.5));
                        ui.end_row();
                    }
                });
            ui.add_space(4.0);
        }
        Block::StateDiagram => {
            ui.add_space(12.0);
            state_diagram(ui);
            ui.add_space(6.0);
        }
        Block::Demo(d) => {
            ui.add_space(12.0);
            let state = st.demos.entry(d.id).or_default();
            demo::show(ui, d, state);
            ui.add_space(6.0);
        }
        Block::Scheduler(scenario) => {
            ui.add_space(12.0);
            let state = st
                .scheds
                .entry(scenario.id)
                .or_insert_with(|| SchedState::new(scenario));
            gantt::show(ui, scenario, state);
            ui.add_space(6.0);
        }
        Block::SeeAlso(t, screen) => {
            ui.add_space(10.0);
            egui::Frame::new()
                .inner_margin(egui::Margin::symmetric(12, 9))
                .corner_radius(8.0)
                .fill(ui.visuals().faint_bg_color)
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("↪").size(15.0).color(ACCENT));
                        ui.label(egui::RichText::new(*t).size(13.5));
                        if ui.button(format!("Abrir  {}", screen.title())).clicked() {
                            goto = Some(*screen);
                        }
                    });
                });
        }
    }
    goto
}

fn callout(ui: &mut egui::Ui, icon: &str, text: &str, fill: egui::Color32) {
    ui.add_space(10.0);
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(12, 10))
        .corner_radius(8.0)
        .fill(fill)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal_top(|ui| {
                ui.label(egui::RichText::new(icon).size(15.0));
                ui.add_space(2.0);
                ui.label(egui::RichText::new(text).size(14.0));
            });
        });
    ui.add_space(4.0);
}

fn bullet_row(ui: &mut egui::Ui, marker: &str, text: &str) {
    ui.horizontal_top(|ui| {
        ui.add_space(12.0);
        ui.label(
            egui::RichText::new(marker)
                .size(14.0)
                .color(ACCENT)
                .monospace(),
        );
        ui.add_space(2.0);
        ui.label(egui::RichText::new(text).size(14.5));
    });
    ui.add_space(3.0);
}

fn pager(ui: &mut egui::Ui, articles: &'static [Article], selected: usize) -> Option<usize> {
    let mut next = None;
    ui.separator();
    ui.add_space(8.0);
    ui.horizontal_wrapped(|ui| {
        if selected > 0 {
            let prev = &articles[selected - 1];
            if ui
                .button(format!("⬅  {}  {}", prev.icon, prev.title))
                .clicked()
            {
                next = Some(selected - 1);
            }
        }
        if selected + 1 < articles.len() {
            let following = &articles[selected + 1];
            if ui
                .button(format!("{}  {}  ➡", following.icon, following.title))
                .clicked()
            {
                next = Some(selected + 1);
            }
        }
    });
    next
}

fn references(ui: &mut egui::Ui, art: &Article) {
    ui.separator();
    ui.add_space(8.0);
    ui.label(egui::RichText::new("Referências").size(15.0).strong());
    ui.add_space(6.0);

    ui.horizontal_top(|ui| {
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(format!("{TANENBAUM} {}.", art.section))
                .size(12.5)
                .color(egui::Color32::GRAY),
        );
    });

    // Artigos sem equivalente no OSTEP citam só o Tanenbaum.
    if art.ostep.is_empty() {
        return;
    }

    ui.add_space(6.0);
    ui.horizontal_top(|ui| {
        ui.add_space(6.0);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 3.0;
            ui.label(
                egui::RichText::new(OSTEP)
                    .size(12.5)
                    .color(egui::Color32::GRAY),
            );
            for r in art.ostep {
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("↳")
                            .size(12.0)
                            .color(egui::Color32::GRAY),
                    );
                    ui.hyperlink_to(egui::RichText::new(r.label).size(12.5), r.url);
                });
            }
            ui.add_space(1.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("Livro completo:")
                        .size(12.0)
                        .color(egui::Color32::GRAY),
                );
                ui.hyperlink_to(egui::RichText::new(OSTEP_URL).size(12.0), OSTEP_URL);
            });
        });
    });
}

// ---------------------------------------------------------------------------
// Diagrama dos estados de um processo (seção 2.1.5)
// ---------------------------------------------------------------------------

/// Desenha os três estados e as quatro transições, numeradas como na lista.
fn state_diagram(ui: &mut egui::Ui) {
    let width = ui.available_width().min(560.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 240.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let visuals = ui.visuals().clone();

    let half = egui::vec2(78.0, 24.0);
    // Recuo lateral das caixas de baixo, sem deixar que elas vazem do quadro.
    let inset = (width * 0.21).max(half.x + 6.0);
    let base_y = rect.bottom() - half.y - 34.0;
    let running = egui::pos2(rect.center().x, rect.top() + half.y + 10.0);
    let blocked = egui::pos2(rect.left() + inset, base_y);
    let ready = egui::pos2(rect.right() - inset, base_y);

    let line = visuals.text_color().gamma_multiply(0.55);
    let stroke = egui::Stroke::new(1.6, line);

    let edge = |center: egui::Pos2, dir: egui::Vec2| -> egui::Pos2 {
        let tx = if dir.x.abs() > 1e-6 {
            half.x / dir.x.abs()
        } else {
            f32::INFINITY
        };
        let ty = if dir.y.abs() > 1e-6 {
            half.y / dir.y.abs()
        } else {
            f32::INFINITY
        };
        center + dir * tx.min(ty)
    };

    let arrow = |a: egui::Pos2, b: egui::Pos2, offset: f32, n: &str| {
        let dir = (b - a).normalized();
        let perp = egui::vec2(-dir.y, dir.x) * offset;
        let start = edge(a, dir) + perp;
        let end = edge(b, -dir) + perp;
        painter.arrow(start, end - start, stroke);
        let mid = start + (end - start) * 0.5;
        painter.circle_filled(mid, 9.5, visuals.extreme_bg_color);
        painter.circle_stroke(mid, 9.5, egui::Stroke::new(1.2, line));
        painter.text(
            mid,
            egui::Align2::CENTER_CENTER,
            n,
            egui::FontId::proportional(11.0),
            visuals.text_color(),
        );
    };

    arrow(running, blocked, 0.0, "1");
    arrow(running, ready, 13.0, "2");
    arrow(ready, running, 13.0, "3");
    arrow(blocked, ready, 0.0, "4");

    let state_box = |center: egui::Pos2, label: &str, accent: bool| {
        let r = egui::Rect::from_center_size(center, half * 2.0);
        let fill = if accent {
            ACCENT.gamma_multiply(0.22)
        } else {
            visuals.widgets.inactive.bg_fill
        };
        let border = if accent { ACCENT } else { line };
        painter.rect(
            r,
            9.0,
            fill,
            egui::Stroke::new(1.4, border),
            egui::StrokeKind::Inside,
        );
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(14.0),
            visuals.text_color(),
        );
    };

    state_box(running, "Em execução", true);
    state_box(blocked, "Bloqueado", false);
    state_box(ready, "Pronto", false);

    painter.text(
        egui::pos2(rect.center().x, rect.bottom() - 10.0),
        egui::Align2::CENTER_CENTER,
        "As transições 2 e 3 são decisão do escalonador.",
        egui::FontId::proportional(11.5),
        visuals.weak_text_color(),
    );
}

/// Validação de conteúdo, compartilhada pelos três módulos de artigos.
///
/// Fica aqui para os três serem checados pelas mesmas regras — e para uma
/// variante nova de [`Block`] quebrar a compilação dos testes até ser coberta.
#[cfg(test)]
pub mod check {
    use super::*;
    use crate::ui::demo;

    pub fn articles(list: &'static [Article]) {
        assert!(!list.is_empty(), "lista de artigos vazia");
        for a in list {
            assert!(!a.title.trim().is_empty(), "artigo sem título");
            assert!(!a.summary.trim().is_empty(), "'{}' sem resumo", a.title);
            assert!(!a.blocks.is_empty(), "'{}' sem blocos", a.title);
            assert!(
                a.section.starts_with('2'),
                "'{}' deveria citar uma seção do capítulo 2, veio '{}'",
                a.title,
                a.section
            );
            for r in a.ostep {
                assert!(
                    r.label.starts_with("capítulo "),
                    "rótulo do OSTEP malformado: '{}'",
                    r.label
                );
                assert!(
                    r.url.starts_with("https://pages.cs.wisc.edu/~remzi/OSTEP/")
                        && r.url.ends_with(".pdf"),
                    "URL do OSTEP inesperada em '{}': {}",
                    a.title,
                    r.url
                );
            }
            for b in a.blocks {
                block(b, a.title);
            }
        }
    }

    fn block(b: &'static Block, title: &str) {
        let nonempty = |t: &str| assert!(!t.trim().is_empty(), "texto vazio em '{title}'");
        match b {
            Block::H(t) | Block::P(t) | Block::Note(t) | Block::Warn(t) => nonempty(t),
            Block::L(items) | Block::N(items) | Block::Code(items) => {
                assert!(!items.is_empty(), "lista vazia em '{title}'");
                items.iter().for_each(|i| nonempty(i));
            }
            Block::Defs(pairs) => {
                assert!(!pairs.is_empty(), "definições vazias em '{title}'");
                for (t, d) in *pairs {
                    nonempty(t);
                    nonempty(d);
                }
            }
            Block::Table2(ha, hb, rows) => {
                nonempty(ha);
                nonempty(hb);
                assert!(!rows.is_empty(), "tabela vazia em '{title}'");
                // A 2ª coluna pode ficar vazia quando um lado tem menos itens.
                rows.iter().for_each(|(x, _)| nonempty(x));
            }
            Block::SeeAlso(t, _) => nonempty(t),
            Block::Demo(d) => demo::check(d),
            Block::Scheduler(s) => {
                nonempty(s.hint);
                assert!(!s.jobs.is_empty(), "cenário '{}' sem processos", s.id);
                assert!(s.quantum >= 1, "cenário '{}' com quantum 0", s.id);
                for j in s.jobs {
                    assert!(j.burst >= 1, "processo '{}' com duração 0", j.name);
                }
            }
            Block::StateDiagram => {}
        }
    }
}
