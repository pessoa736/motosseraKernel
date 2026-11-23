# ROADMAP — motosseraKernel

Documento de análise e priorização, iniciado em 2026-11-20 a partir
do commit `e66e9be` e atualizado em 2026-11-21 após a revisão da reorganização,
dos builds multiarquitetura e dos primeiros boots em QEMU. A referência de
partida é o commit `6cf568d` mais as alterações locais revistas nessa data.
Não é um plano com prazos; é uma fila de trabalho para revisar a cada etapa.

## 1. Estado do projeto

Kernel bare-metal hobby em Rust `#![no_std]`, com Limine como bootloader, heap
próprio, PMM por bitmap, framebuffer com backbuffer, serial para debug, um shell
com Rhai embutido e um gerador de fontes TTF separado. O manifesto, o Makefile,
a toolchain e os linker scripts ficam na raiz; o código está em `source/`, com
constantes compartilhadas extraídas para `source/definitions/`.

O manifesto voltou a declarar o binário e os builds release de x86_64 e
AArch64 softfloat passam. O Makefile gera as duas ISOs. No x86_64, o QEMU
alcança serial, PMM, framebuffer e a tela de boas-vindas. No ARM, o firmware
inicia, mas o kernel não emite log: o build não aplica `linker_aarch64.ld` ao
target softfloat. O shell ainda não foi alcançado nem testado, `cargo test` em
host falha com `core` duplicado, e a página WebAssembly é apenas um esqueleto.
A prioridade é corrigir a configuração de link ARM e chegar ao prompt; depois,
fechar o fluxo comando -> resposta e os testes automatizados.

## 2. O que já existe e o que foi verificado

### Produto e arquitetura (inspecionado)

- Entrada `_start` em `source/main.rs:193`, requests Limine (base revision,
  stack, framebuffer, HHDM, memmap) e inicialização em ordem: serial -> heap ->
  PMM -> vídeo -> shell (`source/main.rs:74`).
- Heap com `linked_list_allocator` sobre a maior região `MEMMAP_USABLE`,
  limitada a 32 MiB (`source/memory.rs:28`).
- PMM por bitmap com `allocate_frame`/`free_frame` (`source/pmm.rs:101`).
- Vídeo com `DMatrix<u32>` de backbuffer, alpha blending e fonte bitmap 16x16
  (`source/video.rs`, `source/db_font.rs`).
- Serial x86 (COM1) e ARM (PL011 em `0x0900_0000`) para I/O e log
  (`source/io.rs`, `source/debuglog.rs`).
- Shell com `TermUI`, histórico, buffer de saída e `rhai::Engine`
  (`source/shell.rs:94`).
- Gerador de fontes TTF -> `[[u16;16];128]` (`tools/fontGenerator`).

### Executado na análise inicial (commit `e66e9be`, estrutura `kernel/`)

- `cargo build --release --target x86_64-unknown-none -Zbuild-std=core,alloc`
  com `CARGO_PROFILE_RELEASE_PANIC=abort`: PASSOU, com 43 warnings.
  Sem `-Zbuild-std` falha neste Termux (`can't find crate for core`).
- `cargo build --release --target aarch64-unknown-none -Zbuild-std=core,alloc`
  com `CARGO_PROFILE_RELEASE_PANIC=abort`: PASSOU, com 54 warnings.
- `cargo build --release --target aarch64-linux-android ...`: FALHOU no link:
  `cc: error: unknown argument: '-zmax-page-size=0x1000'`.
- `cargo test --target aarch64-linux-android`: FALHOU,
  `duplicate lang item panic_impl` (`kernel/src/debuglog.rs:62` no layout antigo).
- `cargo build --manifest-path tools/fontGenerator/Cargo.toml`: PASSOU.
- ISO ARM montada em scratch e bootada em QEMU `virt` + UEFI (EDK2):
  - com 128 MiB o próprio Limine aborta: `High memory allocator: Out of memory`;
  - com 256/512 MiB o Limine imprime `Loading executable boot():/kernel.bin`
    e o kernel não emite nenhum log próprio.
