//! Tela "Explorador de Arquivos": o sistema de arquivos real, ao vivo.
//!
//! É a contraparte prática do capítulo 4. Para cada entrada de um diretório de
//! verdade mostramos o número de i-node (o nome de baixo nível da seção 4.1), a
//! contagem de ligações, o tipo (seção 4.1.3) e os metadados que a chamada stat
//! devolve. Entradas que são hard links do mesmo arquivo ganham a mesma cor, o
//! que torna visível o compartilhamento da seção 4.3.4.

use std::path::{Path, PathBuf};

use eframe::egui;

use crate::app::OsPlaygroundApp;
use crate::sys::fs::{human_size, Entry, Kind, Listing};
use crate::ui::gantt::series_color;
use crate::ui::ACCENT;

pub struct ExplorerState {
    listing: Option<Listing>,
    selected: Option<PathBuf>,
    /// Mostra o diretório como a lista de pares (nome, i-node) da seção 4.3.3.
    pairs_view: bool,
}

impl Default for ExplorerState {
    fn default() -> Self {
        Self {
            listing: None,
            selected: None,
            pairs_view: false,
        }
    }
}

impl ExplorerState {
    fn load(&mut self, dir: &Path) {
        self.listing = Some(Listing::read(dir));
        self.selected = None;
    }

    fn ensure_loaded(&mut self) {
        if self.listing.is_none() {
            let start = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
            self.load(&start);
        }
    }

    pub(crate) fn ui(&mut self, ui: &mut egui::Ui) {
        self.ensure_loaded();
        let dir = self.listing.as_ref().unwrap().dir.clone();

        // ---- Barra de ferramentas ----
        let mut goto: Option<PathBuf> = None;
        egui::Panel::top("fs_toolbar").exact_size(38.0).show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                if ui.button("🔄 Reler").clicked() {
                    goto = Some(dir.clone());
                }
                if ui
                    .add_enabled(dir.parent().is_some(), egui::Button::new("⬆ Acima"))
                    .on_hover_text("Ir para o diretório pai (..)")
                    .clicked()
                {
                    if let Some(p) = dir.parent() {
                        goto = Some(p.to_path_buf());
                    }
                }
                ui.separator();
                if ui.button("🏠 Projeto").clicked() {
                    goto = std::env::current_dir().ok();
                }
                if let Some(home) = std::env::var_os("HOME") {
                    if ui.button("👤 Home").clicked() {
                        goto = Some(PathBuf::from(home));
                    }
                }
                if ui
                    .button("🔌 /dev")
                    .on_hover_text("Dispositivos: arquivos especiais de bloco e de caractere")
                    .clicked()
                {
                    goto = Some(PathBuf::from("/dev"));
                }
                ui.separator();
                ui.checkbox(&mut self.pairs_view, "Ver como pares (nome, i-node)")
                    .on_hover_text("Como o diretório é armazenado no disco (seção 4.3.3)");
            });
        });

        // ---- Caminho atual (trilha de navegação) ----
        egui::Panel::top("fs_path").exact_size(30.0).show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.label(egui::RichText::new("📂").size(15.0));
                if let Some(p) = breadcrumb(ui, &dir) {
                    goto = Some(p);
                }
            });
        });

        // ---- Painel de detalhes (direita) ----
        let selected_entry = self.selected.clone().and_then(|sel| {
            self.listing
                .as_ref()
                .and_then(|l| l.entries.iter().find(|e| e.path == sel).cloned())
        });
        egui::Panel::right("fs_details")
            .resizable(true)
            .default_size(320.0)
            .min_size(240.0)
            .max_size(460.0)
            .show(ui, |ui| {
                details_panel(ui, selected_entry.as_ref());
            });

        // ---- Lista do diretório (centro) ----
        egui::CentralPanel::default().show(ui, |ui| {
            let listing = self.listing.as_ref().unwrap();
            if let Some(err) = &listing.error {
                ui.add_space(16.0);
                ui.colored_label(egui::Color32::from_rgb(0xff, 0x8a, 0x5b), err);
                return;
            }

            // Resumo do diretório.
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                let dirs = listing.entries.iter().filter(|e| e.kind == Kind::Dir).count();
                let files = listing.entries.len() - dirs;
                ui.label(
                    egui::RichText::new(format!(
                        "{} entradas  ·  {dirs} diretórios  ·  {files} arquivos  ·  {} em dados",
                        listing.entries.len(),
                        human_size(listing.total_size())
                    ))
                    .size(12.0)
                    .color(egui::Color32::GRAY),
                );
            });
            ui.add_space(4.0);
            ui.separator();

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if self.pairs_view {
                        goto = pairs_table(ui, listing, &mut self.selected).or(goto.take());
                    } else {
                        goto = entry_table(ui, listing, &mut self.selected).or(goto.take());
                    }
                });
        });

        if let Some(target) = goto {
            self.load(&target);
        }
    }
}

