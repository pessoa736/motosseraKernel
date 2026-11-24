# ROADMAP — motosseraKernel

Fila de trabalho do projeto, na ordem em que deve ser executada. Não tem prazos.
Revise a cada etapa concluída. Atualizado em 2026-11-23.

**Como ler**

- `[x]` só com prova (comando executado, log ou leitura do ELF). `[ ]` é pendente.
- "Reconferido" = lido/rodado em 2026-11-23. "Da análise anterior" = registrado
  em 2026-11-20/21 e não repetido hoje (principalmente os boots em QEMU).
- Dúvidas sem resposta em [DUVIDAS.md](DUVIDAS.md) não são decisões aprovadas.
- Cada pendência aparece em **uma** etapa. Bugs foram movidos para a etapa que
  os corrige, não ficam mais numa lista separada.

---

## 1. Estado atual

Kernel bare-metal `#![no_std]` em Rust, com Limine, heap próprio, PMM por bitmap,
framebuffer com backbuffer, serial, shell com Rhai e gerador de fontes TTF.
Código em `source/`, constantes em `source/definitions.rs`, manifesto na raiz,
workspace com `xtask/`.

| Área | Estado | Evidência |
|---|---|---|
| Build release x86_64 | Passa (37 warnings no crate + 2 do manifesto) | reconferido |
| Build release AArch64 softfloat | Passa (mesma contagem) | reconferido |
| Linker ARM | **Aplicado**: entrada `0xffffffff80009598` (antes `0x28c6f0`) | `readelf`, reconferido; config commitada em `f66d14e` |
| Linker x86_64 | Entrada `0xffffffff80209410` | `readelf`, reconferido |
| Boot x86_64 (QEMU) | Chega a serial, PMM, framebuffer e `welcome`; sem prompt | da análise anterior |
| Boot AArch64 (QEMU) | Só log do firmware, sem `[ BOOT ] A` | da análise anterior, **antes** do linker corrigido; não repetido |
| Shell | Não fecha o ciclo comando -> resposta | código inspecionado |
| `cargo test` em host | Falha (`core` duplicado por `build-std` global) | da análise anterior |
| Testes / CI | Inexistentes | — |
| Web (WASM) | Nada: `kernel/web/index.html` foi apagado (não commitado) | `git status` |

Ferramentas presentes neste PC (Arch Linux): `qemu-system-x86_64`,
`qemu-system-aarch64`, `xorriso`, `nasm` e `/usr/share/edk2/aarch64/QEMU_EFI.fd`.

**Hoje o gargalo é:** provar o boot ARM com o linker correto, chegar ao prompt no
x86_64 e fechar o ciclo do shell. Nada além disso deve ser começado antes.

### Mudanças locais não commitadas que afetam este roadmap

`rust-toolchain.toml`, `limine.conf`,
`source/{main,io,debuglog,definitions}.rs`, remoção de `kernel/web/index.html` e
`limine.h`. Arquivos soltos: `catkernelsource.txt`, `catxtasksource.txt`, `log`,
`source/shell.rs.bak`. Não foram tocados por esta reorganização.

---

## 2. Decisões já tomadas

- [x] **x86_64 e AArch64 desde o MVP.** Nenhuma arquitetura fica "para depois".
- [x] **MVP = console serial interativo + Rhai.** Framebuffer só com banner até a
  Etapa 4.
- [x] **Projeto autoral e de aprendizado**, com identidade visual própria, mas com
  exigência de performance, eficiência e modularidade. Estética é requisito de
  produto, não desculpa para pular a base de OS.
- [x] **Três ambientes:** QEMU nativo no PC, QEMU headless + VNC no Termux,
  WebAssembly no navegador. O navegador roda a **mesma imagem bootável** emulada,
  não o kernel recompilado para `wasm32`.
- [x] **Touch ≠ mouse**, em todas as camadas do driver. Teclado virtual é futuro e
  não substitui o teclado físico.

### Decisão pendente que bloqueia a Etapa 1

