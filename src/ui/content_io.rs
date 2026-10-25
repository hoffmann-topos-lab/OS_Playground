//! Conteúdo de "Entrada e Saída" (seções 5.1, 5.2 e 5.3).
//!
//! Vem das anotações em `Anotações/5 - Entrada e Saída.md` (Tanenbaum & Bos,
//! capítulo 5), com o OSTEP (capítulo 36, I/O Devices) como leitura
//! complementar. Os diagramas e o simulador ficam em `ioview.rs`.

use crate::app::Screen;
use crate::ui::reader::{ostep, Article, Block};

pub const ARTICLES: &[Article] = &[
    // ---------------------------------------------------------------- 5.1.1
    Article {
        icon: "🔌",
        title: "Dispositivos de entrada e saída",
        section: "5.1 e 5.1.1",
        ostep: &[ostep::IO_DEVICES, ostep::HDD],
        summary: "Dispositivos de bloco e de caractere.",
        blocks: &[
            Block::P(
                "Controlar os dispositivos de entrada e saída é uma das tarefas do sistema \
                 operacional. Ele emite comandos, trata interrupções e erros e oferece uma \
                 interface comum para cada classe de dispositivo.",
            ),
            Block::P(
                "Um dispositivo pode ser visto pelo engenheiro, que olha os componentes físicos, \
                 ou pelo programador, que olha a interface de software. Este curso adota a \
                 segunda visão: como o hardware é programado.",
            ),
            Block::H("Duas classes"),
            Block::Defs(&[
                (
                    "Dispositivo de bloco",
                    "guarda a informação em blocos de tamanho fixo, cada um com seu endereço. \
                     Cada bloco é lido ou escrito de forma independente. Exemplos: HDs e SSDs.",
                ),
                (
                    "Dispositivo de caractere",
                    "envia ou recebe um fluxo de bytes, sem blocos nem endereço nem busca. \
                     Exemplos: impressoras, interfaces de rede, mouses e teclados.",
                ),
            ]),
            Block::Note(
                "A divisão não é perfeita. Relógios e telas sensíveis ao toque não se encaixam \
                 bem em nenhuma das duas. Ainda assim, ela guia a escrita dos drivers.",
            ),
            Block::SeeAlso(
                "No UNIX essa divisão é concreta: veja os dispositivos de /dev da sua máquina.",
                Screen::DeviceBrowser,
            ),
        ],
    },
    // ---------------------------------------------------------------- 5.1.2 e 5.1.3
    Article {
        icon: "🎛",
        title: "Controladores e registradores",
        section: "5.1.2 e 5.1.3",
        ostep: &[ostep::IO_DEVICES],
        summary: "A parte eletrônica do dispositivo e como a CPU fala com ela.",
        blocks: &[
            Block::P(
                "Uma unidade de entrada e saída tem uma parte mecânica, o dispositivo em si, e \
                 uma parte eletrônica, o controlador. Em um computador pessoal, o controlador é \
                 um chip na placa-mãe ou uma placa encaixada em um slot.",
            ),
            Block::P(
                "O controlador expõe alguns registradores. Lendo e escrevendo neles, o sistema \
                 operacional comanda o dispositivo, envia e recebe dados e descobre o estado da \
                 operação.",
            ),
            Block::DeviceModel,
            Block::H("Duas formas de chegar aos registradores"),
            Block::Defs(&[
                (
                    "Portas de E/S",
                    "cada registrador recebe um número, a porta. A CPU usa instruções próprias, \
                     como IN e OUT no x86, para ler e escrever nessas portas. São instruções \
                     privilegiadas.",
                ),
                (
                    "E/S mapeada na memória",
                    "os registradores aparecem como se fossem endereços de memória. A CPU lê e \
                     escreve com as mesmas instruções de sempre, e o hardware roteia o acesso \
                     para o dispositivo.",
                ),
            ]),
            Block::Note(
                "Nenhuma das duas é claramente melhor. A E/S mapeada na memória dispensa \
                 instruções novas. As duas abordagens ainda convivem nos sistemas de hoje.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 5.1.4
    Article {
        icon: "⚡",
        title: "Acesso direto à memória (DMA)",
        section: "5.1.4",
        ostep: &[ostep::IO_DEVICES],
        summary: "Transferir dados sem ocupar a CPU com a cópia.",
        blocks: &[
            Block::P(
                "Quando a própria CPU copia os dados entre o dispositivo e a memória, palavra por \
                 palavra, ela gasta tempo em uma tarefa trivial. O acesso direto à memória, o \
                 DMA, tira esse trabalho da CPU.",
            ),
            Block::DmaDiagram,
            Block::P(
                "A CPU apenas configura o controlador DMA: diz onde os dados estão, para onde \
                 vão e quanto copiar. A partir daí ela segue com outro processo. O controlador \
                 move os dados sozinho e, ao terminar, gera uma interrupção para avisar.",
            ),
            Block::Note(
                "Exemplo: ao copiar 2 GB de um SSD para a RAM, a CPU só programa o controlador \
                 com a origem, o destino e o tamanho. O DMA faz a cópia inteira enquanto a CPU \
                 roda outros processos.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 5.2.1
    Article {
        icon: "🎯",
        title: "Objetivos do software de E/S",
        section: "5.2 e 5.2.1",
        ostep: &[ostep::IO_DEVICES],
        summary: "Independência de dispositivo, nomes uniformes, tratamento de erro.",
        blocks: &[
            Block::P(
                "O conceito central do software de entrada e saída é a independência do \
                 dispositivo. Um programa deve acessar qualquer dispositivo sem saber de antemão \
                 qual é.",
            ),
            Block::P(
                "Um programa que lê um arquivo deve funcionar igual quer o arquivo esteja num HD, \
                 num SSD ou num pen drive. Lidar com as diferenças fica a cargo do sistema \
                 operacional, não do programa.",
            ),
            Block::L(&[
                "A nomeação uniforme ajuda: o caminho absoluto identifica o recurso sem revelar \
                 em qual dispositivo ele está.",
                "O tratamento de erro acontece no nível mais baixo possível. O controlador tenta \
                 resolver primeiro e, se não consegue, o driver cuida do erro.",
                "As transferências podem ser síncronas, em que o processo espera bloqueado, ou \
                 assíncronas, orientadas a interrupção.",
            ]),
        ],
    },
    // ---------------------------------------------------------------- 5.2.2 a 5.2.4
    Article {
        icon: "⏱",
        title: "As três formas de fazer E/S",
        section: "5.2.2, 5.2.3 e 5.2.4",
        ostep: &[ostep::IO_DEVICES],
        summary: "Por programa, por interrupção e por DMA, lado a lado.",
        blocks: &[
            Block::P(
                "Há três formas de conduzir uma operação de entrada e saída. Elas diferem no que \
                 a CPU faz enquanto o dispositivo, que é lento, trabalha.",
            ),
            Block::Defs(&[
                (
                    "Por programa",
                    "a CPU faz todo o trabalho e fica em laço consultando o dispositivo até \
                     terminar. É simples, mas desperdiça a CPU na espera ativa.",
                ),
                (
                    "Por interrupção",
                    "a CPU inicia a operação e troca para outro processo. Quando o dispositivo \
                     termina, ele gera uma interrupção que traz a CPU de volta.",
                ),
                (
                    "Por DMA",
                    "a CPU só programa o controlador DMA, que faz a transferência sozinho e \
                     interrompe apenas no fim.",
                ),
            ]),
            Block::P(
                "Use o simulador abaixo para ver a diferença. Compare quanto da CPU cada forma \
                 aproveita enquanto a mesma operação de E/S acontece.",
            ),
            Block::IoModes,
            Block::Note(
                "A interrupção nem sempre vence. Para um dispositivo muito rápido, o custo de \
                 trocar de processo e tratar a interrupção pode passar do custo de só esperar.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 5.3
    Article {
        icon: "🧱",
        title: "As camadas do software de E/S",
        section: "5.3, 5.3.1 e 5.3.2",
        ostep: &[ostep::IO_DEVICES],
        summary: "Do tratador de interrupção ao programa do usuário.",
        blocks: &[
            Block::P(
                "O software de entrada e saída é organizado em camadas. Cada uma tem um papel bem \
                 definido e conversa só com as vizinhas. Isso é o que esconde os detalhes de cada \
                 dispositivo do resto do sistema.",
            ),
            Block::IoLayers,
            Block::H("Tratadores de interrupção"),
            Block::P(
                "São a camada mais baixa. O driver que iniciou a operação fica bloqueado até ela \
                 terminar. A interrupção do dispositivo acorda o driver, que então conclui o \
                 pedido.",
            ),
            Block::H("Drivers de dispositivo"),
            Block::P(
                "Cada classe de dispositivo tem um driver, o código que conhece em detalhe como \
                 aquele hardware funciona. O driver roda em modo núcleo e decide quais comandos, \
                 e em qual ordem, vão para o dispositivo.",
            ),
            Block::H("Software independente do dispositivo"),
            Block::P(
                "Acima dos drivers fica a parte comum a todos os dispositivos. Ela uniformiza a \
                 interface, cuida dos buffers, relata erros e aloca dispositivos. Para cada \
                 classe, o sistema define as funções que todo driver precisa oferecer.",
            ),
            Block::Note(
                "Os drivers são escritos por terceiros e são a maior parte do código do núcleo. \
                 Por isso os projetistas isolam a região dos drivers do resto do sistema.",
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
        check::articles(ARTICLES, '5');
    }
}
