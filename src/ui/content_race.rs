//! Conteúdo da tela "Race Conditions" — seção 2.4 das anotações.
//!
//! Cada algoritmo de exclusão mútua ganha uma explicação curta (o detalhamento
//! é em sala) e uma demonstração passo a passo do entrelaçamento que revela o
//! problema — ou que mostra por que a solução resiste a ele.

use crate::app::Screen;
use crate::ui::demo::{Demo, Frame, Outcome};
use crate::ui::reader::{ostep, Article, Block};

// ---------------------------------------------------------------------------
// Roteiros das demonstrações
// ---------------------------------------------------------------------------

/// A race condition clássica: duas threads incrementam o mesmo contador.
static LOST_UPDATE: Demo = Demo {
    id: "lost_update",
    actors: ["Thread A", "Thread B"],
    code: [
        &["reg = counter", "reg = reg + 1", "counter = reg"],
        &["reg = counter", "reg = reg + 1", "counter = reg"],
    ],
    frames: &[
        Frame {
            pc: [Some(0), None],
            vars: &[("counter", "0"), ("regA", "–"), ("regB", "–")],
            caption: "A lê o contador compartilhado e guarda 0 no seu registrador.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [Some(0), None],
            vars: &[("counter", "0"), ("regA", "0"), ("regB", "–")],
            caption: "regA vale 0. Até aqui, nada de errado.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [None, Some(0)],
            vars: &[("counter", "0"), ("regA", "0"), ("regB", "–")],
            caption: "O escalonador troca de processo bem no meio da operação de A.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [None, Some(0)],
            vars: &[("counter", "0"), ("regA", "0"), ("regB", "0")],
            caption: "B lê o mesmo contador e também vê 0 — o incremento de A ainda não \
                      foi escrito.",
            in_cs: [false, false],
            danger: true,
        },
        Frame {
            pc: [None, Some(1)],
            vars: &[("counter", "0"), ("regA", "0"), ("regB", "1")],
            caption: "B calcula 0 + 1 = 1.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [None, Some(2)],
            vars: &[("counter", "1"), ("regA", "0"), ("regB", "1")],
            caption: "B escreve 1 no contador e termina sua parte.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [Some(1), None],
            vars: &[("counter", "1"), ("regA", "1"), ("regB", "1")],
            caption: "A volta do ponto onde parou e calcula 0 + 1 = 1, usando o valor \
                      velho que tinha guardado.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [Some(2), None],
            vars: &[("counter", "1"), ("regA", "1"), ("regB", "1")],
            caption: "A escreve 1 por cima do 1 que B já tinha escrito.",
            in_cs: [false, false],
            danger: true,
        },
    ],
    outcome: Outcome::Breaks(
        "Dois incrementos, resultado 1 — o esperado era 2. Uma das atualizações se perdeu.",
    ),
};

/// Variável de trava: a mesma corrida, um nível acima.
static LOCK_VARIABLE: Demo = Demo {
    id: "lock_variable",
    actors: ["Processo A", "Processo B"],
    code: [
        &[
            "while (lock == 1) ;",
            "lock = 1",
            "  região crítica",
            "lock = 0",
        ],
        &[
            "while (lock == 1) ;",
            "lock = 1",
            "  região crítica",
            "lock = 0",
        ],
    ],
    frames: &[
        Frame {
            pc: [Some(0), None],
            vars: &[("lock", "0")],
            caption: "A testa a trava e encontra 0: o caminho está livre.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [None, Some(0)],
            vars: &[("lock", "0")],
            caption: "O escalonador troca de processo entre o teste de A e a escrita dele.",
            in_cs: [false, false],
            danger: true,
        },
        Frame {
            pc: [None, Some(1)],
            vars: &[("lock", "1")],
            caption: "B também encontrou 0, e agora escreve 1 na trava.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [None, Some(2)],
            vars: &[("lock", "1")],
            caption: "B entra na região crítica, convencido de que a trava é dele.",
            in_cs: [false, true],
            danger: false,
        },
        Frame {
            pc: [Some(1), Some(2)],
            vars: &[("lock", "1")],
            caption: "A retoma do ponto onde parou — ele já tinha testado a trava — e \
                      escreve 1 nela.",
            in_cs: [false, true],
            danger: false,
        },
        Frame {
            pc: [Some(2), Some(2)],
            vars: &[("lock", "1")],
            caption: "A entra na região crítica. Os dois estão dentro ao mesmo tempo.",
            in_cs: [true, true],
            danger: true,
        },
    ],
    outcome: Outcome::Breaks(
        "A trava só empurrou o problema: testar e escrever são dois passos, e a corrida \
         voltou a acontecer entre eles.",
    ),
};