- [ ] **Makefile ou `xtask`?** O commit `49b22ca` adicionou o `xtask`
  ("sem necessidade do makefile"), mas o `Makefile` continua rastreado e README e
  AGENTS ainda documentam `make run ARCHI=...`. Escolher um como interface
  oficial e remover ou rebaixar o outro. Até lá, não corrigir os dois.

---

## 3. Etapas

### Etapa 1 — build e boot reproduzíveis

**Objetivo:** num checkout limpo, um comando gera a ISO e boota nas duas
arquiteturas com o mesmo marco de log.

Toolchain e configuração:

- [x] Binário declarado no `Cargo.toml`, `panic = "abort"` e `build-std`
  configurados no projeto (não em variável de shell).
- [x] Linker ARM aplicado ao target softfloat (ver tabela do §1); commitado em
  `f66d14e`.
- [ ] Alinhar `rust-toolchain.toml`: ainda lista `aarch64-linux-android` e
  `aarch64-unknown-none`; deixar só os targets realmente usados. Separar o target
  `wasm32` do toolchain do kernel (vai para o crate web, Etapa 5).
- [ ] Decidir e versionar o `Cargo.lock` da raiz: hoje existe, mas o
  `.gitignore` (`*.lock`, `Cargo.lock`) o ignora (confirmado com
  `git check-ignore`). Para kernel, commitar é a escolha sensata.
- [ ] Remover `cc` de `[build-dependencies]` e o `build.rs` vazio (warning do
  manifesto).

Boot AArch64:

- [ ] **Repetir o boot ARM com o linker correto** (QEMU `virt`, EDK2, >= 256 MiB)
  e registrar o resultado. É o primeiro passo da etapa: a hipótese anterior
  (NEON/exceção em `0x200`) só vale se o boot continuar mudo agora.
- [ ] Se continuar mudo: stub de entrada em assembly que habilita FP/SIMD antes
  do Rust (Limine base revision 6 entrega FP/SIMD desligado) e vetores de exceção
  que logam. `_start` em `source/main.rs:194` ainda não tem stub nem vetores.
- [ ] UART PL011 fixa em `0x0900_0000` (`source/definitions.rs:10`): aceitável só
  para QEMU `virt`; documentar a limitação.

Boot x86_64:

- [ ] Medir quanto tempo dura a intro e validar a transição `welcome` -> loop do
  shell (o smoke de 20 s não chegou ao prompt). A intro "de 1 s" é 1000
  iterações de loop, não tempo (ver Etapa 2).

Execução e automação (depois da decisão Makefile/xtask):

- [ ] `run` no PC com aceleração **detectada**: hoje o `xtask` ativa
  `-enable-kvm` sempre que host e target coincidem (`xtask/src/main.rs`), usa
  firmware ARM em caminho fixo e exige GTK. KVM opcional com fallback TCG;
  firmware e display configuráveis por variável de ambiente.
- [ ] Serial para arquivo e `timeout --signal=KILL` externo; `qemu.log` não deve
  nascer na raiz do repositório (hoje `-D qemu.log`).
- [ ] Bootstrap do Limine idempotente: o `xtask` já tem stamp por arquitetura
  (`kernel/limine/.built-for`, `xtask/src/limine.rs`) e clona o Limine do GitHub
  se faltar; **não foi exercitado**. O Makefile ainda refaz o bootstrap a cada
  build. Documentar dependências (NASM, mtools, xorriso). O checkout
  `kernel/limine/` é gerado e ignorado: reconfigurar ao mover o projeto, não
  editar o `GNUmakefile`.
- [ ] `run-vnc` para o Termux: `-display none`, VNC local (`127.0.0.1`) e
  websocket, encerramento limpo do QEMU. Sem VNC exposto sem autenticação.
- [ ] Smoke automatizado por arquitetura: boot com timeout e assertivas sobre a
  serial (`[ BOOT ] A`, PMM, framebuffer).

**Saída:** `run` no PC e `run-vnc` no Termux chegam ao mesmo marco de log, nas
duas arquiteturas, com comandos documentados.

