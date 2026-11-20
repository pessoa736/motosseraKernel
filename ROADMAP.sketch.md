# ROADMAP.sketch — motosseraKernel

Documento temporário de análise e priorização, gerado em 2026-11-20 a partir do
commit `e66e9be` (branch `master`) mais a alteração local não commitada em
`kernel/Cargo.lock`. Não é um plano com prazos; é uma fila de trabalho para
revisar a cada etapa concluída.

## 1. Estado do projeto

Kernel bare-metal hobby em Rust `#![no_std]`, com Limine como bootloader, heap
próprio, PMM por bitmap, framebuffer com backbuffer, serial para debug, um shell
com Rhai embutido e um gerador de fontes TTF separado.

Avaliação direta: hoje é um esqueleto de boot com bastante código pela frente,
não um sistema utilizável. O x86_64 compila com ajustes de ambiente; o caminho
ARM que o Makefile oferece não linka; e o shell, mesmo quando o boot progride,
não exibe texto nem resultados. O maior ganho agora não é adicionar recurso:
é fechar um fluxo mínimo de ponta a ponta (boot -> log -> comando -> resposta)
e só depois expandir.

## 2. O que já existe e o que foi verificado

### Produto e arquitetura (inspecionado)

- Entrada `_start` em `kernel/src/main.rs:183`, requests Limine (base revision,
  stack, framebuffer, HHDM, memmap) e inicialização em ordem: serial -> heap ->
  PMM -> vídeo -> shell (`kernel/src/main.rs:63`).
- Heap com `linked_list_allocator` sobre a maior região `MEMMAP_USABLE`,
  limitada a 32 MiB (`kernel/src/memory.rs:28`).
- PMM por bitmap com `allocate_frame`/`free_frame` (`kernel/src/pmm.rs:101`).
- Vídeo com `DMatrix<u32>` de backbuffer, alpha blending e fonte bitmap 16x16
  (`kernel/src/video.rs:96`, `kernel/src/db_font.rs`).
- Serial x86 (COM1) e ARM (PL011 em `0x0900_0000`) para I/O e log
  (`kernel/src/io.rs`, `kernel/src/debuglog.rs`).
- Shell com `TermUI`, histórico, buffer de saída e `rhai::Engine`
  (`kernel/src/shell.rs:96`).
- Gerador de fontes TTF -> `[[u16;16];128]` (`tools/fontGenerator`).

### Executado nesta análise

- `cargo build --release --target x86_64-unknown-none -Zbuild-std=core,alloc`
  com `CARGO_PROFILE_RELEASE_PANIC=abort`: PASSOU, com 43 warnings.
  Sem `-Zbuild-std` falha neste Termux (`can't find crate for core`).
- `cargo build --release --target aarch64-unknown-none -Zbuild-std=core,alloc`
  com `CARGO_PROFILE_RELEASE_PANIC=abort`: PASSOU, com 54 warnings.
- `cargo build --release --target aarch64-linux-android ...`: FALHOU no link:
  `cc: error: unknown argument: '-zmax-page-size=0x1000'`.
- `cargo test --target aarch64-linux-android`: FALHOU,
  `duplicate lang item panic_impl` (`kernel/src/debuglog.rs:62`).
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

## 3. Bugs e inconsistências confirmados, por impacto

### P0 — impede boot ou uso básico

- [ ] Makefile ARM usa o alvo errado: `aarch64-linux-android` para um kernel
  bare-metal (`kernel/Makefile:12`). Além de ser um target de userspace
  Android, o link falha porque o driver `cc` rejeita `-zmax-page-size`
  (`kernel/.cargo/config.toml:11`). O alvo coerente é `aarch64-unknown-none`,
  que compilou nesta análise.