/// Alternância explícita: funciona, mas viola a terceira condição.
static STRICT_ALTERNATION: Demo = Demo {
    id: "strict_alternation",
    actors: ["Processo 0", "Processo 1"],
    code: [
        &[
            "while (turn != 0) ;",
            "  região crítica",
            "turn = 1",
            "  região não-crítica",
        ],
        &[
            "while (turn != 1) ;",
            "  região crítica",
            "turn = 0",
            "  região não-crítica",
        ],
    ],
    frames: &[
        Frame {
            pc: [Some(0), Some(0)],
            vars: &[("turn", "0")],
            caption: "turn vale 0. O processo 0 passa direto; o 1 fica girando no teste.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [Some(1), Some(0)],
            vars: &[("turn", "0")],
            caption: "O processo 0 entra na região crítica. O 1 continua em espera ocupada, \
                      queimando CPU.",
            in_cs: [true, false],
            danger: false,
        },
        Frame {
            pc: [Some(2), Some(0)],
            vars: &[("turn", "1")],
            caption: "O processo 0 sai e passa a vez: turn = 1.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [Some(3), Some(1)],
            vars: &[("turn", "1")],
            caption: "Agora o 1 entra, e o 0 vai cuidar da sua região não-crítica. \
                      Até aqui a exclusão mútua se sustenta.",
            in_cs: [false, true],
            danger: false,
        },
        Frame {
            pc: [Some(0), Some(3)],
            vars: &[("turn", "0")],
            caption: "O 1 saiu e devolveu a vez. O processo 0, que é rápido, já terminou \
                      sua parte não-crítica e quer entrar de novo.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [Some(1), Some(3)],
            vars: &[("turn", "0")],
            caption: "O 0 entra pela segunda vez e sai. Mas agora ele quer uma terceira, \
                      e turn está com 1.",
            in_cs: [true, false],
            danger: false,
        },
        Frame {
            pc: [Some(0), Some(3)],
            vars: &[("turn", "1")],
            caption: "O processo 0 fica travado esperando, mesmo com a região crítica \
                      livre — porque o 1 está ocupado fora dela e não devolveu a vez.",
            in_cs: [false, false],
            danger: true,
        },
    ],
    outcome: Outcome::Breaks(
        "A exclusão mútua vale, mas a condição 3 foi violada: um processo fora da região \
         crítica bloqueou o outro.",
    ),
};

/// Solução de Peterson: combina a variável de vez com o sinal de interesse.
static PETERSON: Demo = Demo {
    id: "peterson",
    actors: ["Processo 0", "Processo 1"],
    code: [
        &[
            "interested[0] = true",
            "turn = 0",
            "while (turn == 0 &&",
            "   interested[1]) ;",
            "  região crítica",
            "interested[0] = false",
        ],
        &[
            "interested[1] = true",
            "turn = 1",
            "while (turn == 1 &&",
            "   interested[0]) ;",
            "  região crítica",
            "interested[1] = false",
        ],
    ],
    frames: &[
        Frame {
            pc: [Some(0), None],
            vars: &[("interested", "[F, F]"), ("turn", "–")],
            caption: "O processo 0 chama enter_region e declara que está interessado.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [Some(1), None],
            vars: &[("interested", "[V, F]"), ("turn", "–")],
            caption: "Interesse declarado. Agora ele escreve o próprio número em turn.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [Some(2), Some(0)],
            vars: &[("interested", "[V, F]"), ("turn", "0")],
            caption: "Bem aqui o escalonador troca: o processo 1 também quer entrar.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [Some(2), Some(1)],
            vars: &[("interested", "[V, V]"), ("turn", "0")],
            caption: "Os dois estão interessados ao mesmo tempo — exatamente o caso que \
                      derrubou a variável de trava.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [Some(2), Some(2)],
            vars: &[("interested", "[V, V]"), ("turn", "1")],
            caption: "O processo 1 escreve 1 em turn, sobrescrevendo o 0. É essa \
                      sobrescrita que desempata.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [Some(2), Some(3)],
            vars: &[("interested", "[V, V]"), ("turn", "1")],
            caption: "O 1 testa: turn == 1 e o 0 está interessado — então ele espera.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [Some(4), Some(3)],
            vars: &[("interested", "[V, V]"), ("turn", "1")],
            caption: "O 0 testa: turn não é mais 0, então ele passa e entra. Só um entrou.",
            in_cs: [true, false],
            danger: false,
        },
        Frame {
            pc: [Some(5), Some(3)],
            vars: &[("interested", "[V, V]"), ("turn", "1")],
            caption: "O 0 termina e chama leave_region.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [None, Some(4)],
            vars: &[("interested", "[F, V]"), ("turn", "1")],
            caption: "Com interested[0] em falso, o teste do 1 falha e ele entra por sua vez.",
            in_cs: [false, true],
            danger: false,
        },
    ],
    outcome: Outcome::Holds(
        "Quem escreve por último em turn cede a vez. Um entra, o outro espera, e nenhum \
         processo fora da região crítica bloqueia o outro.",
    ),
};