### Etapa 2 — memória e tempo confiáveis

**Objetivo:** alocadores testados em host e tempo real medido.

- [ ] Separar a configuração de testes em host do runtime bare-metal
  (`build-std` global quebra `cargo test`). **Não** remover o panic handler do
  kernel para isso.
- [ ] Extrair bitmap do PMM e alocador para código testável em host: alocação,
  liberação, esgotamento, alinhamento, double-free. `free_frame`
  (`source/pmm.rs:59`) não valida alinhamento, limites nem double-free;
  `allocate_frame` varre o bitmap linearmente.
- [ ] Reservar explicitamente heap, bitmap, kernel, bootloader e framebuffer no
  PMM; validar as entradas do memmap do Limine.
- [ ] Revisar `unsafe { bitmap.set_len(..) }` + escrita volátil
  (`source/pmm.rs:125`): ponto de clareza, não bug demonstrado.
- [ ] Usar `KERNEL_HEAP_SIZE_LIMIT` em vez do `32 * 1024 * 1024` repetido em
  `source/memory.rs:46-47`; testar heap com memmap fragmentado.
- [ ] x86_64: IDT + PIT (ou HPET) para ticks reais. `TIMER_TICKS`
  (`source/timer.rs:7`) existe mas é `static mut` sem fonte de interrupção;
  renomear `TimeInteration` para contador de frames ou torná-la baseada em tempo.
- [ ] AArch64: vetores de exceção úteis (se não saírem na Etapa 1) e timer
  genérico.

**Saída:** `cargo test` verde em host e contador de tempo avançando no QEMU.

### Etapa 3 — shell serial de ponta a ponta

**Objetivo:** o MVP funcional. Não depende de timer nem de vídeo.

- [ ] Uma única instância de serial. Hoje `Kernel.io` e `Shell.io` chamam `init`
  separadamente (`source/main.rs:80`, `source/shell.rs:131`).
- [ ] Leitura não bloqueante e console com eco, CR/LF, backspace e histórico.
- [ ] Separar prompt, linha de entrada, histórico e saída; o prompt nunca entra
  na string executada.
- [ ] Corrigir o editor de linha: `clear_input_line` (`source/shell.rs:140`) não
  limpa a `String`; `current_line_pos` nunca é atualizado e é usado em
  `insert(...)` do histórico e da saída (`shell.rs:152`, `shell.rs:170`).
- [ ] Rhai aceitando `Dynamic` (hoje `eval::<String>`, `source/shell.rs:210`) e
  mostrando erro real com posição, não `"erro"`.
- [ ] Comandos embutidos: `help`, `clear`, `mem`, `version`.
- [ ] Script de teste que digita comandos na serial e confere a saída.

**Saída:** no QEMU, `1 + 2` -> `3`, `"a" + "b"` -> `ab`, erro de sintaxe com
mensagem e posição, `mem` mostrando estatísticas do heap — verificado por script.

### Etapa 4 — terminal gráfico correto

**Objetivo:** o mesmo shell da Etapa 3, visível no framebuffer.

- [ ] `Vga::init` retorna `bool` ignorado em `source/main.rs:91`; tratar a falha
  com diagnóstico em vez de `unwrap` posterior.
- [ ] Backbuffer vem das dimensões reais do framebuffer, não de
  `MAX_WIDTH/MAX_HEIGHT = 1920x1080` (`source/video.rs:82-83`).
- [ ] Respeitar `bpp`, pitch e máscaras de cor do Limine, ou rejeitar o modo com
  log explícito (hoje assume 32 bpp ARGB).
- [ ] Uma geometria só para fonte e célula: `TermUI` usa células de 8 px,
  `draw_char` desenha 16x16 e `draw_string` avança 16 px.
- [ ] `TermUI::draw` (`source/shell.rs:72`): x/y trocados em `draw_char`,
  `view_pos` (células) subtraído como pixels e `view_mut` com scroll > 0 fora
  dos limites; `TermUI::writechar` (`shell.rs:67`) está vazio.