- ISO x86_64 montada em scratch e `limine bios-install`: PASSOU.
  Não foi bootada: não existe `qemu-system-x86_64` instalado neste Termux.
- `make -n kernel.bin ARCHI=x86_64`: FALHOU no Limine. O `limine/GNUmakefile`
  gerado anteriormente aponta para `/home/motosseraKernel/...`, mas o projeto
  está em `/home/Documemts/motosseraKernel/...` (configuração obsoleta após
  mover o diretório).

### Verificação da reorganização e das alterações atuais (2026-11-21)

- O pull até `6cf568d` foi concluído por fast-forward e as alterações locais
  foram preservadas nos caminhos novos. Depois disso, `[[bin]]` passou a
  apontar para `source/main.rs` e `wasm-pack` foi retirado das dependências
  do kernel, eliminando os erros de `std` nas dependências.
- `cargo build --release --target x86_64-unknown-none`: PASSOU, com 39
  warnings no crate. `cargo build --release --target
  aarch64-unknown-none-softfloat`: PASSOU, também com 39 warnings.
- `llvm-readelf`: o x86_64 tem entrada `0xffffffff8020a940`, compatível com
  `linker.ld`. O ARM tem entrada `0x28c6f0` e segmentos a partir de
  `0x200000`; portanto, `linker_aarch64.ld` não foi aplicado.
- `make -s kernel.iso ARCHI=x86_64` e `make -s kernel.iso ARCHI=aarch64`,
  com artefatos direcionados ao scratch: PASSARAM. O Limine configurou e
  compilou os ports x86_64 BIOS/UEFI e AArch64 UEFI.
- QEMU x86_64 com timeout de 20 s: exit 137 esperado pelo timeout; a serial
  registrou `[ BOOT ] A/B`, PMM, framebuffer e início de `welcome`. Não
  alcançou o prompt nem executou comandos.
- QEMU AArch64 `virt` + EDK2 com timeout de 20 s: exit 137 esperado; a serial
  mostrou apenas firmware/UEFI, sem `[ BOOT ] A` do kernel.
- `cargo test --target x86_64-unknown-linux-gnu`: FALHOU com `duplicate lang
  item`/`core` duplicado devido ao `build-std` global. Nenhuma suíte passou.

## 3. Bugs e inconsistências confirmados, por impacto

### P0 — impede boot ou uso básico

- [ ] O Makefile seleciona `aarch64-unknown-none-softfloat`, mas
  `.cargo/config.toml` só configura `aarch64-unknown-none` e
  `aarch64-linux-android`. Assim, `linker_aarch64.ld` não é aplicado: o ELF
  atual entra em `0x28c6f0` em vez de `0xffffffff80000000`. Criar a seção de
  rustflags para o target softfloat e substituir o target Android residual em
  `rust-toolchain.toml`; depois repetir build, leitura do ELF, ISO e boot.
- [ ] O boot ARM continua sem `[ BOOT ] A`. A hipótese anterior de NEON com
  FP/SIMD desativado e exceção em `0x200` ainda precisa ser reavaliada depois
  que o linker correto for aplicado. O stub de entrada e os vetores de
  exceção continuam pendentes.
- [ ] O x86_64 chega à tela de boas-vindas, mas não ao prompt no smoke test de
  20 s. É preciso medir a duração da intro e validar a transição para o loop
  do shell antes de testar comandos.
- [ ] Shell não produz saída visível nem fecha o ciclo de comando:
  `TermUI::writechar` está vazio, o buffer gráfico não recebe o conteúdo, o
  prompt entra na linha executada e `status` não retorna ao prompt após Enter.
- [ ] `cargo test --target x86_64-unknown-linux-gnu` falha com múltiplas
  definições de `core` devido ao `build-std` global. Separar a configuração de
  testes em host do runtime bare-metal antes de adicionar a suíte.

### P1 — comportamento errado quando o caminho é exercitado

- [ ] `make run` já separa máquina e CPU, mas ativa KVM sempre que host e
  target coincidem, usa caminho fixo de firmware ARM e exige GTK. Detectar
  aceleração com fallback TCG, configurar firmware/display e capturar a serial
  em arquivo. O bootstrap do Limine também continua sendo refeito a cada build;
  falta um alvo idempotente.
