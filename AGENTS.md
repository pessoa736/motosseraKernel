# AGENTS.md — motosseraKernel

Instruções para agentes que trabalharem neste repositório. Este arquivo cobre o
projeto; `kernel/limine/AGENTS.md` e `kernel/limine/CONTRIBUTING.md` prevalecem
dentro do checkout do Limine, mas não use suas regras de estilo C no kernel Rust.

## 1. Antes de mexer

- Leia `README.md`, `ROADMAP.sketch.md` e `DUVIDAS.md`.
- Rode `git status --short --branch` antes de editar. O working tree pode ter
  trabalho do Davi, inclusive `kernel/Cargo.lock` modificado.
- Não sobrescreva, restaure, stageie ou commite alterações alheias por acidente.
  Não use `git add -A`, `git add .`, `git reset --hard` ou `git checkout --`.
- Não faça commit/push sem pedido explícito do Davi.
- Não implemente etapas futuras por impulso. O roadmap é a ordem de trabalho;
  dúvidas sem resposta em `DUVIDAS.md` não são decisões aprovadas.

### Propósito e licença

- Este repositório existe principalmente para o estudo de Davi e para auxiliar
  seu desenvolvimento como programador. O sistema representa o que ele quer
  aprender e construir; não é um produto estável para produção.
- O código e a documentação original usam a licença MIT (`LICENSE`). Uso,
  estudo, modificação, redistribuição e venda de versões derivadas são
  permitidos, inclusive em projetos fechados, desde que o aviso de copyright
  de Davi Passos e o texto da licença sejam preservados.
- Ao contribuir, não altere essa licença nem o copyright sem pedido explícito
  do Davi. Preserve também os avisos de dependências, Limine, fontes TTF e
  outros materiais de terceiros; a licença do projeto não os relicencia.
- Não invente licença para fontes ou recursos externos sem termos conhecidos.
  Documente origem e licença real; se estiverem ausentes, registre a pendência.

## 2. Verdade técnica e escopo

- O projeto é um kernel bare-metal `#![no_std]`, não um programa Android,
  Linux ou WASM comum.
- Arquiteturas-alvo: `x86_64-unknown-none` e `aarch64-unknown-none`, ambas desde
  o MVP. Não volte a usar `aarch64-linux-android` para o kernel.
- Não confunda compilar ELF, gerar ISO e bootar. Cada uma é uma verificação
  diferente; "funciona" exige comportamento observado e log/teste correspondente.
- Marque claramente hipóteses, código inspecionado, comandos que passaram e
  comportamento não testado. Não invente saídas de build/QEMU.
- Faça a menor mudança que resolve a etapa atual. Sem reescritas, reformatações
  ou limpezas de arquivos não relacionados.
- Preserve a identidade autoral do projeto, mas não sacrifique modularidade,
  correção ou medição de desempenho para adiantar efeitos visuais.

## 3. Ambiente e build

Ambiente principal atual: Termux/Android em host AArch64. Rust nightly e
`rust-src` são necessários; `rustup` pode não existir. Não presuma os pacotes
instalados: inspecione `rustc --version`, `cargo --version`, `pkg list-installed`
e os binários QEMU disponíveis.

Comandos de build verificados na análise inicial, executados em `kernel/`:

```sh
CARGO_PROFILE_RELEASE_PANIC=abort cargo build --release \
  --target x86_64-unknown-none -Zbuild-std=core,alloc

CARGO_PROFILE_RELEASE_PANIC=abort cargo build --release \
  --target aarch64-unknown-none -Zbuild-std=core,alloc
```

- Os dois passaram na análise inicial, com warnings. O target Android falhou na
  linkagem; `cargo test` falhou por `panic_impl` duplicado.
- Ao corrigir o Makefile/toolchain, torne `panic = "abort"` e `-Zbuild-std`
  configuração explícita do projeto, em vez de depender de variáveis de shell
  esquecidas pelo próximo desenvolvedor.