- [ ] Medir FPS e tempo de `swap_buffers` (cópia pixel a pixel com escrita
  volátil, `source/video.rs:192`) **antes** de otimizar; `set_pixel` ainda faz
  blend após gravar cor opaca (`video.rs:155`). Depois, redesenhar só regiões
  sujas.

**Saída:** shell visível, com rolagem sem panic em resolução menor que
1920x1080 e captura de tela como evidência.

### Etapa 5 — WebAssembly no navegador, nas duas arquiteturas

**Objetivo:** provar que a imagem existente roda num emulador WASM. O frontend só
vem depois do spike.

Ponto de partida: nada. `kernel/web/index.html` foi removido. O frontend deve
ficar em diretório próprio (ex.: `web/`), fora do código do kernel.

- [ ] Spike x86_64 com `ktock/qemu-wasm` (experimental), usando a imagem
  **inalterada**.
- [ ] Spike AArch64 no mesmo runtime: máquina `virt`, EDK2, ISO e firmware
  empacotados. A demo Raspberry Pi do runtime não prova que o nosso kernel roda.
- [ ] Decidir o runtime pelo resultado. `v86` upstream está descartado (sem
  64 bits); `portlandhodl/v86_64` só como alternativa x86_64.
- [ ] Verificar COOP/COEP, workers, `SharedArrayBuffer` e navegadores
  (Chrome/Firefox no PC, Chrome no Android). Não prometer GitHub Pages antes
  disso; sem os headers, usar build sem threads ou outra hospedagem.
- [ ] Configuração declarativa por arquitetura (imagem, memória, firmware,
  máquina, dispositivos) compartilhada com os comandos QEMU nativos.
- [ ] Frontend: seletor explícito de arquitetura, xterm.js para a serial,
  iniciar/pausar/reset, emulação em worker; medir tempo até o primeiro log, FPS e
  RAM.
- [ ] Teste web com timeout e assertiva sobre o log serial de cada arquitetura;
  publicar logs e limitações junto do demo.
- [ ] Deploy público só depois de validar localmente; é opcional.

**Saída:** página local que inicializa x86_64 e AArch64 por seleção explícita,
mostra o shell serial e aceita comandos, sem backend.

### Etapa 6 — entrada física, touch distinto de mouse e teclado virtual

**Pré-requisito:** console funcionando e timer confiável (Etapas 3 e 2).

Modelo e drivers:

- [ ] Modelo de eventos com origem (`keyboard`, `mouse`, `tablet`, `touch`),
  fase (`press`, `move`, `release`, `cancel`), posição e tracking ID/slot. Atalho
  touch -> ação de UI é política da UI, nunca conversão no driver.
- [ ] Caminho virtio-input: PCI, virtqueues, IRQ, `EV_KEY/EV_REL/EV_ABS/EV_SYN`,
  validado primeiro por log. QEMU: `virtio-keyboard-pci`, `virtio-mouse-pci`,
  `virtio-tablet-pci`, `virtio-multitouch-pci`.
- [ ] Teclado físico primeiro: layout documentado, repetição, modificadores,
  mapeamento para o editor do shell. PS/2 só se for necessário.
- [ ] Mouse e tablet separados (relativo/absoluto, botões, scroll); tablet nunca
  vira touch.
- [ ] Touch com slot, início/movimento/fim/cancelamento; um contato, depois
  vários. Gestos (tap, long press, swipe, pinch) em camada própria, sobre o
  timer real.

UI e plataformas:

- [ ] Painel de diagnóstico (origem, ID, coordenadas, fase); testar mouse e touch
  alternados e simultâneos.
- [ ] Teclado virtual no framebuffer, sob demanda, no mesmo pipeline lógico do
  físico, mas marcado como virtual.
- [ ] Captura do contato touch até `release/cancel`, sem ativação dupla nem
  toque atravessando o teclado.
- [ ] Browser: preservar `pointerType`, `pointerId`, `isPrimary`; bloquear mouse
  events de compatibilidade após touch.
