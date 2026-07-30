//! Tela "Árvore de Processos": hierarquia real do sistema, ao vivo.
//!
//! O desenho em si (caixas + ramos, com pan/zoom) fica em [`crate::ui::tree_view`];
//! aqui ficam a captura, a barra de ferramentas, o filtro e o painel de detalhes.

use std::collections::HashSet;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use eframe::egui;

use crate::app::OsPlaygroundApp;
use crate::sys::ProcessSnapshot;
use crate::ui::tree_view::{self, Orientation, TreeViewState};
use crate::ui::ACCENT;

pub struct ProcessTreeState {
    snapshot: Option<ProcessSnapshot>,
    filter: String,
    selected: Option<u32>,
    auto_refresh: bool,
    last_capture: Option<Instant>,
    tree: TreeViewState,
}

impl Default for ProcessTreeState {
    fn default() -> Self {
        Self {
            snapshot: None,
            filter: String::new(),
            selected: None,
            auto_refresh: false,
            last_capture: None,
            tree: TreeViewState::default(),
        }
    }
}

impl ProcessTreeState {
    fn refresh(&mut self) {
        self.snapshot = Some(ProcessSnapshot::capture());
        self.last_capture = Some(Instant::now());
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        if self.snapshot.is_none() {
            self.refresh();
        }
        // Auto-refresh a cada ~2s, se ligado.
        if self.auto_refresh {
            let due = self
                .last_capture
                .map(|t| t.elapsed() >= Duration::from_secs(2))
                .unwrap_or(true);
            if due {
                self.refresh();
            }
            ui.ctx().request_repaint_after(Duration::from_millis(500));
        }

        // ---- Barra de ferramentas ----
        let mut fit_requested = false;
        let mut expand_all = false;
        let mut collapse_all = false;
        egui::Panel::top("proc_toolbar")
            .exact_size(38.0)
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    if ui.button("🔄 Atualizar").clicked() {
                        self.refresh();
                    }
                    ui.checkbox(&mut self.auto_refresh, "Auto (2s)");
                    ui.separator();
                    ui.label("🔎");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.filter)
                            .hint_text("filtrar por nome ou PID")
                            .desired_width(180.0),
                    );
                    if !self.filter.is_empty() && ui.small_button("✕").clicked() {
                        self.filter.clear();
                    }
                    ui.separator();

                    // Controles do desenho da árvore.
                    let vertical = self.tree.orientation == Orientation::Vertical;
                    let (icon, hint) = if vertical {
                        ("⬇", "Raiz em cima — clique para deitar a árvore")
                    } else {
                        ("➡", "Raiz à esquerda — clique para levantar a árvore")
                    };
                    if ui.button(icon).on_hover_text(hint).clicked() {
                        self.tree.orientation = if vertical {
                            Orientation::Horizontal
                        } else {
                            Orientation::Vertical
                        };
                        fit_requested = true;
                    }
                    fit_requested |= ui
                        .button("⛶")
                        .on_hover_text("Ajustar à tela")
                        .clicked();
                    expand_all = ui
                        .button("⊞")
                        .on_hover_text("Expandir tudo")
                        .clicked();
                    collapse_all = ui
                        .button("⊟")
                        .on_hover_text("Recolher tudo")
                        .clicked();

                    if let Some(snap) = &self.snapshot {
                        ui.separator();
                        ui.label(
                            egui::RichText::new(format!(
                                "{} processos · zoom {:.0}%",
                                snap.total,
                                self.tree.zoom * 100.0
                            ))
                            .color(egui::Color32::GRAY),
                        );
                    }
                });
            });

        let filter = self.filter.trim().to_lowercase();
        let mut selected = self.selected;
        let snap = match self.snapshot.as_ref() {
            Some(s) => s,
            None => return,
        };

        if fit_requested {
            self.tree.request_fit();
        }
        if expand_all {
            self.tree.expand_all();
        }
        if collapse_all {
            self.tree.collapse_all(snap);
        }

        // Conjunto de PIDs visíveis quando há filtro (correspondências +
        // ancestrais + subárvore de cada correspondência).
        let keep: Option<HashSet<u32>> = if filter.is_empty() {
            None
        } else {
            Some(compute_keep(snap, &filter))
        };

        // ---- Painel de detalhes (direita) ----
        egui::Panel::right("proc_details")
            .resizable(true)
            .default_size(330.0)
            .min_size(240.0)
            .max_size(460.0)
            .show(ui, |ui| {
                details_panel(ui, snap, selected);
            });

        // ---- Árvore desenhada (centro) ----
        egui::CentralPanel::default().show(ui, |ui| {
            tree_view::show(ui, &mut self.tree, snap, keep.as_ref(), &mut selected);
        });

        self.selected = selected;
    }
}