- [ ] Boot ARM trava antes do primeiro log do kernel (reproduzido em QEMU).
  Evidência forte de causa: o Limine na base revision 6 entrega AArch64 com
  FP/SIMD desativado (CPACR_EL1 = 0), mas o `_start` gerado pelo Rust executa
  NEON logo após `boot_marker` (`movi v0.2d`, `ldr q...`, `stp q...`), e não
  há vetor de exceções instalado — o trace do QEMU termina em laço de
  `Prefetch Abort` em `0x200`. Tratar como causa provável muito bem
  fundamentada; confirmar com single-step/GDB ou com o stub da etapa 1.
  Endereço de entrada observado: `0xffffffff8020f2b4`.
- [ ] 128 MiB padrão do Makefile não basta para o Limine carregar este kernel
  em ARM/UEFI (`kernel/Makefile:2`). Com 256 MiB o carregamento passa.
- [ ] Shell não produz saída visível nem fecha o ciclo de comando:
  `TermUI::writechar` está vazio (`kernel/src/shell.rs:69`), nada escreve no
  `buffer` gráfico para renderizar, e o prompt é inserido na própria linha de
  entrada (`kernel/src/shell.rs:229`). Após Enter, `status` permanece em 1,
  então nem resultado nem novo prompt aparecem (`kernel/src/shell.rs:226`).
- [ ] `cargo test` não compila por causa do `#[panic_handler]` no binário de
  testes (`kernel/src/debuglog.rs:62`). Não existe nenhum teste hoje, então
  a suíte precisa de estrutura, não só de correção pontual.
- [ ] `make limine` falha após a mudança de caminho do projeto; o Makefile
  também sempre copia `BOOTX64.EFI`, mesmo em ARM (`kernel/Makefile:52`), e
  sempre força BIOS + UEFI x86 no configure (`kernel/Makefile:32`).

### P1 — comportamento errado quando o caminho é exercitado

- [ ] `make run` usa opções x86 para qualquer arquitetura: `-machine q35`,
  `-cpu host,+topoext`, `-enable-kvm` (`kernel/Makefile:68`). Em ARM precisa
  de `virt`, firmware UEFI e CPU válida; KVM não é portátil.
- [ ] A interface Rhai só aceita resultado `String`: `eval::<String>`
  (`kernel/src/shell.rs:212`) faz `1 + 2` cair no ramo de erro, e todo erro
  vira a string genérica `"erro"` (`kernel/src/shell.rs:217`).
- [ ] `clear_input_line` sobrescreve bytes com espaços mas não chama
  `String::clear` (`kernel/src/shell.rs:142`), deixando `len()` antigo;
  `current_line_pos` nunca é atualizado, e `save_in_historic_command` usa
  `insert(pos)` com essa posição (`kernel/src/shell.rs:151`).
- [ ] Geometria do terminal é incompatível com a fonte: `TermUI` usa células
  de 8 px (`kernel/src/shell.rs:13`), enquanto `draw_char` desenha 16x16 e
  `draw_string` avança 16 px (`kernel/src/video.rs:238`, `kernel/src/video.rs:261`).
- [ ] `TermUI::draw` chama `draw_char` com x/y trocados e subtrai `view_pos`
  (células) como se fosse pixels (`kernel/src/shell.rs:88`);
  `view_mut(view_pos, self.size)` com scroll > 0 estoura os limites da matriz
  (`kernel/src/shell.rs:79`).
- [ ] `Vga::init` ignora o retorno `bool` no chamador (`kernel/src/main.rs:80`)
  e aloca backbuffer fixo de 1920x1080 antes de saber o modo real
  (`kernel/src/video.rs:124`). Falha de framebuffer vira uso posterior de
  `unwrap`, não diagnóstico.
- [ ] Escrita de pixels assume layout ARGB e 32 bpp, ignorando `bpp`,
  `red_mask_shift`, `blue_mask_shift` etc. do Limine (`kernel/src/video.rs:138`).
  Funciona no caso comum do QEMU, mas não é o contrato completo do protocolo.
