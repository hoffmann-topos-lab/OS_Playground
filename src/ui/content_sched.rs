//! Conteúdo da tela "Escalonamento" — seção 2.5 das anotações.
//!
//! Cada algoritmo ganha uma explicação curta (o detalhamento é em sala) e um
//! cenário vivo no simulador, com os tempos editáveis para o aluno mexer e ver
//! o Gantt e as métricas recalcularem.

use crate::app::Screen;
use crate::ui::gantt::{Job, Policy, Scenario};
use crate::ui::reader::{ostep, Article, Block};

/// Carga de trabalho usada na maioria dos cenários.
///
/// Mesmos processos em todos os algoritmos comparáveis: mudando só a política,
/// dá para ver o efeito dela isolado do efeito da carga.
const BASE: &[Job] = &[
    Job { name: "P1", arrival: 0, burst: 7, priority: 2, tickets: 10 },
    Job { name: "P2", arrival: 2, burst: 4, priority: 5, tickets: 30 },
    Job { name: "P3", arrival: 4, burst: 1, priority: 3, tickets: 20 },
    Job { name: "P4", arrival: 5, burst: 4, priority: 1, tickets: 40 },
];

static FCFS: Scenario = Scenario {
    id: "fcfs",
    policy: Policy::Fcfs,
    jobs: BASE,
    quantum: 1,
    hint: "Aumente a duração de P1 e veja a fila inteira atrasar atrás dele.",
};

static SJF: Scenario = Scenario {
    id: "sjf",
    policy: Policy::Sjf,
    jobs: BASE,
    quantum: 1,
    hint: "Compare o tempo de espera médio com o do FCFS, na mesma carga.",
};

static SRTF: Scenario = Scenario {
    id: "srtf",
    policy: Policy::Srtf,
    jobs: BASE,
    quantum: 1,
    hint: "Agora P1 é interrompido no meio quando alguém mais curto chega.",
};

static RR: Scenario = Scenario {
    id: "rr",
    policy: Policy::RoundRobin,
    jobs: BASE,
    quantum: 2,
    hint: "Mexa no quantum: pequeno fatia demais, grande vira FCFS.",
};

static PRIORITY: Scenario = Scenario {
    id: "priority",
    policy: Policy::Priority,
    jobs: BASE,
    quantum: 1,
    hint: "Prioridade maior ganha a CPU. Baixe a de P2 e veja P1 destravar.",
};

static MLFQ: Scenario = Scenario {
    id: "mlfq",
    policy: Policy::MultiQueue,
    jobs: BASE,
    quantum: 1,
    hint: "Quem gasta o quantum inteiro desce de classe e ganha um quantum maior.",
};

static LOTTERY: Scenario = Scenario {
    id: "lottery",
    policy: Policy::Lottery,
    jobs: BASE,
    quantum: 1,
    hint: "Mais bilhetes, mais chance por sorteio. Zere os de P4 e observe.",
};

