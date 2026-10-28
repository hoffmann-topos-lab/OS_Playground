//! Conteúdo de "Como Evitar Impasses" (capítulo 6, seção 6.6).
//!
//! Vem das anotações em `Anotações/6 - impasses.md`. O segundo artigo embute o
//! algoritmo do banqueiro; o terceiro, a figura da trajetória de recursos.

use crate::ui::reader::{ostep, Article, Block};

pub const ARTICLES: &[Article] = &[
    // ---------------------------------------------------------------- 6.6.1 a 6.6.6
    Article {
        icon: "🛡",
        title: "Estados seguros e inseguros",
        section: "6.6 e 6.6.4",
        ostep: &[ostep::CONC_BUGS],
        summary: "A noção que permite evitar o impasse durante a alocação.",
        blocks: &[
            Block::P(
                "A terceira estratégia é mais ambiciosa. Em vez de deixar o impasse ocorrer, ela \
                 o evita durante a própria alocação. Na prática, os processos não pedem todos os \
                 recursos de uma vez, e o sistema decide, a cada pedido, se a concessão é \
                 prudente.",
            ),
            Block::P(
                "Existe um algoritmo capaz de conceder recursos sempre de forma segura, desde \
                 que o sistema conheça de antemão o número máximo de recursos que cada processo \
                 poderá pedir. A chave é a noção de estado seguro.",
            ),
            Block::Defs(&[
                (
                    "Estado seguro",
                    "existe alguma ordem de escalonamento que leva todos os processos até o fim, \
                     ainda que cada um peça de uma só vez o seu número máximo de recursos.",
                ),
                (
                    "Estado inseguro",
                    "não oferece essa garantia. Ainda pode terminar bem se os processos tiverem \
                     sorte na ordem dos pedidos, mas o sistema não pode contar com isso.",
                ),
            ]),
            Block::Note(
                "Inseguro não é o mesmo que em impasse. Um estado inseguro é apenas um estado em \
                 que o sistema perdeu a garantia de escapar, não um bloqueio já consumado.",
            ),
            Block::P(
                "A regra da estratégia é simples de enunciar. O sistema só concede um recurso \
                 quando o estado resultante continua seguro. Qualquer pedido que levaria a um \
                 estado inseguro é adiado, mesmo que o recurso esteja livre.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 6.6.7 a 6.6.11
    Article {
        icon: "🏦",
        title: "O algoritmo do banqueiro",
        section: "6.6.7",
        ostep: &[ostep::CONC_BUGS],
        summary: "Conceder crédito só enquanto o estado continua seguro.",
        blocks: &[
            Block::P(
                "Dijkstra deu forma a essa regra no algoritmo do banqueiro, assim chamado por \
                 imitar um banqueiro que concede crédito a clientes. O banqueiro conhece o limite \
                 de crédito de cada cliente, mas não guarda dinheiro para honrar todos os limites \
                 ao mesmo tempo.",
            ),
            Block::P(
                "Antes de atender um pedido, o banqueiro verifica se a entrega do dinheiro deixa \
                 o estado seguro. Se continua seguro, ele libera o valor. Se o estado se torna \
                 inseguro, ele faz o cliente esperar, ainda que tenha caixa para pagar naquele \
                 instante.",
            ),
            Block::P(
                "A verificação de segurança repete a busca por linha que vimos na detecção com \
                 vários recursos, só que a demanda de cada processo é a necessidade que ainda \
                 falta, ou seja, o máximo menos o já alocado. O simulador abaixo decide se o \
                 estado é seguro e, quando é, mostra a ordem que o conclui.",
            ),
            Block::Banker,
            Block::Warn(
                "O algoritmo é correto, mas supõe que o número máximo de recursos de cada \
                 processo é conhecido de antemão. Como esse número quase nunca está disponível na \
                 prática, o algoritmo do banqueiro é pouco utilizado nos sistemas reais.",
            ),
        ],
    },
    // ---------------------------------------------------------------- 6.6.12 a 6.6.15
    Article {
        icon: "📈",
        title: "A trajetória de recursos",
        section: "6.6.12",
        ostep: &[],
        summary: "Uma leitura visual do estado seguro, para dois processos.",
        blocks: &[
            Block::P(
                "A noção de estado seguro ganha uma leitura visual na trajetória de recursos. \
                 Ela se aplica a dois processos que disputam dois recursos e exige que \
                 conheçamos todas as instruções de cada processo.",
            ),
            Block::P(
                "O eixo horizontal traz o avanço do processo A e o vertical, o avanço do \
                 processo B. Um ponto do gráfico representa o progresso conjunto dos dois.",
            ),
            Block::Trajectory,
            Block::P(
                "As regiões em cinza marcam os instantes em que os dois processos precisariam do \
                 mesmo recurso ao mesmo tempo. Como cada recurso é único, essas regiões são \
                 proibidas, e a trajetória nunca pode entrar nelas.",
            ),
            Block::Note(
                "Há ainda uma área menor, encravada junto às regiões proibidas, que é insegura. \
                 Uma trajetória que entra nela fica encurralada e caminha para o impasse sem \
                 saída. Cabe ao escalonador suspender um dos processos a tempo e desviar a \
                 trajetória para fora dela, como faz a linha em destaque.",
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