- [ ] A interface Rhai só aceita resultado `String`: `eval::<String>`
  (`source/shell.rs:210`) faz `1 + 2` cair no ramo de erro, e todo erro
  vira a string genérica `"erro"`.
- [ ] `clear_input_line` sobrescreve bytes com espaços mas não chama
  `String::clear` (`source/shell.rs:140`), deixando `len()` antigo;
  `current_line_pos` nunca é atualizado, e `save_in_historic_command` usa
  `insert(pos)` com essa posição.
- [ ] Geometria do terminal é incompatível com a fonte: `TermUI` usa células
  de 8 px, enquanto `draw_char` desenha 16x16 e `draw_string` avança 16 px
  (`source/video.rs:228`, `source/video.rs:251`).
- [ ] `TermUI::draw` chama `draw_char` com x/y trocados e subtrai `view_pos`
  (células) como se fosse pixels (`source/shell.rs:72-87`);
  `view_mut(view_pos, self.size)` com scroll > 0 estoura os limites da matriz.
- [ ] `Vga::init` ignora o retorno `bool` no chamador (`source/main.rs:90`)
  e aloca backbuffer fixo de 1920x1080 antes de saber o modo real
  (`source/video.rs:114`). Falha de framebuffer vira uso posterior de
  `unwrap`, não diagnóstico.
- [ ] Escrita de pixels assume layout ARGB e 32 bpp, ignorando `bpp` e as
  máscaras de cor do Limine. Funciona no caso comum do QEMU, mas não é o
  contrato completo do protocolo.
- [ ] Não há IDT/vetor de exceções nem fonte de interrupção: `TIMER_TICKS`
  nunca é incrementado (`source/timer.rs:7`), logo `Clock` não mede tempo
  real. `TimeInteration` conta iterações de loop, não duração
  (`source/timer.rs:82`) — a intro "de 1 s" é 1000 iterações.
- [ ] Laços de espera ocupam CPU: loop principal (`source/main.rs:115`) e
  polling da serial (`source/io.rs`). O panic agora usa `hlt`/`wfe`, mas os
  demais loops continuam.

### P2 — higiene e manutenção

- [ ] Serial continua inicializada duas vezes: existe `Kernel.io` e outra
  instância em `Shell.io`, ambas com `init` (`source/main.rs:79`,
  `source/shell.rs:131`). O boot x86_64 confirma a duplicação no log.
- [ ] `unsafe impl Sync for Kernel` + `static mut` + `Mutex::get_mut`
  (`source/main.rs:182`, `source/main.rs:200`) gera referência mutável
  a estático sem travar; o compilador atual ainda emite `static_mut_refs`.
- [ ] Cada build gera 39 warnings no crate: imports/código mortos, nomes fora
  de snake_case, `generic_const_exprs` incompatível com o novo solver e
  `build.rs` vazio com `cc` sem uso. Há também o warning de configuração
  `unused config key build.targets`; a chave correta é `build.target`.
- [ ] `Cargo.lock` do kernel foi removido na reorganização e o nome continua
  coberto por `.gitignore` (`*.lock`, `Cargo.lock`); apenas
  `tools/fontGenerator/Cargo.lock` permanece rastreado. Decidir: versionar o
  lock raiz fora do ignore, ou ignorá-lo de verdade. Para kernel, lock
  commitado é a escolha sensata.
- [ ] `KERNEL_HEAP_SIZE_LIMIT` existe mas o valor 32 MiB está repetido inline
  (`source/memory.rs:20`, `source/memory.rs:48`).

## 4. Riscos e pontos não verificados

- Boot x86_64 em QEMU: executado até a tela de boas-vindas. Prompt, comandos,
  formatos de pixel fora do caso padrão e hardware real continuam sem teste.
- Causa atual do bloqueio ARM: o linker script não é aplicado ao target
  softfloat, comprovado pelo ELF. A hipótese NEON/exceção deve ser reavaliada
  somente depois dessa correção.
