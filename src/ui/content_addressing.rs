//! Conteúdo da tela "Espaços de Endereçamento" — seções 3.1 e 3.2 das
//! anotações (`Anotações/03-memoria-virtual.tex` e
//! `Anotações/gerenciamento_de_memoria.md`; Tanenbaum & Bos, *Sistemas
//! Operacionais Modernos*, capítulo 3), com o OSTEP como leitura complementar.
//!
//! Da memória física exposta ao espaço de endereçamento: as três organizações
//! sem abstração, proteção e realocação, base e limite, troca de processos e o
//! gerenciamento de memória livre — com o simulador de encaixe embutido.

use crate::app::Screen;
use crate::ui::memfit::{Fit, Op, Scenario, Seg};
use crate::ui::reader::{Article, Block, ostep};

/// Memória de 64 unidades: o sistema na base e quatro processos com blocos
/// livres entre eles, como resultado de trocas anteriores.
const INITIAL: &[Seg] = &[
    Seg::Proc("SO", 8),
    Seg::Proc("A", 10),
    Seg::Hole(6),
    Seg::Proc("B", 8),
    Seg::Hole(12),
    Seg::Proc("C", 6),
    Seg::Hole(4),
    Seg::Proc("D", 5),
    Seg::Hole(5),
];

/// Só a memória inicial, para olhar as duas representações lado a lado.
static STRUCTURES: Scenario = Scenario {
    id: "structures",
    initial: INITIAL,
    ops: &[],
    fit: Fit::First,
    hint: "A mesma memória vista como lista de segmentos e como mapa de bits.",
};

/// A mesma memória, agora com uma sequência de pedidos e liberações.
static FITS: Scenario = Scenario {
    id: "fits",
    initial: INITIAL,
    ops: &[
        Op::Alloc("E", 5),
        Op::Alloc("F", 4),
        Op::Free("B"),
        Op::Alloc("G", 9),
        Op::Alloc("H", 3),
        Op::Free("A"),
        Op::Alloc("I", 10),
        Op::Alloc("J", 7),
    ],
    fit: Fit::First,
    hint: "Troque a política e aplique a mesma sequência: repare onde cada uma deixa blocos minúsculos.",
};