- Não troque um erro de configuração por instalação global desnecessária de
  pacotes. Se faltar ferramenta, explique e use o gerenciador adequado ou peça
  confirmação quando a mudança tiver impacto fora do repositório.
- O checkout `kernel/limine/` é gerado/ignorado e pode ter `GNUmakefile` com
  caminho absoluto antigo. Reconfigure-o ao mover o projeto; não corrija o
  arquivo gerado manualmente como solução definitiva.
- Limine requer suas próprias ferramentas, incluindo NASM e mtools. Leia
  `kernel/limine/INSTALL.md` antes de alterar seu build.

## 4. Regras de implementação

### Arquitetura e boot

- `kernel/src/main.rs` contém `_start`; memória está em `memory.rs`/`pmm.rs`,
  serial em `io.rs`/`debuglog.rs`, vídeo em `video.rs` e console em `shell.rs`.
- ARM não é só mudar `cfg(target_arch)`. O entry point precisa respeitar o
  estado entregue pelo Limine: na base revision 6, FP/SIMD começa desativado,
  enquanto o Rust/LLVM pode emitir NEON. Prepare o stub assembly antes de
  executar Rust que use essas instruções.
- Exceções precisam de vetores/handlers úteis. Um laço mudo em `0x200` não é
  diagnóstico aceitável.
- QEMU ARM precisa de máquina `virt`, firmware EDK2 e RAM suficiente. O teste
  inicial falhou no próprio Limine com 128 MiB; use pelo menos 256 MiB até que
  uma configuração diferente seja comprovada.
- TCG é o fallback portátil; KVM/aceleração deve ser opcional e detectada.
  Firmware e caminhos do PC/Termux devem ser configuráveis.

### Memória e unsafe

- Heap e PMM são componentes distintos. Preserve as reservas das regiões usadas
  pelo heap, bitmap, kernel, bootloader e framebuffer.
- Não normalize double-free, endereço desalinhado ou acesso fora do bitmap.
  Adicione testes de fronteira antes de confiar no alocador.
- Não adicione `unsafe`, `Sync`, `Send` ou `static mut` apenas para silenciar o
  compilador. Justifique o invariante e minimize referências mutáveis globais.
- Componentes puros/testáveis devem ser separados do runtime bare-metal; não
  remova o panic handler apenas para fazer `cargo test` passar.

### Console e vídeo

- Prompt, linha de entrada, histórico e saída são estados diferentes. Nunca
  insira o prompt na string que será executada.
- Rhai deve aceitar resultados além de `String` e exibir erros reais com
  posição. `1 + 2` deve produzir `3`; não devolva apenas `"erro"`.
- Fonte/célula, x/y, scroll e dimensões reais do framebuffer precisam usar uma
  mesma geometria. O backbuffer fixo de 1920x1080 não é resolução universal.
- Respeite `bpp`, pitch e máscaras de cor do Limine ou rejeite explicitamente
  modos não suportados.
- Meça FPS/tempo de cópia antes de otimizar `swap_buffers`; evite otimizações
  grandes sem benchmark reproduzível.

### Entrada futura: mouse, touch e teclado virtual

- Mouse, tablet, touch e teclado têm origens diferentes. Nunca converta touch
  em click de mouse no driver.
- Preserve origem, fase, coordenadas, botão e tracking ID/slot. A UI pode
  mapear toque para uma ação equivalente, mas sem perder essas informações.
- Implemente teclado físico antes do virtual. O teclado virtual usa o mesmo
  editor lógico, mantendo origem virtual para diagnóstico.
- Touch exige release/cancel e captura do contato; previna mouse events de
  compatibilidade e ativação atrás do teclado virtual.
- No browser, preserve `PointerEvent.pointerType`, `pointerId` e `isPrimary`.
- O QEMU expõe `virtio-keyboard-pci`, `virtio-mouse-pci`, `virtio-tablet-pci`
  e `virtio-multitouch-pci`; a existência do dispositivo não prova suporte do
  VNC/WebAssembly de ponta a ponta. Teste e documente limitações reais.

