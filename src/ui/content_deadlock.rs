//! Conteúdo de "Impasses — Conceitos" (capítulo 6, seções 6.0, 6.1, 6.3, 6.4 e 6.7).
//!
//! Vem das anotações em `Anotações/6 - impasses.md` (Tanenbaum & Bos, *Sistemas
//! Operacionais Modernos*, capítulo 6), com o OSTEP como leitura complementar.
//! O jantar dos filósofos (6.2), a detecção e a recuperação (6.5) e como evitar
//! impasses (6.6) ficam em telas próprias deste módulo.

use crate::app::Screen;
use crate::ui::reader::{ostep, Article, Block};

pub const ARTICLES: &[Article] = &[
    // ---------------------------------------------------------------- 6.0
    Article {
        icon: "🧭",
        title: "O que é um impasse",
        section: "6.0",
        ostep: &[ostep::CONC_BUGS],
        summary: "O bloqueio permanente, generalizado para além da memória.",
        blocks: &[
            Block::P(
                "No capítulo sobre processos o impasse apareceu como um perigo da \
                 sincronização. Ao coordenar o acesso de dois processos a uma região de memória \
                 compartilhada, uma ordem errada das operações podia bloquear ambos para sempre. \
                 O exemplo mais claro foi o do produtor e consumidor: ao inverter a ordem das \
                 operações sobre o semáforo, o produtor tomava a trava do buffer e dormia logo \
                 em seguida, e o consumidor não conseguia mais entrar.",
            ),
            Block::P(
                "Aquele problema não é próprio da memória. Um bloqueio permanente pode surgir \
                 sempre que processos competem por um recurso, seja de software ou de hardware. \
                 Neste capítulo generalizamos a ideia e estudamos o fenômeno por si mesmo.",
            ),
            Block::Note(
                "Informalmente, um impasse é uma situação em que vários processos disputam um \
                 recurso e um mau controle do acesso deixa um ou mais deles bloqueados para \
                 sempre. Cada um espera por algo que está nas mãos de outro, e nenhum progride.",
            ),
            Block::P(
                "Vamos primeiro caracterizar os recursos e a forma como são adquiridos. Depois \
                 veremos o jantar dos filósofos, a definição formal e as condições para a \
                 ocorrência. Por fim, estudaremos as estratégias para lidar com o problema, da \
                 mais ingênua à mais rigorosa.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 6.1
    Article {
        icon: "🧰",
        title: "Recursos",
        section: "6.1",
        ostep: &[],
        summary: "O que é um recurso, e o perigo de pedi-los em ordem contrária.",
        blocks: &[
            Block::P(
                "A primeira classe de impasses envolve os recursos concedidos a um processo. Um \
                 recurso pode ser um dispositivo, um registro de dados, um arquivo ou uma região \
                 de memória.",
            ),
            Block::Note(
                "Um recurso é qualquer item que um processo precisa adquirir, utilizar e depois \
                 liberar, e que, enquanto está com um processo, não pode servir a outro.",
            ),
            Block::P(
                "Os recursos dividem-se em duas categorias, segundo a possibilidade de serem \
                 retomados à força de quem os detém.",
            ),
            Block::Defs(&[
                (
                    "Preemptível",
                    "pode ser retirado do processo sem causar-lhe prejuízo. A memória principal é \
                     o exemplo típico, pois o sistema pode mover o conteúdo para o disco e \
                     devolvê-lo depois.",
                ),
                (
                    "Não preemptível",
                    "não pode ser tomado do dono atual sem risco de falha. Um exemplo é o scanner \
                     no meio de uma digitalização, pois interromper o trabalho corromperia o \
                     resultado.",
                ),
            ]),
            Block::P(
                "A fronteira entre as duas categorias depende do contexto. Os impasses que vamos \
                 estudar envolvem sempre recursos não preemptíveis, pois são eles que não \
                 admitem uma retirada sem prejuízo.",
            ),
            Block::H("A sequência de uso"),
            Block::N(&[
                "O processo solicita o recurso.",
                "O processo utiliza o recurso.",
                "O processo libera o recurso.",
            ]),
            Block::P(
                "Quando o recurso não está disponível na solicitação, o processo precisa \
                 esperar. Para alguns recursos, como os registros de um banco de dados, cabe aos \
                 próprios processos gerenciar o acesso, associando um semáforo a cada recurso: \
                 cada processo executa um down antes de usar e um up ao terminar.",
            ),
            Block::Warn(
                "O perigo aparece quando dois processos precisam de dois recursos e os solicitam \
                 em ordem contrária. Se A pega o recurso 1 e espera o 2, enquanto B pega o \
                 recurso 2 e espera o 1, nenhum dos dois libera o que tem, e ambos esperam para \
                 sempre.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 6.3.1 a 6.3.5
    Article {
        icon: "📜",
        title: "A definição formal",
        section: "6.3 e 6.3.4",
        ostep: &[ostep::CONC_BUGS],
        summary: "O impasse sem depender de um exemplo, e as quatro condições de Coffman.",
        blocks: &[
            Block::P(
                "O jantar dos filósofos dá a intuição do problema. Podemos agora enunciar o \
                 impasse de forma rigorosa, sem depender de um exemplo particular.",
            ),
            Block::Note(
                "Um conjunto de processos está em impasse (deadlock) quando cada processo do \
                 conjunto espera por um evento que apenas outro processo do mesmo conjunto pode \
                 causar.",
            ),
            Block::P(
                "Como todos esperam, nenhum deles causa o evento que os demais aguardam. A \
                 espera é perpétua. Em geral o evento aguardado é a liberação de um recurso, e \
                 por isso tratamos aqui do impasse de recursos.",
            ),
            Block::H("As quatro condições de Coffman"),
            Block::P(
                "Coffman identificou quatro condições que precisam valer ao mesmo tempo para um \
                 impasse de recursos existir.",
            ),
            Block::Defs(&[
                (
                    "Exclusão mútua",
                    "cada recurso está concedido a exatamente um processo ou está livre, nunca \
                     partilhado por dois ao mesmo tempo.",
                ),
                (
                    "Posse e espera",
                    "um processo que já detém recursos pode solicitar novos sem abrir mão dos que \
                     possui.",
                ),
                (
                    "Não preempção",
                    "um recurso concedido não pode ser tomado à força. Apenas o processo que o \
                     detém pode liberá-lo.",
                ),
                (
                    "Espera circular",
                    "existe uma cadeia circular de dois ou mais processos, em que cada um espera \
                     por um recurso que está com o próximo da cadeia.",
                ),
            ]),
            Block::Note(
                "As quatro são necessárias em conjunto. Se qualquer uma falhar, o impasse de \
                 recursos não ocorre. Esse fato é a base de toda a prevenção estudada ao final do \
                 capítulo.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 6.3.6 a 6.3.10
    Article {
        icon: "🕸",
        title: "Modelagem por grafos",
        section: "6.3.6 e 6.3.8",
        ostep: &[ostep::CONC_BUGS],
        summary: "Processos, recursos e o ciclo que denuncia o impasse.",
        blocks: &[
            Block::P(
                "Holt mostrou que as quatro condições podem ser representadas por grafos \
                 dirigidos. O grafo tem dois tipos de nós: os processos, desenhados como \
                 círculos, e os recursos, desenhados como quadrados.",
            ),
            Block::Defs(&[
                (
                    "Recurso ➡ processo",
                    "indica posse. O recurso foi solicitado, concedido e está no momento com \
                     aquele processo.",
                ),
                (
                    "Processo ➡ recurso",
                    "indica espera. O processo está bloqueado e aguarda a liberação daquele \
                     recurso.",
                ),
            ]),
            Block::P(
                "Com essa notação, o impasse ganha uma forma visível. Há impasse quando o grafo \
                 contém um ciclo que passa por ao menos dois processos e dois recursos. O grafo \
                 abaixo desenha o bloqueio dos dois processos que pedem dois recursos em ordem \
                 contrária.",
            ),
            Block::RagDiagram,
            Block::P(
                "O ciclo percorre A, R2, B, R1 e volta a A, e confirma o impasse. Essa leitura \
                 do grafo também fornece um algoritmo para detectar o problema depois que ele \
                 ocorre.",
            ),
            Block::H("As quatro estratégias"),
            Block::P(
                "Diante do impasse, o sistema operacional pode adotar quatro estratégias, que \
                 organizam o resto do capítulo.",
            ),
            Block::N(&[
                "Ignorar o problema e fingir que ele não existe.",
                "Deixar o impasse ocorrer, detectá-lo e recuperar o sistema.",
                "Evitar o impasse de forma dinâmica, por uma alocação cuidadosa dos recursos.",
                "Prevenir o impasse, negando de forma estrutural uma das quatro condições.",
            ]),
            Block::SeeAlso(
                "A detecção e a recuperação têm tela própria, com os dois algoritmos.",
                Screen::DeadlockDetection,
            ),
            Block::SeeAlso(
                "Como evitar impasses, com o algoritmo do banqueiro, também tem tela própria.",
                Screen::DeadlockAvoidance,
            ),
        ],
    },
    // ---------------------------------------------------------------- 6.4
    Article {
        icon: "🙈",
        title: "O algoritmo do avestruz",
        section: "6.4",
        ostep: &[],
        summary: "A estratégia de não fazer nada, e quando ela é razoável.",
        blocks: &[
            Block::P(
                "A primeira estratégia é a mais simples de todas, que é não fazer nada. Ela \
                 recebe o nome de algoritmo do avestruz, em alusão à ave que, segundo a lenda, \
                 enfia a cabeça na areia para não ver o perigo.",
            ),
            Block::P(
                "A ideia parece irresponsável, mas tem o seu lugar. Se os impasses ocorrem muito \
                 raramente, o custo de preveni-los pode não compensar. O projetista aceita o \
                 risco e trata o impasse como mais um dos erros improváveis do sistema.",
            ),
            Block::Note(
                "A escolha depende de uma comparação honesta. O avestruz funciona quando a \
                 frequência de impasses é baixa em termos absolutos, ou baixa diante dos outros \
                 defeitos que o sistema já tolera. Muitos sistemas de uso geral adotam essa \
                 postura.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 6.7
    Article {
        icon: "🧱",
        title: "Prevenção de impasses",
        section: "6.7",
        ostep: &[ostep::CONC_BUGS],
        summary: "Negar de forma estrutural uma das quatro condições.",
        blocks: &[
            Block::P(
                "A quarta e última estratégia é a mais radical. Evitar o impasse exige conhecer \
                 de antemão os pedidos de cada processo, o que quase nunca é possível. A \
                 prevenção abre mão desse conhecimento e ataca o problema pela raiz.",
            ),
            Block::P(
                "A ideia apoia-se nas condições de Coffman. Como as quatro precisam valer ao \
                 mesmo tempo, basta impedir uma delas para tornar o impasse estruturalmente \
                 impossível. Em vez de vigiar cada alocação, o sistema é projetado de modo que \
                 uma das condições nunca se satisfaça.",
            ),
            Block::Table2(
                "Condição atacada",
                "Como atacá-la",
                &[
                    (
                        "Exclusão mútua",
                        "evitar conceder um recurso quando não é indispensável, por exemplo com \
                         spooling, que deixa um único processo tocar o dispositivo.",
                    ),
                    (
                        "Posse e espera",
                        "exigir que o processo peça todos os recursos de uma vez, antes de \
                         começar.",
                    ),
                    (
                        "Não preempção",
                        "permitir que o sistema retire um recurso, quando a sua natureza admite.",
                    ),
                    (
                        "Espera circular",
                        "impor uma ordem fixa de solicitação, de modo que a cadeia nunca feche um \
                         círculo.",
                    ),
                ],
            ),
            Block::P(
                "O preço é sempre uma perda de flexibilidade, e cabe ao projetista julgar se ele \
                 compensa.",
            ),
        ],
    },
];

#[cfg(test)]
mod tests {
    use crate::ui::reader::check;

    #[test]
    fn conteudo_valido() {
        check::articles(super::ARTICLES, '6');
    }
}