- PMM: `free_frame` não valida alinhamento, limites nem double-free
  (`source/pmm.rs:62`); `allocate_frame` faz varredura linear do bitmap
  (`source/pmm.rs:48`). Nenhum dos dois foi exercido por teste.
- Heap: usa a maior região "usable" sem alinhar explicitamente o início;
  comportamento em mapas de memória fragmentados não foi testado.
- Cor e pitch do framebuffer em hardware real ou modos != 32 bpp: não testado.
- UART ARM está fixa no endereço do QEMU `virt`; outras máquinas ARM precisam
  de DTB ou configuração (`source/definitions/io.rs:8-11`).
- `unsafe { bitmap.set_len(...) }` + escrita volátil no PMM
  (`source/pmm.rs:135`) funciona, mas é um ponto a revisitar por
  clareza/segurança, não um bug demonstrado.
- [ ] Desempenho: `swap_buffers` copia pixel a pixel com escrita volátil
  (`source/video.rs:192`), e `set_pixel` faz blend mesmo após escrever
  cor opaca (`source/video.rs:166`). Medir antes de otimizar.
- QEMU no PC: x86_64 e AArch64 foram executados com TCG e timeout. O alvo
  `make run`, KVM opcional, display GTK e VNC ainda não foram validados de
  ponta a ponta.
- VNC no Termux: o QEMU instalado anuncia suporte a VNC e websocket em
  `-vnc help`, mas nenhuma sessão de cliente foi testada ainda.
- WebAssembly: `qemu-wasm` declara suporte experimental a guests x86_64 e
  AArch64, mas nenhuma imagem do motosseraKernel foi executada nele. Suporte a
  máquina `virt`, Limine, EDK2, framebuffer e entrada deve ser provado no
  spike da Etapa 5. COOP/COEP e hospedagem estática também estão por validar.
- Touch/multitouch no VNC: depende de o cliente e o transporte preservarem
  esse tipo de evento. O QEMU ter dispositivo multitouch não prova suporte do
  cliente Android; testar antes de prometer paridade com o browser.
- Touch/multitouch no `qemu-wasm`: ainda não verificado. O browser fornece
  `pointerType` e `pointerId`, mas falta provar a tradução até o dispositivo
  virtual e o driver do kernel.

## 5. Lacunas de engenharia

- [ ] Nenhum teste unitário, de integração ou smoke automatizado. Existe um
  smoke manual com logs em scratch, mas ainda não há script/CI reproduzível.
- [ ] Nenhuma CI. `panic = "abort"` e `build-std` foram validados nos builds
  release, mas a mesma configuração quebra testes de host com `core`
  duplicado. `build.targets` também precisa ser corrigido para `build.target`.
- [ ] `rust-toolchain.toml` já declara nightly e `rust-src`, mas mantém
  `aarch64-linux-android` enquanto o Makefile usa
  `aarch64-unknown-none-softfloat`. Alinhar a lista e separar o target WASM do
  crate web do toolchain do kernel.
- [ ] Makefile tem condicionais por arquitetura, mas não oferece alvos de
  conveniência separados, stamp de configure do Limine, checagem de
  ferramentas nem `make help`.
- [ ] Código antigo comentado em massa na parte final de `source/shell.rs`;
  renomear tipos como `InputOutputSistem`, `TimeInteration`, `Vga`
  (framebuffer, não VGA) quando tocar cada módulo — sem refactor cosmético
  separado.
- [ ] Dependências pesadas para o estágio atual (Rhai + nalgebra) sem política
  de features revisada; avaliar custo de binário/RAM quando o MVP estiver
  estável, não antes.

## 6. Lacunas de documentação

- [x] README inicial e atualização da estrutura, dos builds verificados e das
  limitações atuais (`README.md`). A anotação antiga `docs/description.md` foi
  removida por estar desatualizada.
- [ ] Atualizar `AGENTS.md`: ainda contém caminhos `kernel/src/` e exemplos de
  toolchain que precisam refletir `source/` e o target ARM softfloat.
- [ ] Expandir o README após a Etapa 1 com os comandos exatos e validados de
  boot por arquitetura.
- [ ] Explicação curta da arquitetura: ordem de init, mapa de memória,
  convenção de cores, papel da serial vs. terminal gráfico.
