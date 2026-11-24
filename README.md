# motosseraKernel

Um kernel feito à mão, em Rust, para aprender o sistema operacional por dentro
e, no caminho, inventar um com cara própria.

O motosseraKernel é um projeto autoral de estudo. Ele junta, no mesmo corpo,
um kernel bare-metal, uma interface visual desenhada pixel a pixel e uma
linguagem interpretada (Rhai) como forma de conversar com o sistema. A estética
importa, mas não substitui a base: a meta é um kernel modular, eficiente e
medido, com funcionalidades que não vêm de um manual de SO.

> **Estado: desenvolvimento inicial.** Compila para x86_64 e AArch64, mas ainda
> não há um sistema utilizável. Este README separa o que existe do que é
> intenção. Detalhes e evidências estão no [ROADMAP](ROADMAP.md).

## Ideias que guiam o projeto

- **Autoral antes de convencional.** O sistema representa o que o autor quer
  aprender e construir. Não é um produto nem um clone de Linux.
- **Duas arquiteturas desde o começo.** x86_64 e AArch64 são MVP, não "depois".
  Uma arquitetura só conta como funcionando se bootou; compilar não basta.
- **Rhai como voz do sistema.** O shell avalia expressões Rhai
  (`1 + 2` deve responder `3`). O papel exato da linguagem ainda é uma pergunta
  em aberto, registrada em [DUVIDAS.md](DUVIDAS.md).
- **Entrada com origem preservada.** Quando houver mouse, tablet, touch e
  teclado, cada um chega ao sistema como o que é. Touch não é tratado como
  click de mouse; se a interface os aproximar, é decisão da UI, nunca perda de
  informação no driver.
- **Estética medida.** Efeito visual entra com medição de custo (tempo de boot,
  tempo por frame, memória), não no escuro.
- **Sem promessa antes de prova.** Nada é dado como pronto sem log ou teste.

## O que já existe

| Peça | Situação |
|---|---|
| Boot pelo Limine (x86_64 e AArch64) | Imagens geradas. Boot x86_64 chegou à tela de boas-vindas no QEMU; o ARM precisa ser retestado com o linker corrigido |
| Heap de 32 MiB sobre a memória informada pelo bootloader | Código existe |
| Alocador de frames físicos por bitmap | Código existe, sem testes |
| Vídeo: backbuffer, pixels, retângulos, fonte bitmap 16x16 | Código existe; geometria ainda assume 1920x1080 |
| Serial de debug (COM1 no x86_64, PL011 no ARM `virt`) | Funciona no x86_64; ARM não verificado |
| Shell com Rhai | Esqueleto: ainda não fecha o ciclo comando -> resposta |
| Gerador de fontes TTF (`tools/fontGenerator`) | Compila |
| Testes automatizados, CI | Não existem |
| VNC no Termux, demo no navegador (WebAssembly) | Planejados, não implementados |

## Rodando

### Requisitos

- Rust **nightly** com `rust-src` (o `rust-toolchain.toml` já pede isso);
- Limine e as ferramentas dele (`xorriso`, NASM, mtools, GNU Make);
- QEMU: `qemu-system-x86_64` e/ou `qemu-system-aarch64`;
- para ARM, firmware EDK2 (`QEMU_EFI.fd`) e pelo menos 256 MiB de RAM na VM.

O desenvolvimento acontece num PC Linux. O Termux é alvo futuro do modo VNC.

### Compilar

```sh
cargo build --release --target x86_64-unknown-none
cargo build --release --target aarch64-unknown-none-softfloat
```

Os dois passaram em 2026-11-23, com warnings. O `readelf` confirma entradas
no espaço higher-half nas duas arquiteturas (`0xffffffff80209410` no x86_64 e
`0xffffffff80009598` no ARM).

### Gerar a ISO e abrir no QEMU

Existem dois caminhos, e ainda não foi decidido qual será o oficial
([ROADMAP](ROADMAP.md), seção 2):

```sh
cargo xtask run x86_64        # ou aarch64; padrão: arquitetura do host
make run ARCHI=x86_64         # ou aarch64
```

