# OS Playground

Ferramenta visual, em **Rust**, para apoiar o ensino de **Sistemas
Operacionais** (foco em UNIX). O conteúdo é baseado em minhas anotações dos livros
(Tanenbaum, *Sistemas Operacionais Modernos*) e 
[OSTEP](https://pages.cs.wisc.edu/~remzi/OSTEP/).



### 1 — Processos e Threads 

| Opção | Status |
|-------|--------|
|  Árvore de Processos |  funcional (árvore desenhada da hierarquia real ao vivo, via `sysinfo`) |
|  Conceitos (textos)  |  funcional (11 artigos a partir das anotações, §2.1–2.2) |
|  Race Conditions     |  funcional (13 artigos, §2.4, com demos passo a passo) |
|  Escalonamento       |  funcional (11 artigos, §2.5, com simulador de Gantt) |

### 2 — Memória Virtual 

| Opção | Status |
|-------|--------|
|  Espaços de Endereçamento |  funcional (7 artigos, §3.1–3.2, com base/limite interativo e simulador de encaixe) |
|  Paginação                |  funcional (6 artigos, §3.3, com tradutor de endereços e TLB) |
|  Substituição de Páginas  |  funcional (11 artigos, §3.4, com simulador dos 9 algoritmos) |
|  Projeto e Implementação  |  funcional (8 artigos, §3.5–3.6, com a curva do tamanho de página) |

###  3 — Sistema de Arquivos 

| Opção | Status |
|-------|--------|
|  Explorador de Arquivos |  funcional (sistema de arquivos real ao vivo: número de i-node, contagem de ligações, hard links, symlink e stat de cada arquivo, via `std::os::unix::fs`) |
|  Arquivos e Diretórios  |  funcional (7 artigos, §4.1–4.2) |
|  Implementação do Sistema de Arquivos |  funcional (8 artigos, §4.3–4.4, com os diagramas de disco e do i-node) |

### 4 — Entrada e Saída 

| Opção | Status |
|-------|--------|
|  Dispositivos ao Vivo |  funcional (os arquivos especiais de `/dev` ao vivo: bloco e caractere, com os números maior e menor) |
|  Conceitos de Entrada e Saída |  funcional (6 artigos, §5.1–5.3, com o simulador animado das três formas de E/S) |

As telas ao vivo (Explorador e Dispositivos) leem o sistema real desta máquina
via `std::os::unix::fs`, então só fazem sentido em UNIX (no macOS e no Linux os
dados existem todos).

## Processos e Threads

### Árvore de Processos

A hierarquia é desenhada como uma árvore de verdade, caixas ligadas por ramos, a
partir do `pid 1` (`launchd`/`init`):

- **arrastar** move a árvore, **rolar** (ou pinçar) dá zoom no ponto do cursor.
- **clicar** num nó abre os detalhes. Clicar no **círculo** pendurado no nó
  recolhe ou expande a subárvore (quando recolhido, ele mostra o nº de filhos).
- `⬇`/`➡` alterna entre raiz em cima e raiz à esquerda, `⛶` ajusta à tela,
  `⊞`/`⊟` expandem ou recolhem tudo.
- o filtro mostra os processos que casam, sua subárvore e o caminho até a raiz.


### Conceitos

Onze artigos navegáveis cobrindo modelo de processo, criação (`fork`) e término, hierarquia e
`init`, estados do processo com diagrama desenhado, tabela de processos, por que
existem threads, modelo clássico, Pthreads, threads em espaço de usuário, de
kernel e híbridas, e uma síntese Processo × Thread.

Cada artigo termina com as referências: a seção correspondente do Tanenbaum & Bos
e os capítulos do OSTEP que tratam do mesmo assunto, com link direto para o PDF de
cada capítulo. Quando o OSTEP não cobre o recorte da seção (é o caso de threads em
espaço de usuário, de kernel e híbridas), o artigo cita apenas o Tanenbaum, em vez
de forçar uma remissão frouxa.

Alguns artigos trazem ainda um atalho para a tela relacionada. O de hierarquia,
por exemplo, abre a árvore de processos ao vivo.

### Race Conditions

Treze artigos. Os primeiros tratam do que é uma race
condition, das regiões críticas e das quatro condições. Os demais percorrem cada
mecanismo de exclusão mútua: desabilitar interrupções, variáveis de trava,
alternância explícita, Peterson, TSL, dormir e despertar, semáforos, mutexes e
futexes, monitores, troca de mensagens e barreiras.

Oito deles trazem uma **demonstração animada** desenhada pelo próprio app: duas
linhas de execução lado a lado, o destaque **deslizando** de uma linha de código
para outra, as variáveis compartilhadas pulsando quando mudam, a borda da região
crítica acendendo, e uma legenda que faz dissolução a cada passo.

Cada roteiro mostra o entrelaçamento que revela o problema, ou que mostra por que
a solução resiste a ele. Os controles `◀` `▶|` andam passo a passo, `▶` toca
contínuo, e ** Apresentar** reproduz do início mais devagar, segurando mais tempo
nos quadros de virada, pensado para projetar em aula sem interagir.

### Escalonamento

Onze artigos cobrindo: introdução e troca de contexto, quando
escalonar, preemptivo × não-preemptivo, categorias e metas, e então cada algoritmo
(FCFS, SJF, SRTF, round-robin, prioridade, múltiplas filas e loteria), mais o
escalonamento de threads.

Sete deles trazem um **simulador de Gantt vivo**: a mesma carga de trabalho em
todos os algoritmos comparáveis, então mudar de tela isola o efeito da política.
Chegada, duração, prioridade e bilhetes são editáveis, o quantum tem controle
deslizante, e as métricas (tempo de espera e de retorno médios) recalculam a cada
ajuste.

O `▶` **anima a linha do tempo**: o diagrama se desenha da esquerda para a direita
com um cursor marcando o instante atual, como se o escalonador estivesse decidindo
ao vivo. `⏭` pula direto para o resultado completo.

## Memória Virtual


### Espaços de Endereçamento 

Sete artigos: a memória sem abstração (com o diagrama das três organizações,
sistema em RAM ou em ROM), a noção de espaço de endereçamento (proteção e
realocação), base e limite, troca de processos (fragmentação externa e interna,
compactação), mapas de bits e listas encadeadas, e os quatro algoritmos de
encaixe.

Dois widgets:

- **Base e limite.** Arraste a base e o programa muda de lugar sem mudar uma
  palavra. Passe o endereço virtual do limite e o acesso vira desvio para o SO.
- **Simulador de encaixe.** Uma memória de 64 unidades vista ao mesmo tempo como
  fita, como **lista encadeada** de segmentos (tipo, início, comprimento,
  ponteiro) e como **mapa de bits**. Uma sequência de pedidos e liberações é
  aplicada passo a passo (`◀` `▶|` `▶` `⏭`) sob *first*, *next*, *best* ou *worst
  fit*. A cada passo o simulador sublinha os blocos examinados, destaca o bloco
  escolhido e explica a decisão (inclusive a fusão com os vizinhos ao liberar e a
  falha por fragmentação externa). Os tamanhos dos pedidos são editáveis.

### Paginação 

Seis artigos: memória virtual e sobreposições, páginas e quadros (com a MMU
desenhada), a decomposição do endereço virtual (com o código em C das anotações),
a tabela de páginas e seus campos (diagrama da entrada), e a TLB.

O **tradutor de endereços** reproduz o exemplo das anotações (64 KB virtuais, 32
KB físicos, páginas de 4 KB) com a tabela de páginas e os quadros desenhados e
ligados por setas. O endereço virtual é editável. O widget mostra os 16 bits
partidos em página + deslocamento, o endereço físico com os mesmos 12 bits de
deslocamento intactos, e a **falta de página** quando o bit de presença é 0 (com
um botão que trata a falta: quadro livre se houver, senão descarta a página mais
antiga). No artigo da TLB o mesmo tradutor ganha uma **TLB de 4 entradas** com
contadores de acerto e falta, histórico dos acessos e o botão que percorre um
vetor de 16 KB, que reproduz as 4096 leituras e 4 faltas da conta das anotações.

### Substituição de Páginas 

Onze artigos: o problema, e então cada algoritmo (ótimo, NRU, FIFO, segunda chance
e relógio, LRU, envelhecimento, conjunto de trabalho e WSClock), mais o resumo
comparativo.

Nove deles trazem o **simulador de substituição**, rodando a mesma cadeia de 20
referências (com duas escritas, para os bits M aparecerem) em 3 quadros. O FIFO
usa a cadeia da **anomalia de Belady** (suba de 3 para 4 quadros e as faltas
aumentam). A grade tem uma coluna por referência e uma linha por quadro, com
`✔`/`✖` embaixo e a página descartada ao lado (laranja quando estava suja e custou
uma escrita em disco). As interrupções de relógio aparecem como traços.
Clicar numa coluna (ou andar com `◀` `▶|`, ou animar com `▶`) abre o detalhe do
passo: a explicação da decisão em uma frase (as classes do NRU, os rótulos do
ótimo, a varredura do conjunto de trabalho…) e a tabela dos quadros com R, M e o
dado que o algoritmo consulta (instante de carga, último uso, contador de
envelhecimento em binário, idade contra τ). No relógio e no WSClock o **anel** é
desenhado de verdade, com o ponteiro.

Cadeia de referências, número de quadros, intervalo do relógio e τ são editáveis.
O artigo de resumo tem um **gráfico de barras** com as faltas de todos os
algoritmos na mesma cadeia, com o ótimo como piso.

### Projeto e Implementação 

Oito artigos: alocação local × global e o PFF, controle de carga e o OOM, política
de limpeza (daemon de paginação), tamanho da página, espaços separados e páginas
compartilhadas (com o *copy-on-write* do `fork`), o envolvimento do SO nos quatro
momentos, e o tratamento de uma falta de página passo a passo.

O artigo do tamanho da página traz a **curva do desperdício** `o(p) = s·e/p + p/2`
com as duas parcelas e o mínimo `p = √(2se)` marcado. Mexa no tamanho do processo e
da entrada e veja o ótimo se deslocar (1 MB e 8 bytes dão 4 KB).

## Sistema de Arquivos


### Explorador de Arquivos

O **Explorador** abre um diretório de verdade e mostra, para cada entrada, o número
de i-node (o nome de baixo nível da seção 4.1), a contagem de ligações, o tipo (a
letra que o `ls -l` põe na primeira coluna) e os metadados que a chamada `stat`
devolve. As entradas que são hard links do mesmo arquivo recebem a mesma cor, o que
torna visível o compartilhamento da seção 4.3.4. Uma ligação simbólica aparece com
uma seta para o alvo.

A trilha de diretórios no topo navega com um clique em cada parte do caminho,
clicar num diretório entra nele, e o painel da direita traz o `stat` completo da
entrada selecionada. O mesmo diretório também aparece como a lista de pares (nome,
i-node) que ele de fato é no disco (seção 4.3.3), no formato das entradas do livro.

### Arquivos e Diretórios 

Sete artigos: o que é um arquivo e seu nome, estrutura e tipos de arquivo, formas
de acesso e atributos, as operações sobre um arquivo, os diretórios e sua
organização, os nomes de caminho (absoluto e relativo), e as operações com
diretórios e as ligações (hard link e ligação simbólica).

### Implementação do Sistema de Arquivos

Oito artigos: o esquema do disco e a inicialização, o layout de um sistema de
arquivos, os métodos que rastreiam os blocos de um arquivo, o i-node e seu índice
de vários níveis, os diretórios e o caminho de acesso, a consistência (diário e
estrutura em log), os sistemas de arquivos virtuais e a memória flash, e o
gerenciamento do espaço em disco.

Os artigos trazem quatro **diagramas desenhados** pelo próprio app, no mesmo estilo
do diagrama de estados do capítulo 2:

- **Esquema da partição** : o MBR ou GPT e o bloco de inicialização, com o
  resto do disco repartido em partições.
- **Layout de um sistema de arquivos** : as regiões de bloco em
  ordem, do superbloco aos mapas de i-nodes e de dados, à tabela de i-nodes e aos
  blocos de dados, com a numeração nos extremos de cada região.
- **O i-node e o índice de vários níveis** : o i-node com os atributos e os
  três grupos de ponteiros, cada nível numa faixa, do direto ao indireto duplo.
- **Métodos de alocação**: a alocação contígua, a lista encadeada, a FAT e
  o i-node lado a lado, para comparar como cada um guarda os blocos de um arquivo.

## Entrada e Saída


### Dispositivos ao Vivo

A tela **Dispositivos** lê os arquivos especiais de `/dev` e os separa nos dois
tipos da seção 5.1.1. Cada linha é um dispositivo, marcado com `b` ou `c` como no
`ls -l /dev`, com o número maior (qual driver o atende, seção 5.3.2) e o número
menor (qual unidade concreta). Os **dispositivos de bloco** leem e escrevem blocos
endereçáveis e independentes, como os discos e as partições. Os **dispositivos de
caractere** entregam um fluxo de bytes, sem endereço nem busca, como os terminais e
as fontes `null`, `zero` e `random`. O topo conta quantos são de cada tipo.

### Conceitos de Entrada e Saída 

Seis artigos: os dispositivos de entrada e saída, os controladores e seus
registradores, o acesso direto à memória (DMA), os objetivos do software de E/S, as
três formas de fazer E/S, e as quatro camadas do software de E/S.

Os artigos trazem três **diagramas desenhados**: o dispositivo canônico, com a
interface de registradores separada do interior (seção 5.1.2), o acesso direto à
memória em três passos (seção 5.1.4), e as quatro camadas do software de E/S, com o
pedido descendo e a resposta subindo (seção 5.3).

O artigo das três formas traz um **simulador animado** que compara, lado a lado, o
que a CPU e o dispositivo fazem no tempo (seções 5.2.2 a 5.2.4). A fase de E/S dura
o mesmo em todos os modos. O que muda é a CPU. Por programa, ela fica presa em
espera ativa e nenhum outro processo avança. Por interrupção, ela troca para outro
processo e uma interrupção a acorda no fim. Por DMA, o controlador move os dados
sozinho e a CPU nem participa da transferência. Um medidor mostra o aproveitamento
útil da CPU (50% por programa, 100% nos outros dois) e uma caixa explica cada passo.
O `▶` toca a linha do tempo, `↺` reinicia, e o controle deslizante vai direto a
qualquer instante.

## Como executar (app)

Requer apenas o toolchain **Rust**.

```bash
cargo run            # modo debug (rápido de compilar)
make dev             # idem
make run             # modo release
```

Para abrir direto numa tela (útil para projetar em aula), use
`OS_PLAYGROUND_START="tela[/artigo]"`, com `tela` em `tree`, `texts`, `race`,
`scheduling`, `addressing`, `paging`, `replacement`, `design`, `explorer`,
`files`, `fsimpl`, `devices`, `io` e o artigo como índice a partir de 0:

```bash
OS_PLAYGROUND_START=replacement/4 cargo run    # segunda chance e relógio
```

### App clicável (macOS)

```bash
make app             # -> dist/"OS Playground".app
```

Monta um bundle `.app` de verdade (com `Info.plist` e o ícone `AppIcon.icns`),
que abre com duplo-clique no Finder, sem janela de Terminal junto.

O bundle é autocontido: todos os recursos que o app usa são embutidos no binário
em tempo de compilação.

## Ícone do programa

O ícone é gerado a partir de `logo.png`. O gerador escolhe sozinho o que fazer com
o fundo:

- se o `logo.png` **já vem recortado** (com canal alfa de verdade), ele é usado
  como está, sem flood-fill, que só estragaria um recorte bom.
- se vier **opaco com fundo branco**, o fundo é removido por flood-fill a partir
  das bordas (o branco *dentro* do retrato é preservado).

Em ambos os casos a arte é aparada até o conteúdo visível e centralizada num
quadrado, para o ícone ocupar bem o quadro.

```bash
make icons
# -> assets/logo_transparent.png, assets/window_icon.png,
#    assets/logo.svg e assets/icons/AppIcon.icns
```

Como o `window_icon.png` é embutido no binário (`include_bytes!`), depois de trocar
o `logo.png` rode **`make icons && make app`** para o ícone novo realmente aparecer.

### "Compilar tudo junto"

```bash
make all     # icons + build + app
```

O projeto depende **apenas do toolchain Rust**. Não há runtime Python, nem
biblioteca de sistema, nem nada para instalar na máquina além do Rust. As animações
e os diagramas são desenhados pelo próprio app.

## Testes

```bash
cargo test                          # simuladores + validação do conteúdo
cargo test snapshots -- --ignored   # renderiza as telas em target/snapshots/*.png
```

Os testes normais cobrem as simulações (escalonamento, encaixe, tradução e
substituição, inclusive a anomalia de Belady, a equivalência segunda chance ≡
relógio e "o ótimo nunca perde") e checam todos os artigos (textos não vazios,
seção do capítulo certo, links do OSTEP bem formados).

Os *snapshots* renderizam cada artigo das telas de Memória Virtual, Sistema de
Arquivos e Entrada e Saída headless (via `egui_kittest`, só dependência de
desenvolvimento) para inspeção visual. Eles precisam de uma GPU acessível ao
`wgpu`, por isso ficam atrás do `--ignored`.

## Estrutura

```
src/
  main.rs              entrada (eframe) + ícone da janela
  app.rs               estado + navegação entre telas (+ OS_PLAYGROUND_START)
  ui/
    home.rs            tela inicial (logo + um botão por tópico)
    topic_menu.rs      os cartões de cada tópico (PROCESSES, MEMORY, FILES, IO)
    reader.rs          leitor de artigos (modelo Block/Article + renderização)
    -- Processos e Threads --
    process_tree.rs    tela da árvore: captura, filtro, detalhes
    tree_view.rs       desenho node-link (caixas + ramos, pan/zoom)
    content.rs         artigos de Processos e Threads (§2.1–2.2)
    content_race.rs    artigos de Race Conditions (§2.4) + roteiros das demos
    content_sched.rs   artigos de Escalonamento (§2.5) + cenários do simulador
    demo.rs            motor das demos passo a passo (dois atores concorrentes)
    gantt.rs           simulação de escalonamento + diagrama de Gantt
    texts.rs / race.rs / scheduling.rs   (invólucros finos do reader)
    -- Memória Virtual --
    content_addressing.rs   artigos de Espaços de Endereçamento (§3.1–3.2)
    content_paging.rs       artigos de Paginação (§3.3)
    content_replacement.rs  artigos de Substituição de Páginas (§3.4)
    content_pagedesign.rs   artigos de Projeto e Implementação (§3.5–3.6)
    memfit.rs          simulador de memória livre (lista, mapa de bits, encaixes)
    memaddr.rs         base/limite, tradutor de endereços + TLB, diagramas,
                       curva do tamanho de página
    pagerepl.rs        simulador de substituição de páginas + comparação
    addressing.rs / paging.rs / replacement.rs / pagedesign.rs   (invólucros)
    -- Sistema de Arquivos --
    content_files.rs   artigos de Arquivos e Diretórios (§4.1–4.2)
    content_fsimpl.rs  artigos de Implementação (§4.3–4.4)
    fsview.rs          diagramas de disco (layout vsfs, i-node, alocação, MBR/GPT)
    explorer.rs        tela ao vivo: i-nodes, ligações, hard links, stat
    files.rs / fsimpl.rs   (invólucros do reader)
    -- Entrada e Saída --
    content_io.rs      artigos de Entrada e Saída (§5.1–5.3)
    ioview.rs          diagramas de E/S + simulador (programa × interrupção × DMA)
    devices.rs         tela ao vivo: os dispositivos de /dev (bloco × caractere)
    io.rs              (invólucro do reader)
    snapshots.rs       renderização headless das telas (só em testes)
  sys/processes.rs     coleta de processos/threads (sysinfo)
  sys/fs.rs            leitura de um diretório real (i-node, stat, hard links)
  sys/devices.rs       leitura de /dev (bloco × caractere, maior/menor)
  bin/make_icons.rs    gerador de ícones (remove fundo branco)
scripts/
  bundle_app.sh        monta o bundle macOS .app (make app)
Makefile               orquestra tudo
```