- [ ] Registrar a decisão de escopo: shell interativo via serial e/ou terminal
  gráfico, e o papel do Rhai (linguagem de comandos do kernel?).
- [ ] Guias de execução: QEMU no PC, VNC local no Termux, build/execução WASM
  no navegador e requisitos de headers para o deploy.
- [ ] Mapa de entrada por plataforma: teclado físico/virtual, mouse, tablet e
  touch; limitações reais de VNC e WebAssembly.
- [ ] Documentar origem, autoria e termos das fontes TTF incluídas. A licença
  MIT do código original (`LICENSE`) não se aplica automaticamente a elas;
  dependências e recursos de terceiros mantêm seus próprios termos.

## 7. Funcionalidade ausente (escolhas de escopo, não defeitos)

- Entrada por teclado PS/2 (structs de scancode existem, sem consumo),
  mouse/tablet/touch, teclado virtual, interrupções, timer real, scheduler,
  userspace, storage/VFS, rede, SMP.
- Nenhum desses entra no MVP. O produto mínimo é: boot reproduzível + logs +
  shell serial com alguns comandos + avaliação Rhai com resultado impresso.
  Terminal gráfico é a etapa seguinte, não pré-requisito do shell.

## 8. Etapas sugeridas

### Etapa 0 — decisões tomadas com o autor

- [x] Manter x86_64 E AArch64 desde já, aceitando o custo do stub ARM na
  Etapa 1 (resposta do Davi). Nenhuma arquitetura fica "para depois".
- [x] MVP = console serial interativo + Rhai; framebuffer só com banner
  estático até a Etapa 4.
- [x] Objetivo do projeto: um kernel autoral, com identidade visual e
  funcionalidades inventadas pelo autor ("corpo artístico"), aprendendo o
  máximo de baixo nível no caminho — mas com exigência explícita de boa
  performance, eficiência e estrutura modular. Ou seja: a estética entra como
  requisito de produto, não como desculpa para pular a base de OS.
- [x] Execução em três ambientes (decisão do Davi): QEMU nativo no PC, QEMU
  headless com VNC no Termux e WebAssembly no navegador para x86_64 E
  AArch64. O navegador executa a mesma imagem bootável emulada, não uma
  reescrita do kernel para `wasm32-unknown-unknown`.
- [x] Entrada futura: touch e mouse devem permanecer dispositivos/eventos
  distintos, inclusive diante de um click. O sistema terá teclado virtual,
  mas isso não entra no MVP nem substitui o suporte a teclado físico.

Saída: decisões registradas no README.

### Etapa 1 — build e boot reproduzíveis

- [x] Restaurar um target válido no `Cargo.toml` apontando para
  `source/main.rs`, integrar `source/definitions/` e repetir os builds release
  das duas arquiteturas. Ambos passaram com 39 warnings.
- [x] Adicionar `rust-toolchain.toml` com nightly e `rust-src`, além dos
  perfis `panic = "abort"` e `build-std`. Resta alinhar os targets ARM e a
  configuração de testes em host.
- [ ] Concluir a correção do Makefile e da configuração: as ISOs x86_64 e
  AArch64 são geradas com RAM >= 256M e EFI por arquitetura; aplicar o linker
  ARM ao target softfloat, alinhar `rust-toolchain.toml` e tornar KVM opcional
  com fallback TCG.
- [ ] Tornar o bootstrap do Limine idempotente e documentar dependências. O
  configure/build atual passou para x86_64 BIOS/UEFI e AArch64 UEFI, mas é
  reexecutado a cada chamada.
- [ ] x86_64: validar o log `[BOOT] A` até o shell. O boot atual alcança
  `welcome`, mas não houve prompt nem comandos no smoke de 20 s.
- [ ] ARM: aplicar o linker correto e, depois, implementar stub de entrada que
  habilita FP/SIMD e vetor de exceções mínimo com log. Reavaliar a hipótese
  NEON somente com o ELF corretamente linkado.
- [ ] Transformar o smoke manual em teste automatizado com timeout e assertivas
  sobre a serial das duas arquiteturas.
