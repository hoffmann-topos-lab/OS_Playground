//! Conteúdo da tela "Substituição de Páginas" — seção 3.4 das anotações.
//!
//! Cada algoritmo ganha uma explicação curta e o simulador rodando a mesma
//! cadeia de referências: mudando só o algoritmo, dá para ver o efeito dele
//! isolado do efeito da carga. O FIFO usa a cadeia da anomalia de Belady.

use crate::app::Screen;
use crate::ui::pagerepl::{Algo, Scenario};
use crate::ui::reader::{Article, Block, ostep};

/// Cadeia usada na maioria dos cenários: 20 referências a 8 páginas, com
/// duas escritas para os bits M aparecerem.
const BASE: &str = "7 0 1 2 0 3w 0 4 2 3 0 3w 2 1 2 0 1 7 0 1";

/// A cadeia da anomalia de Belady: com FIFO, 4 quadros faltam mais que 3.
const BELADY: &str = "1 2 3 4 1 2 5 1 2 3 4 5";

static OPTIMAL: Scenario = Scenario {
    id: "optimal",
    algo: Algo::Optimal,
    refs: BASE,
    frames: 3,
    tick: 4,
    tau: 6,
    hint: "Os rótulos dizem quando cada página volta: sai a de rótulo maior. Guarde o total de faltas.",
};

static NRU: Scenario = Scenario {
    id: "nru",
    algo: Algo::Nru,
    refs: BASE,
    frames: 3,
    tick: 4,
    tau: 6,
    hint: "A cada ⏰ os bits R zeram. Veja uma página suja (classe 1) sair antes de uma limpa recém-usada (classe 2).",
};

static FIFO: Scenario = Scenario {
    id: "fifo",
    algo: Algo::Fifo,
    refs: BELADY,
    frames: 3,
    tick: 4,
    tau: 6,
    hint: "Anomalia de Belady: suba de 3 para 4 quadros e conte as faltas.",
};

static SECOND_CHANCE: Scenario = Scenario {
    id: "second_chance",
    algo: Algo::SecondChance,
    refs: BASE,
    frames: 3,
    tick: 4,
    tau: 6,
    hint: "A coluna \"carga\" é a posição na fila: quem ganha a segunda chance vai para o fim.",
};

static CLOCK: Scenario = Scenario {
    id: "clock",
    algo: Algo::Clock,
    refs: BASE,
    frames: 3,
    tick: 4,
    tau: 6,
    hint: "As mesmas decisões da segunda chance, agora só avançando um ponteiro no anel.",
};

static LRU: Scenario = Scenario {
    id: "lru",
    algo: Algo::Lru,
    refs: BASE,
    frames: 3,
    tick: 4,
    tau: 6,
    hint: "Sai quem tem o último uso mais antigo. Compare o total com o do ótimo e com o do FIFO.",
};

static AGING: Scenario = Scenario {
    id: "aging",
    algo: Algo::Aging,
    refs: BASE,
    frames: 3,
    tick: 2,
    tau: 6,
    hint: "A cada ⏰ o contador desloca e recebe o bit R na esquerda. Sai o menor contador.",
};

static WORKING_SET: Scenario = Scenario {
    id: "working_set",
    algo: Algo::WorkingSet,
    refs: BASE,
    frames: 3,
    tick: 4,
    tau: 6,
    hint: "Idade = tempo atual − último uso. Acima de τ, a página está fora do conjunto de trabalho.",
};

static WSCLOCK: Scenario = Scenario {
    id: "wsclock",
    algo: Algo::WsClock,
    refs: BASE,
    frames: 3,
    tick: 4,
    tau: 6,
    hint: "O anel do relógio com idade e bit M: uma página suja fora do conjunto tem a escrita agendada e o ponteiro segue.",
};

static COMPARE: Scenario = Scenario {
    id: "compare",
    algo: Algo::Optimal,
    refs: BASE,
    frames: 3,
    tick: 4,
    tau: 6,
    hint: "Mesma cadeia, todos os algoritmos. Mexa nos quadros e na cadeia e veja quem se aproxima do ótimo.",
};

