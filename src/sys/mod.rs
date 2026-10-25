//! Coleta de informações do sistema operacional (processos, threads, …).

pub mod devices;
pub mod fs;
pub mod processes;

pub use processes::ProcessSnapshot;