/// Ponto de entrada chamado pela navegação do app.
pub fn show(app: &mut OsPlaygroundApp, ui: &mut egui::Ui) {
    app.explorer.ui(ui);
}

/// A trilha de diretórios do caminho atual, cada parte clicável.
fn breadcrumb(ui: &mut egui::Ui, dir: &Path) -> Option<PathBuf> {
    let mut goto = None;
    let mut acc = PathBuf::from("/");
    if ui
        .add(egui::Button::new(egui::RichText::new("/").monospace()).frame(false))
        .clicked()
    {
        goto = Some(acc.clone());
    }
    for comp in dir.components().skip(1) {
        let name = comp.as_os_str().to_string_lossy().into_owned();
        acc.push(&name);
        ui.label(egui::RichText::new("/").color(egui::Color32::GRAY));
        if ui
            .add(egui::Button::new(egui::RichText::new(&name).monospace()).frame(false))
            .clicked()
        {
            goto = Some(acc.clone());
        }
    }
    goto
}

/// A tabela de entradas: tipo, nome, i-node, ligações, tamanho, permissões.
fn entry_table(
    ui: &mut egui::Ui,
    listing: &Listing,
    selected: &mut Option<PathBuf>,
) -> Option<PathBuf> {
    let dark = ui.visuals().dark_mode;
    let mut goto = None;

    egui::Grid::new("fs_entries")
        .num_columns(6)
        .striped(true)
        .spacing([14.0, 5.0])
        .min_col_width(28.0)
        .show(ui, |ui| {
            for h in ["", "nome", "i-node", "links", "tamanho", "permissões"] {
                ui.label(egui::RichText::new(h).size(11.5).strong().color(ACCENT));
            }
            ui.end_row();

            for e in &listing.entries {
                // Coluna do tipo: a letra que o ls -l mostra.
                ui.label(
                    egui::RichText::new(e.kind.ls_char().to_string())
                        .monospace()
                        .size(13.0)
                        .color(kind_color(e.kind)),
                );

                // Nome, clicável. Diretório navega; o resto seleciona.
                let is_sel = selected.as_deref() == Some(e.path.as_path());
                let label = display_name(e);
                let resp = ui.selectable_label(is_sel, egui::RichText::new(label).size(13.0));
                if resp.hovered() && e.kind == Kind::Dir {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if resp.clicked() {
                    if e.kind == Kind::Dir {
                        goto = Some(e.path.clone());
                    } else {
                        *selected = Some(e.path.clone());
                    }
                }
                if resp.double_clicked() && e.kind == Kind::Symlink {
                    // Abrir o alvo de um link para um diretório.
                    if let Ok(real) = std::fs::canonicalize(&e.path) {
                        if real.is_dir() {
                            goto = Some(real);
                        }
                    }
                }

                // i-node, colorido quando faz parte de um grupo de hard links.
                let ino_color = match e.hardlink_group {
                    Some(g) => series_color(g, dark),
                    None => ui.visuals().text_color(),
                };
                ui.label(
                    egui::RichText::new(e.inode.to_string())
                        .monospace()
                        .size(12.0)
                        .color(ino_color),
                );

                // Contagem de ligações; destacada quando maior que 1.
                let nlink = egui::RichText::new(e.nlink.to_string()).monospace().size(12.0);
                ui.label(if e.nlink > 1 { nlink.color(ACCENT).strong() } else { nlink });

                // Tamanho (diretórios e especiais não mostram tamanho útil).
                let size_txt = match e.kind {
                    Kind::Regular | Kind::Symlink => human_size(e.size),
                    _ => "—".to_string(),
                };
                ui.label(egui::RichText::new(size_txt).size(12.0));

                ui.label(
                    egui::RichText::new(e.perm_string())
                        .monospace()
                        .size(11.5)
                        .color(egui::Color32::GRAY),
                );
                ui.end_row();
            }
        });

    if listing.entries.iter().any(|e| e.hardlink_group.is_some()) {
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(
                "Entradas com i-node da mesma cor são hard links do mesmo arquivo: um só \
                 i-node, vários nomes (seção 4.3.4).",
            )
            .size(11.5)
            .color(egui::Color32::GRAY),
        );
    }
    goto
}

