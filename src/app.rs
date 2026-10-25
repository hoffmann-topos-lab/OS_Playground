//! Estado de topo do aplicativo: navegação entre telas e recursos globais.

use eframe::egui;

use crate::ui;

/// As telas navegáveis.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Home,
    // ---- Processos e Threads (capítulo 2) ----
    ProcessesThreadsMenu,
    ProcessTree,
    Texts,
    RaceConditions,
    Scheduling,
    // ---- Memória Virtual (capítulo 3) ----
    MemoryMenu,
    Addressing,
    Paging,
    Replacement,
    PageDesign,
    // ---- Sistema de Arquivos (capítulo 4) ----
    FileSystemMenu,
    FileExplorer,
    FilesConcepts,
    FileSystemImpl,
    // ---- Entrada e Saída (capítulo 5) ----
    IoMenu,
    DeviceBrowser,
    IoConcepts,
}

impl Screen {
    /// Título mostrado na barra superior.
    pub fn title(self) -> &'static str {
        match self {
            Screen::Home => "OS Playground",
            Screen::ProcessesThreadsMenu => "Processos e Threads",
            Screen::ProcessTree => "Árvore de Processos",
            Screen::Texts => "Processos e Threads — Conceitos",
            Screen::RaceConditions => "Race Conditions",
            Screen::Scheduling => "Escalonamento",
            Screen::MemoryMenu => "Memória Virtual",
            Screen::Addressing => "Espaços de Endereçamento",
            Screen::Paging => "Paginação",
            Screen::Replacement => "Substituição de Páginas",
            Screen::PageDesign => "Projeto e Implementação",
            Screen::FileSystemMenu => "Sistema de Arquivos",
            Screen::FileExplorer => "Explorador de Arquivos",
            Screen::FilesConcepts => "Arquivos e Diretórios",
            Screen::FileSystemImpl => "Implementação do Sistema de Arquivos",
            Screen::IoMenu => "Entrada e Saída",
            Screen::DeviceBrowser => "Dispositivos ao Vivo",
            Screen::IoConcepts => "Entrada e Saída — Conceitos",
        }
    }
}

pub struct OsPlaygroundApp {
    screen: Screen,
    /// Pilha de navegação para o botão "voltar".
    history: Vec<Screen>,
    /// Textura do logo (fundo removido), carregada uma vez.
    logo: Option<egui::TextureHandle>,
    /// Estado da árvore de processos (snapshot, filtro, seleção).
    pub process_tree: ui::process_tree::ProcessTreeState,
    /// Estado das telas-leitor (artigo aberto + widgets embutidos).
    pub texts: ui::reader::ReaderState,
    pub race: ui::reader::ReaderState,
    pub scheduling: ui::reader::ReaderState,
    pub addressing: ui::reader::ReaderState,
    pub paging: ui::reader::ReaderState,
    pub replacement: ui::reader::ReaderState,
    pub page_design: ui::reader::ReaderState,
    /// Telas-leitor do Sistema de Arquivos e da Entrada e Saída.
    pub files: ui::reader::ReaderState,
    pub fsimpl: ui::reader::ReaderState,
    pub io: ui::reader::ReaderState,
    /// Telas ao vivo do capítulo 4 e do capítulo 5.
    pub explorer: ui::explorer::ExplorerState,
    pub devices: ui::devices::DevicesState,
}

impl OsPlaygroundApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        ui::theme::apply(&cc.egui_ctx);
        let logo = load_logo_texture(&cc.egui_ctx);
        let mut app = Self {
            screen: Screen::Home,
            history: Vec::new(),
            logo,
            process_tree: ui::process_tree::ProcessTreeState::default(),
            texts: ui::reader::ReaderState::default(),
            race: ui::reader::ReaderState::default(),
            scheduling: ui::reader::ReaderState::default(),
            addressing: ui::reader::ReaderState::default(),
            paging: ui::reader::ReaderState::default(),
            replacement: ui::reader::ReaderState::default(),
            page_design: ui::reader::ReaderState::default(),
            files: ui::reader::ReaderState::default(),
            fsimpl: ui::reader::ReaderState::default(),
            io: ui::reader::ReaderState::default(),
            explorer: ui::explorer::ExplorerState::default(),
            devices: ui::devices::DevicesState::default(),
        };
        if let Some((screen, article)) = start_screen() {
            app.open(screen, article);
        }
        app
    }

    /// Abre `screen` como se o usuário tivesse navegado até ela pelos menus
    /// (o "voltar" funciona), com o artigo `article` selecionado nos leitores.
    fn open(&mut self, screen: Screen, article: usize) {
        let menu = match screen {
            Screen::ProcessTree | Screen::Texts | Screen::RaceConditions | Screen::Scheduling => {
                Some(Screen::ProcessesThreadsMenu)
            }
            Screen::Addressing | Screen::Paging | Screen::Replacement | Screen::PageDesign => {
                Some(Screen::MemoryMenu)
            }
            Screen::FileExplorer | Screen::FilesConcepts | Screen::FileSystemImpl => {
                Some(Screen::FileSystemMenu)
            }
            Screen::DeviceBrowser | Screen::IoConcepts => Some(Screen::IoMenu),
            _ => None,
        };
        if let Some(m) = menu {
            self.navigate(m);
        }
        self.navigate(screen);
        let reader = match screen {
            Screen::Texts => Some(&mut self.texts),
            Screen::RaceConditions => Some(&mut self.race),
            Screen::Scheduling => Some(&mut self.scheduling),
            Screen::Addressing => Some(&mut self.addressing),
            Screen::Paging => Some(&mut self.paging),
            Screen::Replacement => Some(&mut self.replacement),
            Screen::PageDesign => Some(&mut self.page_design),
            Screen::FilesConcepts => Some(&mut self.files),
            Screen::FileSystemImpl => Some(&mut self.fsimpl),
            Screen::IoConcepts => Some(&mut self.io),
            _ => None,
        };
        if let Some(r) = reader {
            r.selected = article;
        }
    }

    /// Navega para `to`, empilhando a tela atual para permitir voltar.
    pub fn navigate(&mut self, to: Screen) {
        if to != self.screen {
            self.history.push(self.screen);
            self.screen = to;
        }
    }

    /// Volta para a tela anterior (se houver).
    pub fn back(&mut self) {
        if let Some(prev) = self.history.pop() {
            self.screen = prev;
        }
    }

    pub fn logo(&self) -> Option<&egui::TextureHandle> {
        self.logo.as_ref()
    }
}

