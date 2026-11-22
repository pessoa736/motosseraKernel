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

Os dois builds release e as duas ISOs já são gerados, mas o ELF ARM ainda não
usa o linker script correto e não emite log próprio no QEMU. No x86_64, o boot
alcança serial, PMM, framebuffer e a tela de boas-vindas; o prompt do shell
ainda não foi validado. Os testes em host também continuam bloqueados e não há
uma suíte automatizada. A análise completa, incluindo comandos e limitações,
está em [ROADMAP.sketch.md](ROADMAP.sketch.md).

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
Cargo.toml           manifesto e dependências do kernel
rust-toolchain.toml  toolchain nightly, rust-src e targets
.cargo/config.toml   build-std e flags de link por arquitetura
Makefile             build, ISO e execução QEMU (em correção no roadmap)
linker.ld            layout x86_64
linker_aarch64.ld    layout ARM (ainda não aplicado ao alvo softfloat)
limine.conf          configuração de boot

source/              código Rust do kernel
  definitions/       constantes compartilhadas de I/O, vídeo e shell

kernel/
  limine/            checkout local do Limine, ignorado pelo Git
  web/               esqueleto inicial da página para emulação

tools/
  fontGenerator/     gerador das fontes bitmap

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

Comandos executados na raiz, em 2026-11-21:

```sh
cargo build --release --target x86_64-unknown-none
cargo build --release --target aarch64-unknown-none-softfloat
```

Os dois comandos passaram, cada um com 39 warnings. O Makefile também gerou as
duas ISOs. O x86_64 usa o linker higher-half esperado; o ARM ainda precisa de
uma seção `[target.aarch64-unknown-none-softfloat]` em `.cargo/config.toml`
para aplicar `linker_aarch64.ld`.

Compilar e gerar ISO não prova boot. O x86_64 chegou à tela de boas-vindas no
QEMU; o ARM mostrou apenas logs do firmware. Detalhes e limitações estão no
roadmap.

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

O Makefile já separa flags por arquitetura, mas ainda precisa detectar KVM com
fallback TCG, tornar o firmware configurável e completar a validação do shell.
Use os resultados do roadmap, não a existência dos alvos, como evidência do
que funciona.

Existe apenas um esqueleto HTML em `kernel/web/`; ainda não há demo WebAssembly
funcional nem build `qemu-wasm` do projeto. Essa etapa depende de boot, shell
serial e validação do emulador nas duas arquiteturas.

## Testes

Não há suíte funcional hoje. `cargo test --target x86_64-unknown-linux-gnu`
falha com múltiplas definições de `core` porque a configuração global de
`build-std` também afeta o build de host. É necessário separar componentes
testáveis e sua configuração do runtime bare-metal, cobrindo especialmente PMM,
console e entrada.

## Documentos de trabalho

- [ROADMAP.sketch.md](ROADMAP.sketch.md): avaliação técnica, bugs priorizados,
  riscos, etapas e critérios de aceite.
- [DUVIDAS.md](DUVIDAS.md): perguntas abertas para o autor, com sugestões de
  padrão quando houver uma escolha recomendada.

O roadmap é temporário e deve ser revisto após cada etapa. Nada é considerado
concluído apenas porque compila: boot exige log, comportamento exige teste.

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