/// Ponto de entrada chamado pela navegação do app.
pub fn show(app: &mut OsPlaygroundApp, ui: &mut egui::Ui) {
    app.process_tree.ui(ui);
}

/// Monta o conjunto de PIDs a manter visíveis dado um filtro textual.
fn compute_keep(snap: &ProcessSnapshot, filter: &str) -> HashSet<u32> {
    let matched: Vec<u32> = snap
        .procs
        .values()
        .filter(|p| p.name.to_lowercase().contains(filter) || p.pid.to_string().contains(filter))
        .map(|p| p.pid)
        .collect();

    let mut keep: HashSet<u32> = HashSet::new();
    // Subárvore de cada correspondência.
    let mut stack = matched.clone();
    while let Some(id) = stack.pop() {
        if keep.insert(id) {
            if let Some(p) = snap.get(id) {
                stack.extend(p.children.iter().copied());
            }
        }
    }
    // Ancestrais de cada correspondência (para mostrar o caminho até a raiz).
    for &m in &matched {
        let mut cur = snap.get(m).and_then(|p| p.parent);
        while let Some(pp) = cur {
            if !keep.insert(pp) {
                break; // já incluído: ancestrais acima também já estão.
            }
            cur = snap.get(pp).and_then(|p| p.parent);
        }
    }
    keep
}

/// Painel lateral com os detalhes do processo selecionado.
fn details_panel(ui: &mut egui::Ui, snap: &ProcessSnapshot, selected: Option<u32>) {
    ui.add_space(6.0);
    ui.heading("Detalhes");
    ui.separator();

    let Some(pid) = selected else {
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new("Selecione um processo na árvore para ver os detalhes.")
                .color(egui::Color32::GRAY),
        );
        return;
    };
    let Some(p) = snap.get(pid) else {
        ui.label("Processo não está mais em execução.");
        return;
    };

    egui::Grid::new("proc_grid")
        .num_columns(2)
        .spacing([12.0, 6.0])
        .striped(true)
        .show(ui, |ui| {
            row(ui, "PID", &p.pid.to_string());
            row(
                ui,
                "PPID",
                &p.parent.map(|x| x.to_string()).unwrap_or_else(|| "—".into()),
            );
            row(ui, "Nome", &p.name);
            row(ui, "Estado", &friendly_status(&p.status));
            row(ui, "CPU", &format!("{:.1}%", p.cpu));
            row(ui, "Memória", &human_bytes(p.memory_bytes));
            row(
                ui,
                "Threads",
                &p.thread_count
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "n/d neste SO".into()),
            );
            row(ui, "Filhos", &p.children.len().to_string());
            row(ui, "Ativo há", &uptime_since(p.start_time));
            ui.end_row();
        });

    if p.pid <= 1 {
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(
                "🌱 Processo raiz da árvore (init/launchd) — de onde descende toda a hierarquia.",
            )
            .color(ACCENT),
        );
    }

    if !p.cmd.is_empty() {
        ui.add_space(8.0);
        ui.label(egui::RichText::new("Linha de comando").strong());
        ui.add_space(2.0);
        ui.label(egui::RichText::new(&p.cmd).monospace().size(12.0));
    }
}

fn row(ui: &mut egui::Ui, k: &str, v: &str) {
    ui.label(egui::RichText::new(k).strong());
    ui.label(v);
    ui.end_row();
}

fn friendly_status(s: &str) -> String {
    let l = s.to_lowercase();
    if l.contains("run") {
        "Executando".into()
    } else if l.contains("sleep") {
        "Dormindo (bloqueado)".into()
    } else if l.contains("idle") {
        "Ocioso".into()
    } else if l.contains("zombie") {
        "Zumbi".into()
    } else if l.contains("stop") {
        "Parado".into()
    } else {
        s.to_string()
    }
}

fn human_bytes(b: u64) -> String {
    const KB: f64 = 1024.0;
    let b = b as f64;
    if b < KB {
        format!("{b:.0} B")
    } else if b < KB * KB {
        format!("{:.1} KB", b / KB)
    } else if b < KB * KB * KB {
        format!("{:.1} MB", b / (KB * KB))
    } else {
        format!("{:.2} GB", b / (KB * KB * KB))
    }
}

fn uptime_since(start_epoch_secs: u64) -> String {
    if start_epoch_secs == 0 {
        return "—".into();
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let secs = now.saturating_sub(start_epoch_secs);
    let (d, h, m) = (secs / 86_400, (secs % 86_400) / 3600, (secs % 3600) / 60);
    if d > 0 {
        format!("{d}d {h}h {m}m")
    } else if h > 0 {
        format!("{h}h {m}m")
    } else {
        format!("{m}m")
    }
}