/// TSL: o teste e a escrita viram uma operação atômica, em hardware.
static TSL: Demo = Demo {
    id: "tsl",
    actors: ["Processo A", "Processo B"],
    code: [
        &[
            "TSL RX, LOCK",
            "CMP RX, 0",
            "JNE volta ao início",
            "  região crítica",
            "MOVE LOCK, 0",
        ],
        &[
            "TSL RX, LOCK",
            "CMP RX, 0",
            "JNE volta ao início",
            "  região crítica",
            "MOVE LOCK, 0",
        ],
    ],
    frames: &[
        Frame {
            pc: [Some(0), None],
            vars: &[("LOCK", "0"), ("RX de A", "–"), ("RX de B", "–")],
            caption: "A executa TSL: numa única instrução indivisível, ela lê LOCK para RX \
                      e escreve um valor diferente de zero em LOCK.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [Some(1), None],
            vars: &[("LOCK", "1"), ("RX de A", "0"), ("RX de B", "–")],
            caption: "RX ficou com o valor antigo (0) e LOCK já está em 1. Ninguém \
                      conseguiu se meter no meio: o barramento fica travado durante a TSL.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [Some(3), Some(0)],
            vars: &[("LOCK", "1"), ("RX de A", "0"), ("RX de B", "–")],
            caption: "A viu 0 no seu RX, então a trava era dele: entra na região crítica. \
                      B chega e executa sua TSL.",
            in_cs: [true, false],
            danger: false,
        },
        Frame {
            pc: [Some(3), Some(1)],
            vars: &[("LOCK", "1"), ("RX de A", "0"), ("RX de B", "1")],
            caption: "O RX de B veio com 1: a trava já estava tomada. LOCK continua 1.",
            in_cs: [true, false],
            danger: false,
        },
        Frame {
            pc: [Some(3), Some(2)],
            vars: &[("LOCK", "1"), ("RX de A", "0"), ("RX de B", "1")],
            caption: "B volta ao início do laço e tenta de novo — espera ocupada.",
            in_cs: [true, false],
            danger: false,
        },
        Frame {
            pc: [Some(4), Some(0)],
            vars: &[("LOCK", "0"), ("RX de A", "0"), ("RX de B", "1")],
            caption: "A sai e zera LOCK. B tenta a TSL mais uma vez.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [None, Some(3)],
            vars: &[("LOCK", "1"), ("RX de A", "–"), ("RX de B", "0")],
            caption: "Agora o RX de B veio com 0: a trava é dele, e ele entra.",
            in_cs: [false, true],
            danger: false,
        },
    ],
    outcome: Outcome::Holds(
        "Como ler e escrever a trava viraram um passo só, não existe mais brecha entre o \
         teste e a escrita. Continua sendo espera ocupada, isso sim.",
    ),
};

/// Produtor-consumidor com sleep/wakeup: o despertar perdido.
static LOST_WAKEUP: Demo = Demo {
    id: "lost_wakeup",
    actors: ["Produtor", "Consumidor"],
    code: [
        &[
            "insere item",
            "count = count + 1",
            "if (count == 1)",
            "   wakeup(consumidor)",
        ],
        &[
            "if (count == 0)",
            "   sleep()",
            "retira item",
            "count = count - 1",
        ],
    ],
    frames: &[
        Frame {
            pc: [None, Some(0)],
            vars: &[("count", "0"), ("buffer", "vazio")],
            caption: "O buffer está vazio. O consumidor testa count e encontra 0.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [Some(0), Some(0)],
            vars: &[("count", "0"), ("buffer", "vazio")],
            caption: "O escalonador troca de processo entre o teste do consumidor e o \
                      sleep dele.",
            in_cs: [false, false],
            danger: true,
        },
        Frame {
            pc: [Some(1), None],
            vars: &[("count", "0"), ("buffer", "1 item")],
            caption: "O produtor insere um item no buffer.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [Some(2), None],
            vars: &[("count", "1"), ("buffer", "1 item")],
            caption: "count vai a 1. O produtor verifica: como era 0, o consumidor deve \
                      estar dormindo.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [Some(3), None],
            vars: &[("count", "1"), ("buffer", "1 item")],
            caption: "O produtor manda um wakeup — mas o consumidor ainda não dormiu. \
                      O sinal se perde no ar.",
            in_cs: [false, false],
            danger: true,
        },
        Frame {
            pc: [None, Some(1)],
            vars: &[("count", "1"), ("buffer", "1 item")],
            caption: "Só agora o consumidor executa o sleep, baseado no teste velho que \
                      tinha feito lá atrás.",
            in_cs: [false, false],
            danger: true,
        },
        Frame {
            pc: [None, None],
            vars: &[("count", "1"), ("buffer", "1 item")],
            caption: "O consumidor dorme com o buffer cheio, e o despertar que o \
                      salvaria já passou.",
            in_cs: [false, false],
            danger: true,
        },
    ],
    outcome: Outcome::Breaks(
        "Despertar perdido: o produtor vai encher o buffer e dormir também. Os dois ficam \
         parados para sempre.",
    ),
};

