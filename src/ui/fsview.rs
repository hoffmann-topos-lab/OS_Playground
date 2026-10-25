//! Diagramas desenhados do capítulo 4 (sistema de arquivos).
//!
//! São figuras estáticas, feitas com o `painter`, no mesmo estilo do diagrama
//! de estados do leitor. Cada uma ilustra uma estrutura de disco descrita no
//! texto: o esquema de uma partição (seção 4.3.1), o layout de um sistema de
//! arquivos simples e o i-node com seu índice de vários níveis (seção 4.3.2), e
//! a comparação entre os métodos de alocação de blocos.

use eframe::egui;

use crate::ui::ACCENT;

const SUPER: egui::Color32 = egui::Color32::from_rgb(0x6f, 0xb3, 0xff);
const IBMAP: egui::Color32 = egui::Color32::from_rgb(0xff, 0xb4, 0x5b);
const DBMAP: egui::Color32 = egui::Color32::from_rgb(0xff, 0x8a, 0x5b);
const INODE: egui::Color32 = egui::Color32::from_rgb(0x7f, 0xd6, 0x9b);
const DATA: egui::Color32 = egui::Color32::from_rgb(0x9a, 0x8c, 0xff);

/// Desenha uma caixa com rótulo centralizado. Devolve o retângulo usado.
fn box_label(
    painter: &egui::Painter,
    rect: egui::Rect,
    fill: egui::Color32,
    stroke: egui::Color32,
    label: &str,
    font: f32,
    text: egui::Color32,
) {
    painter.rect(
        rect,
        5.0,
        fill,
        egui::Stroke::new(1.3, stroke),
        egui::StrokeKind::Inside,
    );
    if !label.is_empty() {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(font),
            text,
        );
    }
}

// ---------------------------------------------------------------------------
// Esquema de uma partição: MBR/GPT e o bloco de inicialização (seção 4.3.1)
// ---------------------------------------------------------------------------

pub fn disk_partition(ui: &mut egui::Ui) {
    let width = ui.available_width().min(620.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 120.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let v = ui.visuals().clone();
    let txt = v.text_color();
    let line = txt.gamma_multiply(0.5);

    let y = rect.top() + 24.0;
    let h = 46.0;
    let x0 = rect.left();
    let total = width;
    // MBR/GPT ocupa uma fatia estreita; o resto são partições.
    let mbr_w = total * 0.16;
    box_label(
        &painter,
        egui::Rect::from_min_size(egui::pos2(x0, y), egui::vec2(mbr_w, h)),
        SUPER.gamma_multiply(0.30),
        SUPER,
        "MBR / GPT",
        11.0,
        txt,
    );
    let parts = [("Partição 1\n(ativa)", 0.34, true), ("Partição 2", 0.26, false), ("Partição 3", 0.24, false)];
    let mut x = x0 + mbr_w;
    for (name, frac, active) in parts {
        let w = total * frac;
        let fill = if active { INODE.gamma_multiply(0.28) } else { v.widgets.inactive.bg_fill };
        let stroke = if active { INODE } else { line };
        box_label(
            &painter,
            egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(w, h)),
            fill,
            stroke,
            name,
            10.5,
            txt,
        );
        x += w;
    }

    painter.text(
        egui::pos2(x0, rect.top() + 6.0),
        egui::Align2::LEFT_TOP,
        "setor 0",
        egui::FontId::proportional(10.0),
        v.weak_text_color(),
    );
    painter.text(
        egui::pos2(x0 + mbr_w + total * 0.17, y + h + 8.0),
        egui::Align2::CENTER_TOP,
        "o bloco de inicialização da partição ativa carrega o sistema operacional",
        egui::FontId::proportional(10.5),
        v.weak_text_color(),
    );
}

// ---------------------------------------------------------------------------
// Layout de um sistema de arquivos simples (vsfs do OSTEP, seções 4.3.1–4.3.2)
// ---------------------------------------------------------------------------

