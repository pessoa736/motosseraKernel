# motosseraKernel

Kernel bare-metal experimental, escrito predominantemente em Rust `no_std` e
iniciado pelo Limine. É um projeto autoral de aprendizado: sistema operacional,
interface visual e experimentos com linguagem interpretada no mesmo corpo.

O objetivo não é apenas fazer algo estiloso. A proposta é construir um kernel
modular, eficiente e mensurável, com identidade própria e funcionalidades
inventadas pelo autor — sem esconder o que ainda não funciona.

## Estado atual

**Em desenvolvimento inicial. Não há um sistema utilizável ainda.**

Já existe código para:

- entrada e requisições do protocolo Limine;
- heap de 32 MiB sobre memória informada pelo bootloader;
- alocador físico de frames por bitmap;
- backbuffer de vídeo, pixels, retângulos e fontes bitmap;
- serial para debug em x86_64 e AArch64;
- esqueleto de terminal/shell com Rhai;
- gerador de fontes TTF em `tools/fontGenerator`.

Também existem problemas importantes: o caminho ARM do Makefile está errado,
o shell ainda não fecha o ciclo de entrada/saída e não há testes automatizados.
A análise completa, incluindo o que foi realmente executado, está em
[ROADMAP.sketch.md](ROADMAP.sketch.md).

## Arquiteturas e ambientes pretendidos

- x86_64 e AArch64, desde o MVP;
- QEMU nativo no PC;
- QEMU headless no Termux, com VNC local;
- navegador, usando emulação da máquina em WebAssembly.

No navegador, a intenção é executar a mesma imagem bootável emulada, não
converter o kernel para `wasm32-unknown-unknown`. O caminho preferido para o
spike é `ktock/qemu-wasm`; compatibilidade, desempenho e deploy ainda não foram
verificados.

## Direção do produto

O primeiro objetivo funcional é pequeno e verificável:

1. boot reproduzível nas duas arquiteturas;
2. logs claros de inicialização;
3. shell serial interativo com Rhai;
4. comandos como `help`, `mem` e `version`;
5. testes do PMM no host;
6. banner no framebuffer.

Depois vêm terminal gráfico, demo WebAssembly e uma camada de entrada com:

- teclado físico e teclado virtual;
- mouse, tablet e touch como origens distintas;
- tracking ID/slot e cancelamento para contatos touch;
- eventos de compatibilidade sem clicks/toques duplicados.

**Touch não será tratado como se fosse click de mouse.** Se uma interação for
reutilizada pela interface, isso será uma decisão da UI, nunca perda de origem
no driver. VNC e WebAssembly terão suas limitações documentadas por teste, não
por suposição.

## Organização

```text
kernel/
  src/               código Rust do kernel
  .cargo/            targets e flags de link
  Makefile           build, ISO e execução QEMU (em correção no roadmap)
  linker.ld          layout x86_64
  linker_aarch64.ld  layout ARM (atualmente não usado pelo .cargo/config)
  limine.conf        configuração de boot
  limine/            checkout local do Limine, ignorado pelo Git

tools/
  fontGenerator/     gerador das fontes bitmap

docs/
  description.md     descrição inicial do projeto

ROADMAP.sketch.md    análise técnica e etapas temporárias
DUVIDAS.md           decisões de produto pendentes com o autor
```

O fluxo atual de inicialização é:

```text
_start
  -> serial/debug
  -> heap
  -> PMM
  -> framebuffer
  -> shell
  -> loop de renderização
```

## Ferramentas

O build exige Rust nightly com código-fonte da biblioteca padrão (`rust-src`),
Cargo com suporte a `-Zbuild-std`, Limine, `xorriso`, GNU Make e QEMU. O Limine
também possui suas próprias dependências de compilação; consulte
`kernel/limine/INSTALL.md` quando o checkout existir.

No Termux, os pacotes Rust são divididos por componente. É necessário verificar
a toolchain nightly e `rust-src` instalados; um `cargo` existente não garante
que os targets bare-metal estejam prontos.

## Build de verificação

Estes são comandos usados na análise, não uma promessa de que `make` já está
pronto para uso diário.

Dentro de `kernel/`:

```sh
CARGO_PROFILE_RELEASE_PANIC=abort cargo build --release \
  --target x86_64-unknown-none -Zbuild-std=core,alloc

CARGO_PROFILE_RELEASE_PANIC=abort cargo build --release \
  --target aarch64-unknown-none -Zbuild-std=core,alloc
```

Os dois comandos compilaram no ambiente da análise, com warnings. O target
`aarch64-linux-android` usado atualmente pelo Makefile é inadequado para este
kernel bare-metal e falha na linkagem.

Gerar uma ISO e compilar não prova que o kernel executa. A validação de boot,
incluindo os problemas encontrados no Limine/QEMU, está descrita no roadmap.

## Execução: planejado, não entregue

Os alvos abaixo são a interface desejada para a Etapa 1 e ainda precisam ser
implementados/corrigidos:

```sh
# QEMU no PC
make run ARCHI=x86_64
make run ARCHI=aarch64

# QEMU headless + VNC no Termux
make run-vnc ARCHI=x86_64
make run-vnc ARCHI=aarch64
```

Não use esses comandos como evidência de suporte atual. O Makefile existente
usa flags x86 para todos os caminhos e precisa de configuração por arquitetura,
firmware configurável e fallback TCG.

A demo WebAssembly também ainda não existe. Ela depende das etapas de boot,
shell serial e validação de `qemu-wasm` nas duas arquiteturas.

## Testes

Não há suíte funcional hoje. `cargo test` falha porque o binário de teste também
carrega o `#[panic_handler]` do kernel. O planejado é separar componentes
testáveis em host do runtime bare-metal e cobrir especialmente PMM, console e
entrada.

## Documentos de trabalho

- [ROADMAP.sketch.md](ROADMAP.sketch.md): avaliação técnica, bugs priorizados,
  riscos, etapas e critérios de aceite.
- [DUVIDAS.md](DUVIDAS.md): perguntas abertas para o autor, com sugestões de
  padrão quando houver uma escolha recomendada.
- [docs/description.md](docs/description.md): anotação inicial, não uma lista
  confiável do que já funciona.

O roadmap é temporário e deve ser revisto após cada etapa. Nada é considerado
concluído apenas porque compila: boot exige log, comportamento exige teste.
