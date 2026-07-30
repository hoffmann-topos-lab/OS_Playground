//! Camada de interface (telas e componentes reutilizáveis).

pub mod content;
pub mod content_race;
pub mod content_sched;
pub mod demo;
pub mod gantt;
pub mod home;
pub mod process_tree;
pub mod race;
pub mod reader;
pub mod scheduling;
pub mod texts;
pub mod theme;
pub mod topic_menu;
pub mod tree_view;

use eframe::egui;

/// Azul de destaque usado para a raiz da árvore (pid 0/1) e realces.
pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(0x6f, 0xb3, 0xff);
