//! Conteúdo da tela "Projeto e Implementação" — seções 3.5 e 3.6 das
//! anotações.
//!
//! As decisões que separam um sistema de paginação que funciona de um que
//! funciona bem — alocação local × global, controle de carga, limpeza, tamanho
//! da página, espaços separados e páginas compartilhadas — e o que o sistema
//! operacional efetivamente faz, passo a passo, numa falta de página.

use crate::app::Screen;
use crate::ui::reader::{Article, Block, ostep};

pub const ARTICLES: &[Article] = &[
    // ---------------------------------------------------------------- 3.5.1
    Article {
        icon: "🌐",
        title: "Alocação local e global",
        section: "3.5 e 3.5.1",
        ostep: &[ostep::SWAP_POLICIES],
        summary: "Como repartir a memória entre os processos, e o PFF.",
        blocks: &[
            Block::P(
                "Conhecemos o mecanismo da paginação e os algoritmos que escolhem a página \
                 descartada. Esta tela trata das decisões que separam um sistema de \
                 paginação que funciona de um que funciona bem. A primeira é como repartir \
                 a memória entre os processos concorrentes, e ela responde à pergunta \
                 deixada em aberto na tela de Substituição.",
            ),
            Block::SeeAlso(
                "A pergunta ficou no primeiro artigo de Substituição de Páginas.",
                Screen::Replacement,
            ),
            Block::P(
                "Suponha três processos A, B e C residentes na memória, e suponha que A \
                 provoque uma falta de página. O algoritmo de substituição precisa \
                 encontrar a página menos utilizada, e há duas maneiras de delimitar a \
                 busca:",
            ),
            Block::Table2(
                "Substituição local",
                "Substituição global",
                &[
                    (
                        "O algoritmo examina apenas as páginas de A. A página descartada \
                         pertence necessariamente ao processo que causou a falta.",
                        "O algoritmo examina todas as páginas residentes. A página descartada \
                         pode pertencer a B ou a C, mesmo que eles não tenham causado falta \
                         alguma.",
                    ),
                    (
                        "Mantém fixo o número de quadros de cada processo: cada falta devolve \
                         ao processo o quadro que ele acabou de perder.",
                        "Deixa o número de quadros de cada processo variar ao longo do tempo.",
                    ),
                ],
            ),
            Block::P(
                "Os algoritmos globais funcionam melhor na prática, e a razão está no \
                 comportamento dos processos. O conjunto de trabalho de um processo cresce \
                 e encolhe conforme ele muda de fase, e uma alocação fixa produz dois \
                 defeitos simultâneos: o processo em fase de expansão sofre ultrapaginação, \
                 enquanto o processo em fase de contração desperdiça quadros que outro \
                 utilizaria.",
            ),
            Block::H("Quantos quadros cada processo merece: o PFF"),
            Block::P(
                "Uma opção é distribuir os quadros proporcionalmente ao tamanho de cada \
                 processo. É simples, mas ignora o comportamento real de cada um. A opção \
                 melhor mede esse comportamento diretamente, e o algoritmo que faz a medição \
                 se chama PFF (Page Fault Frequency).",
            ),
            Block::P(
                "O PFF conta as faltas de página de cada processo por unidade de tempo, \
                 usando uma média sobre uma janela deslizante, e compara a taxa obtida com \
                 dois limiares:",
            ),
            Block::Defs(&[
                (
                    "Acima do limiar superior",
                    "o processo tem quadros de menos. O sistema aumenta sua alocação.",
                ),
                (
                    "Abaixo do limiar inferior",
                    "o processo tem quadros de sobra. O sistema reduz sua alocação e devolve \
                     os quadros aos demais processos.",
                ),
            ]),
            Block::Note(
                "O PFF não decide qual página descartar. Ele decide quantos quadros cada \
                 processo merece, e deixa a escolha da página para o algoritmo de \
                 substituição. As duas decisões são independentes.",
            ),
            Block::Warn(
                "Nem todo algoritmo aceita as duas políticas. Os baseados em filas (FIFO, \
                 relógio, aproximações do LRU) funcionam nas duas. Os baseados no conjunto \
                 de trabalho são diferentes: o conjunto de trabalho é definido para um \
                 processo específico, e tanto ele quanto o WSClock pressupõem, portanto, \
                 uma política local.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 3.5.2
    Article {
        icon: "🧯",
        title: "Controle de carga",
        section: "3.5.2",
        ostep: &[ostep::SWAP_POLICIES],
        summary: "Quando nem a melhor política evita a ultrapaginação: o OOM.",
        blocks: &[
            Block::P(
                "Mesmo com o melhor algoritmo de substituição e com uma política global bem \
                 ajustada, a ultrapaginação ainda pode ocorrer. Ela é inevitável sempre que \
                 a soma dos conjuntos de trabalho de todos os processos ultrapassa a \
                 memória instalada.",
            ),
            Block::P(
                "O sintoma é reconhecível pelo próprio PFF. Quando o algoritmo indica que \
                 vários processos precisam de mais quadros e nenhum processo precisa de \
                 menos, não há redistribuição possível. O problema deixou de ser de \
                 distribuição e passou a ser de excesso de demanda.",
            ),
            Block::H("Reduzir quem disputa a memória"),
            Block::P(
                "A solução é reduzir o número de processos que disputam a memória. A forma \
                 menos agressiva é devolver alguns processos ao disco por completo, \
                 liberando todos os seus quadros de uma vez, e trazê-los de volta quando a \
                 disputa diminuir.",
            ),
            Block::P(
                "Quando nem isso basta, resta encerrar processos. Muitos sistemas mantêm \
                 para isso um mecanismo dedicado, chamado de OOM (Out Of Memory killer), \
                 acionado quando a memória se esgota:",
            ),
            Block::L(&[
                "O OOM examina todos os processos e atribui uma pontuação a cada um, na \
                 qual pesam a memória consumida e a importância do processo para o sistema.",
                "Ele encerra então o menor número possível de processos, evitando os \
                 processos críticos.",
            ]),
            Block::Note(
                "É uma decisão ruim tomada para evitar uma decisão pior, que seria a parada \
                 completa da máquina.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 3.5.3
    Article {
        icon: "🧹",
        title: "Política de limpeza",
        section: "3.5.3",
        ostep: &[ostep::SWAP_MECHANISMS],
        summary: "O daemon de paginação tira a escrita do caminho crítico.",
        blocks: &[
            Block::P(
                "Um sistema de paginação funciona melhor quando existem quadros livres no \
                 instante da falta. Se todos os quadros estiverem ocupados, cada falta \
                 obriga a descartar uma página, e a página descartada pode estar suja, o \
                 que acrescenta uma escrita em disco ao caminho crítico.",
            ),
            Block::P(
                "Para manter essa reserva, os sistemas mantêm um processo em segundo plano \
                 chamado de daemon de paginação. Ele dorme a maior parte do tempo e desperta \
                 periodicamente para examinar o estado da memória.",
            ),
            Block::P(
                "Se o número de quadros livres estiver abaixo de um limiar, o daemon \
                 seleciona páginas pelo algoritmo de substituição em uso e escreve em disco \
                 as que estiverem sujas. As páginas continuam residentes, mas passam a estar \
                 limpas, e podem ser descartadas de imediato.",
            ),
            Block::Note(
                "O trabalho de escrita saiu do caminho crítico da falta de página. Quando a \
                 falta acontece, a vítima já está limpa: basta sobrescrevê-la.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 3.5.4
    Article {
        icon: "📐",
        title: "Tamanho da página",
        section: "3.5.4",
        ostep: &[ostep::SMALLER_TABLES],
        summary: "Um compromisso com resposta numérica: p = √(2se).",
        blocks: &[
            Block::P(
                "O tamanho da página é um parâmetro que o sistema operacional escolhe, \
                 dentro das opções que o hardware oferece, e ele não tem resposta óbvia.",
            ),
            Block::Table2(
                "A favor de páginas pequenas",
                "Contra páginas pequenas",
                &[
                    (
                        "Menos fragmentação interna: um segmento de código ou de dados \
                         dificilmente preenche uma página inteira, e a parte vazia da última \
                         página é memória desperdiçada.",
                        "Um mesmo programa passa a ocupar mais páginas, o que aumenta a tabela \
                         de páginas.",
                    ),
                    (
                        "Menos memória ocupada por páginas inúteis: um programa dividido em \
                         fases carrega apenas a fase corrente.",
                        "O número de transferências com o disco cresce, e transferir muitas \
                         páginas pequenas é mais lento que transferir poucas páginas grandes.",
                    ),
                    (
                        "",
                        "Cada entrada da TLB passa a cobrir menos memória, o que reduz a \
                         cobertura.",
                    ),
                ],
            ),
            Block::H("O compromisso, em números"),
            Block::P(
                "O desperdício de memória por processo é a soma de dois termos: o espaço \
                 da tabela de páginas e a metade média da última página.",
            ),
            Block::Code(&[
                "o(p) = s·e / p  +  p / 2",
                "",
                "s  tamanho médio de um processo, em bytes",
                "e  tamanho de uma entrada da tabela de páginas, em bytes",
                "p  tamanho da página",
                "",
                "derivando em p e igualando a zero:   p = √(2·s·e)",
            ]),
            Block::PageSizeCalc,
            Block::Note(
                "O resultado numérico é revelador. Com processos de 1 MB e entradas de 8 \
                 bytes, p = √(2 × 2²⁰ × 8) = 4096 bytes, exatamente o tamanho de página \
                 adotado pela maioria dos sistemas atuais.",
            ),
            Block::P(
                "Como não existe um valor bom para todos os usos, os sistemas atuais \
                 oferecem mais de um. Páginas de 4 KB atendem o caso comum. Páginas grandes, \
                 de alguns megabytes, atendem processos que utilizam regiões extensas e \
                 contíguas de memória e sofreriam com a cobertura reduzida da TLB.",
            ),
        ],
    },
    // --------------------------------------------------------- 3.5.5, 3.5.6
    Article {
        icon: "🔀",
        title: "Espaços separados e páginas compartilhadas",
        section: "3.5.5 e 3.5.6",
        ostep: &[],
        summary: "Instruções e dados em tabelas distintas, e por que isso ajuda a compartilhar.",
        blocks: &[
            Block::P(
                "A maioria dos computadores mantém um único espaço de endereçamento para \
                 instruções e dados. Enquanto esse espaço é grande o bastante, a organização \
                 não apresenta problema algum.",
            ),
            Block::P(
                "Quando o espaço é pequeno, a alternativa é dividi-lo em dois espaços \
                 independentes: o espaço I, para as instruções, e o espaço D, para os dados. \
                 Cada espaço recebe sua própria tabela de páginas, e a entrada do processo \
                 na tabela de processos passa a conter dois ponteiros, um para cada tabela.",
            ),
            Block::Note(
                "A separação dobra o espaço disponível sem aumentar o número de bits do \
                 endereço. E facilita o compartilhamento de páginas, que é hoje a sua \
                 principal justificativa.",
            ),
            Block::H("Páginas compartilhadas"),
            Block::P(
                "Num sistema multiprogramado, é comum que vários usuários executem o mesmo \
                 programa ao mesmo tempo, e mais comum ainda que programas distintos usem a \
                 mesma biblioteca. Manter uma cópia das mesmas páginas de código para cada \
                 processo é um desperdício evidente: é mais eficiente que os processos \
                 compartilhem os mesmos quadros, com as tabelas de páginas de todos \
                 apontando para os mesmos endereços físicos.",
            ),
            Block::Warn(
                "A questão crítica é o que pode ser compartilhado. Páginas de instruções são \
                 apenas lidas e podem ser compartilhadas sem risco. Páginas de dados são \
                 escritas, e compartilhá-las faria com que a escrita de um processo \
                 aparecesse no outro.",
            ),
            Block::P(
                "A separação entre os espaços I e D resolve o problema de maneira direta: \
                 as tabelas do espaço I apontam para os mesmos quadros em todos os \
                 processos que executam o programa, enquanto as tabelas do espaço D são \
                 privativas de cada processo.",
            ),
            Block::P(
                "O compartilhamento cria uma obrigação para o sistema operacional. Quando \
                 um processo termina, o sistema não pode liberar todos os seus quadros, pois \
                 alguns pertencem também a outros processos. Uma página só é liberada quando \
                 o último processo que a utiliza termina, o que exige um contador de \
                 referências por quadro.",
            ),
            Block::H("Dados compartilhados no UNIX: o fork"),
            Block::P(
                "Nos sistemas do tipo UNIX, páginas de dados também são compartilhadas, mas \
                 apenas entre um processo pai e seu filho logo após o fork. O \
                 compartilhamento dura enquanto nenhum dos dois escreve. A primeira escrita \
                 provoca a duplicação da página afetada, e é esse mecanismo que torna \
                 barata uma chamada que, em princípio, copiaria o processo inteiro.",
            ),
            Block::SeeAlso(
                "O fork está nos Conceitos de Processos e Threads.",
                Screen::Texts,
            ),
        ],
    },
    // ---------------------------------------------------------------- 3.6.1
    Article {
        icon: "🛠",
        title: "O sistema operacional e a paginação",
        section: "3.6 e 3.6.1",
        ostep: &[ostep::SWAP_MECHANISMS],
        summary: "Os quatro momentos em que o sistema atua.",
        blocks: &[
            Block::P(
                "As seções anteriores trataram do mecanismo e das políticas. Agora, o que o \
                 sistema operacional precisa efetivamente fazer para que a paginação \
                 funcione. Ele atua em quatro momentos da vida de um processo:",
            ),
            Block::Defs(&[
                (
                    "Criação do processo",
                    "o sistema estima o tamanho do programa e de seus dados, constrói a \
                     tabela de páginas e reserva espaço em disco para as páginas ausentes. A \
                     localização da tabela e da área em disco fica registrada na tabela de \
                     processos.",
                ),
                (
                    "Execução do processo",
                    "o sistema reinicializa a MMU para apontar à tabela de páginas do \
                     processo escalonado. Se a TLB não usa identificadores de espaço de \
                     endereçamento, ela também precisa ser descarregada.",
                ),
                (
                    "Falta de página",
                    "o sistema descobre qual endereço virtual a provocou, escolhe um quadro, \
                     transfere a página necessária do disco e repete a instrução. É o assunto \
                     do próximo artigo.",
                ),
                (
                    "Término do processo",
                    "o sistema libera a tabela de páginas, os quadros ocupados pelo processo \
                     e a área em disco reservada a ele. Quadros compartilhados só são \
                     liberados quando o último processo que os utiliza termina.",
                ),
            ]),
            Block::SeeAlso(
                "A troca de contexto que reinicializa a MMU está no Escalonamento.",
                Screen::Scheduling,
            ),
        ],
    },
    // ---------------------------------------------------------------- 3.6.2
    Article {
        icon: "🚨",
        title: "Tratamento de uma falta de página",
        section: "3.6.2",
        ostep: &[ostep::SWAP_MECHANISMS],
        summary: "Da interrupção ao retorno ao modo usuário, passo a passo.",
        blocks: &[
            Block::P(
                "O tratamento de uma falta de página é a operação mais delicada de um \
                 sistema de paginação, e vale examiná-la passo a passo:",
            ),
            Block::N(&[
                "O hardware provoca um desvio forçado para o núcleo e salva o contador de \
                 programa na pilha.",
                "Uma rotina em linguagem de montagem salva os registradores gerais e as \
                 demais informações voláteis, e então chama o tratador de faltas de página.",
                "O sistema descobre qual página virtual foi solicitada. Alguns \
                 processadores guardam esse endereço num registrador próprio. Nos demais, o \
                 sistema precisa ler a instrução que falhou e refazer sua decodificação para \
                 determinar o endereço.",
                "Conhecido o endereço, o sistema verifica se ele é válido e se o acesso \
                 respeita os bits de proteção. Se não for, o processo recebe um sinal ou é \
                 encerrado. Se for, o sistema procura um quadro livre.",
                "Se não houver quadro livre, o algoritmo de substituição escolhe uma página \
                 para descartar.",
                "Se a página escolhida estiver suja, ela é escalonada para escrita em \
                 disco. O processo que causou a falta permanece suspenso, e o escalonador \
                 entrega o processador a outro processo enquanto a escrita acontece.",
                "Com o quadro limpo, o sistema localiza em disco o endereço da página \
                 necessária e agenda sua leitura.",
                "Quando a interrupção do disco indica que a página chegou, a tabela de \
                 páginas é atualizada e o quadro passa a constar como residente.",
                "A instrução que falhou é restaurada ao seu estado inicial e o contador de \
                 programa volta a apontar para ela.",
                "O processo que causou a falta é marcado como pronto, e o controle retorna \
                 à rotina em linguagem de montagem.",
                "A rotina recarrega os registradores e as demais informações de estado e \
                 retorna ao modo usuário. O processo retoma a execução como se nada tivesse \
                 acontecido.",
            ]),
            Block::H("Reiniciar a instrução é mais difícil do que parece"),
            Block::P(
                "O passo 9 merece atenção. Uma instrução pode ter modificado registradores \
                 antes de provocar a falta, e reiniciá-la sem desfazer essas modificações \
                 produziria um resultado errado.",
            ),
            Block::Warn(
                "O caso clássico é o de uma instrução que incrementa um registrador de \
                 índice antes de acessar a memória. Se a falta ocorre no acesso, o \
                 registrador já foi incrementado, e repetir a instrução o incrementaria uma \
                 segunda vez.",
            ),
            Block::P(
                "Por essa razão, os processadores projetados para memória virtual armazenam \
                 informações suficientes para desfazer os efeitos parciais de uma instrução \
                 interrompida. É um requisito de hardware, e não uma dificuldade que o \
                 sistema operacional possa contornar sozinho.",
            ),
            Block::H("Fechando o capítulo"),
            Block::Note(
                "Vale reter o gesto que organizou o capítulo inteiro. Partimos de programas \
                 que enxergavam a memória física e chegamos a programas que enxergam um \
                 espaço de endereçamento privativo, contínuo e maior que a memória \
                 instalada. Entre as duas pontas está um mecanismo de tradução em hardware, \
                 um conjunto de políticas em software e a aposta de que os programas \
                 apresentam localidade.",
            ),
        ],
    },
    // ------------------------------------------------------------ exercícios
    Article {
        icon: "📝",
        title: "Exercícios",
        section: "3.5 e 3.6",
        ostep: &[],
        summary: "Para fixar as seções 3.5 e 3.6.",
        blocks: &[
            Block::P(
                "Os exercícios abaixo acompanham as anotações do capítulo. A curva do \
                 tamanho da página faz a conta do exercício 8 para você conferir.",
            ),
            Block::N(&[
                "Diferencie as políticas de substituição local e global. Explique por que a \
                 política global costuma funcionar melhor.",
                "Descreva o algoritmo PFF. Explique por que ele não decide qual página \
                 descartar.",
                "Explique por que os algoritmos baseados no conjunto de trabalho pressupõem \
                 uma política local.",
                "Descreva o sintoma da ultrapaginação identificável pelo PFF. Que \
                 providências o sistema pode tomar?",
                "Explique a função do OOM e os critérios que ele utiliza para escolher um \
                 processo.",
                "Explique a função do daemon de paginação. Por que escrever páginas \
                 antecipadamente reduz o custo de uma falta?",
                "Liste duas vantagens e três desvantagens de páginas pequenas.",
                "Utilizando a expressão p = √(2se), calcule o tamanho de página ideal para \
                 processos de 4 MB e entradas de 8 bytes.",
                "Explique por que os sistemas atuais oferecem mais de um tamanho de página.",
                "Descreva a organização com espaços separados de instruções e de dados. \
                 Qual é hoje a sua principal justificativa?",
                "Explique por que páginas de instruções podem ser compartilhadas e páginas \
                 de dados não podem.",
                "Explique por que o compartilhamento de páginas exige um contador de \
                 referências por quadro.",
                "Liste os quatro momentos em que o sistema operacional atua sobre a \
                 paginação e descreva a tarefa executada em cada um deles.",
                "Descreva, passo a passo, o tratamento de uma falta de página, da \
                 interrupção até o retorno ao modo usuário.",
                "Explique por que reiniciar a instrução que provocou uma falta de página é \
                 mais difícil do que aparenta. Descreva um exemplo que ilustre a \
                 dificuldade.",
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
