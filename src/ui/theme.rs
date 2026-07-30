//! Ajustes visuais globais (espaçamentos e arredondamentos um pouco maiores).

use eframe::egui;

pub fn apply(ctx: &egui::Context) {
    // No egui 0.35 há estilos separados por tema (claro/escuro); ajustamos ambos.
    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(10.0, 6.0);
        for (_text_style, font) in style.text_styles.iter_mut() {
            font.size *= 1.05;
        }
    });
}