/// Semáforos: o down/up resolve o despertar perdido.
static SEMAPHORE: Demo = Demo {
    id: "semaphore",
    actors: ["Produtor", "Consumidor"],
    code: [
        &[
            "down(&vazias)",
            "insere item",
            "up(&cheias)",
        ],
        &[
            "down(&cheias)",
            "retira item",
            "up(&vazias)",
        ],
    ],
    frames: &[
        Frame {
            pc: [None, Some(0)],
            vars: &[("cheias", "0"), ("vazias", "4"), ("buffer", "vazio")],
            caption: "O consumidor chama down em 'cheias', que vale 0.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [None, None],
            vars: &[("cheias", "0"), ("vazias", "4"), ("buffer", "vazio")],
            caption: "Como o semáforo é 0, o down não se completa e o consumidor é posto \
                      para dormir — tudo isso numa operação atômica.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [Some(0), None],
            vars: &[("cheias", "0"), ("vazias", "3"), ("buffer", "vazio")],
            caption: "O produtor faz down em 'vazias', que era 4: decrementa e segue.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [Some(1), None],
            vars: &[("cheias", "0"), ("vazias", "3"), ("buffer", "1 item")],
            caption: "O produtor insere o item.",
            in_cs: [true, false],
            danger: false,
        },
        Frame {
            pc: [Some(2), None],
            vars: &[("cheias", "1"), ("vazias", "3"), ("buffer", "1 item")],
            caption: "up em 'cheias'. Como havia alguém dormindo nesse semáforo, ele é \
                      escolhido para acordar.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [None, Some(0)],
            vars: &[("cheias", "0"), ("vazias", "3"), ("buffer", "1 item")],
            caption: "O consumidor acorda e completa o down que tinha ficado pendente.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [None, Some(1)],
            vars: &[("cheias", "0"), ("vazias", "3"), ("buffer", "vazio")],
            caption: "Ele retira o item.",
            in_cs: [false, true],
            danger: false,
        },
        Frame {
            pc: [None, Some(2)],
            vars: &[("cheias", "0"), ("vazias", "4"), ("buffer", "vazio")],
            caption: "up em 'vazias', devolvendo o espaço ao produtor.",
            in_cs: [false, false],
            danger: false,
        },
    ],
    outcome: Outcome::Holds(
        "O semáforo conta os sinais em vez de deixá-los no ar. Um up dado antes do down \
         não se perde: fica guardado no contador.",
    ),
};

/// Mutex: a versão binária, sem contador.
static MUTEX: Demo = Demo {
    id: "mutex",
    actors: ["Thread A", "Thread B"],
    code: [
        &[
            "mutex_lock(&m)",
            "  região crítica",
            "mutex_unlock(&m)",
        ],
        &[
            "mutex_lock(&m)",
            "  região crítica",
            "mutex_unlock(&m)",
        ],
    ],
    frames: &[
        Frame {
            pc: [Some(0), None],
            vars: &[("m", "destravado")],
            caption: "A chama mutex_lock com o mutex destravado.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [Some(1), None],
            vars: &[("m", "travado")],
            caption: "O mutex passa a travado e A entra na região crítica.",
            in_cs: [true, false],
            danger: false,
        },
        Frame {
            pc: [Some(1), Some(0)],
            vars: &[("m", "travado")],
            caption: "B chama mutex_lock, mas o mutex já está travado.",
            in_cs: [true, false],
            danger: false,
        },
        Frame {
            pc: [Some(1), None],
            vars: &[("m", "travado")],
            caption: "B é bloqueado — não fica girando em espera ocupada, sai da fila da CPU.",
            in_cs: [true, false],
            danger: false,
        },
        Frame {
            pc: [Some(2), None],
            vars: &[("m", "destravado")],
            caption: "A termina e chama mutex_unlock.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [None, Some(1)],
            vars: &[("m", "travado")],
            caption: "Com o mutex livre, B é desbloqueado, toma a trava e entra. Se \
                      houvesse várias threads esperando, uma seria escolhida ao acaso.",
            in_cs: [false, true],
            danger: false,
        },
    ],
    outcome: Outcome::Holds(
        "Duas posições só, travado e destravado — sem contador. É o suficiente para \
         exclusão mútua, e o bloqueio evita queimar CPU.",
    ),
};

/// Barreira: ninguém passa da fase até todos chegarem.
static BARRIER: Demo = Demo {
    id: "barrier",
    actors: ["Processo A", "Processo B"],
    code: [
        &["calcula a fase 1", "barrier()", "calcula a fase 2"],
        &["calcula a fase 1", "barrier()", "calcula a fase 2"],
    ],
    frames: &[
        Frame {
            pc: [Some(0), Some(0)],
            vars: &[("na barreira", "0 de 2"), ("fase", "1")],
            caption: "Os dois processos trabalham na fase 1, cada um no seu ritmo.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [Some(1), Some(0)],
            vars: &[("na barreira", "1 de 2"), ("fase", "1")],
            caption: "A termina primeiro e chama barrier.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [None, Some(0)],
            vars: &[("na barreira", "1 de 2"), ("fase", "1")],
            caption: "A é bloqueado na barreira. Não adianta ele avançar: a fase 2 depende \
                      do resultado que B ainda está produzindo.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [None, Some(1)],
            vars: &[("na barreira", "2 de 2"), ("fase", "1")],
            caption: "B finalmente termina a fase 1 e também chega à barreira.",
            in_cs: [false, false],
            danger: false,
        },
        Frame {
            pc: [Some(2), Some(2)],
            vars: &[("na barreira", "0 de 2"), ("fase", "2")],
            caption: "Com todos presentes, a barreira libera os dois de uma vez e a fase 2 \
                      começa.",
            in_cs: [false, false],
            danger: false,
        },
    ],
    outcome: Outcome::Holds(
        "A barreira sincroniza grupos, não pares: ninguém prossegue até o último chegar.",
    ),
};

