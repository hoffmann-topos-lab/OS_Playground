//! Desenho da árvore de processos como **diagrama node-link**: caixas ligadas
//! por ramos, do jeito que a estrutura matemática é desenhada — e não como uma
//! lista recuada estilo explorador de arquivos.
//!
//! O layout é um "tidy tree" simples: cada folha ocupa um trilho consecutivo e
//! cada nó interno fica centralizado sobre o primeiro e o último filho. Como a
//! árvore real do sistema é larga (no macOS o `launchd` é pai de centenas de
//! processos), a tela tem pan (arrastar), zoom (rolar) e recolher/expandir.

use std::collections::HashSet;

use eframe::egui;

use crate::sys::ProcessSnapshot;
use crate::ui::ACCENT;

/// Como a árvore cresce na tela.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    /// Raiz em cima, ramos descendo (representação clássica).
    Vertical,
    /// Raiz à esquerda, ramos crescendo para a direita.
    Horizontal,
}

// Medidas em coordenadas de mundo (isto é, antes de aplicar o zoom).
const NODE_W: f32 = 150.0;
const NODE_H: f32 = 38.0;
/// Distância entre trilhos de folhas vizinhas.
const SLOT_PITCH_V: f32 = NODE_W + 20.0;
const SLOT_PITCH_H: f32 = NODE_H + 14.0;
/// Distância entre níveis (profundidades) consecutivos.
const LEVEL_PITCH_V: f32 = NODE_H + 62.0;
const LEVEL_PITCH_H: f32 = NODE_W + 90.0;
/// Abaixo deste zoom o texto fica ilegível — não vale o custo de desenhar.
const TEXT_MIN_ZOOM: f32 = 0.45;
/// Raio do botãozinho de recolher/expandir pendurado no nó.
const BADGE_R: f32 = 9.0;
const ZOOM_RANGE: std::ops::RangeInclusive<f32> = 0.05..=3.0;

pub struct TreeViewState {
    pub zoom: f32,
    pub pan: egui::Vec2,
    pub orientation: Orientation,
    collapsed: HashSet<u32>,
    /// Reenquadrar para caber tudo na próxima pintura (botão "Ajustar").
    pending_fit: bool,
    /// Na primeira pintura, centraliza a raiz no topo com zoom 1.
    first_show: bool,
}

impl Default for TreeViewState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
            orientation: Orientation::Vertical,
            collapsed: HashSet::new(),
            pending_fit: false,
            first_show: true,
        }
    }
}

impl TreeViewState {
    pub fn request_fit(&mut self) {
        self.pending_fit = true;
    }

    pub fn expand_all(&mut self) {
        self.collapsed.clear();
    }

    /// Recolhe todo nó que tenha filhos — sobra só o topo de cada raiz.
    pub fn collapse_all(&mut self, snap: &ProcessSnapshot) {
        self.collapsed = snap
            .procs
            .values()
            .filter(|p| !p.children.is_empty())
            .map(|p| p.pid)
            .collect();
    }

    /// Posição do centro do nó em coordenadas de mundo.
    fn world_pos(&self, n: &Laid) -> egui::Pos2 {
        match self.orientation {
            Orientation::Vertical => egui::pos2(
                n.slot * SLOT_PITCH_V,
                n.depth as f32 * LEVEL_PITCH_V,
            ),
            Orientation::Horizontal => egui::pos2(
                n.depth as f32 * LEVEL_PITCH_H,
                n.slot * SLOT_PITCH_H,
            ),
        }
    }

    /// Centro do botãozinho de recolher, pendurado na saída do nó.
    fn badge_world(&self, n: &Laid) -> egui::Pos2 {
        let c = self.world_pos(n);
        match self.orientation {
            Orientation::Vertical => egui::pos2(c.x, c.y + NODE_H / 2.0 + BADGE_R + 1.0),
            Orientation::Horizontal => egui::pos2(c.x + NODE_W / 2.0 + BADGE_R + 1.0, c.y),
        }
    }
}