Ambos gravam a ISO em `kernel/` e abrem o QEMU. **Nenhum dos dois foi validado
de ponta a ponta até o prompt.** Limitações conhecidas do `xtask`: liga KVM
sempre que host e alvo coincidem, usa o firmware ARM em
`/usr/share/edk2/aarch64/QEMU_EFI.fd`, exige display GTK, não tem timeout e
grava `qemu.log` na raiz. Não existe alvo `run-vnc`.

## Como funciona

Inicialização atual:

```text
_start
  -> serial/debug
  -> heap
  -> PMM (frames físicos)
  -> framebuffer
  -> shell
  -> loop de renderização
```

Organização do repositório:

```text
Cargo.toml           kernel e workspace (membro: xtask)
rust-toolchain.toml  nightly, rust-src e targets
.cargo/config.toml   flags de link por arquitetura e alias `cargo xtask`
linker.ld            layout x86_64
linker_aarch64.ld    layout ARM
limine.conf          configuração de boot
Makefile             build, ISO e QEMU (convive com o xtask)

source/              código do kernel
  main.rs              entrada e ordem de init
  memory.rs, pmm.rs    heap e frames físicos
  video.rs, db_font.rs framebuffer e fonte
  io.rs, debuglog.rs   serial e log
  shell.rs             terminal e Rhai
  timer.rs, math.rs, utils.rs
  definitions.rs       constantes compartilhadas

xtask/               build, ISO e QEMU em Rust
tools/fontGenerator/ gerador das fontes bitmap
kernel/              artefatos gerados e checkout do Limine (ignorados pelo Git)
```

## Para onde vai

A ordem completa, com critérios de aceite, está no [ROADMAP](ROADMAP.md). Em
resumo:

1. **Boot reproduzível** nas duas arquiteturas, com log.
2. **Memória e tempo confiáveis:** testes do PMM no host, timer real.
3. **Shell serial de ponta a ponta:** `1 + 2` -> `3`, `help`, `mem`, `version`.
4. **Terminal gráfico** correto, em qualquer resolução.
5. **Navegador:** a mesma imagem bootável, emulada em WebAssembly (não o kernel
   recompilado para `wasm32`). O caminho a testar é `ktock/qemu-wasm`;
   compatibilidade e deploy ainda não foram verificados.
6. **Entrada:** teclado físico e virtual, mouse, tablet e touch como origens
   distintas.
7. **Base para crescer:** CI, limpeza e a escolha do próximo eixo.

O primeiro marco de produto é pequeno e verificável: um shell serial que avalia
Rhai e mostra um banner no framebuffer.

## Uso de IA neste repositório

Como o projeto existe para o autor aprender, a IA entra só como apoio, e não
para escrever o kernel. Ela é usada para:

- **planejar e guiar** o projeto (ordem das etapas, riscos, decisões);
- **ajudar a documentar** (README, roadmap, dúvidas);
- **tirar dúvidas** sobre conceitos e ferramentas.

O código do kernel é do autor. A documentação feita com ajuda de IA é revisada
por ele e pode conter erros: confie nos logs e testes citados, não só no texto.

## Documentos de trabalho

- [ROADMAP.md](ROADMAP.md): estado atual com evidências, etapas, riscos e
  critérios de aceite. É temporário e revisto a cada etapa.
- [DUVIDAS.md](DUVIDAS.md): decisões de produto que dependem do autor.
- [AGENTS.md](AGENTS.md): regras para agentes que trabalham neste repositório.

## Licença

O código e a documentação original do motosseraKernel são distribuídos sob a
[licença MIT](LICENSE). Qualquer pessoa pode usar, estudar, modificar,
redistribuir e vender suas próprias versões, inclusive em produtos fechados,
desde que mantenha o aviso de copyright de Davi Passos e o texto da licença.

O objetivo do repositório é apoiar o aprendizado do autor e servir como base
aberta para outras pessoas. A licença não garante que o kernel esteja pronto
para produção; o software é fornecido sem garantias.

Dependências, Limine, fontes TTF e demais materiais de terceiros continuam sob
suas próprias licenças. A licença do projeto não os relicencia nem substitui
os créditos originais. A origem e os termos das fontes incluídas ainda devem
ser documentados de forma completa antes de uma distribuição pública formal.
