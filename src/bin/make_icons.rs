//! Gerador de ícones do OS Playground.
//!
//! Lê `logo.png` e gera:
//!   - `assets/logo_transparent.png` (1024x1024, com canal alfa)
//!   - `assets/window_icon.png`      (256x256, usado como ícone da janela)
//!   - `assets/logo.svg`             (embute o PNG transparente; escalável)
//!   - `assets/AppIcon.iconset/*`    (tamanhos p/ o `iconutil` gerar .icns)
//!
//! O fundo é tratado de dois modos, escolhidos automaticamente:
//!   - se `logo.png` **já vem recortado** (tem canal alfa de verdade), ele é
//!     usado como está — nada de flood-fill nem feather, que só estragariam
//!     um recorte já bom;
//!   - se vier **opaco com fundo branco**, o fundo é removido por flood-fill a
//!     partir das bordas (sem apagar branco *dentro* do retrato) + feather 1px.
//!
//! Em ambos os casos a arte é aparada até a caixa do conteúdo visível e
//! centralizada num quadrado com uma margem pequena, para o ícone ocupar bem
//! o quadro (senão ele aparece menor que os vizinhos no Dock).
//!
//! Uso: `cargo run --bin make_icons`
//! (depois: `iconutil -c icns assets/AppIcon.iconset -o assets/icons/AppIcon.icns`)

use std::collections::VecDeque;
use std::fs;
use std::path::Path;

use image::{imageops::FilterType, Rgba, RgbaImage};

/// Um pixel é candidato a "fundo" se for quase branco em todos os canais.
const WHITE_THRESHOLD: u8 = 238;
/// Feather: pixels claros na borda do recorte ganham alfa parcial p/ suavizar.
const FEATHER_THRESHOLD: u8 = 210;
/// Alfa mínimo para um pixel contar como "conteúdo" ao aparar as margens.
const CONTENT_ALPHA: u8 = 8;
/// Margem deixada ao redor da arte, como fração do lado do conteúdo.
const PADDING_FRACTION: f32 = 0.04;

/// A origem já veio recortada? (mais de 1% dos pixels não totalmente opacos)
fn has_real_transparency(img: &RgbaImage) -> bool {
    let total = u64::from(img.width()) * u64::from(img.height());
    let non_opaque = img.pixels().filter(|p| p[3] < 250).count() as u64;
    non_opaque * 100 > total
}

/// Caixa delimitadora do conteúdo visível: `(x0, y0, x1, y1)`, inclusiva.
fn content_bbox(img: &RgbaImage) -> Option<(u32, u32, u32, u32)> {
    let (w, h) = img.dimensions();
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0u32, 0u32);
    let mut found = false;
    for y in 0..h {
        for x in 0..w {
            if img.get_pixel(x, y)[3] > CONTENT_ALPHA {
                found = true;
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
            }
        }
    }
    found.then_some((x0, y0, x1, y1))
}

/// Apara as margens transparentes e centraliza num quadrado com margem.
///
/// Sem isto, um `resize` direto para um quadrado distorceria arte não-quadrada
/// e manteria qualquer margem morta que a origem tivesse.
fn trim_to_square(img: &RgbaImage) -> RgbaImage {
    let Some((x0, y0, x1, y1)) = content_bbox(img) else {
        return img.clone(); // imagem inteiramente transparente: nada a aparar
    };
    let (cw, ch) = (x1 - x0 + 1, y1 - y0 + 1);
    let side = cw.max(ch);
    let pad = (side as f32 * PADDING_FRACTION).round() as u32;
    let canvas = side + 2 * pad;

    println!(
        "  conteúdo: {cw}x{ch} em ({x0},{y0}) -> quadrado {canvas}x{canvas} (margem {pad}px)"
    );

    let mut out = RgbaImage::from_pixel(canvas, canvas, Rgba([0, 0, 0, 0]));
    let (ox, oy) = (pad + (side - cw) / 2, pad + (side - ch) / 2);
    for y in 0..ch {
        for x in 0..cw {
            out.put_pixel(ox + x, oy + y, *img.get_pixel(x0 + x, y0 + y));
        }
    }
    out
}

fn is_near_white(p: &Rgba<u8>, threshold: u8) -> bool {
    p[0] >= threshold && p[1] >= threshold && p[2] >= threshold
}