/// Um nó já posicionado pelo layout.
struct Laid {
    pid: u32,
    depth: usize,
    /// Trilho (índice fracionário) ao longo do eixo transversal à profundidade.
    slot: f32,
    /// Índices, em `Vec<Laid>`, dos filhos efetivamente desenhados.
    kids: Vec<usize>,
    /// Filhos diretos que o processo tem, mesmo os escondidos.
    child_count: usize,
    collapsed: bool,
}

/// Posiciona recursivamente `pid` e sua subárvore. Devolve o índice em `out`.
fn place(
    snap: &ProcessSnapshot,
    pid: u32,
    depth: usize,
    collapsed: &HashSet<u32>,
    keep: Option<&HashSet<u32>>,
    next_slot: &mut f32,
    out: &mut Vec<Laid>,
) -> Option<usize> {
    if keep.is_some_and(|k| !k.contains(&pid)) {
        return None;
    }
    let p = snap.get(pid)?;

    // Com filtro ativo o recolhimento é ignorado: o usuário quer ver os achados.
    let is_collapsed = keep.is_none() && collapsed.contains(&pid);

    let idx = out.len();
    out.push(Laid {
        pid,
        depth,
        slot: 0.0,
        kids: Vec::new(),
        child_count: p.children.len(),
        collapsed: is_collapsed,
    });

    let mut kids = Vec::new();
    if !is_collapsed {
        for &c in &p.children {
            if let Some(ci) = place(snap, c, depth + 1, collapsed, keep, next_slot, out) {
                kids.push(ci);
            }
        }
    }

    // Folha ocupa o próximo trilho; nó interno centraliza sobre os filhos.
    let slot = match (kids.first(), kids.last()) {
        (Some(&a), Some(&b)) => (out[a].slot + out[b].slot) / 2.0,
        _ => {
            let s = *next_slot;
            *next_slot += 1.0;
            s
        }
    };
    out[idx].slot = slot;
    out[idx].kids = kids;
    Some(idx)
}

