//! Conteúdo de "Arquivos e Diretórios" (seções 4.1 e 4.2).
//!
//! Vem das anotações em `Anotações/4 - Sistema de Arquivos.md` (Tanenbaum &
//! Bos, capítulo 4), com o OSTEP (capítulo 39, Files and Directories) como
//! leitura complementar e fonte dos exemplos práticos de UNIX. A implementação
//! no disco (4.3 e 4.4) fica na tela própria.

use crate::app::Screen;
use crate::ui::reader::{ostep, Article, Block};

pub const ARTICLES: &[Article] = &[
    // ---------------------------------------------------------------- 4.1.1
    Article {
        icon: "📄",
        title: "O que é um arquivo",
        section: "4.1 e 4.1.1",
        ostep: &[ostep::FILES_DIRS],
        summary: "A abstração que guarda informação além da vida do processo.",
        blocks: &[
            Block::P(
                "Todo programa precisa guardar e recuperar informação. A memória principal não \
                 serve para isso por três razões: é pequena perto do que um programa acumula, \
                 perde tudo quando o processo termina e não deixa vários processos chegarem ao \
                 mesmo dado ao mesmo tempo.",
            ),
            Block::P(
                "A resposta é guardar a informação em disco, em unidades lógicas chamadas \
                 arquivos. Um arquivo é uma sequência de bytes que sobrevive ao processo que o \
                 criou. O usuário lê e escreve sem precisar saber onde, no disco, os bytes estão.",
            ),
            Block::Defs(&[
                (
                    "arquivo",
                    "uma sequência linear de bytes com um nome, que persiste até alguém apagá-lo.",
                ),
                (
                    "sistema de arquivos",
                    "a parte do sistema operacional que cria, nomeia, protege e organiza os \
                     arquivos no disco.",
                ),
            ]),
            Block::H("O nome que o usuário vê e o nome de baixo nível"),
            Block::P(
                "Quando um processo cria um arquivo, dá a ele um nome legível. Por baixo, o \
                 sistema identifica o arquivo por um número, o número de i-node. É esse número, \
                 e não o nome, que o sistema usa para achar o arquivo no disco.",
            ),
            Block::Note(
                "As regras variam de sistema para sistema. É comum permitir nomes de até 255 \
                 caracteres. Alguns sistemas separam maiúsculas de minúsculas e outros não.",
            ),
            Block::SeeAlso(
                "Você pode ver o número de i-node de cada arquivo da sua máquina, ao vivo.",
                Screen::FileExplorer,
            ),
        ],
    },
    // ---------------------------------------------------------------- 4.1.2 e 4.1.3
    Article {
        icon: "🧱",
        title: "Estrutura e tipos de arquivo",
        section: "4.1.2 e 4.1.3",
        ostep: &[ostep::FILES_DIRS],
        summary: "Sequência de bytes, e os quatro tipos que o UNIX conhece.",
        blocks: &[
            Block::P("Há três formas clássicas de estruturar um arquivo por dentro:"),
            Block::Defs(&[
                (
                    "Sequência de bytes",
                    "o arquivo é só uma fila de bytes sem estrutura. Dar sentido a eles é tarefa \
                     do programa, não do sistema. É o modelo do UNIX e do Windows.",
                ),
                (
                    "Sequência de registros",
                    "o arquivo é uma fila de registros de tamanho fixo. Modelo legado dos \
                     primeiros computadores.",
                ),
                (
                    "Árvore de registros",
                    "registros de tamanho variável, ordenados por uma chave. Usado em \
                     computadores de grande porte.",
                ),
            ]),
            Block::P(
                "Este curso adota a visão do UNIX: o arquivo é uma sequência de bytes, e qualquer \
                 estrutura interna é imposta pelo programa que o lê.",
            ),
            Block::H("Os tipos de arquivo"),
            Block::P(
                "Tanto o UNIX quanto o Windows têm arquivos regulares e diretórios. O UNIX ainda \
                 tem dois tipos especiais, herança do fato de que quase tudo no sistema é tratado \
                 como arquivo:",
            ),
            Block::Defs(&[
                ("Arquivo regular", "o arquivo comum do usuário, com seus dados."),
                (
                    "Diretório",
                    "um arquivo especial que guarda a estrutura do sistema de arquivos.",
                ),
                (
                    "Especial de caractere",
                    "representa um dispositivo que fala em fluxo de bytes, como um terminal.",
                ),
                (
                    "Especial de bloco",
                    "representa um dispositivo endereçável por blocos, como um disco.",
                ),
            ]),
            Block::P(
                "O arquivo regular pode ser texto ASCII ou binário. O texto ASCII é legível e \
                 encaixa bem em uma sequência de comandos no terminal. O binário só faz sentido \
                 para o programa que conhece sua estrutura interna.",
            ),
            Block::SeeAlso(
                "Os arquivos especiais de bloco e de caractere têm uma tela só para eles.",
                Screen::DeviceBrowser,
            ),
        ],
    },
    // ---------------------------------------------------------------- 4.1.4 e 4.1.5
    Article {
        icon: "🏷",
        title: "Acesso e atributos",
        section: "4.1.4 e 4.1.5",
        ostep: &[ostep::FILES_DIRS],
        summary: "Leitura sequencial ou aleatória, e os metadados do arquivo.",
        blocks: &[
            Block::H("Acesso"),
            Block::P(
                "Nos primeiros sistemas o acesso era sequencial, porque o armazenamento era em \
                 fita. Para ler o fim do arquivo, era preciso passar por tudo antes. O disco \
                 magnético trouxe o acesso aleatório, que lê qualquer posição direto.",
            ),
            Block::P(
                "Um arquivo aberto guarda uma posição atual, que marca onde a próxima leitura ou \
                 escrita começa. Cada leitura avança essa posição. Para saltar direto a um ponto, \
                 o programa usa a chamada seek.",
            ),
            Block::H("Atributos"),
            Block::P(
                "Além dos dados, o sistema guarda informação sobre cada arquivo: nome, tamanho, \
                 dono, datas de acesso e de modificação, permissões e outras. Chamamos esses \
                 dados sobre o arquivo de atributos ou metadados.",
            ),
            Block::Note(
                "No UNIX, a chamada stat devolve esses metadados. No terminal, o comando stat \
                 mostra o número de i-node, o tamanho, os blocos alocados e a contagem de \
                 ligações do arquivo.",
            ),
            Block::SeeAlso(
                "O Explorador de Arquivos mostra o stat de cada arquivo que você selecionar.",
                Screen::FileExplorer,
            ),
        ],
    },
    // ---------------------------------------------------------------- 4.1.6
    Article {
        icon: "🧰",
        title: "Operações com arquivos",
        section: "4.1.6",
        ostep: &[ostep::FILES_DIRS],
        summary: "As chamadas de sistema que criam, leem e escrevem arquivos.",
        blocks: &[
            Block::P(
                "O sistema oferece um conjunto de chamadas para trabalhar com arquivos. As \
                 principais são:",
            ),
            Block::Defs(&[
                ("create", "cria um arquivo vazio."),
                ("delete", "remove o arquivo do disco."),
                ("open", "prepara o arquivo para uso e devolve um descritor."),
                ("close", "encerra o uso e libera o descritor."),
                ("read", "lê bytes a partir da posição atual."),
                ("write", "escreve bytes a partir da posição atual."),
                ("append", "escreve só no fim, uma forma restrita de write."),
                ("seek", "move a posição atual, para acesso aleatório."),
                ("get e set attributes", "lê e altera os metadados do arquivo."),
                ("rename", "troca o nome do arquivo."),
            ]),
            Block::H("Ler um arquivo, na prática"),
            Block::P(
                "Abrir um arquivo devolve um número inteiro, o descritor de arquivo. Todo processo \
                 já nasce com três descritores abertos: a entrada padrão (0), a saída padrão (1) \
                 e a saída de erro (2). O primeiro open costuma devolver o 3.",
            ),
            Block::Code(&[
                "$ strace cat foo        # Linux (no macOS, dtruss)",
                "open(\"foo\", O_RDONLY)        = 3",
                "read(3, \"hello\\n\", 4096)     = 6",
                "write(1, \"hello\\n\", 6)       = 6",
                "read(3, \"\", 4096)            = 0",
                "close(3)                     = 0",
            ]),
            Block::Note(
                "O read devolve quantos bytes leu. Quando devolve 0, não há mais o que ler: é o \
                 fim do arquivo. Ferramentas como strace (Linux) e dtruss (macOS) mostram essas \
                 chamadas ao vivo.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 4.2.1 e 4.2.2
    Article {
        icon: "🗂",
        title: "Diretórios",
        section: "4.2, 4.2.1 e 4.2.2",
        ostep: &[ostep::FILES_DIRS],
        summary: "De um diretório único à árvore de diretórios.",
        blocks: &[
            Block::P(
                "Um diretório é um arquivo especial que mantém o sistema de arquivos organizado. \
                 Ele guarda uma lista de pares. Cada par liga um nome legível ao número de i-node \
                 do arquivo ou diretório correspondente.",
            ),
            Block::H("Um único nível"),
            Block::P(
                "Os primeiros computadores tinham um só diretório, o diretório raiz, com todos os \
                 arquivos dentro. É simples, mas não escala. Ainda aparece em dispositivos \
                 embarcados bem pequenos.",
            ),
            Block::H("Hierarquia de diretórios"),
            Block::P(
                "Com milhões de arquivos, é preciso agrupar. A solução é deixar um diretório \
                 conter outros, formando uma árvore de diretórios. Além de organizar, a árvore \
                 permite que cada usuário tenha diretórios privados.",
            ),
            Block::Note(
                "No UNIX a árvore começa na raiz, escrita apenas /. Todo arquivo do sistema fica \
                 em algum ponto abaixo dela, mesmo os que estão em discos ou pen drives \
                 diferentes.",
            ),
            Block::SeeAlso(
                "Navegue pela árvore de diretórios real da sua máquina.",
                Screen::FileExplorer,
            ),
        ],
    },
    // ---------------------------------------------------------------- 4.2.3
    Article {
        icon: "🧭",
        title: "Nomes de caminho",
        section: "4.2.3",
        ostep: &[ostep::FILES_DIRS],
        summary: "Caminho absoluto e caminho relativo.",
        blocks: &[
            Block::P(
                "Para apontar um arquivo na árvore, usamos um nome de caminho, a sequência de \
                 diretórios do ponto de partida até o arquivo. Há duas formas de escrevê-lo:",
            ),
            Block::Defs(&[
                (
                    "Caminho absoluto",
                    "parte da raiz. No UNIX começa com /, como em /home/ana/nota.txt. Não depende \
                     de onde o processo está.",
                ),
                (
                    "Caminho relativo",
                    "parte do diretório de trabalho atual. docs/nota.txt resolve a partir de onde \
                     o processo está agora.",
                ),
            ]),
            Block::P(
                "Todo diretório tem duas entradas prontas. A entrada . aponta para ele mesmo e a \
                 entrada .. aponta para o diretório pai. Elas são o que torna possível subir e \
                 ficar no lugar com um caminho relativo.",
            ),
            Block::Note(
                "O UNIX e o Windows escrevem caminhos de formas diferentes, a começar pelo \
                 separador: barra no UNIX e barra invertida no Windows.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 4.2.4 e 4.3.4
    Article {
        icon: "🔗",
        title: "Operações com diretórios e ligações",
        section: "4.2.4 e 4.3.4",
        ostep: &[ostep::FILES_DIRS],
        summary: "Hard links e ligações simbólicas, com um i-node ou vários nomes.",
        blocks: &[
            Block::P("As chamadas para diretórios espelham as de arquivos:"),
            Block::Defs(&[
                ("create e delete", "criam e removem um diretório vazio."),
                ("opendir e closedir", "abrem e fecham um diretório para leitura."),
                ("readdir", "lê a próxima entrada, um par nome e i-node."),
                ("rename", "troca o nome do diretório."),
                ("link", "dá ao arquivo um segundo nome, no mesmo sistema de arquivos."),
                ("unlink", "remove um nome do arquivo."),
            ]),
            Block::H("Hard link: um i-node, vários nomes"),
            Block::P(
                "Criar um arquivo faz duas coisas: monta o i-node, que guarda tamanho, blocos e \
                 atributos, e liga um nome a esse i-node dentro de um diretório. A chamada link \
                 cria um segundo nome que aponta para o mesmo i-node. Não há cópia dos dados.",
            ),
            Block::P(
                "O i-node guarda uma contagem de ligações. A chamada unlink só remove um nome e \
                 diminui a contagem. O arquivo só é apagado de fato quando a contagem chega a \
                 zero. É por isso que a chamada se chama unlink, e não delete.",
            ),
            Block::Code(&[
                "$ echo oi > a            # cria o arquivo a",
                "$ ln a b                 # b passa a ser outro nome do mesmo i-node",
                "$ ls -i a b",
                "671580  a    671580  b   # o mesmo número de i-node",
                "$ rm a                   # some um nome, o i-node continua",
                "$ cat b",
                "oi",
            ]),
            Block::H("Ligação simbólica: um arquivo que guarda um caminho"),
            Block::P(
                "A ligação simbólica, criada com ln -s, é diferente. Ela é um arquivo próprio, de \
                 um terceiro tipo, cujo conteúdo é o caminho do alvo. Se o alvo é apagado, a \
                 ligação fica pendurada e apontando para um nome que não existe mais.",
            ),
            Block::SeeAlso(
                "Hard links aparecem com o mesmo i-node, em cor igual, no Explorador de Arquivos.",
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