/// O diretório visto como a lista de pares (nome, i-node) que ele de fato é no
/// disco (seção 4.3.3), no formato das entradas do livro: i-node e nome.
fn pairs_table(
    ui: &mut egui::Ui,
    listing: &Listing,
    selected: &mut Option<PathBuf>,
) -> Option<PathBuf> {
    let mut goto = None;
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new(
            "Um diretório é apenas um arquivo cujo conteúdo é uma lista de pares. Cada par \
             liga um nome legível ao número do i-node do arquivo.",
        )
        .size(12.5),
    );
    ui.add_space(8.0);

    egui::Grid::new("fs_pairs")
        .num_columns(2)
        .striped(true)
        .spacing([18.0, 5.0])
        .show(ui, |ui| {
            ui.label(egui::RichText::new("i-node").size(11.5).strong().color(ACCENT));
            ui.label(egui::RichText::new("nome").size(11.5).strong().color(ACCENT));
            ui.end_row();
            for e in &listing.entries {
                ui.label(
                    egui::RichText::new(e.inode.to_string())
                        .monospace()
                        .size(12.5),
                );
                let is_sel = selected.as_deref() == Some(e.path.as_path());
                if ui
                    .selectable_label(is_sel, egui::RichText::new(display_name(e)).size(13.0))
                    .clicked()
                {
                    if e.kind == Kind::Dir {
                        goto = Some(e.path.clone());
                    } else {
                        *selected = Some(e.path.clone());
                    }
                }
                ui.end_row();
            }
        });
    goto
}

/// Painel lateral com o `stat` da entrada selecionada.
fn details_panel(ui: &mut egui::Ui, entry: Option<&Entry>) {
    ui.add_space(6.0);
    ui.heading("stat");
    ui.separator();

    let Some(e) = entry else {
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(
                "Clique num arquivo para ver seus metadados. Clique num diretório para entrar \
                 nele.",
            )
            .color(egui::Color32::GRAY),
        );
        ui.add_space(12.0);
        ui.label(
            egui::RichText::new(
                "Os mesmos números aparecem no terminal com os comandos ls -i, stat e, para \
                 seguir as chamadas de sistema, dtruss (macOS) ou strace (Linux).",
            )
            .size(12.0)
            .color(egui::Color32::GRAY),
        );
        return;
    };

    ui.add_space(4.0);
    ui.label(egui::RichText::new(&e.name).strong().size(15.0));
    ui.add_space(6.0);

    egui::Grid::new("fs_stat")
        .num_columns(2)
        .spacing([12.0, 6.0])
        .striped(true)
        .show(ui, |ui| {
            row(ui, "Tipo", e.kind.label());
            row(ui, "i-node", &e.inode.to_string());
            row(ui, "Ligações", &e.nlink.to_string());
            row(ui, "Tamanho", &human_size(e.size));
            row(ui, "Blocos (512 B)", &e.blocks.to_string());
            row(ui, "Permissões", &format!("{} {}", e.kind.ls_char(), e.perm_string()));
            ui.end_row();
        });

    if let Some(target) = &e.symlink_target {
        ui.add_space(8.0);
        ui.label(egui::RichText::new("Aponta para").strong());
        ui.label(egui::RichText::new(target).monospace().size(12.0));
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(
                "Uma ligação simbólica é um arquivo próprio cujo conteúdo é o caminho do alvo. \
                 Se o alvo some, a ligação fica pendurada.",
            )
            .size(11.5)
            .color(egui::Color32::GRAY),
        );
    }

    if e.nlink > 1 && e.kind == Kind::Regular {
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(format!(
                "Este arquivo tem {} nomes apontando para o mesmo i-node. O arquivo só é \
                 apagado quando a contagem chega a zero: é por isso que a chamada se chama \
                 unlink, não delete.",
                e.nlink
            ))
            .size(12.0)
            .color(ACCENT),
        );
    }
}

fn row(ui: &mut egui::Ui, k: &str, v: &str) {
    ui.label(egui::RichText::new(k).strong());
    ui.label(egui::RichText::new(v).monospace().size(12.5));
    ui.end_row();
}

/// Nome mostrado na lista, com uma seta quando é ligação simbólica.
fn display_name(e: &Entry) -> String {
    let mut name = e.name.clone();
    if name.chars().count() > 46 {
        name = format!("{}…", name.chars().take(45).collect::<String>());
    }
    match (e.kind, &e.symlink_target) {
        (Kind::Dir, _) => format!("{name}/"),
        (Kind::Symlink, Some(t)) => format!("{name} ↪ {t}"),
        _ => name,
    }
}

fn kind_color(kind: Kind) -> egui::Color32 {
    match kind {
        Kind::Dir => ACCENT,
        Kind::Symlink => egui::Color32::from_rgb(0x7f, 0xd6, 0x9b),
        Kind::CharDevice | Kind::BlockDevice => egui::Color32::from_rgb(0xff, 0xb4, 0x5b),
        _ => egui::Color32::GRAY,
    }
}
