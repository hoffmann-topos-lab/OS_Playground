//! Leitura ao vivo de `/dev`, para a tela Dispositivos ao Vivo (capítulo 5).
//!
//! A seção 5.1.1 divide os dispositivos de entrada e saída em dois tipos:
//! dispositivos de bloco e dispositivos de caractere. No UNIX essa divisão
//! aparece de forma concreta no sistema de arquivos: cada dispositivo é um
//! arquivo especial em `/dev`, marcado como `b` (bloco) ou `c` (caractere) pelo
//! `ls -l`, e identificado por um par de números maior e menor.
//!
//! Tudo vem de `std::os::unix::fs`, então a tela só faz sentido em UNIX.

use std::os::unix::fs::{FileTypeExt, MetadataExt};

/// A classe de um dispositivo (seção 5.1.1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DevKind {
    /// Dispositivo de bloco: lê e escreve blocos endereçáveis, como um disco.
    Block,
    /// Dispositivo de caractere: um fluxo de bytes, como um terminal.
    Char,
}

impl DevKind {
    pub fn ls_char(self) -> char {
        match self {
            DevKind::Block => 'b',
            DevKind::Char => 'c',
        }
    }
}

/// Um arquivo especial de dispositivo encontrado em `/dev`.
#[derive(Clone)]
pub struct Device {
    pub name: String,
    pub kind: DevKind,
    /// Número maior: identifica o driver que atende o dispositivo (seção 5.3.2).
    pub major: u32,
    /// Número menor: identifica a unidade concreta dentro daquele driver.
    pub minor: u32,
}

/// O resultado da varredura de `/dev`.
pub struct DevList {
    pub devices: Vec<Device>,
    /// Quantos dispositivos de bloco foram encontrados.
    pub block_count: usize,
    /// Quantos dispositivos de caractere foram encontrados.
    pub char_count: usize,
    pub error: Option<String>,
}

impl DevList {
    /// Lê o diretório `/dev` e devolve os arquivos especiais de bloco e de
    /// caractere, ordenados por classe e nome.
    pub fn read() -> Self {
        let mut devices = Vec::new();
        let read = match std::fs::read_dir("/dev") {
            Ok(rd) => rd,
            Err(e) => {
                return DevList {
                    devices,
                    block_count: 0,
                    char_count: 0,
                    error: Some(format!("não foi possível ler /dev: {e}")),
                };
            }
        };

        for item in read.flatten() {
            let path = item.path();
            let meta = match std::fs::symlink_metadata(&path) {
                Ok(m) => m,
                Err(_) => continue,
            };
            let ft = meta.file_type();
            let kind = if ft.is_block_device() {
                DevKind::Block
            } else if ft.is_char_device() {
                DevKind::Char
            } else {
                // Em /dev também há links, FIFOs e diretórios; só queremos os
                // arquivos especiais de bloco e de caractere.
                continue;
            };
            let rdev = meta.rdev();
            let (major, minor) = split_dev(rdev);
            devices.push(Device {
                name: item.file_name().to_string_lossy().into_owned(),
                kind,
                major,
                minor,
            });
        }

        devices.sort_by(|a, b| {
            let ab = a.kind == DevKind::Block;
            let bb = b.kind == DevKind::Block;
            bb.cmp(&ab)
                .then_with(|| a.major.cmp(&b.major))
                .then_with(|| a.minor.cmp(&b.minor))
                .then_with(|| a.name.cmp(&b.name))
        });

        let block_count = devices.iter().filter(|d| d.kind == DevKind::Block).count();
        let char_count = devices.len() - block_count;
        DevList {
            devices,
            block_count,
            char_count,
            error: None,
        }
    }
}

/// Decompõe o identificador de dispositivo (st_rdev) em maior e menor.
///
/// A codificação é específica do sistema, então tratamos os dois casos que
/// interessam ao projeto: macOS (BSD) e Linux (glibc).
#[cfg(target_os = "macos")]
fn split_dev(rdev: u64) -> (u32, u32) {
    let major = ((rdev >> 24) & 0xff) as u32;
    let minor = (rdev & 0x00ff_ffff) as u32;
    (major, minor)
}

#[cfg(target_os = "linux")]
fn split_dev(rdev: u64) -> (u32, u32) {
    let major = (((rdev >> 8) & 0xfff) | ((rdev >> 32) & !0xfff)) as u32;
    let minor = ((rdev & 0xff) | ((rdev >> 12) & !0xff)) as u32;
    (major, minor)
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn split_dev(rdev: u64) -> (u32, u32) {
    // Fallback genérico: aproximação usada por vários UNIX.
    (((rdev >> 8) & 0xff) as u32, (rdev & 0xff) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_dev_and_finds_devices() {
        let list = DevList::read();
        // Em qualquer UNIX há dispositivos de caractere clássicos em /dev
        // (null, zero, tty). Se /dev existe, a lista não deve vir vazia.
        if list.error.is_none() {
            assert!(
                !list.devices.is_empty(),
                "/dev deveria ter arquivos especiais"
            );
            assert_eq!(
                list.block_count + list.char_count,
                list.devices.len(),
                "a soma das classes bate com o total"
            );
        }
    }
}