pub fn fs_layout(ui: &mut egui::Ui) {
    const N: usize = 64;
    let width = ui.available_width();
    let cell = ((width - 2.0) / N as f32).clamp(9.0, 20.0);
    let grid_w = cell * N as f32;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, cell + 58.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let v = ui.visuals().clone();
    let x0 = rect.left() + (width - grid_w) / 2.0;
    let y = rect.top() + 26.0;

    // Região de cada bloco: S (0), i (1), d (2), I (3..7), D (8..63).
    let region = |i: usize| -> (egui::Color32, &'static str) {
        match i {
            0 => (SUPER, "S"),
            1 => (IBMAP, "i"),
            2 => (DBMAP, "d"),
            3..=7 => (INODE, "I"),
            _ => (DATA, "D"),
        }
    };

    for i in 0..N {
        let (color, tag) = region(i);
        let r = egui::Rect::from_min_size(
            egui::pos2(x0 + i as f32 * cell, y),
            egui::vec2(cell - 1.0, cell),
        );
        box_label(&painter, r, color.gamma_multiply(0.55), color, "", 0.0, v.text_color());
        if cell >= 12.0 {
            painter.text(
                r.center(),
                egui::Align2::CENTER_CENTER,
                tag,
                egui::FontId::monospace(cell * 0.6),
                egui::Color32::WHITE,
            );
        }
    }

    // Números dos blocos nos extremos de cada região.
    for i in [0usize, 8, 63] {
        painter.text(
            egui::pos2(x0 + i as f32 * cell + cell / 2.0, y - 4.0),
            egui::Align2::CENTER_BOTTOM,
            i.to_string(),
            egui::FontId::proportional(9.5),
            v.weak_text_color(),
        );
    }

    // Legenda.
    let legend = [
        (SUPER, "S", "superbloco"),
        (IBMAP, "i", "bitmap de i-nodes"),
        (DBMAP, "d", "bitmap de dados"),
        (INODE, "I", "tabela de i-nodes"),
        (DATA, "D", "região de dados"),
    ];
    let ly = y + cell + 10.0;
    let mut lx = x0;
    for (color, tag, name) in legend {
        let sw = egui::Rect::from_min_size(egui::pos2(lx, ly + 1.0), egui::vec2(11.0, 11.0));
        box_label(&painter, sw, color.gamma_multiply(0.55), color, "", 0.0, v.text_color());
        let text = format!("{tag} = {name}");
        painter.text(
            egui::pos2(lx + 16.0, ly + 6.0),
            egui::Align2::LEFT_CENTER,
            &text,
            egui::FontId::proportional(10.5),
            v.text_color(),
        );
        lx += 16.0 + text.len() as f32 * 6.0 + 16.0;
    }
}

// ---------------------------------------------------------------------------
// O i-node e o índice de vários níveis (seção 4.3.2, i-nodes)
// ---------------------------------------------------------------------------