- [ ] VNC: mouse/tablet como caminho básico. Se o cliente só envia pointer,
  documentar a limitação; não exigir passthrough de `/dev/input`.
- [ ] Testes: unitários (coordenadas, ciclo do contato, gestos, layout), QMP
  `input-send-event` com multitouch, manuais no PC, VNC Android e browser Android.

**Saída:** mouse move/clica, touch arrasta com semântica visível, dois dedos com
slots independentes, teclado virtual escrevendo no shell sem eventos duplicados.

### Etapa 7 — base para crescer

- [ ] CI: build das duas arquiteturas, testes em host e smoke de boot.
- [ ] `clippy` e `fmt` na CI; warnings do crate perto de zero.
- [ ] Higiene do código:
  - remover `unsafe impl Sync for Kernel` + `static mut KERNEL` + `get_mut`
    sem trava (`source/main.rs:183-205`) ou justificar o invariante;
  - apagar código comentado em `source/shell.rs` e renomear
    `InputOutputSistem`, `TimeInteration`, `Vga` ao tocar cada módulo;
  - corrigir `generic_const_exprs` incompatível com o novo solver.
- [ ] Laços de espera ocupam CPU (`source/main.rs:116`, polling da serial em
  `io.rs`); usar `hlt`/`wfe` ou IRQ. O panic handler já usa.
- [ ] Revisar o custo de Rhai e nalgebra (binário, RAM, features), só com o MVP
  estável.
- [ ] Regenerar as fontes por alvo documentado; documentar origem e licença das
  fontes TTF.
- [ ] Escolher o próximo eixo, nesta ordem de preferência: (a) base de OS
  (interrupções, drivers, scheduler); (b) identidade autoral como camada
  separada do núcleo; (c) performance medida, com orçamento explícito.

**Saída:** pipeline verde e uma decisão de produto registrada para o próximo ciclo.

---

## 4. Critérios de aceite

### MVP — todos verdadeiros, sem ressalva

- [ ] `git clone` limpo + comandos do README geram a ISO sem edição manual.
- [ ] `run` x86_64 boota no QEMU e prova serial, heap, PMM e vídeo por log; falha
  de qualquer etapa tem mensagem clara.
- [ ] Console serial: prompt, eco, backspace, Enter executa e volta a um novo
  prompt.
- [ ] `1 + 2` -> `3`; `"a" + "b"` -> `ab`; erro de sintaxe com diagnóstico real.
- [ ] `help`, `mem`, `version` respondem; `mem` reflete alocações.
- [ ] `cargo test` passa em host, incluindo fronteiras do PMM.
- [ ] `run` aarch64 atinge o mesmo marco no QEMU `virt` + UEFI. Compilar não é
  funcionar.
- [ ] `run-vnc` nas duas arquiteturas funciona no Termux e o banner aparece no
  cliente VNC local.
- [ ] No navegador, o usuário escolhe x86_64 ou AArch64, a imagem inicializa e o
  shell serial responde `3` a `1 + 2`; repetido no PC e no Android, com
  restrições documentadas.
- [ ] Nenhuma afirmação de "funciona" sem log ou teste anexado.

### Pós-MVP (Etapa 6)

- [ ] Click de mouse chega com origem `mouse`, botão e coordenadas; toque chega
  com origem `touch`, slot e fase própria.
- [ ] Mouse e touch alternam sem trocar a origem nem duplicar eventos.
- [ ] Teclado virtual abre/fecha, respeita foco e escreve no mesmo editor do
  teclado físico, inclusive no browser Android.
- [ ] VNC documenta a capacidade real do cliente testado.

---

## 5. Riscos e itens não verificados

- **Hardware real, formatos de pixel != 32 bpp ARGB e pitch incomum:** sem teste.
- **QEMU nativo:** boots anteriores usaram TCG e timeout; `run` com KVM, GTK e
  VNC não foi validado de ponta a ponta.
