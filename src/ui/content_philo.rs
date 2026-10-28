//! Conteúdo de "O Jantar dos Filósofos" (capítulo 6, seção 6.2).
//!
//! Vem das anotações em `Anotações/6 - impasses.md`. O segundo artigo embute o
//! simulador animado da mesa, com as três soluções das anotações.

use crate::ui::reader::{ostep, Article, Block};

pub const ARTICLES: &[Article] = &[
    // ---------------------------------------------------------------- 6.2.1 a 6.2.4
    Article {
        icon: "🍝",
        title: "O problema",
        section: "6.2",
        ostep: &[ostep::CONC_BUGS],
        summary: "Cinco filósofos, cinco hashis e um jantar que pode travar.",
        blocks: &[
            Block::P(
                "Dijkstra propôs o problema do jantar dos filósofos para ilustrar a dificuldade \
                 dos impasses. Ele é hoje o exemplo clássico da área e reúne, em uma cena \
                 simples, todos os ingredientes do bloqueio.",
            ),
            Block::P(
                "Cinco filósofos estão sentados em torno de uma mesa redonda. Diante de cada um \
                 há um prato de macarrão. Entre cada par de pratos há um único hashi, de modo que \
                 a mesa tem cinco hashis ao todo.",
            ),
            Block::P(
                "Cada filósofo alterna entre duas tarefas, pensar e comer. Para comer, ele \
                 precisa dos dois hashis vizinhos, o da esquerda e o da direita. Cada hashi é um \
                 recurso, e cada filósofo é um processo que disputa esse recurso com os vizinhos.",
            ),
            Block::Note(
                "Surge então a pergunta que interessa a este capítulo. É possível escrever um \
                 programa para os filósofos de modo que nenhum deles fique bloqueado à espera de \
                 um hashi?",
            ),
        ],
    },
    // ---------------------------------------------------------------- 6.2.5 a 6.2.8
    Article {
        icon: "🧪",
        title: "As três soluções",
        section: "6.2.5 e 6.2.8",
        ostep: &[ostep::CONC_BUGS],
        summary: "Da ingênua, que trava, à definitiva, que nunca bloqueia.",
        blocks: &[
            Block::P(
                "Experimente as três estratégias no simulador abaixo. A cor de cada filósofo diz \
                 o que ele faz, e os hashis em azul estão em uso.",
            ),
            Block::Defs(&[
                (
                    "Ingênua",
                    "cada filósofo pega o hashi da esquerda e espera o da direita sem soltá-lo. \
                     Se todos pegam a esquerda ao mesmo tempo, nenhum encontra a direita livre e \
                     o jantar trava. Esse bloqueio coletivo é o impasse.",
                ),
                (
                    "Com sorteio",
                    "um filósofo que não consegue o hashi da direita devolve o da esquerda e \
                     espera um tempo sorteado antes de recomeçar. Isso desfaz a maior parte dos \
                     bloqueios, mas ainda falha quando o sorteio entrega o mesmo tempo a dois \
                     vizinhos, e por isso não resolve o problema em definitivo.",
                ),
                (
                    "Definitiva",
                    "controla o estado de cada filósofo e só o deixa comer quando os dois \
                     vizinhos não estão comendo. Com ela, até dois filósofos comem ao mesmo \
                     tempo, e nenhum fica bloqueado.",
                ),
            ]),
            Block::Philosophers,
            Block::Note(
                "Repare que a ingênua satisfaz as quatro condições de Coffman ao mesmo tempo: \
                 cada hashi é exclusivo, cada filósofo segura um e espera outro, ninguém é \
                 forçado a soltar, e a espera fecha um círculo em torno da mesa. A definitiva \
                 quebra a posse e espera, pois toma os dois hashis de uma vez ou nenhum.",
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
