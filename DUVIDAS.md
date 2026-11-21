# Dúvidas para o Davi

Perguntas que afetam arquitetura ou experiência, não preferências cosméticas
que eu possa escolher sozinho. Responde por número, no formato que preferir.
Pode responder só uma parte; o que ficar em aberto permanece como hipótese,
não como decisão tomada.

As decisões anteriores já registradas são: x86_64 + AArch64 desde o MVP,
shell serial + Rhai, QEMU no PC, VNC no Termux, WebAssembly no navegador,
touch separado de mouse e teclado virtual no futuro.

## Produto e identidade

1. Qual é a primeira demonstração que faria você dizer "isso é o meu kernel"?
   Um shell com comandos autorais, uma animação/interação visual, uma pequena
   aplicação embutida ou outra coisa?

2. Quais são três princípios visuais/de interação que o sistema deve manter
   mesmo quando a estética mudar? Exemplos: tipografia grande, movimento
   suave, terminal como centro, visual retrô, formas orgânicas, transparência.

3. O Rhai será principalmente:
   - linguagem do shell e de comandos internos;
   - plataforma para aplicações do usuário;
   - ferramenta para modificar o sistema em execução;
   - uma mistura dessas?

4. Que funcionalidade autoral você mais quer experimentar primeiro depois da
   base? Não precisa ser uma lista de sistema operacional tradicional.

## Shell e linguagem

5. Quais comandos iniciais além de `help`, `clear`, `mem` e `version` fazem
   sentido para você? Minha sugestão é `echo`, `history`, `uptime` e `fbinfo`
   somente quando o timer/framebuffer correspondente estiver pronto.

6. O shell deve aceitar diretamente qualquer expressão Rhai ou ter comandos
   nomeados como fronteira principal? Sugestão: comandos nomeados para
   administração e Rhai livre para expressões, sem impor prefixo.

7. Uma expressão sem resultado (`()`), um valor numérico e um erro devem ser
   exibidos como? Sugestão: valores normais formatados com tipo; `()` sem
   linha de resultado; erro com mensagem e posição.

## Touch, mouse e teclado virtual

8. O touch terá multitouch real ou um contato já atende à primeira versão?
   Sugestão: preservar slots/IDs desde o modelo inicial, mas só tornar gestos
   de um dedo obrigatórios; multitouch visual vem logo depois.

9. Quais gestos importam primeiro? Sugestão mínima: tap, arrastar e long
   press; swipe/pinch só quando houver uma tela que realmente os utilize.

10. Toque simples deve poder gerar a mesma ação de um botão na UI, mantendo a
    origem `touch`? Sugestão: sim, a UI pode mapear para `activate`; o driver
    nunca transforma touch em mouse.

11. O teclado virtual abre automaticamente ao focar o shell ou apenas por um
    botão? Sugestão: botão explícito no MVP gráfico, para não cobrir metade da
    tela; opção automática pode virar preferência depois.

12. Layout inicial do teclado: ABNT2 com acentos ou US simplificado? Ele será
    exclusivamente touch ou também navegável por mouse?

13. Quais teclas são indispensáveis além do alfabeto? Sugestão: Shift, Ctrl,
    Backspace, Enter, espaço, Tab, setas e uma camada de símbolos/números.

14. Pressão, inclinação da caneta e hover devem entrar no modelo inicial?
    Sugestão: reservar campos sem implementar comportamento ainda; não criar
    driver de pen antes de haver uso real.

## VNC, navegador e compatibilidade

15. Qual aplicativo de VNC você pretende usar no Android? Se já tiver um
    preferido, ele será o cliente de referência para testar mouse e descobrir
    se touch chega apenas como pointer ou com eventos multitouch.

16. Se o VNC escolhido não transportar touch real, qual comportamento é
    aceitável?
    - suporte completo no navegador e limitação documentada no VNC;
    - escolher outro cliente/protocolo;
    - canal alternativo de entrada, mantendo o kernel igual.

    Sugestão: primeira opção. Não vale distorcer o modelo do kernel por causa
    de uma limitação do cliente.

17. No navegador, o usuário escolhe a arquitetura manualmente ou a página deve
    iniciar uma padrão? Sugestão: x86_64 por padrão no PC e AArch64 por padrão
    no Android, com seletor sempre disponível; sem "detecção" mágica.

18. O demo web deve ser somente uma tela interativa ou ter apresentação,
    controles e instruções? Sugestão: canvas/terminal no centro, seletor de
    arquitetura, iniciar/pausar/reset e um pequeno painel de logs/limitações.

19. O deploy público será obrigatório quando a prova local funcionar ou
    continua opcional? E há preferência entre GitHub Pages, Vercel ou outra
    hospedagem caso COOP/COEP seja necessário?

## Identidade visual e performance

20. Existe uma resolução de referência para a interface? Sugestão: projetar
    para 1280x720 e escalar células/componentes; não fixar toda a UI em
    1920x1080 como o backbuffer atual.

21. O visual deve ser o mesmo no PC, VNC e navegador, com apenas escala e
    densidade variando, ou cada plataforma pode ter uma apresentação diferente?
    Sugestão: mesma linguagem visual e adaptação por tamanho/entrada.

22. Que orçamento de performance é importante para você? Sugestão inicial para
    medir, sem transformar em promessa prematura: tempo até primeiro log,
    tempo até prompt, latência de tecla/toque, FPS durante redraw e memória
    ocupada por arch/ambiente.

## Escopo técnico posterior

23. Depois do shell e da entrada, o próximo subsistema de OS deve ser
    scheduler/processos, VFS/storage ou outro? Minha recomendação é VFS
    simples + carregamento de módulos Rhai antes de userspace/scheduler.

24. Você pretende ter aplicações escritas em Rhai salvas na imagem do sistema?
    Isso decide se a VFS entra cedo e se haverá um formato de pacote/modulo.

25. Alguma restrição inegociável de dependências ou arquitetura além de Rust,
    modularidade e desempenho? Exemplo: evitar novas bibliotecas grandes,
    aceitar C apenas no bootloader/stub, zero alocação em handlers de IRQ.