- [ ] Não há IDT/vetor de exceções nem fonte de interrupção: `TIMER_TICKS`
  nunca é incrementado (`kernel/src/timer.rs:7`), logo `Clock` não mede tempo
  real. `TimeInteration` conta iterações de loop, não duração
  (`kernel/src/timer.rs:82`) — a intro "de 1 s" é 1000 iterações.
- [ ] Laços de espera ocupam 100% da CPU: loop principal (`kernel/src/main.rs:105`),
  `panic` (`kernel/src/debuglog.rs:69`), polling da serial (`kernel/src/io.rs:53`).

### P2 — higiene e manutenção

- [ ] Serial inicializada duas vezes: existe `Kernel.io` e outra instância em
  `Shell.io`, ambas com `init` (`kernel/src/main.rs:59`, `kernel/src/shell.rs:100`).
  O driver de debug em `debuglog.rs` é uma terceira implementação, sem
  espera de transmissão pronta.
- [ ] `panic_handler` só imprime detalhes em x86_64; em ARM entra mudo no loop
  (`kernel/src/debuglog.rs:61`).
- [ ] `unsafe impl Sync for Kernel` + `static mut` + `Mutex::get_mut`
  (`kernel/src/main.rs:172`, `kernel/src/main.rs:179`) gera referência mutável
  a estático sem travar; o compilador já emite `static_mut_refs`.
- [ ] 43–54 warnings por build: imports mortos, código morto, nomes fora de
  snake_case, `generic_const_exprs` incompatível com o novo solver
  (`kernel/src/main.rs:3`), `build.rs` vazio com dependência `cc` sem uso.
- [ ] `Cargo.lock` está versionado mas também coberto por `.gitignore`
  (`*.lock`, `Cargo.lock`), e a alteração local com nalgebra/rhai/spin ainda
  não foi commitada. Decidir: lock rastreado e fora do ignore, ou ignorado de
  verdade. Para kernel, lock commitado é a escolha sensata.
- [ ] `KERNEL_HEAP_SIZE_LIMIT` existe mas o valor 32 MiB está repetido inline
  (`kernel/src/memory.rs:20`, `kernel/src/memory.rs:48`).

## 4. Riscos e pontos não verificados

- Boot x86_64 em QEMU: não executado (emulador ausente neste ambiente).
  A ISO foi gerada e o BIOS instalado; nada além disso está provado.
- Causa exata do travamento ARM: cadeia NEON/exceção é a hipótese com melhor
  evidência; ainda não foi demonstrada por depuração instrução a instrução.
- PMM: `free_frame` não valida alinhamento, limites nem double-free
  (`kernel/src/pmm.rs:62`); `allocate_frame` faz varredura linear do bitmap
  (`kernel/src/pmm.rs:48`). Nenhum dos dois foi exercido por teste.
- Heap: usa a maior região "usable" sem alinhar explicitamente o início;
  comportamento em mapas de memória fragmentados não foi testado.
- Cor e pitch do framebuffer em hardware real ou modos != 32 bpp: não testado.
- UART ARM está fixa no endereço do QEMU `virt`; outras máquinas ARM precisam
  de DTB ou configuração (`kernel/src/io.rs:9`).
- `unsafe { bitmap.set_len(...) }` + escrita volátil no PMM
  (`kernel/src/pmm.rs:135`) funciona, mas é um ponto a revisitar por
  clareza/segurança, não um bug demonstrado.
- [ ] Desempenho: `swap_buffers` copia pixel a pixel com escrita volátil
  (`kernel/src/video.rs:202`), e `set_pixel` faz blend mesmo após escrever
  cor opaca (`kernel/src/video.rs:176`). Medir antes de otimizar.
- QEMU no PC: não testado nesta análise; os comandos serão projetados para
  TCG como fallback, mas precisam ser validados no hardware dele.
- VNC no Termux: o QEMU instalado anuncia suporte a VNC e websocket em
  `-vnc help`, mas nenhuma sessão de cliente foi testada ainda.