// ---------------------------------------------------------------------------
// Artigos
// ---------------------------------------------------------------------------

pub const ARTICLES: &[Article] = &[
    Article {
        icon: "💥",
        title: "O que é uma race condition",
        section: "2.4 e 2.4.1",
        ostep: &[ostep::THREADS, ostep::CONC_BUGS],
        summary: "Quando o resultado depende de quem chegou primeiro.",
        blocks: &[
            Block::P(
                "Processos que trabalham em conjunto naturalmente compartilham uma memória \
                 em comum, na qual ambos podem ler e escrever. Esse recurso compartilhado \
                 pode ser tanto uma estrutura de dados na memória quanto um arquivo.",
            ),
            Block::P(
                "As race conditions ocorrem quando dois ou mais processos tentam trabalhar ao \
                 mesmo tempo com um mesmo recurso, de maneira que as ações de um podem \
                 impactar de forma inesperada os resultados do outro.",
            ),
            Block::Note(
                "Os mesmos problemas e as mesmas soluções valem para threads. Aliás, elas \
                 são o caso mais comum: compartilham o espaço de endereços por construção.",
            ),
            Block::H("A comunicação entre processos"),
            Block::P(
                "O assunto maior aqui é a comunicação entre processos (IPC, interprocess \
                 communication), que se organiza em torno de três questões:",
            ),
            Block::N(&[
                "Como processos podem trocar informações entre si.",
                "Como garantir que não haja interferência de um no outro.",
                "Como garantir o sequenciamento adequado quando há dependência entre eles.",
            ]),
            Block::H("Veja acontecer"),
            Block::P(
                "Abaixo, duas threads incrementam o mesmo contador. Cada incremento são três \
                 passos — ler, somar, escrever — e é entre esses passos que o escalonador \
                 pode trocar de processo. Avance passo a passo:",
            ),
            Block::Demo(&LOST_UPDATE),
            Block::Warn(
                "Repare que nenhuma das duas threads tem bug. O código de cada uma, lido \
                 isoladamente, está correto. O defeito só existe na combinação — e só em \
                 alguns entrelaçamentos, o que torna esse tipo de erro difícil de reproduzir.",
            ),
        ],
    },
    Article {
        icon: "🚧",
        title: "Regiões críticas",
        section: "2.4.2",
        ostep: &[ostep::LOCKS],
        summary: "As quatro condições que uma solução precisa satisfazer.",
        blocks: &[
            Block::P(
                "Para evitar race conditions, precisamos de alguma maneira de proibir que um \
                 processo leia ou escreva em dados compartilhados enquanto outro processo \
                 ainda não tiver encerrado sua atividade sobre eles.",
            ),
            Block::Defs(&[
                (
                    "Exclusão mútua",
                    "o mecanismo que garante que apenas um de cada vez trabalhe sobre o \
                     recurso compartilhado.",
                ),
                (
                    "Região crítica",
                    "o trecho do programa que acessa esse recurso compartilhado.",
                ),
            ]),
            Block::H("As quatro condições"),
            Block::P(
                "Uma solução boa não é só uma que impede dois processos de entrarem juntos. \
                 São quatro exigências, e várias tentativas ingênuas falham nas duas últimas:",
            ),
            Block::N(&[
                "Dois processos jamais podem estar simultaneamente dentro de suas regiões \
                 críticas.",
                "Não devem ser feitas suposições sobre a velocidade ou o número de CPUs.",
                "Nenhum processo executando fora da sua região crítica pode bloquear \
                 qualquer outro processo.",
                "Nenhum processo deve ser obrigado a esperar indefinidamente para entrar na \
                 sua região crítica.",
            ]),
            Block::Note(
                "Guarde as condições 3 e 4: elas são o critério pelo qual as tentativas das \
                 próximas telas vão ser reprovadas. A alternância explícita, por exemplo, \
                 garante exclusão mútua e mesmo assim é uma solução ruim — porque tropeça \
                 na condição 3.",
            ),
        ],
    },
    Article {
        icon: "🔌",
        title: "Desabilitando interrupções",
        section: "2.4.3",
        ostep: &[ostep::LOCKS],
        summary: "A solução mais simples — e por que ela não serve.",
        blocks: &[
            Block::P(
                "Num sistema de processador único, a solução mais simples é fazer com que \
                 cada processo desabilite todas as interrupções ao entrar na região crítica \
                 e as reabilite antes de sair. Sem interrupção de relógio, não há troca de \
                 processo, e portanto ninguém interrompe a região crítica no meio.",
            ),
            Block::H("Por que não serve"),
            Block::L(&[
                "Bloqueia um único núcleo. Em computadores modernos, com várias CPUs, os \
                 outros processadores continuam rodando e acessando a memória compartilhada.",
                "Dá a um programa de usuário o poder de desligar todas as interrupções. Se \
                 ele esquecer de reabilitá-las, o sistema inteiro para.",
            ]),
            Block::Warn(
                "Ou seja: pode ser usada para algumas ações específicas dentro do kernel, \
                 mas nunca nos programas do usuário.",
            ),
        ],
    },
    Article {
        icon: "🔑",
        title: "Variáveis do tipo trava",
        section: "2.4.3",
        ostep: &[ostep::LOCKS],
        summary: "A primeira tentativa em software — que repete o erro original.",
        blocks: &[
            Block::P(
                "A ideia é começar com uma única variável de trava, configurada em zero. \
                 Quando um processo tenta entrar na região crítica, ele verifica a trava: se \
                 estiver em 0, muda para 1 e entra; se já estiver em 1, espera até que ela \
                 volte a zero.",
            ),
            Block::P(
                "Parece resolver. Só que temos aqui exatamente a mesma falha da race \
                 condition original — só que um nível acima, agora sobre a variável de trava.",
            ),
            Block::Demo(&LOCK_VARIABLE),
            Block::Note(
                "A lição que fica: verificar e depois agir são duas operações. Enquanto \
                 houver uma brecha entre elas, existe um entrelaçamento que quebra a solução. \
                 É essa brecha que a instrução TSL vai fechar.",
            ),
        ],
    },
    Article {
        icon: "🔄",
        title: "Alternância explícita",
        section: "2.4.3",
        ostep: &[ostep::LOCKS],
        summary: "Funciona, mas viola a terceira condição.",
        blocks: &[
            Block::P(
                "Usamos uma variável de controle, turn, iniciada em zero, que registra de \
                 quem é a vez de entrar na região crítica.",
            ),
            Block::P(
                "O processo 0 inspeciona turn, encontra zero e entra. O processo 1 também \
                 inspeciona, encontra zero e fica aguardando — testando a variável \
                 continuamente até que ela mude para 1. Essa espera em laço é a espera \
                 ocupada, e uma trava usada assim é chamada de trava giratória.",
            ),
            Block::Demo(&STRICT_ALTERNATION),
            Block::Warn(
                "O problema aparece quando um processo é bem mais rápido que o outro: o \
                 rápido termina sua parte não-crítica, quer entrar de novo, e não consegue \
                 — porque a vez está com o lento, que nem sequer está na região crítica. Um \
                 processo fora da região crítica bloqueou o outro, violando a condição 3.",
            ),
        ],
    },
    Article {
        icon: "🤝",
        title: "Solução de Peterson",
        section: "2.4.3",
        ostep: &[ostep::LOCKS],
        summary: "Exclusão mútua em software, sem violar as condições.",
        blocks: &[
            Block::P(
                "O algoritmo de Peterson é uma forma estritamente mais simples de realizar a \
                 exclusão mútua, e resolve o defeito da alternância explícita combinando duas \
                 informações: quem está interessado em entrar, e de quem é a vez.",
            ),
            Block::L(&[
                "Antes de entrar na região crítica, cada processo chama enter_region \
                 passando o seu próprio número. Essa chamada o faz esperar até que seja \
                 seguro entrar.",
                "Ao terminar, o processo chama leave_region para indicar que saiu e passar a \
                 vez para o outro.",
            ]),
            Block::Demo(&PETERSON),
            Block::Note(
                "O truque está no fato de os dois escreverem em turn: quem escreve por \
                 último acaba cedendo a vez. Se só um está interessado, ele entra direto — \
                 e é isso que salva a condição 3, que a alternância explícita violava.",
            ),
        ],
    },
    Article {
        icon: "⚛",
        title: "A instrução TSL",
        section: "2.4.3",
        ostep: &[ostep::LOCKS],
        summary: "Quando o hardware fecha a brecha entre testar e escrever.",
        blocks: &[
            Block::P(
                "Esta é a solução que recorre a um pouco de hardware. Alguns computadores \
                 projetados para multiprocessadores oferecem instruções como TSL, RX e LOCK.",
            ),
            Block::Defs(&[
                (
                    "TSL",
                    "test and set lock. Lê o conteúdo de LOCK, envia para o registrador RX e \
                     armazena um valor diferente de zero em LOCK — tudo isso de forma \
                     indivisível.",
                ),
                (
                    "XCHG",
                    "a versão equivalente da TSL na arquitetura Intel x86.",
                ),
            ]),
            Block::P(
                "É a indivisibilidade que muda tudo. Na variável de trava, o intervalo entre \
                 ler e escrever era a porta de entrada da race condition. Aqui essa porta não \
                 existe: não há instante em que a trava foi lida mas ainda não foi marcada.",
            ),
            Block::Demo(&TSL),
        ],
    },
    Article {
        icon: "😴",
        title: "Dormir e despertar",
        section: "2.4.4",
        ostep: &[ostep::COND_VARS],
        summary: "Sair da espera ocupada — e o despertar que se perde.",
        blocks: &[
            Block::P(
                "A solução de Peterson e a TSL estão corretas, mas ambas usam espera ocupada: \
                 o processo fica queimando CPU verificando se a região crítica desocupou.",
            ),
            Block::L(&[
                "Isso desperdiça tempo de processador que outro processo poderia usar.",
                "E não leva em conta a criticidade dos processos, o que abre espaço para o \
                 problema da inversão de prioridade.",
            ]),
            Block::P(
                "Um dos mecanismos mais simples para bloquear em vez de girar é o par \
                 sleep / wakeup. O sleep é uma chamada de sistema que bloqueia o processo que \
                 a chamou até que outro o desperte; o wakeup recebe como parâmetro o processo \
                 que deve ser despertado.",
            ),
            Block::H("O problema do produtor-consumidor"),
            Block::P(
                "Dois processos compartilham um buffer de tamanho fixo. O produtor insere \
                 informações; o consumidor retira. O problema aparece quando o produtor quer \
                 inserir e o buffer está cheio — a solução é fazê-lo dormir e ser despertado \
                 quando o consumidor tiver removido algum item. O raciocínio inverso vale \
                 quando o consumidor encontra o buffer vazio.",
            ),
            Block::P(
                "Só que a race condition volta, agora na variável que conta os itens do \
                 buffer:",
            ),
            Block::Demo(&LOST_WAKEUP),
            Block::Note(
                "Uma correção possível é adicionar um bit de espera para o sinal de \
                 despertar, guardando o wakeup que chegou cedo demais. Os semáforos \
                 generalizam justamente essa ideia — trocando um bit por um contador.",
            ),
        ],
    },
    Article {
        icon: "🚦",
        title: "Semáforos",
        section: "2.4.5",
        ostep: &[ostep::SEMAPHORES],
        summary: "Contar os sinais de despertar, em vez de perdê-los.",
        blocks: &[
            Block::P(
                "Os semáforos foram um mecanismo desenvolvido por Dijkstra para contar o \
                 número de sinais de despertar. Um semáforo é uma variável inteira não \
                 negativa que registra quantos sinais estão guardados.",
            ),
            Block::Defs(&[
                (
                    "down",
                    "confere se o semáforo é maior que zero. Se for, decrementa (gasta um \
                     sinal) e continua. Se for 0, o processo é posto para dormir sem \
                     completar o down.",
                ),
                (
                    "up",
                    "incrementa o valor do semáforo. Se houver processos dormindo naquele \
                     semáforo, um deles é escolhido para completar o down que ficou pendente.",
                ),
            ]),
            Block::Warn(
                "Cada uma dessas operações é atômica. É essa atomicidade que impede outro \
                 processo de mexer no semáforo no meio da operação — sem ela, estaríamos de \
                 volta ao problema da variável de trava.",
            ),
            Block::Demo(&SEMAPHORE),
            Block::H("O problema dos leitores e escritores"),
            Block::P(
                "O mesmo mecanismo up/down resolve outro problema clássico de concorrência, \
                 típico do acesso a bancos de dados: vários leitores podem consultar a base \
                 ao mesmo tempo, mas assim que um escritor aparece, todos os leitores \
                 precisam ser interrompidos.",
            ),
        ],
    },
    Article {
        icon: "🔒",
        title: "Mutexes e futexes",
        section: "2.4.6",
        ostep: &[ostep::LOCKS, ostep::LOCKED_DATA],
        summary: "O semáforo binário, e a versão rápida do Linux.",
        blocks: &[
            Block::P(
                "O mutex é um gerenciador de exclusão mútua de recursos compartilhados — uma \
                 versão mais simples dos semáforos, em que não é necessário um contador. É \
                 uma variável compartilhada com apenas dois estados: travado ou destravado. \
                 É especialmente útil para pacotes de threads executados no espaço do usuário.",
            ),
            Block::P(
                "Quando uma thread precisa acessar a região crítica, ela chama mutex_lock. Se \
                 o mutex estiver destravado, ela entra. Caso contrário, é bloqueada até que a \
                 thread que está na região crítica termine. Havendo várias threads \
                 bloqueadas, a próxima é escolhida ao acaso.",
            ),
            Block::Demo(&MUTEX),
            Block::H("Futexes"),
            Block::P(
                "Com o paralelismo crescendo, achar um mecanismo de sincronização eficiente \
                 ficou cada vez mais importante — e as duas famílias que vimos têm forças \
                 opostas:",
            ),
            Block::Table2(
                "Travas giratórias",
                "Mecanismos bloqueantes",
                &[
                    (
                        "Rápidas quando a espera é curta.",
                        "Funcionam bem quando há disputa pesada.",
                    ),
                    (
                        "Desperdiçam CPU se houver muita disputa ou espera longa.",
                        "As trocas de contexto ficam caras quando há pouca disputa.",
                    ),
                ],
            ),
            Block::P(
                "O futex (fast userspace mutex) é o recurso do Linux que combina as duas: \
                 implementa o mecanismo de mutex evitando chamadas desnecessárias ao núcleo. \
                 Ele tem duas partes:",
            ),
            Block::L(&[
                "Um serviço de núcleo, com uma fila de espera onde múltiplos processos \
                 aguardam por uma trava. Colocar um processo nessa fila exige uma chamada de \
                 sistema.",
                "Uma biblioteca de usuário. Não havendo disputa, tudo se resolve no espaço \
                 do usuário: a thread pega a trava com um decremento e teste atômico e \
                 inspeciona o resultado. Só quando a trava já está com outro é que a \
                 biblioteca recorre à chamada de sistema para entrar na fila.",
            ]),
        ],
    },
    Article {
        icon: "🏛",
        title: "Monitores",
        section: "2.4.8",
        ostep: &[ostep::COND_VARS],
        summary: "Empurrar a exclusão mútua para o compilador.",
        blocks: &[
            Block::P(
                "Mesmo com todos esses mecanismos, ainda podem surgir situações em que todos \
                 os processos ficam bloqueados e nunca terminam — os deadlocks, assunto do \
                 capítulo 6. Os monitores foram criados para tratar dessas questões num nível \
                 mais alto.",
            ),
            Block::P(
                "Um monitor é um conjunto de procedimentos, variáveis e estruturas de dados \
                 reunidos em um módulo ou pacote. Processos podem chamar os procedimentos do \
                 monitor sempre que quiserem, mas não podem acessar diretamente as estruturas \
                 de dados internas por fora dele.",
            ),
            Block::Note(
                "A propriedade decisiva: apenas um processo pode estar ativo dentro do \
                 monitor por vez. E quem implementa isso é o compilador, tipicamente usando \
                 mutex ou semáforos binários — o programador não escreve a exclusão mútua. \
                 Transformar todas as regiões críticas em procedimentos de monitores garante, \
                 por construção, que dois processos nunca estarão nas suas regiões críticas \
                 ao mesmo tempo.",
            ),
            Block::H("Variáveis de condição"),
            Block::P(
                "Ainda falta uma maneira de bloquear processos dentro do monitor. A solução \
                 são as variáveis de condição, com as operações wait e signal:",
            ),
            Block::L(&[
                "Quando um procedimento do monitor descobre que não pode continuar, executa \
                 um wait sobre alguma variável de condição — por exemplo, 'full'. Isso \
                 bloqueia o processo que chamou e permite que outro entre no monitor.",
                "O processo que estava bloqueado pode ser liberado por um signal aplicado à \
                 variável de condição que ele estava esperando.",
            ]),
            Block::P(
                "A implementação de monitores depende do paradigma de orientação a objetos.",
            ),
        ],
    },
    Article {
        icon: "✉",
        title: "Troca de mensagens",
        section: "2.4.8",
        ostep: &[],
        summary: "Comunicar em vez de compartilhar.",
        blocks: &[
            Block::P(
                "Todos os mecanismos anteriores pressupõem memória compartilhada. A troca de \
                 mensagens segue outro caminho: a informação é transmitida através de duas \
                 chamadas de sistema.",
            ),
            Block::Code(&["send(destination, &message)", "receive(source, &message)"]),
            Block::P(
                "Uma abordagem robusta implementa também a confirmação de recebimento, para \
                 que o remetente saiba que a mensagem chegou.",
            ),
        ],
    },
    Article {
        icon: "🚧",
        title: "Barreiras",
        section: "2.4.9",
        ostep: &[],
        summary: "Sincronizar grupos de processos, não pares.",
        blocks: &[
            Block::P(
                "O último mecanismo de sincronização se aplica a grupos de processos. \
                 Aplicações cujas tarefas são divididas em fases exigem que nenhum processo \
                 prossiga até que todos tenham completado a fase atual — e é para isso que \
                 serve uma barreira em cada fase.",
            ),
            Block::P(
                "Quando um processo atinge a barreira, ele é bloqueado até que os processos \
                 restantes também cheguem. Só então todos são liberados de uma vez. A \
                 operação é a chamada de sistema barrier.",
            ),
            Block::Demo(&BARRIER),
            Block::Note(
                "Em baixo nível existem as barreiras de memória, que implementam um \
                 mecanismo análogo para garantir a sincronia das operações de memória.",
            ),
            Block::SeeAlso(
                "O outro lado da moeda: como o sistema decide quem roda em cada momento.",
                Screen::Scheduling,
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
        check::articles(ARTICLES);
    }
}