- **VNC no Termux:** o QEMU anuncia VNC/websocket, mas nenhuma sessão de cliente
  foi testada. Touch depende do cliente e do transporte.
- **`qemu-wasm`:** suporte a `virt`, Limine, EDK2, framebuffer, entrada e touch
  dentro do emulador é hipótese até o spike da Etapa 5.
- **Mapas de memória fragmentados:** o heap usa a maior região usable sem alinhar
  explicitamente o início.
- **UART ARM fixa** no endereço do QEMU `virt`; outras máquinas precisam de DTB
  ou configuração.
- **Ambiente:** o desenvolvimento acontece num PC Arch Linux; o Termux é alvo
  do VNC e nada nele foi reverificado desde 2026-11-21.

---

## 6. Documentação pendente

- [x] README e AGENTS atualizados em 2026-11-23: caminhos novos, `ROADMAP.md`,
  Makefile e `xtask` documentados lado a lado e uso de IA restrito a
  planejamento, documentação e dúvidas. Rever quando a decisão Makefile/xtask
  for tomada.
- [ ] README: comandos exatos e validados de boot por arquitetura (após a
  Etapa 1).
- [ ] Arquitetura em poucas linhas: ordem de init, mapa de memória, convenção de
  cores, papel da serial vs. terminal gráfico.
- [ ] Papel do Rhai e escopo do shell (decidir via DUVIDAS.md, itens 3, 6 e 7).
- [ ] Guias: QEMU no PC, VNC no Termux, build WASM e headers de deploy.
- [ ] Mapa de entrada por plataforma e limitações de VNC/WebAssembly.
- [ ] Origem, autoria e termos das fontes TTF (a MIT do projeto não as cobre).

---

## 7. Fora do MVP (escolhas de escopo, não defeitos)

Entrada por teclado PS/2, mouse/tablet/touch, teclado virtual, scheduler,
userspace, storage/VFS, rede e SMP. Nenhum entra antes do MVP. O terminal
gráfico é a etapa seguinte ao shell serial, não pré-requisito dele.

---

## Apêndice — registro de verificações

**2026-11-23 (reconferido)**

- `cargo build --release --target x86_64-unknown-none`: passou, 37 warnings no
  crate + 2 do manifesto (`cc` sem uso, `build-override`).
- `cargo build --release --target aarch64-unknown-none-softfloat`: passou, mesma
  contagem.
- `readelf -h`: entrada x86_64 `0xffffffff80209410`; ARM `0xffffffff80009598`.
- Presença de `qemu-system-x86_64`, `qemu-system-aarch64`, `xorriso`, `nasm` e
  EDK2 aarch64. Boots em QEMU **não** foram repetidos.
- Bugs de shell, vídeo, timer e PMM listados nas Etapas 2-4 reconferidos no
  código por leitura/grep.

**2026-11-21 (Termux/Android, commit `6cf568d` + alterações locais)**

- Mesmos dois builds: passaram, 39 warnings. ARM com entrada `0x28c6f0`
  (linker script não aplicado).
- `make kernel.iso` das duas arquiteturas passou (Limine compilou x86_64
  BIOS/UEFI e AArch64 UEFI).
- QEMU x86_64, 20 s: serial com `[ BOOT ] A/B`, PMM, framebuffer e início do
  `welcome`; sem prompt. Exit 137 = timeout.
- QEMU AArch64 `virt` + EDK2, 20 s: só firmware/UEFI, sem `[ BOOT ] A`.
- `cargo test --target x86_64-unknown-linux-gnu`: falhou, `core` duplicado.

**2026-11-20 (commit `e66e9be`, layout `kernel/` antigo)**

- Builds com `-Zbuild-std=core,alloc` passaram nas duas arquiteturas; target
  `aarch64-linux-android` falhou no link e nos testes (`panic_impl` duplicado).
- ARM com 128 MiB: o próprio Limine aborta (`High memory allocator: Out of
  memory`); com 256/512 MiB carrega `kernel.bin` e o kernel fica mudo.
- `fontGenerator` compila.
