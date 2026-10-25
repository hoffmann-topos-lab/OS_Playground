//! Leitura ao vivo de um diretório real, para a tela Explorador de Arquivos
//! (capítulo 4). Mostra na prática o que o texto descreve: o número de i-node
//! de cada arquivo, a contagem de ligações (hard links), o alvo de uma ligação
//! simbólica e os metadados que a chamada stat devolve.
//!
//! Tudo vem de `std::os::unix::fs` (lstat por baixo), então a tela só faz
//! sentido em sistemas UNIX. No macOS e no Linux os campos existem todos.

use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::path::{Path, PathBuf};

/// O tipo de um arquivo, na classificação do capítulo 4 (seção 4.1.3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// Arquivo regular (comum do usuário).
    Regular,
    /// Diretório.
    Dir,
    /// Ligação simbólica (soft link).
    Symlink,
    /// Arquivo especial de caractere (seção 4.1.3): teclado, terminal, /dev/null.
    CharDevice,
    /// Arquivo especial de bloco (seção 4.1.3): um disco ou partição.
    BlockDevice,
    /// Cano nomeado (FIFO).
    Fifo,
    /// Soquete local.
    Socket,
    /// Qualquer outro.
    Other,
}

impl Kind {
    /// A letra que o `ls -l` mostra na primeira coluna.
    pub fn ls_char(self) -> char {
        match self {
            Kind::Regular => '-',
            Kind::Dir => 'd',
            Kind::Symlink => 'l',
            Kind::CharDevice => 'c',
            Kind::BlockDevice => 'b',
            Kind::Fifo => 'p',
            Kind::Socket => 's',
            Kind::Other => '?',
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Kind::Regular => "arquivo regular",
            Kind::Dir => "diretório",
            Kind::Symlink => "ligação simbólica",
            Kind::CharDevice => "especial de caractere",
            Kind::BlockDevice => "especial de bloco",
            Kind::Fifo => "cano nomeado (FIFO)",
            Kind::Socket => "soquete",
            Kind::Other => "outro",
        }
    }
}

/// Uma entrada de diretório, já com os metadados do i-node.
#[derive(Clone)]
pub struct Entry {
    pub name: String,
    pub path: PathBuf,
    pub kind: Kind,
    /// Número do i-node (o nome de baixo nível do arquivo, seção 4.1).
    pub inode: u64,
    /// Identificador do dispositivo que contém o arquivo.
    pub dev: u64,
    /// Contagem de ligações (hard links) para este i-node.
    pub nlink: u64,
    /// Tamanho em bytes.
    pub size: u64,
    /// Blocos de 512 bytes alocados ao arquivo.
    pub blocks: u64,
    /// Bits de permissão e tipo (st_mode).
    pub mode: u32,
    /// Alvo da ligação, quando a entrada é simbólica.
    pub symlink_target: Option<String>,
    /// Marca de grupo de hard links: entradas com o mesmo i-node e dispositivo
    /// neste diretório recebem o mesmo índice de cor; `None` quando nlink == 1.
    pub hardlink_group: Option<usize>,
}

impl Entry {
    /// As nove letras de permissão (rwx para dono, grupo e outros).
    pub fn perm_string(&self) -> String {
        let bits = self.mode;
        let tri = |shift: u32| {
            let r = if bits & (0o4 << shift) != 0 { 'r' } else { '-' };
            let w = if bits & (0o2 << shift) != 0 { 'w' } else { '-' };
            let x = if bits & (0o1 << shift) != 0 { 'x' } else { '-' };
            format!("{r}{w}{x}")
        };
        format!("{}{}{}", tri(6), tri(3), tri(0))
    }
}

/// O conteúdo de um diretório lido do disco.
pub struct Listing {
    /// O caminho absoluto do diretório lido.
    pub dir: PathBuf,
    pub entries: Vec<Entry>,
    /// Mensagem de erro quando o diretório não pôde ser lido.
    pub error: Option<String>,
}

