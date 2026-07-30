<p align="center">
  <img src="assets/logo_transparent.png" alt="OS Playground" width="200">
</p>

# OS Playground

Programa em Rust + egui feito para ser usado na disciplina de **Sistemas
Operacionais**. Ele visualiza os conceitos vistos em aula, com base nas
anotações do curso (Tanenbaum & Bos, *Sistemas Operacionais Modernos*) e no
[OSTEP](https://pages.cs.wisc.edu/~remzi/OSTEP/).

## O que já está implementado

Tópico **Processos e Threads** (capítulo 2), com quatro telas:

- **Árvore de Processos** — hierarquia real da máquina, ao vivo, desenhada como
  árvore (caixas e ramos) com pan, zoom, recolhimento de subárvores, filtro e
  painel de detalhes.
- **Conceitos (§2.1–2.2)** — 11 artigos: modelo de processo, criação e término,
  hierarquia e `init`, estados, tabela de processos, threads (modelo clássico,
  Pthreads, usuário/kernel/híbridas) e a síntese Processo × Thread.
- **Race Conditions (§2.4)** — 13 artigos sobre região crítica e exclusão mútua
  (interrupções, trava, alternância, Peterson, TSL, dormir/despertar, semáforos,
  mutexes, monitores, mensagens, barreiras), 8 deles com demonstração animada
  passo a passo.
- **Escalonamento (§2.5)** — 11 artigos (FCFS, SJF, SRTF, round-robin,
  prioridade, múltiplas filas, loteria, threads), 7 deles com simulador de Gantt
  editável e métricas de espera e retorno.

Todos os artigos terminam com as referências do Tanenbaum & Bos e os capítulos
correspondentes do OSTEP.

## Como executar

Requer apenas o toolchain Rust.

```bash
cargo run      # modo debug
make run       # modo release
make app       # bundle macOS -> dist/"OS Playground".app
```

## Próximos passos

- Detalhe de threads por processo.
- Novos tópicos de SO (memória, sistemas de arquivos).
