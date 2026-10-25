//! Conteúdo de "Implementação do Sistema de Arquivos" (seções 4.3 e 4.4).
//!
//! Vem das anotações em `Anotações/4 - Sistema de Arquivos.md` (Tanenbaum &
//! Bos, capítulo 4), com o OSTEP (capítulos 40 e 42) como leitura complementar.
//! Os diagramas de disco ficam em `fsview.rs`.

use crate::app::Screen;
use crate::ui::reader::{ostep, Article, Block};

pub const ARTICLES: &[Article] = &[
    // ---------------------------------------------------------------- 4.3.1
    Article {
        icon: "💽",
        title: "Esquema do disco e inicialização",
        section: "4.3.1",
        // Inicialização (MBR/UEFI) não tem capítulo próprio no OSTEP.
        ostep: &[],
        summary: "Partições, MBR, GPT e o bloco de inicialização.",
        blocks: &[
            Block::P(
                "Um sistema de arquivos vive em um disco, que pode ser dividido em partições. \
                 Cada partição pode conter um sistema de arquivos independente. O começo do \
                 disco guarda a informação que liga a máquina.",
            ),
            Block::DiskPartition,
            Block::H("MBR"),
            Block::P(
                "Nos sistemas antigos, o setor 0 do disco é o MBR (Master Boot Record). No fim do \
                 MBR fica a tabela de partições, com o início e o fim de cada uma. Uma partição é \
                 marcada como ativa.",
            ),
            Block::P(
                "A BIOS executa o MBR, que localiza a partição ativa e executa o bloco de \
                 inicialização que está nela. Esse bloco, por sua vez, carrega o sistema \
                 operacional daquela partição.",
            ),
            Block::H("UEFI"),
            Block::P(
                "O UEFI é o esquema moderno, feito para discos maiores e partida mais rápida. Ele \
                 localiza a GPT, a tabela de partições que fica logo no início do disco e tem uma \
                 cópia de segurança no fim.",
            ),
            Block::Note(
                "Com a GPT, o firmware UEFI já sabe ler um sistema de arquivos simples e carregar \
                 de lá os programas de inicialização, em vez de depender de um único bloco de 512 \
                 bytes como no MBR.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 4.3.1 (layout)
    Article {
        icon: "🧩",
        title: "O layout de um sistema de arquivos",
        section: "4.3.1 e 4.3.2",
        ostep: &[ostep::FS_IMPL],
        summary: "Superbloco, bitmaps, tabela de i-nodes e região de dados.",
        blocks: &[
            Block::P(
                "Dentro de uma partição, o sistema de arquivos divide o espaço em blocos de \
                 tamanho fixo, por exemplo 4 KB. O OSTEP usa um sistema simples, o vsfs, para \
                 mostrar as estruturas que quase todo sistema de arquivos tem.",
            ),
            Block::FsLayout,
            Block::L(&[
                "O superbloco guarda os parâmetros do sistema de arquivos: quantos i-nodes e \
                 blocos existem e onde cada região começa.",
                "O bitmap de i-nodes e o bitmap de dados marcam, com um bit cada, o que está \
                 livre e o que está ocupado.",
                "A tabela de i-nodes guarda um i-node por arquivo. Com i-nodes de 256 bytes, \
                 cabem 16 por bloco de 4 KB.",
                "A região de dados ocupa quase todo o disco e guarda o conteúdo dos arquivos e \
                 dos diretórios.",
            ]),
            Block::Note(
                "Ao montar o sistema de arquivos, o sistema operacional lê o superbloco primeiro. \
                 A partir dele descobre onde ficam as outras regiões.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 4.3.2 (alocação)
    Article {
        icon: "🧮",
        title: "Como os blocos de um arquivo são rastreados",
        section: "4.3.2",
        ostep: &[ostep::FS_IMPL],
        summary: "Contígua, lista encadeada, FAT e i-node.",
        blocks: &[
            Block::P(
                "A questão central da implementação é saber quais blocos do disco pertencem a \
                 cada arquivo. Há quatro métodos clássicos, cada um com um compromisso diferente.",
            ),
            Block::AllocMethods,
            Block::H("Alocação contígua"),
            Block::P(
                "O arquivo ocupa uma sequência de blocos vizinhos. Basta guardar o primeiro bloco \
                 e o tamanho. A leitura é rápida, mas remover arquivos abre lacunas e a \
                 fragmentação logo impede alocar arquivos grandes.",
            ),
            Block::H("Lista encadeada"),
            Block::P(
                "Cada bloco guarda, no fim, o ponteiro para o próximo. Não há fragmentação, mas o \
                 acesso aleatório fica lento, porque chegar ao bloco n exige percorrer os \
                 anteriores.",
            ),
            Block::H("Lista encadeada com tabela (FAT)"),
            Block::P(
                "Os ponteiros saem dos blocos e vão para uma tabela em memória, a FAT. Com ela na \
                 RAM, o percurso da lista é rápido. O custo é manter a tabela inteira carregada, \
                 o que não escala para discos muito grandes.",
            ),
            Block::H("I-node"),
            Block::P(
                "Cada arquivo tem uma estrutura própria, o i-node, que lista os atributos e os \
                 endereços dos seus blocos. A vantagem é que o i-node só precisa estar na memória \
                 enquanto o arquivo está aberto, o que ocupa bem menos espaço que a FAT.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 4.3.2 (i-node)
    Article {
        icon: "🌲",
        title: "O i-node e o índice de vários níveis",
        section: "4.3.2",
        ostep: &[ostep::FS_IMPL],
        summary: "Ponteiros diretos e indiretos para arquivos de qualquer tamanho.",
        blocks: &[
            Block::P(
                "O i-node precisa apontar para os blocos do arquivo. Um conjunto fixo de \
                 ponteiros diretos resolve o caso dos arquivos pequenos, mas não daria conta de \
                 um arquivo enorme. A saída é o índice de vários níveis.",
            ),
            Block::InodeIndex,
            Block::L(&[
                "Alguns ponteiros diretos apontam direto para blocos de dados. Doze é um número \
                 comum.",
                "O ponteiro indireto aponta para um bloco cheio de ponteiros, cada um para um \
                 bloco de dados.",
                "O ponteiro indireto duplo aponta para um bloco de ponteiros para blocos de \
                 ponteiros, e assim por diante com o triplo.",
            ]),
            Block::Note(
                "A árvore é desbalanceada de propósito. A maioria dos arquivos é pequena, então \
                 vale otimizar esse caso: um arquivo pequeno usa só os ponteiros diretos e nem \
                 toca nos indiretos.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 4.3.3
    Article {
        icon: "🗺",
        title: "Diretórios e o caminho de acesso",
        section: "4.3.3",
        ostep: &[ostep::FS_IMPL],
        summary: "Do nome ao i-node, e o que o disco lê para abrir um arquivo.",
        blocks: &[
            Block::P(
                "Abrir um arquivo começa pelo nome do caminho. O sistema precisa traduzir esse \
                 nome na informação que localiza os dados: os blocos (na alocação contígua), o \
                 bloco inicial (na lista encadeada) ou o i-node.",
            ),
            Block::P(
                "O diretório faz essa tradução. Ele é uma lista de entradas, cada uma com o nome \
                 e a informação para achar o arquivo. No modelo de i-nodes, a entrada guarda o \
                 nome e o número do i-node.",
            ),
            Block::H("Onde ficam os atributos"),
            Block::P(
                "Os metadados do arquivo podem ficar na própria entrada do diretório ou no \
                 i-node. No UNIX ficam no i-node, e o diretório guarda apenas o par nome e número \
                 de i-node. Assim o mesmo i-node pode ter vários nomes.",
            ),
            Block::H("O caminho de acesso"),
            Block::P(
                "Para abrir /foo/bar, o sistema lê o i-node da raiz, depois os dados da raiz para \
                 achar foo, depois o i-node de foo, depois os dados de foo para achar bar, e por \
                 fim o i-node de bar. Cada nível do caminho custa mais uma leitura.",
            ),
            Block::Note(
                "Ler um arquivo não consulta os bitmaps. O i-node já tem os endereços dos blocos. \
                 Os bitmaps só entram quando é preciso alocar espaço novo.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 4.3.5 e 4.3.6
    Article {
        icon: "📓",
        title: "Consistência: diário e estrutura em log",
        section: "4.3.5 e 4.3.6",
        ostep: &[ostep::JOURNALING, ostep::LFS],
        summary: "Como o sistema de arquivos sobrevive a uma falha no meio de uma operação.",
        blocks: &[
            Block::P(
                "Uma operação do sistema de arquivos costuma tocar várias estruturas. Se a \
                 máquina cai no meio, elas podem ficar inconsistentes. O caso clássico é remover \
                 um arquivo, que são três passos:",
            ),
            Block::N(&[
                "Remover a entrada do arquivo no diretório.",
                "Devolver o i-node ao conjunto de i-nodes livres.",
                "Devolver os blocos ao conjunto de blocos livres.",
            ]),
            Block::Warn(
                "Uma falha entre os passos deixa recurso desperdiçado, ou pior, um i-node e um \
                 diretório apontando para blocos que já não são do arquivo.",
            ),
            Block::H("Journaling"),
            Block::P(
                "A ideia do diário (journaling) é anotar primeiro o que vai ser feito, antes de \
                 fazer. Se o sistema cair, ao reiniciar ele lê o diário e refaz ou desfaz a \
                 operação que ficou pela metade. Quase todo sistema de arquivos moderno usa isso.",
            ),
            Block::H("Sistema estruturado em log"),
            Block::P(
                "O LFS foi um sistema experimental que tratava o disco inteiro como um log, \
                 escrevendo sempre no fim. A busca em disco é o gargalo, e escrever em sequência \
                 a evita. O LFS não se popularizou direto, mas suas ideias estão no kernel Linux.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 4.3.7 e 4.3.8
    Article {
        icon: "🔀",
        title: "Sistemas de arquivos virtuais e memória flash",
        section: "4.3.7 e 4.3.8",
        ostep: &[ostep::SSD],
        summary: "Um disco, vários sistemas de arquivos, e o hardware dos SSDs.",
        blocks: &[
            Block::P(
                "É comum uma máquina ter vários sistemas de arquivos ao mesmo tempo: o disco \
                 interno, um pen drive, um formato legado. O desafio é apresentar tudo de forma \
                 uniforme.",
            ),
            Block::Table2(
                "Windows",
                "Linux",
                &[
                    (
                        "Lista cada sistema por uma letra (C:, D:). É simples, mas quebra a \
                         hierarquia única.",
                        "Introduz o VFS (Virtual File System), que encaixa todos os sistemas em \
                         uma só árvore.",
                    ),
                    (
                        "",
                        "O VFS abstrai o que é comum a todos os sistemas e serve de interface. \
                         Uma chamada passa primeiro pelo VFS, que a encaminha ao sistema certo.",
                    ),
                ],
            ),
            Block::H("Memória flash"),
            Block::P(
                "Os SSDs são feitos de memória flash, com portas NAND, um hardware bem diferente \
                 do disco magnético. Não há cabeça nem rotação, então o custo de acesso aleatório \
                 cai muito, o que muda o que vale a pena otimizar no sistema de arquivos.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 4.4
    Article {
        icon: "📊",
        title: "Gerenciamento de espaço em disco",
        section: "4.4, 4.4.1 e 4.4.2",
        ostep: &[ostep::FS_IMPL, ostep::FFS],
        summary: "Tamanho do bloco e como rastrear os blocos livres.",
        blocks: &[
            Block::H("Tamanho do bloco"),
            Block::P(
                "Escolhido o bloco de tamanho fixo, resta decidir qual tamanho. É o mesmo \
                 compromisso da alocação de memória. Blocos grandes desperdiçam espaço em \
                 arquivos pequenos. Blocos pequenos exigem muitos ponteiros e deixam a leitura \
                 lenta.",
            ),
            Block::H("Monitorar os blocos livres"),
            Block::P("Há duas formas de saber quais blocos estão livres:"),
            Block::Defs(&[
                (
                    "Lista encadeada de blocos livres",
                    "uma lista que liga os blocos livres. Quanto maior o disco, maior a lista, o \
                     que gasta espaço para guardá-la.",
                ),
                (
                    "Mapa de bits",
                    "um bit por bloco, dizendo se está livre ou ocupado. É compacto e o mesmo \
                     esquema que o bitmap do layout usa.",
                ),
            ]),
            Block::Note(
                "Ao criar um arquivo, o sistema procura no mapa de bits os blocos livres de que \
                 precisa, marca cada um como ocupado e atualiza o mapa no disco.",
            ),
            Block::SeeAlso(
                "Os tamanhos e os blocos alocados de cada arquivo aparecem no Explorador.",
                Screen::FileExplorer,
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
        check::articles(ARTICLES, '4');
    }
}
