//! Captura da hierarquia de processos ao vivo via `sysinfo`.
//!
//! Constrói a árvore pai → filho a partir dos PIDs. No UNIX a raiz costuma ser
//! o `pid 1` (`launchd` no macOS, `init`/`systemd` no Linux) — exatamente o
//! processo raiz descrito nas anotações (seção 2.1.4, Hierarquia de processos).

use std::collections::HashMap;

use sysinfo::{Pid, System};

/// Informação de um único processo, já normalizada para a UI.
#[derive(Clone)]
pub struct ProcInfo {
    pub pid: u32,
    pub parent: Option<u32>,
    pub name: String,
    pub cmd: String,
    pub status: String,
    pub memory_bytes: u64,
    pub cpu: f32,
    pub start_time: u64,
    /// Nº de threads (tasks). Disponível no Linux; `None` onde o SO não expõe.
    pub thread_count: Option<usize>,
    /// PIDs dos filhos diretos, ordenados por PID.
    pub children: Vec<u32>,
}

/// Um retrato instantâneo de todos os processos e sua hierarquia.
pub struct ProcessSnapshot {
    pub procs: HashMap<u32, ProcInfo>,
    /// Processos raiz (sem pai visível), ordenados por PID.
    pub roots: Vec<u32>,
    pub total: usize,
}

impl ProcessSnapshot {
    /// Coleta um novo retrato do sistema.
    pub fn capture() -> Self {
        let mut sys = System::new();
        // Dois refreshes com um respiro no meio permitem medir uso de CPU
        // (a 1ª leitura é sempre 0; a 2ª compara com a anterior).
        sys.refresh_all();
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        sys.refresh_all();

        let mut procs: HashMap<u32, ProcInfo> = HashMap::new();
        for (pid, p) in sys.processes() {
            let pid = pid.as_u32();
            let cmd = p
                .cmd()
                .iter()
                .map(|s| s.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ");
            procs.insert(
                pid,
                ProcInfo {
                    pid,
                    parent: p.parent().map(Pid::as_u32),
                    name: p.name().to_string_lossy().into_owned(),
                    cmd,
                    status: p.status().to_string(),
                    memory_bytes: p.memory(),
                    cpu: p.cpu_usage(),
                    start_time: p.start_time(),
                    thread_count: p.tasks().map(|t| t.len()),
                    children: Vec::new(),
                },
            );
        }

        // Liga cada processo ao pai; o que não tiver pai visível vira raiz.
        let mut roots: Vec<u32> = Vec::new();
        let ids: Vec<u32> = procs.keys().copied().collect();
        for id in ids {
            let parent = procs[&id].parent;
            match parent {
                Some(pp) if pp != id && procs.contains_key(&pp) => {
                    procs.get_mut(&pp).unwrap().children.push(id);
                }
                _ => roots.push(id),
            }
        }

        // Ordena filhos e raízes por PID para estabilidade visual.
        for p in procs.values_mut() {
            p.children.sort_unstable();
        }
        roots.sort_unstable();

        let total = procs.len();
        ProcessSnapshot {
            procs,
            roots,
            total,
        }
    }

    pub fn get(&self, pid: u32) -> Option<&ProcInfo> {
        self.procs.get(&pid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captures_a_consistent_tree() {
        let snap = ProcessSnapshot::capture();
        assert!(snap.total > 0, "deveria existir ao menos um processo");
        assert!(!snap.roots.is_empty(), "deveria existir ao menos uma raiz");

        // Todo filho referenciado precisa existir no mapa (árvore consistente).
        for p in snap.procs.values() {
            for child in &p.children {
                assert!(snap.procs.contains_key(child), "filho {child} sem entrada");
            }
        }
        // Nenhuma raiz tem pai visível dentro do mapa.
        for &r in &snap.roots {
            if let Some(pp) = snap.procs[&r].parent {
                assert!(!snap.procs.contains_key(&pp), "raiz {r} tem pai visível {pp}");
            }
        }
    }
}