- WebAssembly: `qemu-wasm` declara suporte experimental a guests x86_64 e
  AArch64, mas nenhuma imagem do motosseraKernel foi executada nele. Suporte a
  máquina `virt`, Limine, EDK2, framebuffer e entrada deve ser provado no
  spike da Etapa 5. COOP/COEP e hospedagem estática também estão por validar.

## 5. Lacunas de engenharia

- [ ] Nenhum teste unitário, de integração ou smoke de boot.
- [ ] Nenhuma CI; builds dependem de flags não registradas
  (`-Zbuild-std=core,alloc`, `CARGO_PROFILE_RELEASE_PANIC=abort`).
- [ ] Sem `rust-toolchain.toml`: o projeto exige nightly e componentes
  `rust-src` (e alvos), mas isso só existe na máquina do autor.
- [ ] Makefile sem alvos separados por arquitetura, sem stamp de configure do
  Limine, sem checagem de ferramentas e sem `make help`.
- [ ] Código antigo comentado em massa em `kernel/src/shell.rs:256` em diante;
  renomear tipos como `InputOutputSistem`, `TimeInteration`, `Vga`
  (framebuffer, não VGA) quando tocar cada módulo — sem refactor cosmético
  separado.
- [ ] Dependências pesadas para o estágio atual (Rhai + nalgebra) sem política
  de features revisada; avaliar custo de binário/RAM quando o MVP estiver
  estável, não antes.

## 6. Lacunas de documentação

- [ ] README com requisitos (nightly, rust-src, QEMU, xorriso, Limine),
  comandos exatos por arquitetura e como bootar.
- [ ] Explicação curta da arquitetura: ordem de init, mapa de memória,
  convenção de cores, papel da serial vs. terminal gráfico.
- [ ] `docs/description.md` existe, mas descreve intenção, não o estado real;
  atualizar após o MVP.
- [ ] Registrar a decisão de escopo: shell interativo via serial e/ou terminal
  gráfico, e o papel do Rhai (linguagem de comandos do kernel?).
- [ ] Guias de execução: QEMU no PC, VNC local no Termux, build/execução WASM
  no navegador e requisitos de headers para o deploy.

## 7. Funcionalidade ausente (escolhas de escopo, não defeitos)

- Entrada por teclado PS/2 (structs de scancode existem, sem consumo),
  interrupções, timer real, scheduler, userspace, storage/VFS, rede, SMP.
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

Saída: registrar essas decisões no README quando ele for criado.

### Etapa 1 — build e boot reproduzíveis

- [ ] Adicionar `rust-toolchain.toml` (nightly + `rust-src` + alvos) e perfil
  `panic = "abort"`; registrar o uso de `-Zbuild-std=core,alloc`.
- [ ] Corrigir o Makefile: `aarch64-unknown-none`, RAM >= 256M, EFI por
  arquitetura (`BOOTAA64.EFI` vs `BOOTX64.EFI`), flags de QEMU separadas,
  sem KVM obrigatório (TCG no Android/Termux).
- [ ] Reconfigurar/regerar o Limine no caminho atual e transformar bootstrap
  em alvo idempotente; documentar dependências (`nasm`, `mtools`, etc.).
- [ ] x86_64: instalar QEMU x86 e validar o log `[BOOT] A` até o shell.
- [ ] ARM: stub de entrada em assembly que habilita FP/SIMD antes de chamar
  Rust, e confirma a hipótese NEON; adicionar vetor de exceções mínimo com
  log, para que falhas deixem de ser um laço mudo em `0x200`.
- [ ] Smoke test de boot com timeout e assertiva sobre o log serial.
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
- [ ] Resolver o conflito do panic handler nos testes (separar runtime
  bare-metal de biblioteca testável) em vez de apagar o handler.
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
  a serial no MVP e canvas para o framebuffer na Etapa 6; teclado vai para o
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

### Etapa 6 — base para crescer

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
