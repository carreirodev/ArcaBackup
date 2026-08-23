//! A porta das operacoes do proprio sistema.
//!
//! O `bcdedit` tem porta desde a E0 porque e o gerenciador de boot. A E6 traz
//! duas coisas que **nao sao firmware** e mesmo assim atravessam a fronteira:
//! a Inicializacao Rapida (B-5) e o `chkdsk` (B-6). Elas nao cabem em
//! [`crate::portas::Firmware`] — pendura-las la faria a porta mentir sobre o
//! que ela e —, e nao podem ficar soltas num `Command::new` no meio de um
//! comando, porque ai B-5 e B-6 deixariam de ter teste sem hardware.
//!
//! # S-1 nao e violado por nenhuma das duas
//!
//! Isto ja esta resolvido no documento, e vale repetir aqui: a correcao D5 do
//! plano delimitou S-1 a **acesso raw ao dispositivo**, e o proprio S-1 diz
//! que `powercfg` e `chkdsk` sao operacoes do sistema, pelas quais o Windows
//! responde. O WMI cai na mesma categoria. Nenhuma assinatura deste modulo
//! entrega handle de dispositivo, caminho bruto nem deslocamento em setores.
//!
//! # O contrato entrega o codigo de saida e o texto bruto, sem julgar
//!
//! Pelo mesmo motivo de [`crate::portas::Firmware`]: quem julga e codigo puro,
//! testavel sem hardware. E ha uma razao a mais, medida na E2 e confirmada
//! nesta etapa — **o texto vem traduzido**. O `chkdsk` desta maquina responde
//! "Nao ha problemas no sistema de arquivos", e o `powercfg /a` responde
//! "Esta acao esta desabilitada na politica do sistema atual". Interpretar
//! frase e o que C-3 existe para evitar; quem decide e o codigo de saida.

use crate::erro::Resultado;
use crate::resumo::Algoritmo;
use std::path::Path;

/// O que uma ferramenta de console respondeu.
///
/// O texto ja vem decodificado pela pagina de codigo em que a ferramenta
/// escreveu — medido nesta etapa: o `chkdsk` escreve em CP850 mesmo chamado de
/// um console em UTF-8. Serve para **mostrar** a quem lê, nunca para decidir.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaidaDeFerramenta {
    pub codigo: i32,
    pub texto: String,
}

impl SaidaDeFerramenta {
    /// As primeiras `quantas` linhas nao vazias, para caber numa tela.
    ///
    /// O `chkdsk` desta maquina imprime cento e poucas linhas, quase todas
    /// barra de progresso. Despejar tudo no pre-voo esconderia o resto do
    /// dialogo do §5.2.
    pub fn resumo(&self, quantas: usize) -> String {
        self.texto
            .lines()
            .map(str::trim)
            .filter(|linha| !linha.is_empty())
            .take(quantas)
            .collect::<Vec<_>>()
            .join(" · ")
    }
}

pub trait Sistema {
    /// O `HiberbootEnabled` do registro, ou `None` quando o valor nao esta la.
    ///
    /// # Por que o registro, e nao o `powercfg`
    ///
    /// O plano manda "verificar Inicializacao Rapida", e a leitura obvia seria
    /// `powercfg /a`. Medido em 22/08/2026, nesta maquina: ele roda sem
    /// elevacao, sai com codigo 0 e responde **em portugues** — a Inicializacao
    /// Rapida aparece sob "estados de suspensao nao disponiveis", com a frase
    /// "Esta acao esta desabilitada na politica do sistema atual". Parsear
    /// frase traduzida e exatamente o erro que a E2 nomeou e que o parser do
    /// `bcdedit` foi construido para evitar. Pior: a frase nao distingue
    /// "desativada pelo usuario" de "indisponivel por outro motivo".
    ///
    /// O valor mora no registro, como numero, e numero nao tem idioma:
    ///
    /// ```text
    /// HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Power
    ///   HiberbootEnabled : REG_DWORD
    /// ```
    ///
    /// `None` e ausencia de verdade, e nao um erro engolido: quem lê decide o
    /// que fazer com "o registro nao diz", e o que **nao** pode acontecer e
    /// isso virar "esta desativada".
    fn inicializacao_rapida(&self) -> Resultado<Option<u32>>;

    /// Roda `chkdsk <letra>: /scan` e devolve o que ele respondeu.
    ///
    /// `/scan`, e nunca `/f`: o `/scan` roda com o volume montado e nao
    /// escreve nada. Agendar o `/f` e oferta de B-6, e quem decide e o usuario.
    fn conferir_volume(&self, letra: char) -> Resultado<SaidaDeFerramenta>;

    /// Roda `certutil -hashfile <caminho> <algoritmo>` e devolve o que ele
    /// respondeu (V-1, PR-1).
    ///
    /// # Por que aqui, e nao em [`crate::portas::Arquivos`]
    ///
    /// Porque quem faz o trabalho e uma **ferramenta do console do Windows**,
    /// e este modulo e o das ferramentas do console — o `powercfg` de B-5, o
    /// `chkdsk` de B-6 e o `shutdown` da E7. A porta dos arquivos entrega
    /// conteudo e metadado por API; esta entrega codigo de saida e texto
    /// bruto, que e exatamente o que se precisa aqui: as tres linhas da
    /// resposta do `certutil` vem **traduzidas**, e quem as julga e codigo
    /// puro em [`crate::resumo::do_certutil`].
    ///
    /// # S-1 continua valendo
    ///
    /// Resumir um arquivo e lê-lo por caminho, pela API do proprio Windows.
    /// Nao ha handle de dispositivo, caminho bruto nem deslocamento em setores
    /// — nem nesta assinatura, nem no que o `certutil` faz.
    ///
    /// # Nao julga, e o motivo e o mesmo do `chkdsk`
    ///
    /// Codigo diferente de zero **nao** vira erro aqui. E resposta: um arquivo
    /// que sumiu entre a leitura do `MD5SUMS` e a conferencia dele responde
    /// `0x80070002`, e isso e uma linha da tela de V-1 — nao o fim do comando.
    /// Quem verifica trinta e nove arquivos nao pode parar no primeiro que
    /// falta e deixar os outros trinta e oito sem resposta.
    fn resumir(&self, caminho: &Path, algoritmo: Algoritmo) -> Resultado<SaidaDeFerramenta>;