pub const ARTICLES: &[Article] = &[
    // ------------------------------------------------------------ 3.1, 3.1.1
    Article {
        icon: "📜",
        title: "Sem abstração de memória",
        section: "3.1 e 3.1.1",
        ostep: &[ostep::ADDRESS_SPACES],
        summary: "O caso mais simples: cada programa enxerga a memória física.",
        blocks: &[
            Block::P(
                "O capítulo anterior tratou da multiplexação do processador no tempo: o \
                 sistema operacional interrompe os programas, salva o estado de cada um e \
                 devolve a cada processo a ilusão de um processador dedicado. Chegou a hora \
                 da outra metade do problema, a multiplexação da memória no espaço.",
            ),
            Block::P(
                "Todo programador gostaria de uma memória infinitamente grande, \
                 infinitamente rápida e não volátil. Ela não existe. O que existe é uma \
                 hierarquia: alguns megabytes de cache, rápida e cara, alguns gigabytes de \
                 memória principal, alguns terabytes de disco, lento e barato. Cabe ao \
                 sistema operacional transformar essa hierarquia desigual num recurso \
                 utilizável.",
            ),
            Block::H("Nenhuma abstração"),
            Block::P(
                "A forma mais simples de abstração de memória é não ter abstração alguma. \
                 Foi o esquema dos primeiros computadores, e ele ainda aparece em sistemas \
                 embarcados muito simples. Cada programa enxerga diretamente a memória \
                 física, vista como um grande vetor de células de tamanho fixo, endereçadas \
                 de 0 até M−1.",
            ),
            Block::P(
                "Uma instrução que referencia o endereço 1000 lê ou escreve, literalmente, a \
                 célula física de número 1000. Não existe tradução, não existe verificação e \
                 não existe camada nenhuma entre o programa e o hardware de memória.",
            ),
            Block::Warn(
                "Com a memória física exposta assim, não dá para manter dois programas na \
                 memória ao mesmo tempo. Se os dois escrevem no endereço 1000, o segundo \
                 destrói o valor do primeiro, e nenhum dos dois funciona.",
            ),
            Block::H("Onde colocar o sistema operacional"),
            Block::P(
                "Mesmo sem abstração, ainda é preciso decidir onde fica o próprio sistema \
                 operacional. Há três organizações possíveis:",
            ),
            Block::MemLayouts,
            Block::Defs(&[
                (
                    "(a) Sistema na base da RAM",
                    "o sistema operacional ocupa a parte inferior e o programa do usuário a \
                     superior. Foi usada nos primeiros computadores de grande porte. Hoje é \
                     apenas legado.",
                ),
                (
                    "(b) Sistema em ROM, no topo",
                    "o sistema operacional fica em ROM na parte superior, e o programa do \
                     usuário na RAM, embaixo. Usada em sistemas embarcados e nos primeiros \
                     computadores portáteis.",
                ),
                (
                    "(c) Drivers em ROM, sistema na base",
                    "os drivers de dispositivo ficam em ROM no topo, o sistema na base da RAM \
                     e o programa do usuário no meio. Foi a organização dos primeiros \
                     computadores pessoais, e a porção em ROM ganhou o nome de BIOS (Basic \
                     Input Output System).",
                ),
            ]),
            Block::Note(
                "As organizações (a) e (c) compartilham a mesma fraqueza: o sistema está em \
                 RAM, e nada impede que um erro do programa do usuário escreva sobre ele. Um \
                 único ponteiro inválido derruba a máquina inteira. A organização com o \
                 sistema em ROM não sofre desse problema, mas paga o preço de não poder ser \
                 atualizada.",
            ),
            Block::H("Vários programas, sem abstração"),
            Block::P(
                "Ainda é possível manter vários programas em andamento sem qualquer \
                 abstração, desde que apenas um deles esteja na memória a cada instante. A \
                 técnica se chama troca de processos (swapping): o sistema escreve em disco \
                 a imagem inteira do programa corrente e carrega em seguida a imagem do \
                 próximo. Como só existe um programa residente por vez, o conflito de \
                 endereços desaparece.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 3.2.1
    Article {
        icon: "🗺",
        title: "A noção de espaço de endereçamento",
        section: "3.2 e 3.2.1",
        ostep: &[ostep::ADDRESS_SPACES],
        summary: "Proteção e realocação, resolvidas com uma abstração nova.",
        blocks: &[
            Block::P(
                "Expor a memória física aos programas do usuário produz dois problemas \
                 distintos, e qualquer solução aceitável precisa resolver os dois:",
            ),
            Block::Defs(&[
                (
                    "Proteção",
                    "um programa não pode ler nem escrever na memória de outro programa, e \
                     muito menos na memória do sistema operacional.",
                ),
                (
                    "Realocação",
                    "um programa precisa executar corretamente em qualquer posição da memória \
                     física, pois o compilador não tem como saber, na hora de compilar, onde \
                     o programa será carregado.",
                ),
            ]),
            Block::P(
                "A solução para os dois problemas é a mesma, e consiste em inventar uma nova \
                 abstração de memória.",
            ),
            Block::Note(
                "Definição: um espaço de endereçamento é o conjunto de endereços que um \
                 processo pode utilizar para endereçar memória, criado pelo sistema \
                 operacional e independente dos endereços físicos da máquina.",
            ),
            Block::P(
                "Cada processo recebe seu próprio espaço de endereçamento. O endereço 1000 \
                 do processo A e o endereço 1000 do processo B correspondem a posições \
                 distintas da memória física, e nenhum dos dois processos precisa saber \
                 disso.",
            ),
            Block::H("O mesmo gesto do capítulo 2"),
            Block::P(
                "Assim como o processo é um processador virtual privativo, o espaço de \
                 endereçamento é uma memória virtual privativa. Nos dois casos o sistema \
                 operacional multiplexa um recurso físico único e entrega a cada programa a \
                 ilusão de possuí-lo inteiro.",
            ),
            Block::SeeAlso(
                "O processo como CPU virtual está nos Conceitos de Processos e Threads.",
                Screen::Texts,
            ),
        ],
    },
    // ---------------------------------------------------------------- 3.2.1
    Article {
        icon: "📏",
        title: "Registradores de base e limite",
        section: "3.2.1",
        ostep: &[ostep::ADDRESS_TRANSLATION],
        summary: "A implementação mais simples da ilusão: somar e comparar.",
        blocks: &[
            Block::P(
                "A maneira mais simples de implementar o espaço de endereçamento usa dois \
                 registradores do processador, chamados de base e limite. Quando um processo \
                 é escalonado, o sistema operacional carrega os dois: a base recebe o \
                 endereço físico em que a região do processo começa, e o limite recebe o \
                 tamanho dessa região. Só o modo kernel pode alterá-los.",
            ),
            Block::P(
                "A cada referência à memória, o hardware soma o conteúdo da base ao endereço \
                 produzido pelo programa:",
            ),
            Block::Code(&[
                "f = v + b",
                "",
                "v  endereço virtual produzido pelo programa",
                "b  conteúdo do registrador de base",
                "f  endereço físico entregue à memória",
                "",
                "acesso autorizado somente se  v < limite",
            ]),
            Block::BaseLimit,
            Block::H("Os dois problemas, resolvidos de uma vez"),
            Block::Defs(&[
                (
                    "Realocação",
                    "o programa pode ser carregado em qualquer posição física. Basta ajustar \
                     o registrador de base, e nenhuma palavra do programa precisa ser \
                     modificada.",
                ),
                (
                    "Proteção",
                    "nenhum endereço fora do intervalo autorizado chega à memória. Uma \
                     tentativa de ultrapassar o limite provoca um desvio forçado para o \
                     sistema operacional, que encerra o processo infrator.",
                ),
            ]),
            Block::Table2(
                "Vantagem",
                "Desvantagens",
                &[
                    (
                        "É barata e exige pouco hardware.",
                        "Obriga o processador a somar e comparar a cada referência à memória.",
                    ),
                    (
                        "",
                        "Obriga cada processo a ocupar uma região física contígua, restrição \
                         que só a paginação vai remover.",
                    ),
                ],
            ),
            Block::H("Olhando o espaço de endereçamento de perto"),
            Block::P(
                "Dá para observar o espaço de endereçamento de um processo diretamente. Em \
                 C:",
            ),
            Block::Code(&[
                "#include <stdio.h>",
                "#include <stdlib.h>",
                "",
                "int global = 1337;                        // (1)",
                "",
                "int main(void) {",
                "    int local = 42;                       // (2)",
                "    int *dinamico = malloc(sizeof(int));  // (3)",
                "",
                "    printf(\"codigo : %p\\n\", (void *) main);",
                "    printf(\"global : %p\\n\", (void *) &global);",
                "    printf(\"heap   : %p\\n\", (void *) dinamico);",
                "    printf(\"pilha  : %p\\n\", (void *) &local);",
                "",
                "    free(dinamico);",
                "    return 0;",
                "}",
            ]),
            Block::L(&[
                "A variável em (1) fica na região de dados, pois existe durante toda a \
                 execução do programa.",
                "A variável em (2) fica na pilha, pois pertence à chamada corrente de main().",
                "O bloco obtido em (3) fica no heap, pois foi alocado durante a execução por \
                 malloc().",
            ]),
            Block::Note(
                "Os quatro valores impressos são endereços virtuais, não posições reais da \
                 memória física. Duas execuções simultâneas do mesmo programa imprimem \
                 valores próximos ou idênticos, embora ocupem regiões físicas distintas. Os \
                 sistemas atuais ainda deslocam as regiões aleatoriamente a cada execução, \
                 então os valores mudam entre execuções sucessivas, mas a ordem relativa \
                 entre as quatro regiões permanece.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 3.2.2
    Article {
        icon: "🔄",
        title: "Troca de processos",
        section: "3.2.2",
        ostep: &[ostep::SWAP_MECHANISMS, ostep::FREE_SPACE],
        summary: "Quando a memória não cabe todo mundo: swapping e fragmentação.",
        blocks: &[
            Block::P(
                "Se a memória física fosse grande o bastante para conter todos os processos, \
                 os registradores de base e limite bastariam. Não é o que acontece: a soma da \
                 memória exigida pelos processos de um sistema típico excede com folga a \
                 memória instalada. Duas técnicas lidam com esse excesso:",
            ),
            Block::Defs(&[
                (
                    "Troca de processos (swapping)",
                    "traz um processo inteiro para a memória, executa-o por algum tempo e \
                     devolve-o ao disco.",
                ),
                (
                    "Memória virtual",
                    "permite que um processo execute enquanto apenas parte dele está na \
                     memória principal. É o assunto da tela de Paginação.",
                ),
            ]),
            Block::SeeAlso("A memória virtual tem tela própria.", Screen::Paging),
            Block::H("Fragmentação externa e compactação"),
            Block::P(
                "A troca de processos cria um problema próprio. Processos entram e saem da \
                 memória em ordens diferentes, e o resultado é uma memória perfurada por \
                 blocos livres de tamanhos variados. Esse fenômeno se chama fragmentação \
                 externa.",
            ),
            Block::P(
                "A solução direta é a compactação de memória, que empurra todos os processos \
                 para a parte inferior e reúne os blocos numa única região livre. Funciona, \
                 mas é cara: compactar 16 GB a 8 GB/s consome cerca de 2 segundos, e nesse \
                 tempo nenhum processo avança.",
            ),
            Block::H("Quanta memória conceder"),
            Block::P(
                "Há ainda a questão de quanta memória dar a um processo que acaba de ser \
                 carregado:",
            ),
            Block::Table2(
                "Alocação exata",
                "Alocação com folga",
                &[
                    (
                        "Qualquer crescimento do heap ou da pilha obriga o sistema a mover o \
                         processo inteiro, para um bloco maior, ou de volta ao disco.",
                        "O processo cresce sem ser movido, mas a folga não utilizada é memória \
                         desperdiçada: é a fragmentação interna.",
                    ),
                    (
                        "",
                        "Quando o processo volta ao disco, só a parte efetivamente usada \
                         precisa ser escrita, a folga é apenas espaço reservado.",
                    ),
                ],
            ),
            Block::Note(
                "Os dois nomes valem a pena fixar: fragmentação externa é o espaço \
                 desperdiçado entre as regiões alocadas. Fragmentação interna é o espaço \
                 desperdiçado dentro de uma região alocada.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 3.2.3
    Article {
        icon: "🧱",
        title: "Mapas de bits e listas encadeadas",
        section: "3.2.3",
        ostep: &[ostep::FREE_SPACE],
        summary: "Como o sistema sabe o que está livre.",
        blocks: &[
            Block::P(
                "Quando a memória é alocada dinamicamente, o sistema operacional precisa \
                 saber quais regiões estão livres e quais estão ocupadas. Há duas maneiras \
                 de rastrear esse uso: mapas de bits e listas encadeadas.",
            ),
            Block::Allocator(&STRUCTURES),
            Block::H("Mapas de bits"),
            Block::P(
                "A memória é dividida em unidades de alocação de tamanho fixo, de algumas \
                 palavras a alguns quilobytes. Cada unidade recebe um bit no mapa: 0 quando \
                 está livre, 1 quando está ocupada.",
            ),
            Block::P(
                "Quanto menor a unidade, maior o mapa. Com unidades de 4 bytes, cada 32 bits \
                 de memória exigem 1 bit de mapa, e o mapa consome 1/33 da memória total. \
                 Quanto maior a unidade, menor o mapa, e maior a memória desperdiçada na \
                 última unidade de cada processo.",
            ),
            Block::Warn(
                "A grande desvantagem aparece na alocação: para atender um processo que \
                 precisa de k unidades, o gerenciador precisa localizar k zeros consecutivos \
                 no mapa. Essa busca é lenta, e é por isso que os mapas de bits raramente \
                 são usados na prática.",
            ),
            Block::H("Listas encadeadas"),
            Block::P(
                "A alternativa é manter uma lista encadeada de segmentos, na qual cada \
                 segmento é um processo ou um bloco livre entre dois processos. Cada entrada \
                 tem quatro campos:",
            ),
            Block::N(&[
                "o tipo do segmento, processo (P) ou bloco livre (H)",
                "o endereço em que ele se inicia",
                "o seu comprimento",
                "um ponteiro para a entrada seguinte.",
            ]),
            Block::P(
                "A lista é mantida ordenada por endereço. Essa escolha tem uma consequência \
                 importante: quando um processo termina, o sistema examina imediatamente as \
                 entradas vizinhas e funde blocos livres adjacentes num único bloco maior.",
            ),
            Block::H("O que acontece quando um processo termina"),
            Block::P(
                "O término produz uma entre quatro situações, conforme os vizinhos do \
                 segmento liberado sejam processos ou blocos livres:",
            ),
            Block::L(&[
                "Os dois vizinhos são processos: a entrada apenas muda de tipo.",
                "Um dos vizinhos é um bloco livre: as duas entradas se fundem em uma.",
                "Os dois vizinhos são blocos livres: as três entradas se fundem em uma, e a \
                 lista encolhe.",
            ]),
            Block::Note(
                "No simulador acima a memória de 64 unidades aparece das duas formas ao \
                 mesmo tempo. Repare que o mapa de bits sabe dizer se a unidade 30 está \
                 livre, mas não sabe onde começa nem termina o bloco a que ela pertence, \
                 é a lista que guarda essa informação.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 3.2.3
    Article {
        icon: "🧩",
        title: "Primeiro, próximo, melhor e pior encaixe",
        section: "3.2.3",
        ostep: &[ostep::FREE_SPACE],
        summary: "Qual bloco escolher quando há mais de um candidato.",
        blocks: &[
            Block::P(
                "Resta decidir qual bloco escolher quando existe mais de um candidato. Essa \
                 decisão cabe ao algoritmo de alocação, e há quatro escolhas clássicas:",
            ),
            Block::Defs(&[
                (
                    "Primeiro encaixe (first fit)",
                    "percorre a lista desde o início e para no primeiro bloco grande o \
                     bastante. O bloco é dividido em duas partes: uma para o processo e outra \
                     que continua livre. É o algoritmo mais rápido, pois procura o mínimo \
                     possível.",
                ),
                (
                    "Próximo encaixe (next fit)",
                    "funciona como o anterior, mas recomeça a busca de onde parou da última \
                     vez, para não varrer o início da lista repetidamente. Simulações mostram \
                     desempenho ligeiramente pior que o do primeiro encaixe.",
                ),
                (
                    "Melhor encaixe (best fit)",
                    "percorre a lista inteira e escolhe o menor bloco capaz de conter o \
                     processo. É mais lento, pois sempre percorre tudo, e desperdiça mais \
                     memória: produz blocos minúsculos, pequenos demais para servir a \
                     qualquer processo.",
                ),
                (
                    "Pior encaixe (worst fit)",
                    "escolhe sempre o maior bloco, na esperança de que o resto continue \
                     utilizável. Simulações mostram que a ideia também não funciona bem.",
                ),
            ]),
            Block::Allocator(&FITS),
            Block::Note(
                "Aplique a sequência inteira em cada política e compare a fita ao final. O \
                 melhor encaixe é o que mais deixa blocos de uma ou duas unidades, \
                 exatamente o desperdício que as anotações descrevem. E repare no pedido de \
                 J, que pode não caber mesmo havendo memória livre suficiente no total: é a \
                 fragmentação externa na prática.",
            ),
            Block::H("Duas listas em vez de uma"),
            Block::P(
                "Uma otimização possível é manter duas listas separadas, uma para os \
                 processos e outra para os blocos livres. A busca passa a examinar só os \
                 blocos e fica mais rápida. Se a lista de blocos estiver ordenada por \
                 tamanho, o melhor encaixe fica tão rápido quanto o primeiro encaixe.",
            ),
            Block::Warn(
                "Como desvantagem, a liberação fica mais cara: ao terminar um processo, é \
                 preciso removê-lo da lista de processos e inseri-lo na lista de blocos. E a \
                 fusão com os vizinhos exige a ordem por endereço, que a ordem por tamanho \
                 acabou de destruir.",
            ),
            Block::P(
                "Nenhum desses algoritmos elimina a fragmentação externa, e todos herdam a \
                 exigência de que o processo ocupe uma região contígua. Enquanto essa \
                 exigência permanecer, o gerenciamento de memória continuará sendo um \
                 problema de encaixar blocos rígidos em blocos de tamanhos arbitrários.",
            ),
            Block::SeeAlso(
                "É a paginação que remove a exigência de contiguidade.",
                Screen::Paging,
            ),
        ],
    },
    // ------------------------------------------------------------ exercícios
    Article {
        icon: "📝",
        title: "Exercícios",
        section: "3.1 e 3.2",
        ostep: &[],
        summary: "Para fixar as seções 3.1 e 3.2.",
        blocks: &[
            Block::P(
                "Os exercícios abaixo acompanham as anotações do capítulo. As respostas \
                 estão nos artigos desta tela, e o simulador de encaixe ajuda a conferir as \
                 contas.",
            ),
            Block::N(&[
                "Explique por que um sistema sem abstração de memória não consegue manter \
                 dois programas na memória ao mesmo tempo.",
                "Descreva as três organizações possíveis da memória em um sistema sem \
                 abstração. Compare-as quanto à proteção do sistema operacional.",
                "Diferencie os problemas da proteção e da realocação. Por que uma solução \
                 aceitável precisa resolver os dois?",
                "Defina espaço de endereçamento. Explique em que sentido ele é, para a \
                 memória, o que o processo é para o processador.",
                "Descreva o funcionamento dos registradores de base e limite. Explique como \
                 cada um deles contribui para a proteção e para a realocação.",
                "Liste as duas desvantagens dos registradores de base e limite. Qual delas é \
                 removida pela paginação?",
                "Defina fragmentação externa e fragmentação interna. Descreva, para cada uma, \
                 uma técnica que a reduza.",
                "Calcule o tempo necessário para compactar uma memória de 32 GB a uma taxa \
                 de 8 GB/s. Explique o resultado do ponto de vista de um sistema interativo.",
                "Compare o mapa de bits e a lista encadeada como formas de rastrear o uso da \
                 memória. Por que o mapa de bits raramente é utilizado?",
                "Uma memória de 1 MB é gerenciada por um mapa de bits com unidades de \
                 alocação de 8 bytes. Calcule o tamanho do mapa e a fração da memória que \
                 ele consome.",
                "Descreva os quatro campos de uma entrada da lista de segmentos. Por que a \
                 lista é mantida ordenada por endereço?",
                "Diferencie os algoritmos de primeiro encaixe, melhor encaixe e pior \
                 encaixe. Explique por que o melhor encaixe desperdiça mais memória que o \
                 primeiro encaixe.",
                "Explique a vantagem e a desvantagem de manter listas separadas para \
                 processos e para blocos.",
            ]),
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::reader::check;

    #[test]
    fn conteudo_valido() {
        check::articles(ARTICLES, '3');
    }
}
