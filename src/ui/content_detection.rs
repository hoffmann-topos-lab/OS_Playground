//! Conteúdo de "Detecção e Recuperação" (capítulo 6, seção 6.5).
//!
//! Vem das anotações em `Anotações/6 - impasses.md`. O primeiro artigo embute o
//! grafo de alocação com detecção de ciclo; o segundo, a detecção por matrizes.

use crate::ui::reader::{ostep, Article, Block};

pub const ARTICLES: &[Article] = &[
    // ---------------------------------------------------------------- 6.5.2 a 6.5.5
    Article {
        icon: "🔎",
        title: "Um recurso de cada tipo",
        section: "6.5 e 6.5.2",
        ostep: &[ostep::CONC_BUGS],
        summary: "O grafo de alocação e a busca por um ciclo.",
        blocks: &[
            Block::P(
                "A segunda estratégia não tenta impedir o impasse. Ela deixa o bloqueio \
                 acontecer, mantém um mecanismo que o detecta e, uma vez detectado, recupera o \
                 sistema.",
            ),
            Block::P(
                "O caso mais simples é aquele em que existe um único recurso de cada tipo. A \
                 cada instante sabemos quais recursos cada processo detém e quais aguarda, e com \
                 isso construímos o grafo de alocação. Se o grafo contém um ciclo, há impasse. Se \
                 não contém, não há.",
            ),
            Block::P(
                "Resta então decidir se um grafo dirigido possui um ciclo. O algoritmo abaixo \
                 resolve essa questão. Ele usa uma lista L de nós e marca os arcos já \
                 percorridos. Para cada nó tomado como ponto de partida, executamos os passos \
                 seguintes.",
            ),
            Block::N(&[
                "Inicie L vazia e marque todos os arcos como não percorridos.",
                "Adicione o nó atual ao fim de L. Se o nó já aparecia em L, o grafo tem um ciclo \
                 e o algoritmo termina.",
                "A partir do nó atual, procure um arco de saída ainda não percorrido.",
                "Se existir, escolha um, marque-o, siga até o novo nó atual e volte ao passo 2.",
                "Se não existir e o nó atual é o de partida, não há ciclo a partir dele: retire-o \
                 e recomece com o próximo nó de partida.",
            ]),
            Block::P(
                "A ideia é construir, nó a nó, os caminhos que saem de cada ponto de partida. Um \
                 nó que reaparece em L fecha um caminho sobre si mesmo, e esse retorno é o ciclo \
                 que denuncia o impasse. No grafo abaixo você monta o cenário: ajuste quantos \
                 processos e recursos existem, marque em cada cruzamento quem detém e quem pede, \
                 ou parta de um exemplo pronto, e então revele a lista L passo a passo.",
            ),
            Block::ResourceGraph,
        ],
    },
    // ---------------------------------------------------------------- 6.5.6 a 6.5.15
    Article {
        icon: "🧮",
        title: "Vários recursos de cada tipo",
        section: "6.5.6 e 6.5.11",
        ostep: &[ostep::CONC_BUGS],
        summary: "Vetores, matrizes e a marcação dos processos que terminam.",
        blocks: &[
            Block::P(
                "Quando há várias unidades de um mesmo recurso, o ciclo no grafo deixa de ser \
                 prova de impasse, e precisamos de um método numérico. Descrevemos a situação \
                 com vetores e matrizes.",
            ),
            Block::Defs(&[
                (
                    "Existente E",
                    "quantas unidades de cada classe de recurso existem ao todo.",
                ),
                (
                    "Disponível A",
                    "quantas unidades de cada classe estão livres no momento.",
                ),
                (
                    "Alocação C",
                    "a linha i registra o que o processo Pi possui de cada classe.",
                ),
                (
                    "Requisição R",
                    "a linha i registra o que o processo Pi ainda pede de cada classe.",
                ),
            ]),
            Block::P(
                "As unidades não desaparecem, de modo que a soma do que está alocado em cada \
                 classe com o que está livre reproduz o total existente. Escrevemos A menor ou \
                 igual a B quando cada componente de A não supera o de B.",
            ),
            Block::P(
                "Cada processo começa não marcado. Marcamos os que conseguem terminar, pois um \
                 processo que termina devolve os recursos que detinha. Quando o algoritmo para, \
                 qualquer processo ainda não marcado está em impasse.",
            ),
            Block::N(&[
                "Procure um processo não marcado cuja linha em R seja menor ou igual a A.",
                "Se encontrar, some a sua linha de C ao vetor A, marque o processo e repita.",
                "Se nenhum processo satisfaz a condição, o algoritmo termina.",
            ]),
            Block::P(
                "O simulador abaixo já traz o exemplo das anotações, com três processos e quatro \
                 classes de recurso. Edite as matrizes e veja a marcação recalcular.",
            ),
            Block::DeadlockMatrix,
        ],
    },
    // ---------------------------------------------------------------- 6.5.16 a 6.5.19
    Article {
        icon: "🛟",
        title: "Recuperação de um impasse",
        section: "6.5.16",
        ostep: &[],
        summary: "Preempção, retrocesso e eliminação, nenhum confortável.",
        blocks: &[
            Block::P(
                "Detectado o impasse, o sistema precisa agir para desfazê-lo. Há três caminhos, \
                 e convém dizer desde já que nenhum deles é confortável.",
            ),
            Block::Defs(&[
                (
                    "Por preempção",
                    "retira um recurso de um processo e o entrega a outro, devolvendo-o depois. \
                     Depende muito da natureza do recurso, pois impressoras e scanners toleram \
                     mal uma retirada no meio do trabalho.",
                ),
                (
                    "Por retrocesso",
                    "apoia-se em pontos de controle salvos de tempos em tempos. Diante de um \
                     impasse, um processo retrocede até o ponto anterior à aquisição do recurso \
                     disputado.",
                ),
                (
                    "Por eliminação",
                    "encerra um ou mais processos do ciclo. É drástica, pois descarta todo o \
                     trabalho feito, mas tem a vantagem de ser sempre aplicável. Em geral \
                     escolhemos o processo cujo reinício custe menos.",
                ),
            ]),
            Block::Warn(
                "Toda recuperação paga um preço: um recurso retirado no momento errado, um \
                 trabalho refeito desde o ponto de controle, ou um processo morto com tudo o que \
                 havia calculado. Por isso evitar o impasse antes é tão atraente.",
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