impl Listing {
    /// Lê `dir` e devolve suas entradas, ordenadas (diretórios primeiro, depois
    /// por nome). As ligações simbólicas são detectadas com lstat, então um
    /// link para um diretório aparece como link, não como diretório.
    pub fn read(dir: &Path) -> Self {
        let dir = std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
        let mut entries = Vec::new();
        let read = match std::fs::read_dir(&dir) {
            Ok(rd) => rd,
            Err(e) => {
                return Listing {
                    dir,
                    entries,
                    error: Some(format!("não foi possível ler o diretório: {e}")),
                };
            }
        };

        for item in read.flatten() {
            let path = item.path();
            // lstat: não segue a ligação, para distinguir link de alvo.
            let meta = match std::fs::symlink_metadata(&path) {
                Ok(m) => m,
                Err(_) => continue,
            };
            let ft = meta.file_type();
            let kind = if ft.is_symlink() {
                Kind::Symlink
            } else if ft.is_dir() {
                Kind::Dir
            } else if ft.is_file() {
                Kind::Regular
            } else if ft.is_char_device() {
                Kind::CharDevice
            } else if ft.is_block_device() {
                Kind::BlockDevice
            } else if ft.is_fifo() {
                Kind::Fifo
            } else if ft.is_socket() {
                Kind::Socket
            } else {
                Kind::Other
            };
            let symlink_target = if kind == Kind::Symlink {
                std::fs::read_link(&path)
                    .ok()
                    .map(|t| t.to_string_lossy().into_owned())
            } else {
                None
            };
            entries.push(Entry {
                name: item.file_name().to_string_lossy().into_owned(),
                path,
                kind,
                inode: meta.ino(),
                dev: meta.dev(),
                nlink: meta.nlink(),
                size: meta.size(),
                blocks: meta.blocks(),
                mode: meta.mode(),
                symlink_target,
                hardlink_group: None,
            });
        }

        entries.sort_by(|a, b| {
            let ad = a.kind == Kind::Dir;
            let bd = b.kind == Kind::Dir;
            bd.cmp(&ad).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });

        mark_hardlink_groups(&mut entries);

        Listing {
            dir,
            entries,
            error: None,
        }
    }

    /// Soma dos tamanhos dos arquivos regulares, para o rodapé.
    pub fn total_size(&self) -> u64 {
        self.entries
            .iter()
            .filter(|e| e.kind == Kind::Regular)
            .map(|e| e.size)
            .sum()
    }
}

/// Atribui um índice de grupo a entradas que dividem o mesmo i-node, ou seja,
/// são hard links do mesmo arquivo dentro deste diretório. Isso torna visível
/// o conceito da seção 4.3.4 (arquivos compartilhados).
fn mark_hardlink_groups(entries: &mut [Entry]) {
    use std::collections::HashMap;
    let mut seen: HashMap<(u64, u64), Vec<usize>> = HashMap::new();
    for (i, e) in entries.iter().enumerate() {
        if e.nlink > 1 && e.kind == Kind::Regular {
            seen.entry((e.dev, e.inode)).or_default().push(i);
        }
    }
    let mut group = 0usize;
    let mut groups: Vec<(&(u64, u64), &Vec<usize>)> =
        seen.iter().filter(|(_, v)| v.len() > 1).collect();
    // Ordem estável: pelo primeiro índice de aparição.
    groups.sort_by_key(|(_, v)| v[0]);
    for (_, idxs) in groups {
        for &i in idxs {
            entries[i].hardlink_group = Some(group);
        }
        group += 1;
    }
}

/// Converte bytes para uma string curta (B, KB, MB, GB), base 1024.
pub fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut v = bytes as f64;
    let mut u = 0;
    while v >= 1024.0 && u < UNITS.len() - 1 {
        v /= 1024.0;
        u += 1;
    }
    if u == 0 {
        format!("{bytes} B")
    } else {
        format!("{v:.1} {}", UNITS[u])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_current_directory() {
        let here = std::env::current_dir().unwrap();
        let listing = Listing::read(&here);
        assert!(listing.error.is_none(), "deveria ler o diretório atual");
        assert!(
            !listing.entries.is_empty(),
            "o diretório do projeto não está vazio"
        );
        // Todo i-node é um número positivo e o modo traz bits de tipo.
        for e in &listing.entries {
            assert!(e.inode > 0, "i-node inválido para {}", e.name);
        }
    }

    #[test]
    fn human_size_scales() {
        assert_eq!(human_size(6), "6 B");
        assert_eq!(human_size(4096), "4.0 KB");
        assert_eq!(human_size(1024 * 1024), "1.0 MB");
    }

    #[test]
    fn permission_string_has_nine_chars() {
        let here = std::env::current_dir().unwrap();
        let listing = Listing::read(&here);
        if let Some(e) = listing.entries.first() {
            assert_eq!(e.perm_string().len(), 9);
        }
    }
}