pub const ARTICLES: &[Article] = &[
    Article {
        icon: "🎛",
        title: "Introdução ao escalonamento",
        section: "2.5 e 2.5.1",
        ostep: &[ostep::SCHEDULING],
        summary: "Quem decide qual processo roda, e por que a troca custa caro.",
        blocks: &[
            Block::P(
                "Computadores modernos executam múltiplos processos e threads ao mesmo tempo, \
                 competindo pelo processamento da CPU. O mecanismo que escolhe quem roda é o \
                 escalonador, e a regra que ele segue é o algoritmo de escalonamento.",
            ),
            Block::Note(
                "Boa parte das questões de escalonamento de processos e de threads é a \
                 mesma, então vale focar primeiro nos aspectos comuns.",
            ),
            Block::P(
                "Com a quantidade atual de recursos de processamento, a CPU deixou de ser o \
                 gargalo dos computadores pessoais. Ainda assim, escolher um bom algoritmo \
                 continua sendo relevante para servidores, IoT e celulares.",
            ),
            Block::H("A troca de contexto é cara"),
            Block::P(
                "Além de escolher qual processo executar, o escalonador precisa usar a CPU de \
                 forma eficiente, porque trocar de processo tem um custo próprio:",
            ),
            Block::N(&[
                "É preciso passar do modo usuário para o modo kernel.",
                "O processo atual precisa ser salvo: seus registradores vão para a tabela de \
                 processos, junto com o mapa de memória. É a troca de contexto.",
                "Um novo processo é selecionado.",
                "A MMU carrega o mapa de memória do novo processo.",
            ]),
            Block::Warn(
                "Guarde isso ao mexer no quantum do round-robin mais adiante: cada fatia a \
                 mais no Gantt é uma troca de contexto a mais, e o custo dela não aparece no \
                 desenho.",
            ),
            Block::H("Comportamento dos processos"),
            Block::P(
                "Quase todos os processos alternam entre surtos de computação e solicitações \
                 de entrada/saída. Onde eles passam a maior parte do tempo os separa em duas \
                 famílias:",
            ),
            Block::Defs(&[
                (
                    "Vinculados à computação",
                    "passam a maior parte do tempo calculando: longos períodos de computação \
                     com curtas esperas por I/O.",
                ),
                (
                    "Vinculados a I/O",
                    "passam a maior parte do tempo esperando por uma operação de \
                     entrada/saída.",
                ),
            ]),
        ],
    },
    Article {
        icon: "⏰",
        title: "Quando escalonar",
        section: "2.5.1",
        ostep: &[ostep::SCHEDULING],
        summary: "Os momentos de decisão, e a divisão preemptivo × não-preemptivo.",
        blocks: &[
            Block::P(
                "O algoritmo de escalonamento precisa decidir em vários momentos distintos:",
            ),
            Block::L(&[
                "Quando um novo processo é criado: roda o pai ou o filho?",
                "Quando um processo termina: outro precisa ser escolhido do conjunto dos \
                 prontos. Se nenhum estiver pronto, roda um processo ocioso gerado pelo \
                 sistema.",
                "Quando um processo é bloqueado, por qualquer razão, é preciso selecionar \
                 outro.",
                "Quando há uma interrupção de I/O: se ela veio de um dispositivo que concluiu \
                 seu trabalho, o processo que estava bloqueado esperando por ela agora pode \
                 executar.",
            ]),
            Block::H("Preemptivo ou não"),
            Block::P(
                "Se o hardware de relógio fornece interrupções periódicas, uma decisão de \
                 escalonamento pode ser tomada a cada interrupção, ou a cada n interrupções. \
                 É aí que os algoritmos se dividem em duas classes:",
            ),
            Block::Table2(
                "Não preemptivo",
                "Preemptivo",
                &[
                    (
                        "Escolhe um processo e o deixa executar até que ele bloqueie ou \
                         libere a CPU voluntariamente.",
                        "Escolhe um processo e o deixa executar por um tempo máximo fixado.",
                    ),
                    (
                        "Nenhuma decisão é tomada durante a interrupção de relógio: ela é \
                         concluída e o controle volta ao processo que estava rodando.",
                        "Uma interrupção de relógio marca o fim do intervalo, e nesse ponto \
                         o escalonador pode trocar de processo.",
                    ),
                ],
            ),
            Block::Note(
                "Nas demonstrações a seguir, essa diferença fica visível de imediato: nos \
                 algoritmos não-preemptivos cada processo aparece como um bloco contínuo no \
                 Gantt. Nos preemptivos, ele aparece picotado.",
            ),
        ],
    },
    Article {
        icon: "🎯",
        title: "Categorias e metas",
        section: "2.5.1",
        ostep: &[ostep::SCHEDULING],
        summary: "O que otimizar muda conforme o ambiente.",
        blocks: &[
            Block::P(
                "Ambientes diferentes exigem algoritmos diferentes: o que o escalonador deve \
                 otimizar muda conforme o sistema. Vale considerar três ambientes.",
            ),
            Block::Defs(&[
                (
                    "Lote",
                    "ainda usados para operações contáveis, como folhas de pagamento, estoque, \
                     cálculo de juros. Não há usuário aguardando resposta no terminal, o que \
                     permite algoritmos não-preemptivos, ou preemptivos com fatias longas.",
                ),
                (
                    "Interativo",
                    "com vários usuários e serviços, a preempção é fundamental para que \
                     nenhum processo domine a CPU.",
                ),
                (
                    "Tempo real",
                    "o foco é cumprir as tarefas o mais rápido possível, e os algoritmos não \
                     precisam necessariamente ser preemptivos. A diferença para os sistemas \
                     interativos é que aqui o sistema é modelado para uma tarefa específica.",
                ),
            ]),
            Block::H("As metas"),
            Block::P("Algumas metas valem para todos os sistemas:"),
            Block::L(&[
                "Imparcialidade: dar a cada processo uma porção justa da CPU.",
                "Aplicação de políticas: verificar se as políticas estabelecidas estão sendo \
                 cumpridas.",
                "Equilíbrio: manter todas as partes do sistema ocupadas.",
            ]),
            Block::P("Outras são específicas de cada ambiente:"),
            Block::Table2(
                "Ambiente",
                "Metas próprias",
                &[
                    (
                        "Lote",
                        "Vazão (maximizar tarefas por hora), tempo de retorno (minimizar o \
                         tempo entre submissão e término), utilização de CPU (mantê-la \
                         ocupada o tempo todo).",
                    ),
                    (
                        "Interativo",
                        "Tempo de resposta (responder rápido às requisições), \
                         proporcionalidade (atender às expectativas do usuário).",
                    ),
                    (
                        "Tempo real",
                        "Cumprimento de prazos (evitar perda de dados), previsibilidade \
                         (evitar degradação da qualidade em sistemas multimídia).",
                    ),
                ],
            ),
            Block::Note(
                "As duas métricas que o simulador mostra, tempo de espera e tempo de \
                 retorno médios, são metas de sistemas em lote. Um algoritmo pode ser ruim \
                 nelas e ainda assim ser o certo para um sistema interativo, onde o que \
                 importa é o tempo de resposta.",
            ),
        ],
    },
    Article {
        icon: "1️⃣",
        title: "Primeiro a chegar, primeiro a ser servido",
        section: "2.5.2",
        ostep: &[ostep::SCHEDULING],
        summary: "FCFS, o não-preemptivo mais simples que existe.",
        blocks: &[
            Block::P(
                "É o algoritmo não-preemptivo mais simples: a CPU é distribuída unicamente \
                 pela ordem de requisição. Quem chegou primeiro roda primeiro, e roda até \
                 terminar.",
            ),
            Block::Table2(
                "Vantagem",
                "Desvantagem",
                &[(
                    "Fácil de compreender e de programar, com uma única lista encadeada \
                     controla todos os processos.",
                    "O uso do tempo não é otimizado: um processo longo na frente segura todo \
                     mundo atrás dele.",
                )],
            ),
            Block::Scheduler(&FCFS),
            Block::Note(
                "Repare no tempo de espera de P3, que precisa de uma unidade só de CPU e \
                 mesmo assim espera P1 e P2 inteiros. Guarde esse número para comparar com \
                 o próximo algoritmo.",
            ),
        ],
    },
    Article {
        icon: "2️⃣",
        title: "Tarefa mais curta primeiro",
        section: "2.5.2",
        ostep: &[ostep::SCHEDULING],
        summary: "SJF, ótimo quando dá para saber a duração de antemão.",
        blocks: &[
            Block::P(
                "Algoritmo não-preemptivo que presume o conhecimento prévio do tempo de \
                 execução de cada processo. Havendo vários processos de igual importância, o \
                 escalonador escolhe aquele com o tempo de execução mais curto.",
            ),
            Block::Warn(
                "A hipótese é forte: só é bom quando todas as tarefas já estão disponíveis e \
                 suas durações são conhecidas. Num sistema interativo real, ninguém sabe \
                 quanto tempo um processo vai levar.",
            ),
            Block::Scheduler(&SJF),
            Block::Note(
                "Mesma carga do FCFS, só a política mudou. Compare o tempo de espera médio \
                 entre as duas telas. A diferença é o efeito isolado do algoritmo.",
            ),
        ],
    },
    Article {
        icon: "3️⃣",
        title: "Tempo restante mais curto",
        section: "2.5.2",
        ostep: &[ostep::SCHEDULING],
        summary: "SRTF, a versão preemptiva do anterior.",
        blocks: &[
            Block::P(
                "É a versão preemptiva do escalonamento por tarefa mais curta. Em vez de \
                 comparar a duração total, o escalonador compara o tempo que falta: se um \
                 processo que chega tem tempo restante menor que o do processo em execução, \
                 este é interrompido.",
            ),
            Block::Scheduler(&SRTF),
            Block::Note(
                "Aqui aparece pela primeira vez o Gantt picotado: P1 é interrompido no meio \
                 quando alguém mais curto chega, e só volta depois. Esse picote é a \
                 preempção acontecendo.",
            ),
        ],
    },
    Article {
        icon: "🔁",
        title: "Escalonamento circular (round-robin)",
        section: "2.5.3",
        ostep: &[ostep::SCHEDULING],
        summary: "O quantum, e o custo escondido de cada troca.",
        blocks: &[
            Block::P(
                "É o algoritmo mais simples entre os interativos, e o implementado na maioria \
                 dos casos. Cada processo recebe um intervalo de tempo, o quantum, durante o \
                 qual tem permissão para operar. Se não tiver terminado ao fim do quantum, a \
                 CPU sofre preempção e passa ao próximo.",
            ),
            Block::P(
                "A implementação é uma fila circular de processos: quem consome todo o seu \
                 quantum sem terminar vai para o fim da fila.",
            ),
            Block::H("O tamanho do quantum"),
            Block::P(
                "A única questão realmente relevante é definir o tamanho do quantum, porque é \
                 ele que determina o número de trocas entre processos, e trocas são caras:",
            ),
            Block::Table2(
                "Quantum muito curto",
                "Quantum muito longo",
                &[(
                    "Trocas demais. O tempo gasto trocando de contexto reduz a eficiência.",
                    "Reduz o tempo de resposta médio. No limite, vira FCFS.",
                )],
            ),
            Block::Note(
                "Algo em torno de 20 a 50 ms costuma ser o desejável. No simulador abaixo, \
                 arraste o quantum de 1 até 8 e observe os dois extremos: com 1, o Gantt \
                 vira uma grade fina. Com 8, os blocos se fundem e o resultado fica igual ao \
                 do FCFS.",
            ),
            Block::Scheduler(&RR),
        ],
    },
    Article {
        icon: "⭐",
        title: "Escalonamento por prioridade",
        section: "2.5.3",
        ostep: &[ostep::SCHEDULING],
        summary: "Nem todo processo é igualmente urgente.",
        blocks: &[
            Block::P(
                "Cada processo recebe uma prioridade, e o processo de prioridade mais alta \
                 que estiver pronto para executar é o escolhido.",
            ),
            Block::H("O risco da inanição"),
            Block::P(
                "Sem cuidado, um processo de prioridade alta pode monopolizar a CPU \
                 indefinidamente. Há duas saídas:",
            ),
            Block::L(&[
                "O escalonador reduz a prioridade do processo em execução a cada ciclo de \
                 relógio, até que outro processo passe a ter prioridade maior.",
                "Ou define-se um quantum, como no round-robin.",
            ]),
            Block::P(
                "A atribuição de prioridades pode ser estática, com valores fixos, ou \
                 dinâmica, com o peso de cada processo determinado em tempo de execução.",
            ),
            Block::Scheduler(&PRIORITY),
            Block::Warn(
                "O simulador usa prioridades estáticas, para deixar o efeito puro do \
                 critério visível. Repare que P1, de prioridade baixa, só roda quando não \
                 sobra ninguém melhor. É a inanição em miniatura. Baixe a prioridade de P2 \
                 e veja P1 destravar.",
            ),
        ],
    },
    Article {
        icon: "🪜",
        title: "Múltiplas filas",
        section: "2.5.3",
        ostep: &[ostep::MLFQ],
        summary: "Classes de prioridade, com o quantum dobrando a cada degrau.",
        blocks: &[
            Block::P(
                "Como a troca entre processos é custosa, dá para refinar o escalonamento por \
                 prioridade agrupando os processos em classes.",
            ),
            Block::P(
                "Um processo na classe mais alta recebe um quantum. Quando termina esse \
                 quantum sem concluir, ele é rebaixado uma classe, e o número de quanta que \
                 ele recebe é multiplicado por dois.",
            ),
            Block::Note(
                "A lógica é econômica: processos longos acabam descendo para classes baixas, \
                 onde rodam com fatias grandes e portanto sofrem menos trocas de contexto. Já \
                 os processos curtos e interativos ficam no topo, onde a resposta é rápida.",
            ),
            Block::Scheduler(&MLFQ),
            Block::P(
                "No simulador, o quantum ajustável é o da classe mais alta, e cada classe \
                 abaixo dobra esse valor.",
            ),
        ],
    },
    Article {
        icon: "🎟",
        title: "Escalonamento por loteria",
        section: "2.5.3",
        ostep: &[ostep::LOTTERY],
        summary: "Distribuir bilhetes e sortear quem roda.",
        blocks: &[
            Block::P(
                "São distribuídos bilhetes de loteria para cada processo, que os usa para \
                 disputar recursos. A cada decisão de escalonamento é sorteado quem recebe \
                 acesso à CPU.",
            ),
            Block::P(
                "Processos com maior prioridade recebem mais bilhetes, o que aumenta a chance \
                 de serem sorteados, mas não a garante. A proporção de CPU que cada processo \
                 recebe tende à proporção de bilhetes que ele tem.",
            ),
            Block::Scheduler(&LOTTERY),
            Block::Note(
                "O sorteio do simulador é determinístico: a mesma carga produz sempre o \
                 mesmo Gantt, para a demonstração ser reprodutível em aula. Ajuste os \
                 bilhetes de um processo e veja a fatia dele crescer ou encolher.",
            ),
        ],
    },
    Article {
        icon: "🧵",
        title: "Escalonamento de threads",
        section: "2.5.6",
        ostep: &[ostep::THREADS],
        summary: "Dois níveis de decisão, quando há threads no meio.",
        blocks: &[
            Block::P(
                "Cada processo pode ter múltiplas threads, e por isso o escalonamento passa a \
                 acontecer em dois níveis. O mecanismo depende de o sistema dar suporte a \
                 threads de usuário, de kernel, ou ambas.",
            ),
            Block::P(
                "Nas threads de usuário, o kernel enxerga apenas o processo. Ele escalona \
                 esse processo normalmente, e então um segundo algoritmo de escalonamento, \
                 semelhante aos que estudamos, decide como repartir, entre as threads, os \
                 recursos computacionais que couberam ao processo.",
            ),
            Block::Warn(
                "Há uma diferença importante nesse segundo nível: não existe um sinal de \
                 relógio para interromper as threads. Sem interrupção de relógio, o \
                 escalonador de threads de usuário não tem como preemptar uma thread que não \
                 ceda a vez por conta própria.",
            ),
            Block::SeeAlso(
                "Os modelos de thread de usuário, de kernel e híbridos estão nos Conceitos.",
                Screen::Texts,
            ),
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::reader::check;

    #[test]
    fn conteudo_valido() {
        check::articles(ARTICLES, '2');
    }
}