pub fn show(
    ui: &mut egui::Ui,
    st: &mut TreeViewState,
    snap: &ProcessSnapshot,
    keep: Option<&HashSet<u32>>,
    selected: &mut Option<u32>,
) {
    // ---- 1. Layout ----
    let mut laid: Vec<Laid> = Vec::new();
    let mut next_slot = 0.0f32;
    for &root in &snap.roots {
        if place(snap, root, 0, &st.collapsed, keep, &mut next_slot, &mut laid).is_some() {
            next_slot += 1.0; // respiro entre raízes distintas
        }
    }

    let visuals = ui.visuals().clone();
    let (resp, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
    let clip = painter.clip_rect();

    if laid.is_empty() {
        painter.text(
            clip.center(),
            egui::Align2::CENTER_CENTER,
            "Nenhum processo corresponde ao filtro.",
            egui::FontId::proportional(14.0),
            visuals.weak_text_color(),
        );
        return;
    }

    // ---- 2. Enquadramento ----
    if st.first_show {
        st.first_show = false;
        st.zoom = 1.0;
        let root = st.world_pos(&laid[0]).to_vec2();
        st.pan = egui::vec2(clip.width() * 0.5, 50.0) - root * st.zoom;
    }
    if st.pending_fit {
        st.pending_fit = false;
        let mut bounds = egui::Rect::NOTHING;
        for n in laid.iter() {
            let c = st.world_pos(n);
            bounds = bounds.union(egui::Rect::from_center_size(
                c,
                egui::vec2(NODE_W, NODE_H),
            ));
        }
        let margin = 28.0;
        let avail = (clip.size() - egui::vec2(margin * 2.0, margin * 2.0)).max(egui::vec2(1.0, 1.0));
        let fit = (avail.x / bounds.width().max(1.0)).min(avail.y / bounds.height().max(1.0));
        st.zoom = fit.clamp(*ZOOM_RANGE.start(), *ZOOM_RANGE.end());
        st.pan = clip.size() * 0.5 - bounds.center().to_vec2() * st.zoom;
    }

    // ---- 3. Interação ----
    if resp.dragged() {
        st.pan += resp.drag_delta();
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
    } else if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
    }

    if resp.hovered() {
        let (scroll, pinch) = ui.input(|i| (i.smooth_scroll_delta.y, i.zoom_delta()));
        let factor = pinch * (scroll * 0.0015).exp();
        if (factor - 1.0).abs() > 1e-4
            && let Some(p) = resp.hover_pos()
        {
            // Mantém fixo o ponto do mundo que está sob o cursor.
            let anchor = p - clip.min - st.pan;
            let new_zoom = (st.zoom * factor).clamp(*ZOOM_RANGE.start(), *ZOOM_RANGE.end());
            st.pan += anchor * (1.0 - new_zoom / st.zoom);
            st.zoom = new_zoom;
        }
    }

    let to_screen = |w: egui::Pos2| clip.min + st.pan + w.to_vec2() * st.zoom;
    let node_rect = |n: &Laid| {
        egui::Rect::from_center_size(
            to_screen(st.world_pos(n)),
            egui::vec2(NODE_W, NODE_H) * st.zoom,
        )
    };

    // Clique: primeiro nos botõezinhos, depois nas caixas.
    let mut hit_badge: Option<u32> = None;
    let mut hit_node: Option<u32> = None;
    if resp.clicked()
        && let Some(p) = resp.interact_pointer_pos()
    {
        for n in laid.iter().filter(|n| n.child_count > 0) {
            if (p - to_screen(st.badge_world(n))).length() <= BADGE_R * st.zoom + 2.0 {
                hit_badge = Some(n.pid);
                break;
            }
        }
        if hit_badge.is_none() {
            hit_node = laid
                .iter()
                .find(|n| node_rect(n).contains(p))
                .map(|n| n.pid);
        }
    }

    // ---- 4. Ramos ----
    let branch = visuals.text_color().gamma_multiply(0.42);
    let stroke = egui::Stroke::new((1.5 * st.zoom).clamp(0.5, 3.0), branch);
    for n in laid.iter() {
        let pc = st.world_pos(n);
        for &ci in &n.kids {
            let cc = st.world_pos(&laid[ci]);
            let (a, b, corner_a, corner_b) = match st.orientation {
                Orientation::Vertical => {
                    let a = egui::pos2(pc.x, pc.y + NODE_H / 2.0);
                    let b = egui::pos2(cc.x, cc.y - NODE_H / 2.0);
                    let mid = (a.y + b.y) * 0.5;
                    (a, b, egui::pos2(a.x, mid), egui::pos2(b.x, mid))
                }
                Orientation::Horizontal => {
                    let a = egui::pos2(pc.x + NODE_W / 2.0, pc.y);
                    let b = egui::pos2(cc.x - NODE_W / 2.0, cc.y);
                    let mid = (a.x + b.x) * 0.5;
                    (a, b, egui::pos2(mid, a.y), egui::pos2(mid, b.y))
                }
            };
            // Descarta ramos inteiramente fora da tela.
            let span = egui::Rect::from_two_pos(to_screen(a), to_screen(b));
            if !clip.expand(4.0).intersects(span) {
                continue;
            }
            painter.line_segment([to_screen(a), to_screen(corner_a)], stroke);
            painter.line_segment([to_screen(corner_a), to_screen(corner_b)], stroke);
            painter.line_segment([to_screen(corner_b), to_screen(b)], stroke);
        }
    }

    // ---- 5. Nós ----
    let corner = (6.0 * st.zoom).clamp(1.0, 8.0);
    let draw_text = st.zoom >= TEXT_MIN_ZOOM;
    for n in laid.iter() {
        let r = node_rect(n);
        if !clip.intersects(r) {
            continue;
        }
        let Some(p) = snap.get(n.pid) else { continue };

        let is_sel = *selected == Some(n.pid);
        let is_root = n.pid <= 1;
        let fill = if is_sel {
            visuals.selection.bg_fill
        } else if is_root {
            ACCENT.gamma_multiply(0.22)
        } else {
            visuals.widgets.inactive.bg_fill
        };
        let border = if is_sel {
            visuals.selection.stroke.color
        } else if is_root {
            ACCENT
        } else {
            branch
        };
        painter.rect(
            r,
            corner,
            fill,
            egui::Stroke::new((1.2 * st.zoom).clamp(0.5, 2.5), border),
            egui::StrokeKind::Inside,
        );

        if draw_text {
            let tp = painter.with_clip_rect(r.intersect(clip));
            let pad = 9.0 * st.zoom;
            let name_color = if is_sel {
                visuals.strong_text_color()
            } else {
                visuals.text_color()
            };
            tp.text(
                egui::pos2(r.min.x + pad, r.center().y - 7.0 * st.zoom),
                egui::Align2::LEFT_CENTER,
                &p.name,
                egui::FontId::proportional(13.0 * st.zoom),
                name_color,
            );
            let threads = p
                .thread_count
                .map(|t| format!("  ·  {t} thr"))
                .unwrap_or_default();
            tp.text(
                egui::pos2(r.min.x + pad, r.center().y + 8.0 * st.zoom),
                egui::Align2::LEFT_CENTER,
                format!("pid {}{}", p.pid, threads),
                egui::FontId::proportional(10.5 * st.zoom),
                visuals.weak_text_color(),
            );
        }

        // Botãozinho de recolher/expandir.
        if n.child_count > 0 {
            let bc = to_screen(st.badge_world(n));
            let br = BADGE_R * st.zoom;
            if br >= 3.0 {
                painter.circle_filled(bc, br, visuals.extreme_bg_color);
                painter.circle_stroke(bc, br, egui::Stroke::new((1.2 * st.zoom).clamp(0.5, 2.0), border));
                if draw_text {
                    let label = if n.collapsed {
                        n.child_count.to_string()
                    } else {
                        "−".to_string()
                    };
                    painter.text(
                        bc,
                        egui::Align2::CENTER_CENTER,
                        label,
                        egui::FontId::proportional(9.5 * st.zoom),
                        visuals.text_color(),
                    );
                }
            }
        }
    }

    // ---- 6. Aplica os cliques (depois de desenhar: só vale no próximo quadro) ----
    if let Some(pid) = hit_badge
        && !st.collapsed.remove(&pid)
    {
        st.collapsed.insert(pid);
    }
    if let Some(pid) = hit_node {
        *selected = Some(pid);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Roda o layout sobre a árvore real do sistema, sem GUI.
    fn layout_live() -> (ProcessSnapshot, Vec<Laid>) {
        let snap = ProcessSnapshot::capture();
        let collapsed = HashSet::new();
        let mut laid = Vec::new();
        let mut next = 0.0;
        for &root in &snap.roots {
            if place(&snap, root, 0, &collapsed, None, &mut next, &mut laid).is_some() {
                next += 1.0;
            }
        }
        (snap, laid)
    }

    #[test]
    fn layout_cobre_todos_os_processos() {
        let (snap, laid) = layout_live();
        assert_eq!(
            laid.len(),
            snap.total,
            "todo processo deveria ser posicionado exatamente uma vez"
        );
    }

    #[test]
    fn pai_fica_centralizado_sobre_os_filhos() {
        let (_snap, laid) = layout_live();
        let mut internos = 0;
        for n in &laid {
            if n.kids.is_empty() {
                continue;
            }
            internos += 1;
            let first = laid[n.kids[0]].slot;
            let last = laid[*n.kids.last().unwrap()].slot;
            let esperado = (first + last) / 2.0;
            assert!(
                (n.slot - esperado).abs() < 1e-3,
                "pid {} no trilho {} deveria estar em {esperado}",
                n.pid,
                n.slot
            );
            // Nenhum filho pode ficar num nível acima do pai.
            for &k in &n.kids {
                assert_eq!(laid[k].depth, n.depth + 1, "profundidade inconsistente");
            }
        }
        assert!(internos > 0, "a árvore real deveria ter nós internos");
    }

    #[test]
    fn folhas_nao_dividem_trilho() {
        let (_snap, laid) = layout_live();
        let mut slots: Vec<f32> = laid
            .iter()
            .filter(|n| n.kids.is_empty())
            .map(|n| n.slot)
            .collect();
        slots.sort_by(|a, b| a.partial_cmp(b).unwrap());
        for w in slots.windows(2) {
            assert!(
                (w[1] - w[0]) >= 1.0 - 1e-3,
                "duas folhas ocuparam o mesmo trilho ({} e {})",
                w[0],
                w[1]
            );
        }
    }
}