impl eframe::App for OsPlaygroundApp {
    // No eframe 0.35 o método principal é `ui`, recebendo um `Ui` raiz.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Barra superior: botão voltar + título da tela atual.
        egui::Panel::top("appbar").exact_size(48.0).show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.add_space(8.0);
                let can_go_back = !self.history.is_empty();
                if ui
                    .add_enabled(can_go_back, egui::Button::new("  ⬅  Voltar  "))
                    .clicked()
                {
                    self.back();
                }
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);
                ui.heading(self.screen.title());
            });
        });

        egui::CentralPanel::default().show(ui, |ui| match self.screen {
            Screen::Home => ui::home::show(self, ui),
            Screen::ProcessesThreadsMenu => {
                ui::topic_menu::show(self, ui, ui::topic_menu::PROCESSES)
            }
            Screen::ProcessTree => ui::process_tree::show(self, ui),
            Screen::Texts => ui::texts::show(self, ui),
            Screen::RaceConditions => ui::race::show(self, ui),
            Screen::Scheduling => ui::scheduling::show(self, ui),
            Screen::MemoryMenu => ui::topic_menu::show(self, ui, ui::topic_menu::MEMORY),
            Screen::Addressing => ui::addressing::show(self, ui),
            Screen::Paging => ui::paging::show(self, ui),
            Screen::Replacement => ui::replacement::show(self, ui),
            Screen::PageDesign => ui::pagedesign::show(self, ui),
            Screen::FileSystemMenu => ui::topic_menu::show(self, ui, ui::topic_menu::FILES),
            Screen::FileExplorer => ui::explorer::show(self, ui),
            Screen::FilesConcepts => ui::files::show(self, ui),
            Screen::FileSystemImpl => ui::fsimpl::show(self, ui),
            Screen::IoMenu => ui::topic_menu::show(self, ui, ui::topic_menu::IO),
            Screen::DeviceBrowser => ui::devices::show(self, ui),
            Screen::IoConcepts => ui::io::show(self, ui),
        });
    }
}

/// Tela inicial pedida por `OS_PLAYGROUND_START="tela[/artigo]"`.
///
/// Útil para abrir o app direto na tela que vai ser projetada em aula (e para
/// conferir uma tela durante o desenvolvimento). Telas: `tree`, `texts`,
/// `race`, `scheduling`, `addressing`, `paging`, `replacement`, `design`; o
/// artigo é o índice no índice da tela, a partir de 0.
fn start_screen() -> Option<(Screen, usize)> {
    let spec = std::env::var("OS_PLAYGROUND_START").ok()?;
    let (name, article) = spec.split_once('/').unwrap_or((spec.as_str(), "0"));
    let screen = match name.trim().to_ascii_lowercase().as_str() {
        "processes" => Screen::ProcessesThreadsMenu,
        "tree" => Screen::ProcessTree,
        "texts" => Screen::Texts,
        "race" => Screen::RaceConditions,
        "scheduling" => Screen::Scheduling,
        "memory" => Screen::MemoryMenu,
        "addressing" => Screen::Addressing,
        "paging" => Screen::Paging,
        "replacement" => Screen::Replacement,
        "design" => Screen::PageDesign,
        "filesystem" | "fsmenu" => Screen::FileSystemMenu,
        "explorer" => Screen::FileExplorer,
        "files" => Screen::FilesConcepts,
        "fsimpl" => Screen::FileSystemImpl,
        "iomenu" => Screen::IoMenu,
        "devices" => Screen::DeviceBrowser,
        "io" => Screen::IoConcepts,
        _ => return None,
    };
    Some((screen, article.trim().parse().unwrap_or(0)))
}

/// Carrega `assets/window_icon.png` como textura para exibir na tela inicial.
fn load_logo_texture(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let bytes = include_bytes!("../assets/window_icon.png");
    let img = image::load_from_memory(bytes).ok()?.to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    let color = egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw());
    Some(ctx.load_texture("logo", color, egui::TextureOptions::LINEAR))
}
