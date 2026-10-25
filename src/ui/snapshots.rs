//! Renderização headless das telas de Memória Virtual, para inspeção visual
//! durante o desenvolvimento (não há asserções sobre os pixels).
//!
//! Ignorados por padrão, porque precisam de uma GPU acessível ao `wgpu`:
//!
//! ```text
//! cargo test snapshots -- --ignored
//! ```
//!
//! As imagens vão para `target/snapshots/` (ou para o diretório em
//! `OS_PLAYGROUND_SNAPSHOT_DIR`).

use std::path::PathBuf;

use eframe::egui;
use egui_kittest::Harness;

use crate::ui::{
    content, content_addressing, content_files, content_fsimpl, content_io, content_pagedesign,
    content_paging, content_replacement, devices, explorer, reader, theme,
};

fn out_dir() -> PathBuf {
    let dir = std::env::var("OS_PLAYGROUND_SNAPSHOT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("target/snapshots"));
    std::fs::create_dir_all(&dir).expect("criar diretório dos snapshots");
    dir
}

/// Renderiza `ui_fn` num quadro de `size` e salva como PNG.
fn save(name: &str, size: egui::Vec2, ui_fn: impl FnMut(&mut egui::Ui)) {
    let mut harness = Harness::builder().with_size(size).build_ui(ui_fn);
    theme::apply(&harness.ctx);
    // Alguns quadros para o layout assentar (painéis redimensionáveis, etc.).
    for _ in 0..3 {
        harness.run();
    }
    let image = harness.render().expect("renderizar com wgpu");
    let path = out_dir().join(format!("{name}.png"));
    image.save(&path).expect("salvar png");
    eprintln!("snapshot: {}", path.display());
}

/// Uma tela-leitor inteira, com o artigo `article` aberto. Alta o bastante
/// para os widgets embutidos aparecerem sem rolar.
fn reader_screen(name: &str, articles: &'static [reader::Article], article: usize) {
    let mut st = reader::ReaderState::default();
    st.selected = article;
    save(name, egui::vec2(1100.0, 2200.0), move |ui| {
        reader::show(ui, articles, "snapshot", &mut st);
    });
}

#[test]
#[ignore]
fn snapshots_addressing() {
    for (i, art) in content_addressing::ARTICLES.iter().enumerate() {
        reader_screen(
            &format!("addressing_{i}_{}", slug(art.title)),
            content_addressing::ARTICLES,
            i,
        );
    }
}

#[test]
#[ignore]
fn snapshots_paging() {
    for (i, art) in content_paging::ARTICLES.iter().enumerate() {
        reader_screen(
            &format!("paging_{i}_{}", slug(art.title)),
            content_paging::ARTICLES,
            i,
        );
    }
}

#[test]
#[ignore]
fn snapshots_replacement() {
    for (i, art) in content_replacement::ARTICLES.iter().enumerate() {
        reader_screen(
            &format!("replacement_{i}_{}", slug(art.title)),
            content_replacement::ARTICLES,
            i,
        );
    }
}

#[test]
#[ignore]
fn snapshots_pagedesign() {
    for (i, art) in content_pagedesign::ARTICLES.iter().enumerate() {
        reader_screen(
            &format!("design_{i}_{}", slug(art.title)),
            content_pagedesign::ARTICLES,
            i,
        );
    }
}

#[test]
#[ignore]
fn snapshots_files() {
    for (i, art) in content_files::ARTICLES.iter().enumerate() {
        reader_screen(
            &format!("files_{i}_{}", slug(art.title)),
            content_files::ARTICLES,
            i,
        );
    }
}

#[test]
#[ignore]
fn snapshots_fsimpl() {
    for (i, art) in content_fsimpl::ARTICLES.iter().enumerate() {
        reader_screen(
            &format!("fsimpl_{i}_{}", slug(art.title)),
            content_fsimpl::ARTICLES,
            i,
        );
    }
}

#[test]
#[ignore]
fn snapshots_io() {
    for (i, art) in content_io::ARTICLES.iter().enumerate() {
        reader_screen(&format!("io_{i}_{}", slug(art.title)), content_io::ARTICLES, i);
    }
}

/// As duas telas ao vivo, lendo o sistema de arquivos real desta máquina.
#[test]
#[ignore]
fn snapshots_live() {
    let mut ex = explorer::ExplorerState::default();
    save("live_explorer", egui::vec2(1100.0, 900.0), move |ui| ex.ui(ui));
    let mut dv = devices::DevicesState::default();
    save("live_devices", egui::vec2(1100.0, 1400.0), move |ui| dv.ui(ui));
}

/// Os menus de tópico, para conferir o tamanho uniforme dos cartões.
#[test]
#[ignore]
fn snapshots_menus() {
    use crate::ui::topic_menu;
    for (name, opts) in [
        ("menu_memory", topic_menu::MEMORY),
        ("menu_files", topic_menu::FILES),
        ("menu_io", topic_menu::IO),
    ] {
        save(name, egui::vec2(980.0, 440.0), move |ui| {
            ui.add_space(16.0);
            egui::ScrollArea::vertical().show(ui, |ui| {
                for opc in opts {
                    let _ = topic_menu::card(ui, opc);
                    ui.add_space(12.0);
                }
            });
        });
    }
}

/// Na largura mínima da janela (820 px): nada pode transbordar a coluna.
#[test]
#[ignore]
fn snapshots_narrow() {
    for (name, articles, i) in [
        ("narrow_wsclock", content_replacement::ARTICLES, 8),
        ("narrow_fits", content_addressing::ARTICLES, 5),
        ("narrow_tlb", content_paging::ARTICLES, 4),
        ("narrow_pagesize", content_pagedesign::ARTICLES, 3),
        ("narrow_iomodes", content_io::ARTICLES, 4),
        ("narrow_inode", content_fsimpl::ARTICLES, 3),
        ("narrow_links", content_files::ARTICLES, 6),
    ] {
        let mut st = reader::ReaderState::default();
        st.selected = i;
        save(name, egui::vec2(820.0, 2000.0), move |ui| {
            reader::show(ui, articles, "snapshot", &mut st);
        });
    }
}

/// Uma tela antiga, para comparar o comportamento do leitor.
#[test]
#[ignore]
fn snapshots_texts_reference() {
    reader_screen("texts_0", content::ARTICLES, 0);
    reader_screen("texts_9", content::ARTICLES, 9);
}

/// Quais glifos a fonte padrão do egui tem: para escolher símbolos que rendem.
#[test]
#[ignore]
fn snapshots_glyphs() {
    save("glyphs", egui::vec2(700.0, 200.0), |ui| {
        ui.heading("💡 ⚠ ✔ ✘ ✓ ✗ → ← ➡ ⬅ ↪ ↳ ⏰ ◀ ▶ ⏭ ↺ ≤ ≥ ≈ √ τ × − · ⌊ ⌋ ² ⁰ ¹ 🎯 🔮 📥");
        ui.label("💡 ⚠ ✔ ✘ ✓ ✗ → ← ➡ ⬅ ↪ ↳ ⏰ ◀ ▶ ⏭ ↺ ≤ ≥ ≈ √ τ × − · ⌊ ⌋ ² ⁰ ¹ 🎯 🔮 📥");
        ui.label(egui::RichText::new("💡 ⚠ ✔ ✘ ✓ ✗ → ← ➡ ⬅ ↪ ↳ ⏰ ◀ ▶ ⏭ ↺ ≤ ≥ ≈ √ τ").monospace());
        ui.label("A: ✖ ❌ ✕ ⨯ ⊗ | B: ⇒ ⟶ ⇢ ↦ ➔ ➜ ⇨ ⏵ ▸ ‣ | C: ⇐ ⟵ ⇦ ⬆ ⬇ | D: ⌊v/p⌋ ⎣ ⎦");
    });
}

fn slug(title: &str) -> String {
    title
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == ' ')
        .collect::<String>()
        .to_ascii_lowercase()
        .split_whitespace()
        .take(3)
        .collect::<Vec<_>>()
        .join("_")
}