- [ ] QEMU nativo no PC: alvo `make run ARCHI=...`, com aceleração opcional
  detectada (KVM quando existir; TCG sempre disponível), firmware EDK2
  configurável por variável de ambiente e logs separados por arquitetura.
- [ ] VNC no Termux: alvo `make run-vnc ARCHI=...` usando `-display none`,
  VNC local e websocket; documentar endereço/porta, cliente VNC no Android e
  encerramento limpo do QEMU. Não exigir X11 nem túnel externo para uso local.
- [ ] Sem VNC exposto à internet sem autenticação: qualquer acesso remoto
  deve ser explícito, protegido e documentado separadamente do modo local.

Saída: `make run ARCHI=...` no PC e `make run-vnc ARCHI=...` no Termux
chegam ao mesmo marco de log, num checkout limpo, com comandos documentados.

### Etapa 2 — memória e tempo confiáveis

- [ ] Extrair o bitmap do PMM e o alocador para código testável em host;
  testes de alocação/liberação, esgotamento, double-free e alinhamento.
- [ ] Separar a configuração de testes em host do runtime bare-metal,
  incluindo `build-std` e panic handler, sem apagar o handler do kernel.
- [ ] Reservar explicitamente heap/bitmap no PMM e validar entradas do Limine.
- [ ] IDT no x86_64 + PIT (ou HPET) para ticks reais; remover a falsa noção de
  tempo de `TimeInteration` ou renomeá-la para contador de frames.

Saída: `cargo test` verde em host e contador de tempo avançando no QEMU.

### Etapa 3 — shell serial de ponta a ponta

- [ ] Uma única instância de serial, com API não bloqueante de leitura e um
  console que faça eco, trate CR/LF, backspace e histórico por `push`.
- [ ] Rhai avaliando `Dynamic` e formatando resultado/erro com posição;
  comandos embutidos mínimos: `help`, `clear`, `mem`, `version`.
- [ ] Nunca inserir o prompt na linha de entrada; separar prompt, input,
  histórico e saída.

Saída: no QEMU, digitar `1 + 2` produz `3`, um erro de sintaxe mostra a
mensagem real, e `mem` imprime estatísticas do heap — verificado por script.

### Etapa 4 — terminal gráfico correto

- [ ] Unificar célula/fonte em 16x16 (ou parametrizar), corrigir x/y,
  scrolling com limites e dimensões vindas do framebuffer real.
- [ ] Respeitar bpp/máscaras de cor do Limine ou rejeitar modos não suportados
  com log explícito.
- [ ] Redesenhar apenas regiões sujas; medir FPS antes de qualquer otimização
  mais sofisticada.

Saída: o mesmo shell da Etapa 3 visível no framebuffer, com rolagem sem panic
em resolução menor que 1920x1080.

### Etapa 5 — WebAssembly no navegador, nas duas arquiteturas

Escopo: usar um emulador de sistema compilado para WebAssembly para executar a
ISO/ELF existente. Compilar o próprio kernel para WASM seria outro projeto e
não atende ao requisito de rodar o mesmo sistema bare-metal.

Estado atual: `kernel/web/index.html` é apenas um esqueleto que importa um
wrapper JS ainda não gerado. Não há build `qemu-wasm`, integração da ISO,
servidor com COOP/COEP ou testes web.

- [ ] Prova de viabilidade x86_64 com `ktock/qemu-wasm` (experimental), usando
  um runtime publicado ou build containerizado do projeto; testar a imagem
  inalterada antes de integrar qualquer frontend.
- [ ] Prova de viabilidade AArch64 com o mesmo runtime, máquina `virt` e o
  firmware/EDK2 necessário; empacotar ISO, firmware e arquivos auxiliares.
  O fato de haver demo AArch64 (Raspberry Pi) não prova que nossa máquina ou
  nosso kernel já funciona.
- [ ] Decidir o runtime após o spike: priorizar `qemu-wasm` para compartilhar
  configuração e comportamento entre archs. Alternativa apenas para x86_64:
  avaliar `portlandhodl/v86_64`; v86 upstream fica descartado porque não
  suporta kernels de 64 bits. Não misturar emuladores sem necessidade.