    /// Baixa `url` para `destino` com o `curl` do `System32` (PR-1).
    ///
    /// # Por que aqui, e por que `curl` e nao um crate de HTTP
    ///
    /// Mesma categoria das outras: **ferramenta do console do Windows**, com o
    /// contrato de sempre — codigo de saida e texto bruto, e quem julga e
    /// codigo puro. O `curl.exe` 8.21.0 esta no `System32` desde o Windows 10
    /// 1803, medido nesta maquina em 23/08/2026.
    ///
    /// Um crate de HTTP traria TLS, resolucao de nome e a arvore que vem com
    /// os dois — dezenas de caixas — para **uma** transferencia que acontece
    /// uma vez por dispositivo preparado. As tres dependencias continuam tres.
    ///
    /// # O que ele NAO faz e o ponto de PR-1
    ///
    /// Baixar nao verifica nada. Quem verifica e [`Sistema::resumir`] contra a
    /// constante compilada em [`crate::pacote::SHA256`], e essa ordem —
    /// baixar, conferir, so entao extrair — e o que separa este comando de um
    /// que instala o que quer que tenha chegado pelo fio.
    ///
    /// # S-1 continua valendo
    ///
    /// Escrever um arquivo por caminho, pela API do proprio Windows. Nao ha
    /// handle de dispositivo, caminho bruto nem deslocamento em setores.
    fn baixar(&self, url: &str, destino: &Path) -> Resultado<SaidaDeFerramenta>;

    /// Extrai `pacote` dentro de `destino` com o `bsdtar` do `System32`.
    ///
    /// # A armadilha do nome, e ela foi medida
    ///
    /// **`tar` no `PATH` pode nao ser o `bsdtar`.** Medido em 23/08/2026 nesta
    /// maquina: com o Git para Windows instalado, `tar` resolve para o **GNU
    /// tar 1.35** do `/usr/bin`, que **nao abre zip** — ele responde *"This
    /// does not look like a tar archive"* e sai com erro.
    ///
    /// Quem abre zip e o `C:\Windows\System32\tar.exe`, que e o `bsdtar 3.8.8`
    /// / libarchive 3.8.8. O campo que os separa sem ambiguidade e o
    /// `OriginalFilename` do executavel: `bsdtar` num, `tar` no outro.
    ///
    /// Por isso o adaptador chama pelo **caminho absoluto**, e nunca pelo nome.
    /// Confiar no `PATH` faria o `arca prepare` falhar na maquina de quem tem
    /// Git instalado — e falhar **depois de o disco ja ter sido apagado**.
    ///
    /// E o mesmo padrao que o plano da E10 ja tinha registrado com outro nome:
    /// a versao do `tar` estava medida no `ProductVersion` do Windows em vez do
    /// `FileVersion` do bsdtar. Duas vezes a mesma ferramenta enganou por
    /// homonimia.
    fn extrair(&self, pacote: &Path, destino: &Path) -> Resultado<SaidaDeFerramenta>;

    /// Lista o que ha dentro de um pacote, sem extrair.
    ///
    /// Existe para a conferencia de [`crate::pacote::o_que_falta`] poder
    /// acontecer **antes** de escrever no dispositivo, e nao depois: um zip
    /// sem o `bootx64.efi` produz um dispositivo que nao boota, e isso so se
    /// descobriria depois de o Windows ter sido apagado.
    fn listar_pacote(&self, pacote: &Path) -> Resultado<SaidaDeFerramenta>;

    /// Reinicia a maquina agora.
    ///
    /// # Por que atras de porta, e nao um `Command::new` no comando
    ///
    /// Porque sem porta o comando que arma deixa de ter teste. Todo o resto da
    /// E7 — montar o bloco, gravar o estado, marcar o boot unico, conferir com
    /// C-3 — e verificavel sem hardware **desde que a ultima linha nao
    /// reinicie de verdade**. Um teste que reinicia a maquina de quem o roda
    /// nao e um teste.
    ///
    /// # Por que aqui e nao numa porta propria
    ///
    /// E a mesma categoria das outras duas deste modulo: uma operacao do
    /// **proprio sistema**, pela qual o Windows responde, e nao um acesso ao
    /// disco. A correcao D5 do plano delimitou S-1 a acesso raw ao
    /// dispositivo; reiniciar nao chega perto disso.
    ///
    /// # O contrato nao promete voltar
    ///
    /// Em exito esta chamada devolve `Ok(())` e a maquina desliga logo em
    /// seguida — nao ha como saber quanto depois. Quem a chama nao pode ter
    /// nada a fazer com o retorno alem de propagar a falha: **tudo que precisa
    /// acontecer antes do reinicio ja aconteceu quando ela e chamada**.
    fn reiniciar(&self) -> Resultado<()>;
}
