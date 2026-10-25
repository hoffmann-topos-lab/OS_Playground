//! Conteúdo da tela "Paginação" — seção 3.3 das anotações.
//!
//! Memória virtual e sobreposições, páginas e quadros, a decomposição do
//! endereço virtual, a tabela de páginas e a TLB. O tradutor de endereços usa
//! o exemplo das anotações (64 KB virtuais, 32 KB físicos, páginas de 4 KB).

use crate::app::Screen;
use crate::ui::memaddr::TranslateScenario;
use crate::ui::reader::{Article, Block, ostep};

static TRANSLATE: TranslateScenario = TranslateScenario {
    id: "translate",
    tlb: false,
    hint: "Arraste o endereço: a MMU troca o número da página pelo do quadro e não toca no deslocamento.",
};

static TLB: TranslateScenario = TranslateScenario {
    id: "tlb",
    tlb: true,
    hint: "Acesse endereços vizinhos e veja a TLB acertar. Salte de página em página e veja-a faltar.",
};

pub const ARTICLES: &[Article] = &[
    // ------------------------------------------------------------------ 3.3
    Article {
        icon: "🧠",
        title: "Memória virtual",
        section: "3.3",
        ostep: &[ostep::SWAP_MECHANISMS],
        summary: "Executar um programa que não cabe inteiro na memória.",
        blocks: &[
            Block::P(
                "Os registradores de base e limite resolvem a proteção e a realocação, mas \
                 exigem que o processo caiba inteiro na memória física. Enquanto os \
                 programas eram pequenos, essa exigência era razoável. Ela deixou de ser há \
                 muito tempo.",
            ),
            Block::P(
                "O problema não é só o de um programa maior que a memória. É também o de \
                 manter muitos programas grandes disponíveis ao mesmo tempo. Mover um \
                 programa de 1 GB entre a memória e o disco, a 1 GB/s, consome cerca de 1 \
                 segundo em cada sentido. Nenhum sistema interativo tolera essa espera a \
                 cada troca de processo.",
            ),
            Block::H("A primeira solução: sobreposições"),
            Block::P(
                "Historicamente, a primeira saída foi a sobreposição (overlay). O programa \
                 era dividido em pedaços, e apenas os pedaços necessários ficavam na \
                 memória:",
            ),
            Block::N(&[
                "Quando o programa era iniciado, apenas o gerenciador de sobreposições era \
                 carregado, e ele carregava a sobreposição 0.",
                "Ao terminar a sobreposição 0, o gerenciador carregava a sobreposição 1, \
                 acima da anterior se houvesse espaço, ou sobre ela caso contrário.",
                "O gerenciador executava a troca sozinho, mas a divisão do programa em \
                 sobreposições cabia ao programador: um trabalho manual, demorado e fácil \
                 de errar.",
            ]),
            Block::P(
                "Surge então a pergunta que organiza o resto do capítulo: seria possível \
                 transferir esse trabalho para o computador? A resposta é sim, e o método \
                 recebeu o nome de paginação.",
            ),
            Block::Note(
                "Definição: a memória virtual é a técnica na qual o espaço de endereçamento \
                 de cada processo é dividido em blocos, e apenas os blocos referenciados \
                 precisam estar na memória física.",
            ),
            Block::P(
                "Quando o programa referencia um bloco que está na memória, o hardware \
                 traduz o endereço imediatamente. Quando referencia um bloco ausente, o \
                 sistema operacional busca esse bloco no disco e repete a instrução que \
                 falhou. O programa não percebe a diferença, e é essa a razão do nome.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 3.3.1
    Article {
        icon: "📄",
        title: "Paginação: páginas e quadros",
        section: "3.3.1",
        ostep: &[ostep::PAGING],
        summary: "A MMU, o bit de presença e a falta de página.",
        blocks: &[
            Block::P(
                "Os endereços produzidos por um programa são chamados de endereços \
                 virtuais, e o conjunto de todos eles forma o espaço de endereços virtuais \
                 do processo. Num sistema sem memória virtual, esses endereços vão direto ao \
                 barramento de memória. Num sistema com memória virtual, eles vão para a \
                 MMU (Memory Management Unit), que os traduz em endereços físicos antes de \
                 entregá-los à memória.",
            ),
            Block::MmuDiagram,
            Block::P(
                "O espaço de endereços virtuais é dividido em unidades de tamanho fixo \
                 chamadas de páginas. A memória física é dividida em unidades do mesmo \
                 tamanho, chamadas de quadros de página. Página e quadro têm sempre o mesmo \
                 tamanho, e as transferências entre a memória e o disco são sempre de uma \
                 página inteira.",
            ),
            Block::H("Um exemplo pequeno"),
            Block::P(
                "Um computador com espaço virtual de 64 KB, memória física de 32 KB e \
                 páginas de 4 KB tem 16 páginas virtuais e 8 quadros de página. O tradutor \
                 abaixo mostra um mapeamento possível entre os dois conjuntos: cada página \
                 aponta para o quadro em que está, ou não aponta para lugar nenhum.",
            ),
            Block::Translator(&TRANSLATE),
            Block::P(
                "Há 16 páginas e apenas 8 quadros: a maior parte das páginas não está na \
                 memória, e o sistema precisa registrar esse fato. Esse registro é o bit de \
                 presença, mantido pelo hardware em cada entrada do mapeamento. Quando vale \
                 1, a página está em algum quadro. Quando vale 0, a página está apenas no \
                 disco.",
            ),
            Block::H("A falta de página"),
            Block::P(
                "Se o programa referencia uma página cujo bit de presença vale 0, a MMU não \
                 consegue traduzir o endereço e provoca um desvio forçado para o sistema \
                 operacional. Esse desvio se chama falta de página (page fault). O \
                 tratamento segue quatro passos:",
            ),
            Block::N(&[
                "O sistema operacional escolhe um quadro pouco utilizado.",
                "Escreve o conteúdo dele em disco, caso tenha sido modificado.",
                "Carrega nesse quadro a página referenciada e atualiza o mapeamento.",
                "Repete a instrução que falhou, agora com sucesso.",
            ]),
            Block::Note(
                "No tradutor, escolha um endereço de uma página ausente (o exemplo 20500 \
                 cai na página 5) e clique em \"Tratar a falta de página\". Com o quadro 2 \
                 livre, ele é usado. Sem quadros livres, uma página residente precisa sair\
                 , e escolher qual é o assunto da tela de Substituição de Páginas.",
            ),
            Block::SeeAlso(
                "Os algoritmos que escolhem a página descartada têm tela própria.",
                Screen::Replacement,
            ),
        ],
    },
    // ---------------------------------------------------------------- 3.3.1
    Article {
        icon: "🔢",
        title: "Decompondo o endereço virtual",
        section: "3.3.1",
        ostep: &[ostep::PAGING],
        summary: "Número da página e deslocamento, sem nenhuma divisão.",
        blocks: &[
            Block::P(
                "Falta descrever como a MMU quebra um endereço virtual em duas partes. O \
                 endereço é interpretado como um número de página seguido de um \
                 deslocamento dentro dela:",
            ),
            Block::Code(&[
                "n = v div p        d = v mod p",
                "",
                "v  endereço virtual        p  tamanho da página",
                "n  número da página        d  deslocamento dentro da página",
            ]),
            Block::P(
                "Como p é sempre uma potência de dois, essa aritmética não custa nada. Os \
                 bits mais significativos do endereço já são o número da página, e os menos \
                 significativos já são o deslocamento. Nenhuma divisão é executada de fato.",
            ),
            Block::H("De volta ao exemplo"),
            Block::P(
                "Os endereços virtuais têm 16 bits e as páginas têm 4 KB, de modo que 12 \
                 bits formam o deslocamento e os 4 restantes formam o número da página:",
            ),
            Block::L(&[
                "O endereço virtual 8196 corresponde a n = 2 e d = 4, pois 8196 = 2 × 4096 + 4.",
                "Se a página 2 estiver no quadro 6, o endereço físico será 6 × 4096 + 4 = 24580.",
                "O deslocamento atravessa a MMU intacto: a tradução substitui o número da \
                 página pelo número do quadro e não toca no restante do endereço.",
            ]),
            Block::Note(
                "Volte ao tradutor do artigo anterior e olhe as duas fileiras de bits: os \
                 12 bits laranja do deslocamento são idênticos nas duas, e só os bits azuis \
                 mudam. O endereço físico tem 15 bits porque 8 quadros cabem em 3 bits.",
            ),
            Block::H("Em C"),
            Block::Code(&[
                "#define BITS_DESLOCAMENTO 12                          // (1)",
                "#define TAMANHO_PAGINA    (1u << BITS_DESLOCAMENTO)",
                "",
                "unsigned numero_da_pagina(unsigned virtual) {",
                "    return virtual >> BITS_DESLOCAMENTO;              // (2)",
                "}",
                "",
                "unsigned deslocamento(unsigned virtual) {",
                "    return virtual & (TAMANHO_PAGINA - 1);            // (3)",
                "}",
                "",
                "unsigned traduz(unsigned virtual, unsigned quadro) {",
                "    return (quadro << BITS_DESLOCAMENTO)              // (4)",
                "         | deslocamento(virtual);",
                "}",
            ]),
            Block::N(&[
                "A constante em (1) fixa páginas de 4 KB, pois 2¹² = 4096.",
                "A função em (2) descarta os 12 bits menos significativos e devolve os \
                 restantes, o número da página.",
                "A função em (3) preserva apenas os 12 bits menos significativos, aplicando \
                 uma máscara de 12 bits em 1. O resultado é o deslocamento.",
                "A função em (4) monta o endereço físico: desloca o número do quadro para a \
                 posição alta e recoloca o deslocamento na posição baixa, sem alterá-lo.",
            ]),
        ],
    },
    // ---------------------------------------------------------------- 3.3.2
    Article {
        icon: "📋",
        title: "Tabelas de páginas",
        section: "3.3.2",
        ostep: &[ostep::PAGING, ostep::SMALLER_TABLES],
        summary: "A estrutura que a MMU consulta, e o que cada entrada guarda.",
        blocks: &[
            Block::P(
                "Resta descrever a estrutura que a MMU consulta para descobrir em qual quadro \
                 uma página se encontra.",
            ),
            Block::Note(
                "Definição: a tabela de páginas é a estrutura que mapeia números de páginas \
                 virtuais em números de quadros de página.",
            ),
            Block::P(
                "A tabela funciona como uma função: recebe o número da página virtual como \
                 argumento e devolve o número do quadro como resultado. O número da página é \
                 usado diretamente como índice, o que torna a consulta imediata.",
            ),
            Block::P(
                "Cada processo tem a sua própria tabela de páginas, e é isso que dá a cada \
                 processo um espaço de endereçamento privativo. Trocar de processo \
                 significa, entre outras coisas, apontar a MMU para outra tabela.",
            ),
            Block::H("A estrutura de uma entrada"),
            Block::P(
                "A organização exata de uma entrada varia de máquina para máquina, mas as \
                 informações que ela carrega são praticamente as mesmas em todas. Num \
                 computador atual, cada entrada ocupa 64 bits:",
            ),
            Block::PteDiagram,
            Block::Defs(&[
                (
                    "Número do quadro de página",
                    "o resultado da tradução e o campo mais importante da entrada: 52 bits \
                     para endereçar páginas de 4 KB. Ocupa a maior parte dos 64 bits.",
                ),
                (
                    "Presente / ausente (P)",
                    "indica se a entrada é válida. Quando vale 0, qualquer referência à \
                     página provoca uma falta de página.",
                ),
                (
                    "Proteção",
                    "indica quais acessos são permitidos. Três bits bastam para representar \
                     as permissões de leitura, escrita e execução de forma independente.",
                ),
                (
                    "Supervisor (S)",
                    "indica se a página só pode ser acessada em modo núcleo. É o que impede um \
                     programa do usuário de ler as estruturas do sistema operacional.",
                ),
                (
                    "Modificada (M)",
                    "o bit sujo: ligado pelo hardware sempre que a página sofre uma escrita. \
                     O sistema o consulta na hora de descartar a página, se está ligado, é \
                     preciso salvá-la em disco antes.",
                ),
                (
                    "Referenciada (R)",
                    "ligado pelo hardware a cada leitura ou escrita na página. Alimenta os \
                     algoritmos de substituição.",
                ),
                (
                    "Cache desabilitado (C)",
                    "impede que a página seja mantida em cache. Indispensável nas páginas \
                     mapeadas sobre registradores de dispositivos, nas quais um valor em \
                     cache seria um valor desatualizado.",
                ),
            ]),
            Block::Note(
                "Repare que o endereço em disco de uma página ausente não aparece na \
                 entrada. O hardware nunca precisa dessa informação: ele apenas provoca a \
                 falta de página e entrega o controle ao sistema operacional. O endereço em \
                 disco fica nas estruturas do próprio sistema, em software.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 3.3.3
    Article {
        icon: "⚡",
        title: "Acelerando a tradução: a TLB",
        section: "3.3.3",
        ostep: &[ostep::TLBS],
        summary: "Uma memória associativa dentro da MMU, e a aposta na localidade.",
        blocks: &[
            Block::P(
                "Qualquer implementação de paginação precisa resolver dois problemas de \
                 ordem prática:",
            ),
            Block::L(&[
                "O mapeamento entre endereços virtuais e físicos precisa ser rápido, pois \
                 acontece em toda referência à memória.",
                "A tabela de páginas precisa ser pequena, pois espaços de endereçamento \
                 grandes produzem tabelas enormes.",
            ]),
            Block::Warn(
                "Se a tabela de páginas estiver inteira na memória, cada referência do \
                 programa exige duas referências físicas: uma para ler a entrada da tabela e \
                 outra para ler o dado. O programa fica duas vezes mais lento, e a paginação \
                 deixa de compensar.",
            ),
            Block::P(
                "A observação que torna a técnica viável é a mesma que sustenta todas as \
                 memórias cache: os programas concentram suas referências num número \
                 pequeno de páginas, e as mesmas poucas entradas da tabela são lidas \
                 repetidamente.",
            ),
            Block::Note(
                "Definição: a TLB (Translation Lookaside Buffer) é uma memória associativa \
                 dentro da MMU que armazena um número pequeno de traduções recentes. Com \
                 ela, a MMU traduz um endereço sem consultar a tabela de páginas.",
            ),
            Block::P(
                "Cada entrada da TLB contém o número de uma página virtual, o número do \
                 quadro correspondente e os bits de proteção e de validade daquela página. \
                 As entradas são comparadas todas ao mesmo tempo, e não uma a uma.",
            ),
            Block::H("Funcionamento"),
            Block::Defs(&[
                (
                    "Acerto",
                    "quando um endereço virtual chega à MMU, o hardware verifica se o número \
                     da página está na TLB. Se estiver, e o acesso não violar os bits de \
                     proteção, o número do quadro é tirado diretamente dela. A tabela de \
                     páginas não é consultada.",
                ),
                (
                    "Falta",
                    "se o número da página não estiver na TLB, a MMU consulta a tabela de \
                     páginas na memória, descarta uma das entradas da TLB e coloca a nova \
                     tradução em seu lugar. A instrução é repetida em seguida.",
                ),
            ]),
            Block::Translator(&TLB),
            Block::H("Por que compensa"),
            Block::P(
                "O ganho é grande porque as faltas são raras. Considere um programa que \
                 percorre um vetor de 4096 inteiros de 4 bytes. O vetor ocupa 16 KB, quatro \
                 páginas de 4 KB. As 4096 leituras produzem apenas quatro faltas na TLB, uma \
                 por página, e a taxa de acerto é (4096 − 4) / 4096 ≈ 99,9%.",
            ),
            Block::Note(
                "O botão \"Percorrer um vetor de 16 KB\" reproduz exatamente essa conta a \
                 partir do endereço escolhido. Depois zere os contadores e faça acessos \
                 saltando de página em página: a taxa de acerto despenca.",
            ),
            Block::P(
                "Esse resultado depende inteiramente da localidade. A localidade espacial \
                 faz com que referências vizinhas caiam na mesma página, e a localidade \
                 temporal faz com que uma página recém-usada seja usada de novo em seguida. \
                 Um programa que salta aleatoriamente por um espaço de endereçamento grande \
                 destrói as duas e perde o benefício da TLB.",
            ),
            Block::P(
                "A quantidade de memória alcançável sem faltas na TLB se chama cobertura. \
                 Uma TLB de 64 entradas com páginas de 4 KB cobre 256 KB, pouco para os \
                 programas atuais, e um dos argumentos a favor de páginas maiores.",
            ),
            Block::H("A troca de contexto"),
            Block::P(
                "As traduções armazenadas na TLB pertencem ao processo que estava executando \
                 e não valem para o processo seguinte. Há duas soluções:",
            ),
            Block::Table2(
                "Descarregar a TLB",
                "Identificar o espaço de endereçamento",
                &[(
                    "Simples: a TLB inteira é esvaziada a cada troca de contexto. Mas o novo \
                     processo precisa reconstruir todas as suas traduções do zero.",
                    "Cada entrada ganha um ASID (Address Space Identifier). Entradas de \
                     processos diferentes convivem na TLB sem se confundir, e a troca de \
                     contexto não descarta nada.",
                )],
            ),
            Block::Table2(
                "Vantagem",
                "Desvantagem",
                &[(
                    "Reduz o custo da tradução a praticamente zero na grande maioria das \
                     referências.",
                    "É uma memória cara e pequena, e programas sem localidade continuam \
                     pagando o preço integral de duas referências à memória.",
                )],
            ),
        ],
    },
    // ------------------------------------------------------------ exercícios
    Article {
        icon: "📝",
        title: "Exercícios",
        section: "3.3",
        ostep: &[],
        summary: "Para fixar a seção 3.3.",
        blocks: &[
            Block::P(
                "Os exercícios abaixo acompanham as anotações do capítulo. O tradutor de \
                 endereços faz as contas de página e deslocamento para você conferir.",
            ),
            Block::N(&[
                "Descreva o funcionamento das sobreposições. Qual era a sua principal \
                 desvantagem e como a paginação a elimina?",
                "Defina memória virtual. Explique por que um programa não percebe que \
                 apenas parte dele está na memória.",
                "Diferencie página e quadro de página. Por que os dois têm necessariamente \
                 o mesmo tamanho?",
                "Um computador possui endereços virtuais de 32 bits e páginas de 4 KB. \
                 Calcule quantos bits formam o número da página e quantos formam o \
                 deslocamento.",
                "Para a máquina do exercício anterior, calcule o número da página e o \
                 deslocamento correspondentes ao endereço virtual 20500.",
                "Explique por que a decomposição de um endereço virtual não exige nenhuma \
                 divisão. Qual propriedade do tamanho da página torna esse resultado \
                 possível?",
                "Defina tabela de páginas. Explique por que cada processo precisa da sua.",
                "Liste os campos de uma entrada da tabela de páginas e descreva a função de \
                 cada um deles.",
                "Explique por que o endereço em disco de uma página ausente não é \
                 armazenado na entrada da tabela de páginas.",
                "Explique por que uma tabela de páginas mantida na memória, sem nenhum \
                 mecanismo de aceleração, tornaria a paginação inviável.",
                "Defina TLB e descreva o que acontece quando uma tradução é encontrada e \
                 quando não é encontrada.",
                "Um programa percorre sequencialmente um vetor de 8192 inteiros de 4 bytes \
                 em uma máquina com páginas de 4 KB. Calcule o número de faltas na TLB e a \
                 taxa de acerto.",
                "Explique o que é a cobertura da TLB. Calcule a cobertura de uma TLB de 128 \
                 entradas com páginas de 4 KB.",
                "Compare as duas soluções para o problema da TLB na troca de contexto. Por \
                 que os identificadores de espaço de endereçamento são preferíveis?",
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