/// Remove o fundo branco conectado às bordas. Retorna imagem RGBA transparente.
fn remove_white_background(src: &RgbaImage) -> RgbaImage {
    let (w, h) = src.dimensions();
    let mut out = src.clone();

    // `bg[y*w + x]` = true se o pixel pertence ao fundo (a ser tornado transparente).
    let mut bg = vec![false; (w * h) as usize];
    let mut queue: VecDeque<(u32, u32)> = VecDeque::new();

    // Semeia a fila com todos os pixels de borda que forem quase brancos.
    let seed = |x: u32, y: u32, bg: &mut [bool], queue: &mut VecDeque<(u32, u32)>| {
        let idx = (y * w + x) as usize;
        if !bg[idx] && is_near_white(&src.get_pixel(x, y), WHITE_THRESHOLD) {
            bg[idx] = true;
            queue.push_back((x, y));
        }
    };
    for x in 0..w {
        seed(x, 0, &mut bg, &mut queue);
        seed(x, h - 1, &mut bg, &mut queue);
    }
    for y in 0..h {
        seed(0, y, &mut bg, &mut queue);
        seed(w - 1, y, &mut bg, &mut queue);
    }

    // Flood-fill 4-conectado sobre pixels quase brancos.
    while let Some((x, y)) = queue.pop_front() {
        let visit = |nx: u32, ny: u32, bg: &mut [bool], queue: &mut VecDeque<(u32, u32)>| {
            let idx = (ny * w + nx) as usize;
            if !bg[idx] && is_near_white(&src.get_pixel(nx, ny), WHITE_THRESHOLD) {
                bg[idx] = true;
                queue.push_back((nx, ny));
            }
        };
        if x > 0 {
            visit(x - 1, y, &mut bg, &mut queue);
        }
        if x + 1 < w {
            visit(x + 1, y, &mut bg, &mut queue);
        }
        if y > 0 {
            visit(x, y - 1, &mut bg, &mut queue);
        }
        if y + 1 < h {
            visit(x, y + 1, &mut bg, &mut queue);
        }
    }

    // Aplica transparência ao fundo.
    for y in 0..h {
        for x in 0..w {
            if bg[(y * w + x) as usize] {
                out.put_pixel(x, y, Rgba([0, 0, 0, 0]));
            }
        }
    }

    // Feather de 1px: pixels claros do sujeito, adjacentes ao fundo, ficam
    // semi-transparentes para reduzir a franja branca ao redor do recorte.
    let snapshot = out.clone();
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            if bg[idx] {
                continue;
            }
            let p = *snapshot.get_pixel(x, y);
            if !is_near_white(&p, FEATHER_THRESHOLD) {
                continue;
            }
            let touches_bg = [
                (x.wrapping_sub(1), y),
                (x + 1, y),
                (x, y.wrapping_sub(1)),
                (x, y + 1),
            ]
            .iter()
            .any(|&(nx, ny)| nx < w && ny < h && bg[(ny * w + nx) as usize]);
            if touches_bg {
                let mut np = p;
                np[3] = 128;
                out.put_pixel(x, y, np);
            }
        }
    }

    out
}

/// Codificação base64 padrão (sem dependência externa), para embutir no SVG.
fn base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | (b[2] as u32);
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

fn resize(img: &RgbaImage, size: u32) -> RgbaImage {
    image::imageops::resize(img, size, size, FilterType::Lanczos3)
}

fn png_bytes(img: &RgbaImage) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
        .expect("falha ao codificar PNG");
    buf
}

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_path = root.join("logo.png");
    let assets = root.join("assets");
    let iconset = assets.join("AppIcon.iconset");
    let icons_dir = assets.join("icons");
    fs::create_dir_all(&iconset).expect("criar AppIcon.iconset");
    fs::create_dir_all(&icons_dir).expect("criar assets/icons");
    fs::create_dir_all(assets.join("anim")).ok();

    println!("Lendo {}", src_path.display());
    let src = image::open(&src_path)
        .expect("não consegui abrir logo.png")
        .to_rgba8();
    println!("  original: {}x{}", src.width(), src.height());

    // A origem já está recortada? Então preservamos o alfa dela intacto.
    let cut = if has_real_transparency(&src) {
        println!("logo.png já tem canal alfa — preservando o recorte da origem.");
        src.clone()
    } else {
        println!("logo.png é opaco — removendo fundo branco (flood-fill das bordas)...");
        remove_white_background(&src)
    };

    println!("Aparando margens transparentes e centralizando...");
    let cut = trim_to_square(&cut);

    // Base 1024 transparente + ícone 256 da janela.
    let base = resize(&cut, 1024);
    base.save(assets.join("logo_transparent.png"))
        .expect("salvar logo_transparent.png");
    resize(&cut, 256)
        .save(assets.join("window_icon.png"))
        .expect("salvar window_icon.png");
    println!("  -> assets/logo_transparent.png (1024)");
    println!("  -> assets/window_icon.png (256)");

    // SVG escalável embutindo o PNG transparente (512).
    let svg_png = resize(&cut, 512);
    let b64 = base64_encode(&png_bytes(&svg_png));
    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"512\" height=\"512\" \
viewBox=\"0 0 512 512\">\n  <image width=\"512\" height=\"512\" \
href=\"data:image/png;base64,{b64}\"/>\n</svg>\n"
    );
    fs::write(assets.join("logo.svg"), svg).expect("salvar logo.svg");
    println!("  -> assets/logo.svg (512, embute PNG sem fundo)");

    // Iconset para o macOS (.icns via `iconutil`).
    let variants: &[(&str, u32)] = &[
        ("icon_16x16.png", 16),
        ("icon_16x16@2x.png", 32),
        ("icon_32x32.png", 32),
        ("icon_32x32@2x.png", 64),
        ("icon_128x128.png", 128),
        ("icon_128x128@2x.png", 256),
        ("icon_256x256.png", 256),
        ("icon_256x256@2x.png", 512),
        ("icon_512x512.png", 512),
        ("icon_512x512@2x.png", 1024),
    ];
    for (name, size) in variants {
        resize(&cut, *size)
            .save(iconset.join(name))
            .unwrap_or_else(|_| panic!("salvar {name}"));
    }
    println!("  -> assets/AppIcon.iconset/ ({} tamanhos)", variants.len());
    println!("\nAgora rode:\n  iconutil -c icns assets/AppIcon.iconset -o assets/icons/AppIcon.icns");
}