pub const ARTICLES: &[Article] = &[
    // ------------------------------------------------------------------ 3.4
    Article {
        icon: "🎯",
        title: "O problema da substituição",
        section: "3.4",
        ostep: &[ostep::SWAP_POLICIES],
        summary: "Quando a memória está cheia, qual página sai?",
        blocks: &[
            Block::P(
                "Quando ocorre uma falta de página e não existe quadro livre, o sistema \
                 operacional precisa escolher uma página residente para descartar. Esta \
                 tela estuda os algoritmos que fazem essa escolha.",
            ),
            Block::P(
                "A escolha tem uma consequência imediata sobre o custo da falta. Se a página \
                 descartada tiver sido modificada, o bit M estará ligado e será preciso \
                 escrevê-la em disco. Se não tiver sido modificada, a cópia em disco \
                 continua válida e a página é simplesmente sobrescrita.",
            ),
            Block::Warn(
                "Escolher bem importa por outra razão também: uma página muito referenciada \
                 que seja descartada será trazida de volta em seguida, ao custo de duas \
                 transferências de disco.",
            ),
            Block::H("As duas perguntas"),
            Block::P("Todo algoritmo precisa responder a duas perguntas:"),
            Block::N(&[
                "Como identificar, entre as páginas residentes, aquelas que provavelmente \
                 não serão referenciadas em breve?",
                "A página descartada deve pertencer ao processo que causou a falta, ou pode \
                 pertencer a qualquer processo?",
            ]),
            Block::P(
                "Os artigos seguintes respondem à primeira. A segunda é a escolha entre \
                 alocação local e global, tratada na tela de Projeto e Implementação.",
            ),
            Block::SeeAlso(
                "Alocação local × global está em Projeto e Implementação.",
                Screen::PageDesign,
            ),
            Block::Note(
                "O problema não é exclusivo da memória virtual. Ele reaparece em qualquer \
                 sistema com uma memória rápida e pequena diante de uma memória lenta e \
                 grande: os mesmos algoritmos aparecem nas memórias cache e nos servidores \
                 de páginas da Internet.",
            ),
            Block::H("Como ler o simulador"),
            Block::P(
                "Cada artigo traz o simulador rodando a mesma cadeia de referências. As \
                 colunas são as referências, em ordem. As linhas são os quadros de página, \
                 cada célula mostra a página que está naquele quadro depois da referência. \
                 Abaixo de cada coluna, ✔ é acerto e ✖ é falta, seguida da página que saiu. \
                 Clique numa coluna para ver o estado dos quadros e a explicação da decisão \
                 naquele passo.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 3.4.1
    Article {
        icon: "🔮",
        title: "O algoritmo ótimo",
        section: "3.4.1",
        ostep: &[ostep::SWAP_POLICIES],
        summary: "O melhor possível, e o único impossível de implementar.",
        blocks: &[
            Block::P(
                "No instante de uma falta de página, cada página residente pode ser rotulada \
                 com o número de instruções que serão executadas antes de ela ser \
                 referenciada de novo. Uma página que só será referenciada daqui a 8 milhões \
                 de instruções recebe o rótulo 8 000 000.",
            ),
            Block::P(
                "O algoritmo ótimo descarta a página de maior rótulo, aquela cuja próxima \
                 referência está mais distante no futuro. Adiar a falta o máximo possível \
                 é, por definição, a melhor decisão disponível.",
            ),
            Block::Warn(
                "O problema é evidente: o sistema operacional não conhece o futuro. Não há \
                 como determinar, no instante da falta, quando cada página será referenciada \
                 de novo.",
            ),
            Block::Replacer(&OPTIMAL),
            Block::H("Para que ele serve, então"),
            Block::P(
                "Ainda assim, o algoritmo é útil. Executamos o programa uma vez num \
                 simulador que registra todas as referências à memória, e só então aplicamos \
                 o algoritmo sobre o registro completo. O resultado dá o menor número de \
                 faltas alcançável naquela execução e serve como referência para avaliar os \
                 algoritmos reais.",
            ),
            Block::Note(
                "É exatamente o que o simulador faz: ele conhece a cadeia inteira de \
                 antemão, então consegue \"ver o futuro\". Anote o número de faltas. Nenhum \
                 dos algoritmos seguintes vai fazer menos na mesma cadeia.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 3.4.2
    Article {
        icon: "🏷",
        title: "NRU — não usadas recentemente",
        section: "3.4.2",
        ostep: &[],
        summary: "Quatro classes a partir dos bits R e M.",
        blocks: &[
            Block::P(
                "Os algoritmos implementáveis precisam estimar o futuro a partir do passado, \
                 e para isso usam os bits R e M da entrada da tabela de páginas:",
            ),
            Block::Defs(&[
                (
                    "R",
                    "ligado pelo hardware sempre que a página é referenciada, seja para \
                     leitura ou para escrita.",
                ),
                ("M", "ligado pelo hardware sempre que a página é escrita."),
            ]),
            Block::P(
                "Os dois bits são ligados pelo hardware e só podem ser desligados pelo \
                 sistema operacional. É essa assimetria que permite ao sistema medir o \
                 comportamento do programa sem interferir nele.",
            ),
            Block::P(
                "O sistema desliga o bit R de todas as páginas periodicamente, a cada \
                 interrupção de relógio. Assim, uma página com R ligado é uma página \
                 referenciada desde a última varredura, e não uma página referenciada em \
                 algum momento qualquer do passado.",
            ),
            Block::H("As quatro classes"),
            Block::Table2(
                "Classe (R, M)",
                "Situação da página",
                &[
                    ("0 — R=0, M=0", "não referenciada, não modificada"),
                    ("1 — R=0, M=1", "não referenciada, modificada"),
                    ("2 — R=1, M=0", "referenciada, não modificada"),
                    ("3 — R=1, M=1", "referenciada, modificada"),
                ],
            ),
            Block::Note(
                "A classe 1 parece contraditória, pois uma página modificada teria de ter \
                 sido referenciada. A explicação é a interrupção de relógio: ela desliga o \
                 bit R, mas nunca desliga o bit M.",
            ),
            Block::P(
                "O algoritmo NRU (Not Recently Used) descarta uma página escolhida ao acaso \
                 dentro da classe de menor ordem que não esteja vazia.",
            ),
            Block::Replacer(&NRU),
            Block::P(
                "Repare no julgamento embutido nessa ordem. O algoritmo prefere descartar \
                 uma página modificada e não referenciada, da classe 1, a descartar uma \
                 página limpa e referenciada, da classe 2. Ele aceita pagar uma escrita em \
                 disco para preservar uma página que o programa acabou de utilizar.",
            ),
            Block::Table2(
                "Vantagens",
                "Desvantagem",
                &[(
                    "Fácil de entender, barato de implementar e com desempenho razoável.",
                    "Quatro classes são uma medida grosseira do comportamento do programa, \
                     os algoritmos seguintes fazem melhor.",
                )],
            ),
        ],
    },
    // ---------------------------------------------------------------- 3.4.3
    Article {
        icon: "📥",
        title: "FIFO — primeiro a entrar, primeiro a sair",
        section: "3.4.3",
        ostep: &[ostep::SWAP_POLICIES],
        summary: "Idade não é utilidade, e a anomalia de Belady.",
        blocks: &[
            Block::P(
                "O algoritmo FIFO (First In, First Out) mantém uma lista de todas as \
                 páginas residentes, ordenada pelo instante em que cada uma foi carregada. \
                 Diante de uma falta, ele descarta a página do início da lista e insere a \
                 nova página no final.",
            ),
            Block::Warn(
                "A desvantagem é imediata: idade não é utilidade. A página mais antiga pode \
                 ser exatamente a mais utilizada, como acontece com as páginas de código de \
                 um laço principal ou de uma biblioteca carregada na inicialização.",
            ),
            Block::H("A anomalia de Belady"),
            Block::P(
                "Há ainda um defeito mais profundo. Sob o FIFO, aumentar o número de \
                 quadros disponíveis pode aumentar o número de faltas de página, resultado \
                 conhecido como anomalia de Belady. Um algoritmo em que mais memória produz \
                 pior desempenho é um algoritmo mal construído.",
            ),
            Block::Replacer(&FIFO),
            Block::Note(
                "O simulador está com a cadeia clássica da anomalia. Com 3 quadros são 9 \
                 faltas. Suba para 4 quadros e conte de novo: 10. Mais memória, mais faltas.",
            ),
            Block::P(
                "Por essas duas razões, o FIFO puro raramente é utilizado. Ele interessa \
                 como ponto de partida para os dois algoritmos seguintes, que o corrigem \
                 sem abandonar sua estrutura.",
            ),
        ],
    },
    // --------------------------------------------------------- 3.4.4, 3.4.5
    Article {
        icon: "🕰",
        title: "Segunda chance e o relógio",
        section: "3.4.4 e 3.4.5",
        ostep: &[ostep::SWAP_POLICIES],
        summary: "O FIFO corrigido por um exame do bit R, e a fila vira anel.",
        blocks: &[
            Block::P(
                "O algoritmo da segunda chance corrige o FIFO com um exame adicional. Antes \
                 de descartar a página mais antiga, ele examina o bit R dela:",
            ),
            Block::Defs(&[
                (
                    "R = 0",
                    "a página é antiga e não foi referenciada recentemente. É descartada de \
                     imediato.",
                ),
                (
                    "R = 1",
                    "a página é antiga mas continua em uso. O algoritmo desliga o bit R, move \
                     a página para o final da lista com um novo instante de carga e examina \
                     a página seguinte.",
                ),
            ]),
            Block::P(
                "O nome vem daí: uma página antiga que continua sendo utilizada recebe mais \
                 uma chance antes de ser descartada.",
            ),
            Block::Replacer(&SECOND_CHANCE),
            Block::Note(
                "No pior caso, todas as páginas têm R ligado. O algoritmo dá a volta \
                 completa, desliga todos os bits pelo caminho e retorna à primeira página, \
                 que agora tem R em 0 e é descartada. Nesse caso extremo, a segunda chance \
                 degenera no FIFO puro. É o que acontece logo na primeira falta do \
                 simulador, quando as três páginas acabaram de ser carregadas.",
            ),
            Block::H("O algoritmo do relógio"),
            Block::P(
                "Resta uma ineficiência: mover páginas de uma ponta a outra da lista a cada \
                 inspeção custa tempo e não acrescenta informação alguma. O algoritmo do \
                 relógio elimina esse custo com uma mudança apenas de estrutura de dados. As \
                 páginas ficam numa lista circular, e um ponteiro indica a mais antiga.",
            ),
            Block::P(
                "Diante de uma falta, o algoritmo examina a página apontada. Se o bit R dela \
                 vale 0, a página é descartada, a nova página ocupa seu lugar e o ponteiro \
                 avança uma posição. Se o bit R vale 1, o algoritmo o desliga, avança o \
                 ponteiro e repete o exame.",
            ),
            Block::Replacer(&CLOCK),
            Block::Note(
                "O relógio toma exatamente as mesmas decisões que a segunda chance, \
                 compare as duas grades, coluna a coluna, e é por isso que os dois \
                 costumam ser tratados como um só algoritmo. A diferença está no custo: \
                 avançar um ponteiro é mais barato que remover e reinserir um elemento numa \
                 lista. É um algoritmo simples, barato e realista.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 3.4.6
    Article {
        icon: "📉",
        title: "LRU — usadas menos recentemente",
        section: "3.4.6",
        ostep: &[ostep::SWAP_POLICIES],
        summary: "Aproximar o ótimo olhando o passado recente.",
        blocks: &[
            Block::P(
                "O algoritmo LRU (Least Recently Used) parte de uma aposta razoável sobre o \
                 comportamento dos programas: páginas muito utilizadas nas últimas \
                 instruções continuarão sendo utilizadas nas próximas, e páginas esquecidas \
                 há muito tempo continuarão esquecidas.",
            ),
            Block::P(
                "Sob essa aposta, a decisão é direta: descartar a página que passou mais \
                 tempo sem ser referenciada. O LRU aproxima o algoritmo ótimo olhando para \
                 o passado recente em vez do futuro.",
            ),
            Block::Replacer(&LRU),
            Block::H("Por que é caro"),
            Block::P(
                "A implementação exata é cara. Uma lista encadeada de todas as páginas \
                 residentes, da mais recente à mais antiga, precisa ser atualizada a cada \
                 referência à memória. Remover um elemento do meio da lista e recolocá-lo na \
                 frente a cada acesso é um custo alto demais para ser pago em software.",
            ),
            Block::P(
                "Uma implementação em hardware é possível: o processador mantém um contador \
                 de 64 bits, incrementado a cada instrução, e copia o valor do contador na \
                 entrada da tabela de páginas a cada referência. Na falta, o sistema \
                 descarta a página de menor valor.",
            ),
            Block::Table2(
                "Vantagem",
                "Desvantagem",
                &[(
                    "Excelente: fica próximo do algoritmo ótimo na maioria das cargas de \
                     trabalho.",
                    "Exige um hardware que poucas máquinas oferecem, o que obriga a procurar \
                     aproximações.",
                )],
            ),
            Block::Note(
                "A coluna \"último uso\" do simulador é exatamente esse contador: o instante \
                 da última referência a cada página. O simulador pode mantê-lo de graça, \
                 uma máquina real, não.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 3.4.7
    Article {
        icon: "⏳",
        title: "Envelhecimento",
        section: "3.4.7",
        ostep: &[],
        summary: "A aproximação do LRU que qualquer máquina pode pagar.",
        blocks: &[
            Block::P(
                "A aproximação mais utilizada do LRU se chama envelhecimento (aging) e usa \
                 apenas o bit R, que qualquer máquina com paginação oferece.",
            ),
            Block::P(
                "Cada página recebe um contador de 8 bits, inicialmente em zero. A cada \
                 interrupção de relógio, o sistema desloca esse contador um bit para a \
                 direita, insere o bit R da página na posição mais à esquerda e desliga esse \
                 bit.",
            ),
            Block::P(
                "O contador passa a ser um histórico das últimas oito varreduras, no qual as \
                 referências recentes pesam mais que as antigas. Na falta, o sistema \
                 descarta a página de menor contador.",
            ),
            Block::Replacer(&AGING),
            Block::H("Por que aproxima o LRU"),
            Block::P(
                "Considere duas páginas ao final de oito varreduras. Uma página referenciada \
                 apenas na varredura mais recente tem contador 10000000, que vale 128. Uma \
                 página referenciada nas sete varreduras anteriores, mas não na mais \
                 recente, tem contador 01111111, que vale 127. O algoritmo descarta a \
                 segunda, exatamente a decisão do LRU.",
            ),
            Block::Warn(
                "O contador tem memória finita. Com oito bits, duas páginas não \
                 referenciadas nas últimas oito varreduras ficam ambas com contador zero, e \
                 o algoritmo não consegue distingui-las. Ainda assim, o envelhecimento é \
                 uma boa aproximação do LRU a um custo que qualquer sistema pode pagar.",
            ),
            Block::Note(
                "No simulador, o relógio bate a cada 2 referências para o histórico se \
                 formar mais rápido. Acompanhe a coluna \"contador\" passo a passo: a cada \
                 ⏰ os bits andam para a direita e o bit R entra pela esquerda.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 3.4.8
    Article {
        icon: "👜",
        title: "O conjunto de trabalho",
        section: "3.4.8",
        ostep: &[],
        summary: "Localidade, paginação sob demanda, ultrapaginação e pré-paginação.",
        blocks: &[
            Block::P(
                "Os algoritmos vistos até aqui reagem às faltas de página. Os dois últimos \
                 tentam evitá-las, e para isso precisam de um conceito novo.",
            ),
            Block::P(
                "Na forma mais simples de paginação, um processo começa a executar sem \
                 nenhuma de suas páginas na memória. Cada página é carregada apenas quando \
                 referenciada pela primeira vez. Esse regime se chama paginação sob demanda, \
                 e funciona porque os programas apresentam localidade de referência: \
                 durante qualquer fase de sua execução, um processo referencia apenas uma \
                 fração pequena de suas páginas.",
            ),
            Block::Note(
                "Definição: o conjunto de trabalho de um processo é o conjunto de páginas \
                 que ele referenciou nas k referências mais recentes. Seu tamanho, w(k, t), \
                 cresce rapidamente para valores pequenos de k e depois se estabiliza, a \
                 expressão numérica da localidade.",
            ),
            Block::H("Ultrapaginação e pré-paginação"),
            Block::P(
                "Se o conjunto de trabalho de um processo está na memória, ele executa com \
                 poucas faltas de página. Se não está, o processo provoca uma falta a cada \
                 poucas instruções e passa mais tempo esperando o disco do que executando. \
                 Esse fenômeno se chama ultrapaginação (thrashing).",
            ),
            Block::P(
                "Daí vem a pré-paginação: o sistema registra o conjunto de trabalho de cada \
                 processo e, ao trazer um processo de volta do disco, carrega esse conjunto \
                 antes de deixá-lo executar. O processo retoma sem a rajada de faltas que a \
                 paginação sob demanda produziria.",
            ),
            Block::H("O algoritmo do conjunto de trabalho"),
            Block::P(
                "O algoritmo usa essa informação para escolher a página descartada. Na \
                 prática, ele não conta referências: usa o tempo de processador já consumido \
                 pelo processo, o tempo virtual atual, e um limiar de idade τ. Cada \
                 entrada da tabela guarda o instante do último uso da página. Diante de uma \
                 falta, o algoritmo varre a tabela inteira e examina cada página residente:",
            ),
            Block::Defs(&[
                (
                    "R = 1",
                    "a página está no conjunto de trabalho. O algoritmo apenas atualiza o \
                     instante do último uso e segue para a próxima entrada.",
                ),
                (
                    "R = 0 e idade > τ",
                    "a página está fora do conjunto de trabalho. Ela é descartada e a \
                     varredura termina.",
                ),
                (
                    "R = 0 e idade ≤ τ",
                    "a página ainda pertence ao conjunto de trabalho. O algoritmo a anota \
                     como candidata e segue adiante.",
                ),
            ]),
            Block::P(
                "Se a varredura termina sem encontrar nenhuma página fora do conjunto de \
                 trabalho, o algoritmo descarta a candidata mais antiga. Se nem candidatas \
                 existirem, ele descarta qualquer página limpa.",
            ),
            Block::Replacer(&WORKING_SET),
            Block::Table2(
                "Vantagem",
                "Desvantagem",
                &[(
                    "Evita a ultrapaginação em vez de apenas reagir a ela.",
                    "É caro: cada falta de página exige varrer a tabela de páginas inteira.",
                )],
            ),
            Block::Note(
                "No simulador o tempo virtual é o número da referência, e τ está em \
                 referências. Diminua τ e mais páginas ficam \"fora do conjunto\". Aumente e \
                 o algoritmo passa a cair no caso das candidatas.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 3.4.9
    Article {
        icon: "🧭",
        title: "WSClock",
        section: "3.4.9",
        ostep: &[],
        summary: "O relógio com a informação do conjunto de trabalho.",
        blocks: &[
            Block::P(
                "O algoritmo WSClock combina a estrutura circular do relógio com a \
                 informação do conjunto de trabalho, e é por isso o mais utilizado na \
                 prática.",
            ),
            Block::P(
                "A estrutura de dados é uma lista circular de quadros de página. A lista \
                 começa vazia e cresce à medida que as páginas são carregadas, até fechar o \
                 anel. Cada entrada contém o instante do último uso, o bit R e o bit M.",
            ),
            Block::H("A decisão a cada página apontada"),
            Block::Defs(&[
                (
                    "R = 1",
                    "a página foi utilizada recentemente. O algoritmo desliga o bit, atualiza \
                     o instante de uso, avança o ponteiro e repete o exame.",
                ),
                (
                    "R = 0, idade > τ, página limpa",
                    "está fora do conjunto de trabalho e não precisa ser escrita. O \
                     algoritmo a descarta.",
                ),
                (
                    "R = 0, idade > τ, página suja",
                    "o algoritmo agenda a escrita em disco e avança o ponteiro. Assim ele \
                     evita bloquear e dá tempo à escrita.",
                ),
            ]),
            Block::P(
                "Se o ponteiro completa a volta inteira, dois casos são possíveis. Se ao \
                 menos uma escrita foi agendada, o ponteiro continua girando até que alguma \
                 página fique limpa. Se nenhuma escrita foi agendada, todas as páginas estão \
                 no conjunto de trabalho, e o algoritmo descarta qualquer página limpa que \
                 encontrar.",
            ),
            Block::Replacer(&WSCLOCK),
            Block::Table2(
                "Vantagem",
                "Desvantagem",
                &[(
                    "Une o baixo custo do relógio à resistência à ultrapaginação do conjunto \
                     de trabalho.",
                    "Depende da escolha de τ, um parâmetro sem valor universalmente correto.",
                )],
            ),
            Block::Note(
                "As escritas em 3w deixam a página 3 suja. Quando ela envelhece além de τ, o \
                 simulador mostra a escrita sendo agendada e o ponteiro seguindo em frente, \
                 e a página saindo só na volta seguinte, já limpa.",
            ),
        ],
    },
    // --------------------------------------------------------------- 3.4.10
    Article {
        icon: "📊",
        title: "Resumo dos algoritmos",
        section: "3.4.10",
        ostep: &[ostep::SWAP_POLICIES],
        summary: "Todos lado a lado, na mesma cadeia de referências.",
        blocks: &[
            Block::P(
                "A tabela reúne os algoritmos estudados e o julgamento que fizemos de cada \
                 um deles:",
            ),
            Block::Table2(
                "Algoritmo",
                "Avaliação",
                &[
                    ("Ótimo", "Não implementável, mas útil como referência."),
                    ("NRU", "Muito grosseiro, mas barato."),
                    (
                        "FIFO",
                        "Descarta páginas importantes e sofre da anomalia de Belady.",
                    ),
                    ("Segunda chance", "Grande melhoria sobre o FIFO."),
                    (
                        "Relógio",
                        "As mesmas decisões da segunda chance, mais baratas.",
                    ),
                    ("LRU", "Excelente, mas difícil de implementar com exatidão."),
                    (
                        "Envelhecimento",
                        "Boa aproximação do LRU sem hardware especial.",
                    ),
                    (
                        "Conjunto de trabalho",
                        "Evita a ultrapaginação, mas a varredura é cara.",
                    ),
                    ("WSClock", "Eficiente e bom, o mais utilizado na prática."),
                ],
            ),
            Block::ReplCompare(&COMPARE),
            Block::Note(
                "Os dois melhores algoritmos da tabela são o envelhecimento e o WSClock, e \
                 os dois são aproximações. A lição geral está aí: em gerenciamento de \
                 memória, o algoritmo que vence não é o mais preciso. É o que oferece a \
                 melhor relação entre a qualidade da decisão e o custo de tomá-la.",
            ),
            Block::Warn(
                "Uma cadeia de 20 referências é curta demais para julgar os algoritmos. Ela \
                 serve para entender o mecanismo, não para medir desempenho. Com cadeias \
                 curtas, o NRU e o envelhecimento podem até empatar com o LRU, ou ganhar \
                 dele, por acaso.",
            ),
        ],
    },
    // ------------------------------------------------------------ exercícios
    Article {
        icon: "📝",
        title: "Exercícios",
        section: "3.4",
        ostep: &[],
        summary: "Para fixar a seção 3.4.",
        blocks: &[
            Block::P(
                "Os exercícios abaixo acompanham as anotações do capítulo. Para os que \
                 pedem uma cadeia de referências, o simulador aceita a sua: digite os \
                 números separados por espaço.",
            ),
            Block::N(&[
                "Explique por que o bit M determina o custo de descartar uma página.",
                "Descreva o algoritmo ótimo de substituição. Por que ele é impossível de \
                 implementar e para que ele serve?",
                "Descreva as quatro classes utilizadas pelo algoritmo NRU. Explique por que \
                 a classe 1 não é uma contradição.",
                "Explique por que o NRU prefere descartar uma página da classe 1 a \
                 descartar uma página da classe 2.",
                "Descreva o algoritmo FIFO e seus dois defeitos. Explique o que é a \
                 anomalia de Belady.",
                "Descreva o algoritmo da segunda chance. Em que situação ele degenera no \
                 FIFO puro?",
                "Compare o algoritmo da segunda chance e o algoritmo do relógio. Em que \
                 exatamente eles diferem?",
                "Explique a aposta sobre o comportamento dos programas que sustenta o \
                 algoritmo LRU. Por que a sua implementação exata é cara?",
                "Descreva o algoritmo de envelhecimento. Explique por que ele aproxima o \
                 LRU utilizando apenas o bit R.",
                "Duas páginas têm contadores de envelhecimento 00110000 e 00101111. \
                 Explique qual delas o algoritmo descarta.",
                "Explique a limitação de um contador de envelhecimento de oito bits.",
                "Defina conjunto de trabalho e localidade de referência. Explique a relação \
                 entre os dois conceitos.",
                "Defina ultrapaginação. Explique a relação entre esse fenômeno e o conjunto \
                 de trabalho.",
                "Explique o que é a pré-paginação e qual problema da paginação sob demanda \
                 ela resolve.",
                "Descreva os três casos examinados pelo algoritmo do conjunto de trabalho a \
                 cada entrada da tabela. Qual é a sua principal desvantagem?",
                "Descreva o algoritmo WSClock. Explique como ele combina os dois algoritmos \
                 que lhe deram origem.",
                "Compare os algoritmos estudados quanto à qualidade da decisão e ao custo \
                 de tomá-la. Explique por que os dois melhores são aproximações.",
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