- [ ] Frontend web único, em diretório separado do kernel, com xterm.js para
  a serial no MVP e canvas para o framebuffer na Etapa 7; teclado vai para o
  dispositivo de entrada correto quando esse driver existir.
- [ ] Configuração declarativa por arquitetura compartilhada pelo build web e
  pelos comandos QEMU: imagem, memória, firmware, máquina e dispositivos.
- [ ] Build reproduzível dos artefatos `.wasm`, JavaScript, workers e imagens;
  servir com os headers exigidos pelos workers/`SharedArrayBuffer` (COOP/COEP)
  e documentar compatibilidade real de navegadores.
- [ ] Worker separado para emulação e controles de iniciar/pausar/resetar;
  medir tempo até o primeiro log, FPS e consumo de RAM. Evitar travar a aba.
- [ ] Deploy estático somente depois de validar localmente no Chrome/Firefox
  do PC e no Chrome do Android, incluindo interação por toque/teclado.
- [ ] GitHub Pages precisa de suporte a COOP/COEP para threads; se o runtime
  exigir headers que a hospedagem não permite, escolher build sem threads ou
  outra hospedagem estática configurável. Não prometer Pages antes do spike.
- [ ] Testes web com timeout e assertiva sobre o log serial de cada arch;
  publicar os logs e limitações junto do demo, não só uma captura bonita.

Saída: uma página local que inicializa x86_64 e AArch64 por seleção explícita,
mostra o shell serial da Etapa 3 e permite enviar comandos, sem backend de
aplicação. Deploy público é opcional até essa prova passar.

### Etapa 6 — entrada física, touch distinto de mouse e teclado virtual

Esta etapa começa somente depois de o console funcionar e de existir um timer
confiável. Ela não altera o console serial do MVP nem usa touch para fingir
que existe um mouse.

- [ ] Criar um modelo de eventos com origem explícita (`keyboard`, `mouse`,
  `tablet`, `touch`), fase (`press`, `move`, `release`, `cancel`), posição e
  identificador do contato. Click de mouse e toque nunca compartilham o mesmo
  evento genérico; qualquer atalho por conveniência é uma política da UI, não
  uma perda de informação no driver.
- [ ] Definir o caminho virtio-input: descoberta/configuração PCI, virtqueues,
  IRQ, eventos EV_KEY/EV_REL/EV_ABS e `EV_SYN`, validado primeiro com logs.
  O QEMU instalado expõe `virtio-keyboard-pci`, `virtio-mouse-pci`,
  `virtio-tablet-pci` e `virtio-multitouch-pci`.
- [ ] Suportar teclado físico antes do virtual: layout inicial documentado,
  teclas mantidas/repetição, modificadores e mapeamento para o editor do
  shell. PS/2 só entra como compatibilidade adicional se for necessário.
- [ ] Implementar mouse e tablet separadamente: movimento relativo/absoluto,
  botões e scroll; não converter `usb-tablet` ou `virtio-tablet` em touch.
- [ ] Implementar touch com tracking ID/slot, início, movimento, fim e
  cancelamento; primeiro um contato, depois múltiplos. Gestos (tap, long
  press, swipe, pinch) ficam numa camada própria e usam o timer real.
- [ ] Criar um painel de diagnóstico que mostre origem, ID, coordenadas e fase
  dos eventos; testar mouse e touch alternados e simultâneos para provar que
  a origem não se perde.
- [ ] Teclado virtual renderizado no framebuffer, visível sob demanda, com
  foco de texto, modificadores, backspace/enter e layout adaptado a toque.
  As teclas pressionadas entram no mesmo pipeline lógico do teclado físico,
  mas continuam identificadas como virtuais para a UI/diagnóstico.
- [ ] Touch no teclado virtual usa captura do contato até `release`/`cancel`,
  evitando ativação dupla por eventos de compatibilidade e impedindo que o
  mesmo toque acione também o conteúdo atrás do teclado.