pub fn inode_index(ui: &mut egui::Ui) {
    let width = ui.available_width().min(640.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 232.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let v = ui.visuals().clone();
    let txt = v.text_color();

    // O i-node à esquerda, com atributos e três grupos de ponteiros.
    let inode = egui::Rect::from_min_size(
        egui::pos2(rect.left() + 4.0, rect.top() + 14.0),
        egui::vec2(140.0, 196.0),
    );
    box_label(&painter, inode, INODE.gamma_multiply(0.18), INODE, "", 0.0, txt);
    painter.text(
        egui::pos2(inode.center().x, inode.top() + 13.0),
        egui::Align2::CENTER_CENTER,
        "i-node",
        egui::FontId::proportional(13.0),
        INODE,
    );
    painter.text(
        egui::pos2(inode.center().x, inode.top() + 30.0),
        egui::Align2::CENTER_CENTER,
        "atributos",
        egui::FontId::proportional(10.5),
        v.weak_text_color(),
    );

    // Uma faixa por grupo de ponteiros, cada uma no seu nível vertical.
    let y1 = rect.top() + 72.0;
    let y2 = rect.top() + 128.0;
    let y3 = rect.top() + 184.0;
    let slot_x = inode.left() + 8.0;
    let slot_w = inode.width() - 16.0;
    let slot = |y: f32, label: &str, color: egui::Color32| -> egui::Pos2 {
        let r = egui::Rect::from_center_size(egui::pos2(slot_x + slot_w / 2.0, y), egui::vec2(slot_w, 18.0));
        box_label(&painter, r, color.gamma_multiply(0.30), color, label, 9.5, color);
        egui::pos2(r.right(), r.center().y)
    };
    let a1 = slot(y1, "12 diretos", DATA);
    let a2 = slot(y2, "indireto", SUPER);
    let a3 = slot(y3, "indireto duplo", IBMAP);

    let arrow = |a: egui::Pos2, b: egui::Pos2, color: egui::Color32| {
        painter.arrow(a, b - a, egui::Stroke::new(1.4, color));
    };
    let data_block = |c: egui::Pos2| {
        let r = egui::Rect::from_center_size(c, egui::vec2(24.0, 18.0));
        box_label(&painter, r, DATA.gamma_multiply(0.5), DATA, "D", 10.5, egui::Color32::WHITE);
        r
    };
    let ptr_block = |c: egui::Pos2, color: egui::Color32| -> egui::Rect {
        let r = egui::Rect::from_center_size(c, egui::vec2(70.0, 26.0));
        box_label(&painter, r, color.gamma_multiply(0.22), color, "ponteiros", 10.0, color);
        r
    };

    // Faixa 1: ponteiros diretos para blocos de dados.
    for k in 0..3 {
        let r = data_block(egui::pos2(rect.left() + 206.0 + k as f32 * 30.0, y1));
        arrow(a1, egui::pos2(r.left() - 3.0, r.center().y), DATA.gamma_multiply(0.85));
    }
    painter.text(
        egui::pos2(rect.left() + 206.0 + 90.0, y1),
        egui::Align2::LEFT_CENTER,
        "… 12",
        egui::FontId::proportional(10.0),
        v.weak_text_color(),
    );

    // Faixa 2: indireto simples, um bloco de ponteiros e seus dados.
    let sb = ptr_block(egui::pos2(rect.left() + 224.0, y2), SUPER);
    arrow(a2, egui::pos2(sb.left() - 3.0, sb.center().y), SUPER);
    for k in 0..2 {
        let r = data_block(egui::pos2(rect.left() + 320.0, y2 - 11.0 + k as f32 * 22.0));
        arrow(egui::pos2(sb.right(), sb.center().y), egui::pos2(r.left() - 3.0, r.center().y), SUPER.gamma_multiply(0.7));
    }

    // Faixa 3: indireto duplo, dois níveis de ponteiros até os dados.
    let db = ptr_block(egui::pos2(rect.left() + 224.0, y3), IBMAP);
    arrow(a3, egui::pos2(db.left() - 3.0, db.center().y), IBMAP);
    let mb = ptr_block(egui::pos2(rect.left() + 334.0, y3), SUPER);
    arrow(egui::pos2(db.right(), db.center().y), egui::pos2(mb.left() - 3.0, mb.center().y), IBMAP);
    let dc = data_block(egui::pos2(rect.left() + 420.0, y3));
    arrow(egui::pos2(mb.right(), mb.center().y), egui::pos2(dc.left() - 3.0, dc.center().y), SUPER.gamma_multiply(0.7));
}

// ---------------------------------------------------------------------------
// Métodos de alocação de blocos (seção 4.3.2)
// ---------------------------------------------------------------------------

pub fn alloc_methods(ui: &mut egui::Ui) {
    let width = ui.available_width().min(640.0);
    let row_h = 44.0;
    let rows = 4.0;
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(width, row_h * rows + 10.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let v = ui.visuals().clone();
    let txt = v.text_color();
    let line = txt.gamma_multiply(0.5);

    let label_w = 118.0;
    let bx = rect.left() + label_w;
    let bw = width - label_w - 6.0;
    let bh = 20.0;
    let cell = (bw / 8.0).min(42.0);

    let draw_blocks = |y: f32, idxs: &[usize], color: egui::Color32| {
        for (k, &i) in idxs.iter().enumerate() {
            let c = egui::pos2(bx + k as f32 * cell + cell / 2.0, y);
            let r = egui::Rect::from_center_size(c, egui::vec2(cell - 6.0, bh));
            box_label(&painter, r, color.gamma_multiply(0.5), color, &i.to_string(), 10.5, egui::Color32::WHITE);
        }
    };
    let name = |y: f32, title: &str, sub: &str| {
        painter.text(
            egui::pos2(rect.left() + 2.0, y - 6.0),
            egui::Align2::LEFT_CENTER,
            title,
            egui::FontId::proportional(12.0),
            txt,
        );
        painter.text(
            egui::pos2(rect.left() + 2.0, y + 9.0),
            egui::Align2::LEFT_CENTER,
            sub,
            egui::FontId::proportional(9.5),
            v.weak_text_color(),
        );
    };

    // 1. Contígua: blocos em sequência.
    let y1 = rect.top() + row_h * 0.5;
    name(y1, "Contígua", "início + tamanho");
    draw_blocks(y1, &[4, 5, 6, 7], DATA);

    // 2. Lista encadeada: cada bloco aponta para o próximo.
    let y2 = rect.top() + row_h * 1.5;
    name(y2, "Lista encadeada", "ponteiro em cada bloco");
    let chain = [4usize, 9, 2, 6];
    for (k, &i) in chain.iter().enumerate() {
        let c = egui::pos2(bx + k as f32 * cell + cell / 2.0, y2);
        let r = egui::Rect::from_center_size(c, egui::vec2(cell - 10.0, bh));
        box_label(&painter, r, INODE.gamma_multiply(0.5), INODE, &i.to_string(), 10.5, egui::Color32::WHITE);
        if k + 1 < chain.len() {
            painter.arrow(
                egui::pos2(r.right(), r.center().y),
                egui::vec2(cell - (cell - 10.0) + 4.0, 0.0),
                egui::Stroke::new(1.3, line),
            );
        }
    }

    // 3. FAT: tabela de ponteiros na memória.
    let y3 = rect.top() + row_h * 2.5;
    name(y3, "FAT", "bloco · próximo, na RAM");
    let fat = [("4", "9"), ("9", "2"), ("2", "6"), ("6", "fim")];
    for (k, (blk, nxt)) in fat.iter().enumerate() {
        let c = egui::pos2(bx + k as f32 * (cell + 6.0) + cell / 2.0, y3);
        let r = egui::Rect::from_center_size(c, egui::vec2(cell + 2.0, bh));
        box_label(&painter, r, SUPER.gamma_multiply(0.30), SUPER, &format!("{blk}·{nxt}"), 9.5, txt);
    }

    // 4. I-node: estrutura própria com os ponteiros do arquivo.
    let y4 = rect.top() + row_h * 3.5;
    name(y4, "I-node", "estrutura por arquivo");
    let ir = egui::Rect::from_center_size(egui::pos2(bx + cell, y4), egui::vec2(cell * 1.6, bh + 6.0));
    box_label(&painter, ir, IBMAP.gamma_multiply(0.30), IBMAP, "i-node", 10.0, txt);
    for k in 0..3 {
        let c = egui::pos2(ir.right() + 26.0 + k as f32 * (cell), y4);
        let r = egui::Rect::from_center_size(c, egui::vec2(cell - 6.0, bh));
        box_label(&painter, r, DATA.gamma_multiply(0.5), DATA, "D", 10.0, egui::Color32::WHITE);
        painter.arrow(
            egui::pos2(ir.right(), ir.center().y),
            egui::pos2(r.left() - 4.0, r.center().y) - egui::pos2(ir.right(), ir.center().y),
            egui::Stroke::new(1.2, IBMAP.gamma_multiply(0.8)),
        );
    }
    let _ = ACCENT;
}
