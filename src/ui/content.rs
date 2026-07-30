//! Conteúdo dos textos de "Processos e Threads".
//!
//! Tudo aqui vem das anotações em `Anotações/processos e threads.md` (Tanenbaum
//! & Bos, *Sistemas Operacionais Modernos*, capítulo 2), com o OSTEP como
//! leitura complementar. As seções 2.4 (race conditions) e 2.5 (escalonamento)
//! ficam nas telas próprias, e a 2.3 (servidores orientados a eventos) está
//! fora do curso — aqui cobrimos 2.1 e 2.2.

use crate::app::Screen;
use crate::ui::reader::{ostep, Article, Block};

pub const ARTICLES: &[Article] = &[
    // ---------------------------------------------------------------- 2.1.1
    Article {
        icon: "🧩",
        title: "O modelo de processo",
        section: "2.1 e 2.1.1",
        ostep: &[ostep::PROCESSES],
        summary: "A abstração central do sistema operacional.",
        blocks: &[
            Block::P(
                "Usar um computador significa manter várias tarefas no ar ao mesmo tempo: as \
                 que o usuário iniciou e as que o sistema operacional mantém em segundo plano. \
                 Para dar conta disso, a CPU alterna entre essas tarefas centenas de milhares \
                 de vezes por segundo, criando a ilusão de que todas correm em paralelo.",
            ),
            Block::Note(
                "Essa ilusão não é o paralelismo de verdade. O paralelismo de hardware exige \
                 vários processadores; aqui um único núcleo se reveza entre as tarefas rápido \
                 o bastante para parecer simultâneo.",
            ),
            Block::P(
                "O processo é a abstração criada para orquestrar esse revezamento. Todo o \
                 software executável da máquina — inclusive o próprio sistema operacional — é \
                 organizado como uma coleção de processos sequenciais.",
            ),
            Block::H("O que é um processo"),
            Block::P(
                "Um processo é uma instância em execução de um programa, incluindo os valores \
                 atuais do contador de programa, dos registradores e das variáveis. Cada \
                 processo tem, portanto, um programa, uma entrada, uma saída e um estado.",
            ),
            Block::P(
                "É importante separar as duas coisas: o programa é o texto estático no disco; \
                 o processo é esse programa vivo, em execução, com todo o estado que ele \
                 acumulou até agora.",
            ),
            Block::H("Uma CPU virtual para cada um"),
            Block::L(&[
                "Cada processo se comporta como se rodasse na sua própria CPU virtual.",
                "Na prática existe uma CPU real, que troca a todo momento qual processo está \
                 sendo executado — é a multiprogramação.",
                "O algoritmo de escalonamento é quem determina quando um processo deve ser \
                 interrompido para atender outro.",
            ]),
            Block::SeeAlso(
                "O escalonamento tem tela própria neste módulo.",
                Screen::Scheduling,
            ),
        ],
    },
    // ---------------------------------------------------------------- 2.1.2
    Article {
        icon: "🐣",
        title: "Criação de processos",
        section: "2.1.2",
        ostep: &[ostep::PROCESS_API, ostep::PROCESSES],
        summary: "Os quatro eventos que criam processos, e o fork do UNIX.",
        blocks: &[
            Block::P("Quatro eventos principais fazem processos nascerem:"),
            Block::N(&[
                "A inicialização do sistema.",
                "Uma chamada de sistema de criação de processos, feita por um processo que já \
                 está em execução.",
                "Uma solicitação do usuário para criar um novo processo.",
                "O início de uma tarefa em lote.",
            ]),
            Block::H("Na inicialização do sistema"),
            Block::P(
                "Quando o sistema operacional sobe, uma série de processos é criada. Eles se \
                 dividem em dois tipos:",
            ),
            Block::Defs(&[
                (
                    "Primeiro plano",
                    "interagem com usuários humanos e realizam trabalhos para eles.",
                ),
                (
                    "Segundo plano",
                    "não estão associados a um usuário em particular, mas têm alguma função \
                     específica. São os daemons.",
                ),
            ]),
            Block::H("Por outro processo, ou pelo usuário"),
            Block::P(
                "Um processo em execução pode usar chamadas de sistema para criar um ou mais \
                 processos novos que o auxiliem em alguma tarefa. E quando um usuário executa \
                 um programa pelo terminal, ou dá um duplo-clique num ícone da interface \
                 gráfica, um novo processo é criado para atender a essa tarefa.",
            ),
            Block::H("O fork do UNIX"),
            Block::P(
                "No UNIX existe apenas uma chamada de sistema para iniciar processos: o fork. \
                 O que ele faz é peculiar — em vez de criar um processo a partir de um \
                 programa, ele duplica o processo que o chamou.",
            ),
            Block::L(&[
                "Logo após o fork, pai e filho são idênticos: mesma imagem de memória, mesmas \
                 variáveis de ambiente, mesmos arquivos abertos.",
                "O processo-filho então executa uma chamada de sistema para trocar sua imagem \
                 de memória e passar a executar um programa diferente.",
                "A partir daí, os dois processos têm espaços de endereços distintos: as \
                 mudanças de um não são visíveis para o outro.",
            ]),
            Block::P(
                "No Windows, a criação de processos é resolvida de maneira semelhante pela \
                 CreateProcess, que trata criação e carga do programa numa chamada só.",
            ),
            Block::Note(
                "Os processos em execução podem ser vistos com o comando ps em sistemas \
                 POSIX, ou pelo gerenciador de tarefas no Windows.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 2.1.3
    Article {
        icon: "🚪",
        title: "Término de processos",
        section: "2.1.3",
        ostep: &[ostep::PROCESS_API],
        summary: "As quatro causas de morte de um processo.",
        blocks: &[
            Block::P(
                "Um processo pode ser encerrado por quatro causas — duas voluntárias e duas \
                 involuntárias:",
            ),
            Block::Defs(&[
                (
                    "Saída normal (voluntária)",
                    "o processo terminou sua tarefa como esperado e avisa o sistema \
                     operacional com uma chamada de sistema: exit no UNIX, ExitProcess no \
                     Windows.",
                ),
                (
                    "Saída por erro (voluntária)",
                    "o processo encontrou um bug ou executou uma instrução ilegal durante a \
                     execução.",
                ),
                (
                    "Erro fatal (involuntária)",
                    "os parâmetros informados ao programa não existem ou não estão \
                     acessíveis.",
                ),
                (
                    "Encerrado por outro processo (involuntária)",
                    "algum outro processo executou uma chamada de sistema para matá-lo: kill \
                     no UNIX, TerminateProcess no Windows.",
                ),
            ]),
            Block::Note(
                "A distinção entre voluntário e involuntário é sobre quem decide encerrar, não \
                 sobre haver ou não erro: um processo que morre por um bug seu ainda saiu \
                 \"por vontade própria\", no sentido de que a saída partiu dele.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 2.1.4
    Article {
        icon: "🌳",
        title: "Hierarquia de processos",
        section: "2.1.4",
        ostep: &[ostep::PROCESS_API],
        summary: "Por que os processos formam uma árvore no UNIX — e não no Windows.",
        blocks: &[
            Block::P(
                "Na maioria dos sistemas operacionais, o processo-pai e o processo-filho \
                 continuam associados de alguma maneira depois da criação. Como o filho pode \
                 criar outros processos, o resultado é uma árvore de hierarquia.",
            ),
            Block::H("No UNIX"),
            Block::L(&[
                "Um processo, todos os seus filhos e todos os seus descendentes formam um \
                 grupo de processos.",
                "Quando o sistema é inicializado, um processo especial chamado init — presente \
                 na imagem de inicialização — é o primeiro a subir.",
                "O init lê quantos terminais existem e cria um novo processo para cada um.",
                "É por isso que o init funciona como o processo raiz da árvore de processos: \
                 todo o resto descende dele.",
            ]),
            Block::SeeAlso(
                "Você pode ver essa árvore ao vivo, na sua própria máquina.",
                Screen::ProcessTree,
            ),
            Block::H("No Windows"),
            Block::P(
                "O Windows não tem o conceito de hierarquia de processos: todos os processos \
                 são iguais. O único mecanismo parecido aparece na criação — o pai recebe um \
                 identificador chamado handle, usado para controlar o filho.",
            ),
            Block::Note(
                "Mas esse handle pode ser transferido para outro processo, o que quebra \
                 qualquer relação de hierarquia. Por isso não se pode falar de uma árvore de \
                 processos no Windows como se fala no UNIX.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 2.1.5
    Article {
        icon: "🚦",
        title: "Estados de um processo",
        section: "2.1.5",
        ostep: &[ostep::PROCESSES],
        summary: "Em execução, pronto, bloqueado — e as quatro transições.",
        blocks: &[
            Block::P(
                "Processos muitas vezes precisam interagir entre si. A saída de um pode ser a \
                 entrada de outro, por exemplo. Dependendo da complexidade e do tempo de \
                 execução dos processos que trabalham em conjunto, um deles pode precisar \
                 aguardar o outro para entrar em ação — esse tempo de espera é o estado de \
                 bloqueio.",
            ),
            Block::P(
                "Um processo também pode ser interrompido por outro motivo, bem diferente: a \
                 CPU simplesmente resolveu dar prioridade a outro processo.",
            ),
            Block::H("Os três estados"),
            Block::Defs(&[
                ("Em execução", "está sendo executado pela CPU neste instante."),
                (
                    "Pronto",
                    "é executável, mas está esperando a vez de ser executado pela CPU.",
                ),
                (
                    "Bloqueado",
                    "é incapaz de ser executado até que um evento externo aconteça.",
                ),
            ]),
            Block::Note(
                "A diferença entre pronto e bloqueado é o que separa \"posso rodar, só não é \
                 a minha vez\" de \"não adianta me escalar, eu não tenho como avançar\".",
            ),
            Block::H("As quatro transições"),
            Block::StateDiagram,
            Block::N(&[
                "Em execução → Bloqueado: o sistema operacional descobre que o processo não \
                 pode continuar naquele momento.",
                "Em execução → Pronto: o escalonador decidiu tirá-lo da CPU.",
                "Pronto → Em execução: o escalonador decidiu que é a vez dele.",
                "Bloqueado → Pronto: o evento externo que ele aguardava finalmente aconteceu.",
            ]),
            Block::Note(
                "Repare que não existe transição de Bloqueado direto para Em execução. Um \
                 processo desbloqueado volta para a fila dos prontos e ainda precisa esperar o \
                 escalonador escolhê-lo.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 2.1.6
    Article {
        icon: "🗂",
        title: "Implementação de processos",
        section: "2.1.6",
        ostep: &[ostep::PROCESSES, ostep::DIRECT_EXECUTION],
        summary: "A tabela de processos e o vetor de interrupção.",
        blocks: &[
            Block::P(
                "Para implementar o modelo de processos, o sistema operacional mantém uma \
                 tabela de processos. Cada entrada guarda o que é preciso para suspender um \
                 processo e depois retomá-lo como se nada tivesse acontecido:",
            ),
            Block::L(&[
                "contador do programa;",
                "ponteiro de pilha;",
                "alocação de memória;",
                "estado dos arquivos abertos;",
                "informações de escalonamento;",
                "estado do processo.",
            ]),
            Block::H("Como a ilusão de multiprogramação é criada"),
            Block::L(&[
                "A cada classe de entrada/saída está associado um local chamado vetor de \
                 interrupção.",
                "O vetor de interrupção contém o endereço da rotina de serviço de interrupção \
                 (ISR, interrupt service routine).",
                "Cada interrupção começa salvando o estado dos registradores na entrada da \
                 tabela de processos referente ao processo atual.",
            ]),
            Block::Note(
                "É esse salvamento e restauração de estado que sustenta toda a abstração: sem \
                 ele, um processo interrompido não teria como continuar de onde parou, e a \
                 CPU virtual de cada processo deixaria de existir.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 2.2.1
    Article {
        icon: "🧵",
        title: "Por que existem threads",
        section: "2.2 e 2.2.1",
        ostep: &[ostep::THREADS],
        summary: "Processos dentro de processos, e as três razões para tê-los.",
        blocks: &[
            Block::P(
                "Em sistemas operacionais tradicionais, cada processo tem seu espaço de \
                 endereços e uma única linha de execução. Só que é desejável ter várias linhas \
                 de execução no mesmo espaço de endereços, rodando em paralelo como se fossem \
                 processos paralelos. É isso que as threads oferecem.",
            ),
            Block::Note(
                "Uma forma útil de pensar: threads são processos dentro de processos — \
                 mini-processos que compartilham o espaço de endereços de quem os hospeda.",
            ),
            Block::H("Três razões"),
            Block::Defs(&[
                (
                    "Atividades concorrentes que bloqueiam",
                    "uma aplicação costuma ter várias atividades acontecendo ao mesmo tempo, e \
                     algumas precisam ser bloqueadas de tempos em tempos para aguardar \
                     resultados. Com threads, essas entidades paralelas compartilham o mesmo \
                     espaço de endereços e os mesmos dados entre si.",
                ),
                (
                    "Consumo de recursos",
                    "é mais barato criar e encerrar uma thread do que um processo. Um processo \
                     exige a alocação de um espaço de memória próprio; a thread nasce dentro do \
                     espaço de memória que já existe.",
                ),
                (
                    "Desempenho",
                    "quando há muitos processos ou muitas operações de entrada/saída, o uso de \
                     threads traz ganhos de escala.",
                ),
            ]),
            Block::P(
                "Threads também são úteis quando há paralelismo de verdade, com vários \
                 processadores disponíveis.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 2.2.2
    Article {
        icon: "⚙",
        title: "O modelo clássico de thread",
        section: "2.2.2",
        ostep: &[ostep::THREADS],
        summary: "Separar agrupamento de recursos de linha de execução.",
        blocks: &[
            Block::P(
                "O modelo de processo repousa sobre dois conceitos que são independentes entre \
                 si: o agrupamento de recursos e a execução. Decompor esses dois conceitos é \
                 justamente o que torna a necessidade das threads compreensível.",
            ),
            Block::Table2(
                "O processo agrupa",
                "A thread executa",
                &[
                    (
                        "O espaço de endereços, onde o código do programa e seus dados ficam.",
                        "O contador de programa, que controla a ordem de execução das \
                         instruções.",
                    ),
                    (
                        "Os arquivos abertos e demais recursos do sistema.",
                        "O controle dos registradores.",
                    ),
                    (
                        "É o que se compartilha entre as threads de um mesmo processo.",
                        "A pilha, com o histórico de execução. É o que cada thread tem de seu.",
                    ),
                ],
            ),
            Block::H("Chamadas comuns"),
            Block::Defs(&[
                ("thread_create", "cria uma thread."),
                ("thread_exit", "encerra uma thread."),
                ("thread_join", "faz uma thread aguardar a saída de outra."),
                (
                    "thread_yield",
                    "permite que uma thread abra mão da sua vez na fila de uso da CPU.",
                ),
            ]),
            Block::H("As complicações que aparecem"),
            Block::P(
                "Compartilhar o espaço de endereços resolve problemas, mas cria outros. Três \
                 deles não têm resposta única — cada sistema operacional decide de um jeito:",
            ),
            Block::N(&[
                "Se um processo-pai tem múltiplas threads e é bifurcado, o processo-filho \
                 recebe todas essas threads também?",
                "Como lidar com uma thread que se encerra segurando um recurso que outra \
                 thread está usando?",
                "Se uma thread percebe que precisa de mais memória, faz a solicitação de \
                 alocação e passa a vez, a thread seguinte pode fazer o mesmo pedido — e o \
                 número de solicitações dobra.",
            ]),
        ],
    },
    // ---------------------------------------------------------------- 2.2.3
    Article {
        icon: "📐",
        title: "Threads POSIX (Pthreads)",
        section: "2.2.3",
        ostep: &[ostep::THREAD_API],
        summary: "O padrão IEEE 1003.1c, presente em quase todo UNIX.",
        blocks: &[
            Block::P(
                "O IEEE 1003.1c define um padrão para o uso de threads chamado Pthreads, \
                 suportado por quase todos os sistemas UNIX. O padrão define mais de 60 \
                 chamadas de função.",
            ),
            Block::H("As principais"),
            Block::Defs(&[
                ("pthread_create", "cria uma thread."),
                ("pthread_exit", "encerra uma thread."),
                ("pthread_join", "faz uma thread aguardar a saída de outra."),
                (
                    "pthread_yield",
                    "permite que uma thread abra mão da sua vez na fila de uso da CPU.",
                ),
                (
                    "pthread_attr_init",
                    "cria e inicializa uma estrutura de atributos da thread.",
                ),
                (
                    "pthread_attr_destroy",
                    "remove uma estrutura de atributos da thread.",
                ),
            ]),
            Block::Note(
                "As quatro primeiras são exatamente as chamadas do modelo clássico da seção \
                 anterior, agora com nome padronizado.",
            ),
            Block::H("O que cada thread carrega"),
            Block::L(&[
                "Um identificador.",
                "Um conjunto de registradores.",
                "Um conjunto de atributos, guardados numa estrutura de dados própria.",
            ]),
            Block::P(
                "Essa estrutura de atributos contém o tamanho da pilha, parâmetros de \
                 escalonamento e outros itens secundários — é ela que as chamadas \
                 pthread_attr_init e pthread_attr_destroy manipulam.",
            ),
        ],
    },
    // -------------------------------------------------------- 2.2.4 a 2.2.6
    Article {
        icon: "🏛",
        title: "Onde as threads são implementadas",
        section: "2.2.4, 2.2.5 e 2.2.6",
        // Sem OSTEP aqui de propósito: o livro não tem capítulo que cubra o
        // recorte usuário/kernel/híbrido, e citar o 26 prometeria mais do que
        // ele entrega neste ponto.
        ostep: &[],
        summary: "Espaço de usuário, espaço de kernel, e a abordagem híbrida.",
        blocks: &[
            Block::P(
                "As threads podem ser implementadas em dois lugares — no espaço do usuário ou \
                 no espaço do kernel — e ainda existe uma terceira via, híbrida. Cada uma tem \
                 suas vantagens e desvantagens.",
            ),
            Block::H("No espaço do usuário"),
            Block::P(
                "A primeira abordagem coloca o pacote de threads inteiramente no espaço do \
                 usuário, de maneira que o kernel não enxerga as threads — só os processos. \
                 Nesse modelo as threads são executadas sobre um sistema em tempo de execução, \
                 e é preciso manter uma tabela de threads dentro do processo, parecida com a \
                 que o kernel mantém para tabular seus processos.",
            ),
            Block::Table2(
                "Vantagens",
                "Desvantagens",
                &[
                    (
                        "Dá para implementar threads mesmo em sistemas operacionais que não \
                         as suportam.",
                        "Chamadas de sistema bloqueantes não podem ser tratadas no nível das \
                         threads.",
                    ),
                    (
                        "Permite um algoritmo de escalonamento customizado para cada processo.",
                        "",
                    ),
                ],
            ),
            Block::H("No kernel"),
            Block::P(
                "O segundo método permite que o kernel observe e gerencie as threads. Isso \
                 torna desnecessários tanto o sistema em tempo de execução quanto a tabela de \
                 threads local dentro de cada processo.",
            ),
            Block::Table2(
                "Vantagens",
                "Desvantagens",
                &[
                    (
                        "Sistema de reciclagem de threads.",
                        "O que fazer com processos multithread que são bifurcados?",
                    ),
                    (
                        "Uso de chamadas não-bloqueantes.",
                        "É preciso definir como os sinais são entregues, já que em princípio \
                         eles são endereçados a processos, não a threads.",
                    ),
                ],
            ),
            Block::H("Implementações híbridas"),
            Block::P(
                "A abordagem híbrida usa threads de kernel e então multiplexa threads de \
                 usuário sobre elas. O programador define quantas serão as threads de núcleo e \
                 quantas serão as threads multiplexadas — ficando com o controle fino do \
                 espaço de usuário sem abrir mão do que o kernel enxerga.",
            ),
        ],
    },
    // --------------------------------------------------------------- síntese
    Article {
        icon: "⚖",
        title: "Processo × Thread",
        section: "2.1 e 2.2",
        ostep: &[ostep::PROCESSES, ostep::THREADS],
        summary: "Uma síntese lado a lado dos dois conceitos.",
        blocks: &[
            Block::P(
                "Depois de percorrer as duas metades do capítulo, vale colocar os dois \
                 conceitos lado a lado. A chave é lembrar que o processo agrupa recursos e a \
                 thread é a linha de execução.",
            ),
            Block::Table2(
                "Processo",
                "Thread",
                &[
                    (
                        "Tem um espaço de endereços próprio.",
                        "Compartilha o espaço de endereços do processo que a hospeda.",
                    ),
                    (
                        "É a unidade de agrupamento de recursos: memória, arquivos abertos.",
                        "É a unidade de execução: contador de programa, registradores, pilha.",
                    ),
                    (
                        "Criar exige alocar um espaço de memória — é caro.",
                        "Criar é barato: acontece dentro de um espaço de memória que já existe.",
                    ),
                    (
                        "No UNIX, é criado por fork e forma uma árvore de hierarquia.",
                        "É criada por thread_create / pthread_create, sem hierarquia \
                         equivalente.",
                    ),
                    (
                        "As mudanças de um não são visíveis para o outro.",
                        "Os dados são compartilhados entre as threads do mesmo processo.",
                    ),
                    (
                        "É sempre visível para o kernel.",
                        "Pode ser invisível para o kernel, se implementada no espaço do \
                         usuário.",
                    ),
                ],
            ),
            Block::Note(
                "Justamente porque as threads compartilham dados sem barreira nenhuma entre \
                 si, elas herdam todos os problemas de concorrência dos processos — e sem a \
                 proteção que espaços de endereços separados davam de graça.",
            ),
            Block::SeeAlso(
                "É daí que nascem as race conditions.",
                Screen::RaceConditions,
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
        check::articles(ARTICLES);
    }
}