## 5. Testes obrigatórios por tipo de mudança

- Mudança Rust no kernel: build release nas duas arquiteturas, não apenas
  `cargo check` no target padrão.
- Mudança de boot/linker/Makefile/assembly: boot em QEMU com log serial e
  timeout. Se o emulador de uma arch não existir, diga que ela ficou sem
  verificação; não alegue paridade.
- Mudança em memória: testes host de alocação, liberação, esgotamento,
  alinhamento e double-free quando aplicável.
- Mudança no shell: comandos positivos e negativos, incluindo `1 + 2` -> `3`,
  string concatenada, backspace, Enter repetido e erro de sintaxe com posição.
- Mudança gráfica: validar resolução menor que 1920x1080, limites, scroll e
  formato de pixel; captura de tela quando houver ferramenta disponível.
- Mudança de entrada: teste de origem, IDs, coordenadas, cancelamento,
  alternância mouse/touch e ausência de duplicação. Use QMP
  `input-send-event` quando possível.
- Não declare que uma suíte antiga falhou por sua alteração sem comparar com
  o estado anterior. Distinga falha preexistente de regressão.

## 6. QEMU, VNC e WebAssembly

- Rode QEMU com timeout externo (`timeout --signal=KILL`) e saída serial em
  arquivo fora do Git. Exit 137 pode significar apenas que o timeout encerrou
  uma VM ainda ativa; interprete pelo log, não como build failure automático.
- Use diretório scratch para ISOs e logs temporários. Não sobrescreva artefatos
  gerados do repositório só para investigação.
- Pare emuladores/servidores temporários ao terminar; não deixe QEMU rodando
  sem necessidade.
- Interface planejada, ainda não entregue na análise inicial:
  `make run ARCHI=...` no PC e `make run-vnc ARCHI=...` no Termux.
- VNC deve ser local por padrão. Não exponha serviço sem autenticação.
- Navegador deve emular a mesma imagem bootável; não reescreva o kernel para
  `wasm32-unknown-unknown`. Priorize spike com `ktock/qemu-wasm` nas duas
  arquiteturas antes de construir frontend.
- `v86` upstream não suporta kernel de 64 bits. `portlandhodl/v86_64` é apenas
  alternativa x86_64 a avaliar, não solução ARM.
- COOP/COEP, workers, SharedArrayBuffer, firmware e compatibilidade mobile são
  verificações do spike. Não prometa GitHub Pages antes de testar os headers.

## 7. Documentação, estilo e Git

- Comentários e identificadores podem continuar em português, seguindo o
  estilo local; use snake_case para código novo e imports somente necessários.
- Não faça renames cosméticos em massa. Renomeie tipos antigos ao tocar sua
  implementação por uma razão funcional.
- Atualize README/roadmap quando mudar estado real, comandos, suporte ou
  limitações. Não transforme intenção em capacidade entregue.
- `ROADMAP.sketch.md` é temporário; marque itens concluídos somente com prova.
- Referencie código como `path:line` no relatório final. Seja conciso e diga
  o que passou, falhou e ficou sem teste.
- Faça commits atômicos por concern quando autorizado, somente com os arquivos
  da tarefa. Antes do commit, verifique `git diff --cached --name-only`.
- Antes do push, confirme branch e remoto; depois releia o remoto e
  `git status` para comprovar o resultado.

## 8. Definição de pronto

Uma tarefa só está pronta quando:

1. a mudança pedida está aplicada e limitada ao escopo;
2. as verificações relevantes foram realmente executadas;
3. falhas preexistentes/ambientais estão separadas de regressões;
4. o comportamento exigido tem evidência, não apenas build;
5. documentação afetada foi atualizada;
6. o working tree contém somente as mudanças pretendidas mais o trabalho
   preexistente do usuário, preservado intacto.