- [ ] No browser, mapear Pointer Events sem descartar `pointerType`,
  `pointerId` e `isPrimary`; impedir os mouse events de compatibilidade após
  touch e traduzir para o dispositivo virtual apropriado. A distinção existe
  no browser, mas precisa ser testada também dentro do emulador.
- [ ] Verificar VNC: mouse/tablet funciona como caminho básico; touch real e
  múltiplos contatos dependem de cliente/protocolo. Se o cliente escolhido só
  enviar pointer de mouse, documentar essa limitação em vez de declarar touch
  suportado. Não exigir passthrough de `/dev/input` do Android.
- [ ] Testes unitários para normalização de coordenadas, ciclo de contato,
  gestos e layout; testes QEMU via QMP `input-send-event` com multitouch;
  testes manuais no PC, cliente VNC Android e browser Android.

Saída: uma demo gráfica em que mouse move/clica, touch arrasta/toca com outra
semântica visível, dois dedos têm slots independentes, e o teclado virtual
escreve no shell sem gerar eventos duplicados.

### Etapa 7 — base para crescer

- [ ] CI com build das duas arquiteturas, testes em host e smoke de boot.
- [ ] `cargo clippy`/`fmt` no CI e redução dos warnings para perto de zero.
- [ ] Limpar código morto e decidir o lockfile; regenerar fontes via alvo de
  build documentado.
- [ ] Só então escolher o próximo eixo, nesta ordem de preferência:
  (a) base de OS — interrupções, drivers, scheduler;
  (b) identidade autoral — tema visual, fontes, efeitos e as funcionalidades
  inventadas pelo autor, como camada separada do núcleo;
  (c) performance medida (tempo de boot, custo por frame, alocações por
  comando) com orçamento explícito, não otimização no escuro.

Saída: pipeline verde e uma decisão explícita de produto para o próximo ciclo.

## 9. Critérios temporários de aceite do MVP

O MVP está pronto quando TODOS forem verdade, sem ressalva:

- [ ] `git clone` limpo + comandos do README produzem a ISO sem edição manual.
- [ ] `make run ARCHI=x86_64` boota no QEMU e prova init de serial, heap, PMM e
  vídeo por logs; falha de qualquer etapa tem mensagem clara.
- [ ] No console serial: prompt visível, eco correto, backspace correto,
  Enter executa e sempre retorna a um novo prompt.
- [ ] `1 + 2` imprime `3`; `"a" + "b"` imprime `ab`; erro de sintaxe imprime
  diagnóstico real (não a string `"erro"`).
- [ ] `help`, `mem` e `version` respondem; `mem` reflete alocações.
- [ ] `cargo test` passa em host, incluindo casos de fronteira do PMM.
- [ ] `make run ARCHI=aarch64` atinge o mesmo marco de boot no QEMU `virt` +
  UEFI (decisão do autor: as duas arquiteturas são suportadas desde o MVP).
  Nunca apresentar uma arquitetura como funcionando só porque compila.
- [ ] `make run-vnc ARCHI=x86_64` e `make run-vnc ARCHI=aarch64` funcionam no
  Termux com cliente VNC local, e o mesmo banner é visto no cliente.
- [ ] No navegador, o usuário seleciona x86_64 ou AArch64, a imagem inicializa
  no emulador WASM e o shell serial aceita `1 + 2`, retornando `3`. O teste é
  repetido no PC e no Android; restrições de navegador ficam documentadas.
- [ ] Nenhuma afirmação de "funciona" sem log ou teste anexado.

## 10. Depois do MVP: entrada touch e teclado virtual

Estes critérios pertencem à Etapa 6, não bloqueiam o shell serial:

- [ ] Um click de mouse chega com origem `mouse`, botão e coordenadas; um
  toque chega com origem `touch`, tracking ID/slot e fase própria.
- [ ] Mouse e touch podem ocorrer alternadamente sem trocar a origem nem
  produzir click/toque duplicado por eventos de compatibilidade.
- [ ] O teclado virtual abre/fecha, respeita foco e escreve no mesmo editor do
  shell usado pelo teclado físico, inclusive no browser Android.
- [ ] VNC documenta a capacidade efetiva do cliente testado; touch real não é
  reivindicado quando o transporte entrega apenas pointer de mouse.
