//! A porta das operacoes do sistema, implementada por registro e `chkdsk`.
//!
//! Duas fronteiras diferentes atras da mesma porta, e cada uma pelo caminho
//! que nao depende de idioma:
//!
//! - **B-5 lê o registro**, e nao o `powercfg`. Um `REG_DWORD` nao tem
//!   traducao; a saida do `powercfg /a` tem, e ela nem separa "desativada" de
//!   "indisponivel".
//! - **B-6 roda o `chkdsk` e olha o codigo de saida**, nunca o texto. Medido
//!   nesta maquina: `chkdsk C: /scan` elevado sai com **codigo 0** em 16,3 s,
//!   e o texto vem em **CP850** mesmo chamado de um console em UTF-8 — o mesmo
//!   caso do `bcdedit` da E2, e `de_pagina_de_codigo` resolve.

use crate::erro::{Erro, Resultado};
use crate::portas::{SaidaDeFerramenta, Sistema};
use crate::resumo::Algoritmo;
use std::path::Path;
use std::process::Command;

use super::texto::{de_pagina_de_codigo, pagina_do_console, para_utf16};

/// As ferramentas do `System32`, **por caminho absoluto**.
///
/// # Isto nasceu de uma medicao, e o defeito e silencioso
///
/// Medido em 23/08/2026: com o Git para Windows instalado, `tar` no `PATH`
/// resolve para o **GNU tar 1.35** do `/usr/bin`, e ele **nao abre zip** —
/// responde *"This does not look like a tar archive"* e sai com erro. O que
/// abre zip e o `C:\Windows\System32\tar.exe`, que e o `bsdtar 3.8.8`. Os dois
/// se chamam `tar.exe`; o que os separa e o `OriginalFilename` do executavel,
/// `bsdtar` num e `tar` no outro.
///
/// O `curl` tem o mesmo problema pelo mesmo motivo — ha um em `/mingw64/bin` —,
/// e o `certutil` nao tem homonimo.
///
/// **O modo de falha e caro**: o `arca prepare` extrai o pacote *depois* de ter
/// apagado o disco. Um `tar` que nao entende zip falharia com o dispositivo ja
/// destruido e nada instalado nele.
///
/// Os outros tres comandos que este adaptador roda — `chkdsk`, `certutil`,
/// `shutdown` — continuam pelo nome: eles ja rodavam assim desde a E6 e a E11,
/// nenhum deles tem homonimo conhecido, e mudar isso agora seria uma alteracao
/// sem medicao em caminho ja exercitado em hardware.
struct Ferramentas {
    curl: &'static str,
    bsdtar: &'static str,
}

/// O `System32` desta instalacao.
///
/// Caminho fixo, e nao `%SystemRoot%`: o ARCA ja recusa BIOS legada, RAID e
/// Storage Spaces (§2), e uma instalacao do Windows fora de `C:\Windows` esta
/// na mesma categoria de coisa que este projeto nao suporta. Fixar torna a
/// leitura obvia; ler a variavel daria a impressao de suportar o que nao se
/// testou.
const FERRAMENTAS: Ferramentas = Ferramentas {
    curl: r"C:\Windows\System32\curl.exe",
    bsdtar: r"C:\Windows\System32\tar.exe",
};

/// O codigo de saida e o texto de uma ferramenta de console, decodificados.
///
/// Junta `stdout` e `stderr` porque as duas ferramentas novas escrevem a
/// recusa no `stderr` — o `curl -sS` de proposito — e quem lê a resposta
/// precisa dela inteira. **Nao julga**: codigo diferente de zero e resposta, e
/// quem decide o que fazer com ela e codigo puro, como no `chkdsk` e no
/// `certutil`.
fn resposta(saida: &std::process::Output) -> SaidaDeFerramenta {
    let pagina = pagina_do_console();
    let mut texto = de_pagina_de_codigo(&saida.stdout, pagina);
    if !saida.stderr.is_empty() {
        texto.push_str(&de_pagina_de_codigo(&saida.stderr, pagina));
    }

    SaidaDeFerramenta {
        codigo: saida.status.code().unwrap_or(-1),
        texto,
    }
}

/// Onde a Inicializacao Rapida mora, como numero.
const CHAVE_DA_ENERGIA: &str = r"SYSTEM\CurrentControlSet\Control\Session Manager\Power";

/// O valor. Diferente de zero e Inicializacao Rapida ligada.
const VALOR_DA_INICIALIZACAO_RAPIDA: &str = "HiberbootEnabled";

#[derive(Debug, Clone, Copy, Default)]
pub struct SistemaDoWindows;

impl Sistema for SistemaDoWindows {
    fn inicializacao_rapida(&self) -> Resultado<Option<u32>> {
        ler_dword(CHAVE_DA_ENERGIA, VALOR_DA_INICIALIZACAO_RAPIDA)
    }

    fn conferir_volume(&self, letra: char) -> Resultado<SaidaDeFerramenta> {
        // `/scan`, e nunca `/f`: roda com o volume montado e nao escreve nada.
        let volume = format!("{letra}:");
        let saida = Command::new("chkdsk")
            .args([volume.as_str(), "/scan"])
            .output()
            .map_err(|origem| Erro::Ferramenta {
                ferramenta: "chkdsk",
                origem,
            })?;

        let pagina = pagina_do_console();
        let mut texto = de_pagina_de_codigo(&saida.stdout, pagina);
        if !saida.stderr.is_empty() {
            texto.push_str(&de_pagina_de_codigo(&saida.stderr, pagina));
        }

        // Codigo diferente de zero **nao** vira erro aqui, ao contrario do que
        // o adaptador do `bcdedit` faz. E deliberado: o `chkdsk` usa o codigo
        // de saida para dizer o que achou no disco — 1 e "havia erro e foi
        // corrigido", 2 e "nao deu para conferir", 3 e "acesso negado". Todos
        // sao **resposta**, e quem os interpreta e o pre-voo, que tem teste.
        // Transforma-los em erro aqui faria o pre-voo inteiro parar por causa
        // de um disco que acusou alguma coisa — que e justamente o caso em que
        // B-6 quer falar com o usuario.
        Ok(SaidaDeFerramenta {
            codigo: saida.status.code().unwrap_or(-1),
            texto,
        })
    }

    fn resumir(&self, caminho: &Path, algoritmo: Algoritmo) -> Resultado<SaidaDeFerramenta> {
        // `-hashfile <caminho> <ALGORITMO>`, na ordem medida em 23/08/2026 e
        // preservada em `recursos/capturas/verificacao-md5-medida-2026-08-23.txt`.
        let saida = Command::new("certutil")
            .arg("-hashfile")
            .arg(caminho)
            .arg(algoritmo.como_certutil_o_chama())
            .output()
            .map_err(|origem| Erro::Ferramenta {
                ferramenta: "certutil",
                origem,
            })?;

        // A mesma decodificacao do `chkdsk`: a resposta vem na pagina de
        // codigo do console, e nao em UTF-8. Medido nesta maquina, em
        // portugues: `MD5 hash de ...:` e `comando concluido com exito`.
        let pagina = pagina_do_console();
        let mut texto = de_pagina_de_codigo(&saida.stdout, pagina);
        if !saida.stderr.is_empty() {
            texto.push_str(&de_pagina_de_codigo(&saida.stderr, pagina));
        }

        // Codigo diferente de zero e **resposta**, como no `chkdsk` e ao
        // contrario do `shutdown`. Ver a doc da porta: um arquivo que sumiu e
        // uma linha da tela de V-1, e nao o fim do comando.
        Ok(SaidaDeFerramenta {
            codigo: saida.status.code().unwrap_or(-1),
            texto,
        })
    }

    fn baixar(&self, url: &str, destino: &Path) -> Resultado<SaidaDeFerramenta> {
        // `-L` porque o SourceForge redireciona para um mirror; `-f` para que
        // uma pagina de erro HTTP nao vire um arquivo de 500 bytes com cara de
        // pacote; `--retry` porque meio giga por uma rede domestica cai.
        //
        // `-sS` cala a barra de progresso e mantem a mensagem de erro: o
        // andamento e impresso pelo ARCA, e a barra do `curl` no meio da tela
        // do §7.1 atrapalharia mais do que ajuda.
        let saida = Command::new(FERRAMENTAS.curl)
            .arg("-sS")
            .arg("-L")
            .arg("-f")
            .args(["--retry", "2"])
            .arg("-o")
            .arg(destino)
            .arg(url)
            .output()
            .map_err(|origem| Erro::Ferramenta {
                ferramenta: "curl",
                origem,
            })?;

        Ok(resposta(&saida))
    }

    fn extrair(&self, pacote: &Path, destino: &Path) -> Resultado<SaidaDeFerramenta> {
        // `-x -f <pacote> -C <destino>`. O `bsdtar` reconhece zip pelo
        // conteudo, e nao pela extensao.
        let saida = Command::new(FERRAMENTAS.bsdtar)
            .arg("-x")
            .arg("-f")
            .arg(pacote)
            .arg("-C")
            .arg(destino)
            .output()
            .map_err(|origem| Erro::Ferramenta {
                ferramenta: "bsdtar",
                origem,
            })?;

        Ok(resposta(&saida))
    }

    fn listar_pacote(&self, pacote: &Path) -> Resultado<SaidaDeFerramenta> {
        let saida = Command::new(FERRAMENTAS.bsdtar)
            .arg("-t")
            .arg("-f")
            .arg(pacote)
            .output()
            .map_err(|origem| Erro::Ferramenta {
                ferramenta: "bsdtar",
                origem,
            })?;

        Ok(resposta(&saida))
    }

    fn reiniciar(&self) -> Resultado<()> {
        // # `shutdown /r /t 0`, e nao `ExitWindowsEx`
        //
        // A alternativa da API exige habilitar `SeShutdownPrivilege` no token
        // deste processo antes de chamar — `OpenProcessToken`,
        // `LookupPrivilegeValue`, `AdjustTokenPrivileges` — e depois lidar com
        // o fato de que `AdjustTokenPrivileges` **sai com sucesso mesmo
        // quando nao ajustou tudo**, e quem quiser saber precisa consultar o
        // `GetLastError` a parte. E o mesmo modo de falha do `bcdedit` que
        // C-3 existe para desconfiar: a chamada responde bem e nao fez o que
        // se pediu.
        //
        // O `shutdown.exe` faz esse trabalho, e o codigo de saida dele diz se
        // deu certo. E uma ferramenta do proprio Windows, como o `powercfg` e
        // o `chkdsk` — a mesma categoria que criou esta porta.
        //
        // `/t 0` porque o aviso de C-9 ja foi impresso e a confirmacao de S-2
        // ja foi digitada: uma contagem regressiva aqui so daria a impressao
        // de haver uma chance de desistir que nao existe mais. O ponto sem
        // volta ficou para tras quando o boot unico foi marcado.
        let saida = Command::new("shutdown")
            .args(["/r", "/t", "0"])
            .output()
            .map_err(|origem| Erro::Ferramenta {
                ferramenta: "shutdown",
                origem,
            })?;

        let codigo = saida.status.code().unwrap_or(-1);
        if codigo != 0 {
            // Ao contrario do `chkdsk`, aqui codigo diferente de zero **e**
            // erro: o `shutdown` nao usa o codigo de saida para relatar
            // achados, so para dizer que nao conseguiu. E um reinicio que nao
            // aconteceu com o dispositivo armado nao pode passar por feito —
            // quem lê precisa saber que a maquina continua no Windows com uma
            // receita esperando.
            let pagina = pagina_do_console();
            let mut texto = de_pagina_de_codigo(&saida.stdout, pagina);
            texto.push_str(&de_pagina_de_codigo(&saida.stderr, pagina));
            return Err(Erro::FerramentaRecusou {
                ferramenta: "shutdown",
                codigo,
                saida: texto.trim().to_string(),
            });
        }

        Ok(())
    }
}

/// Um `REG_DWORD` de `HKEY_LOCAL_MACHINE`, ou `None` quando ele nao esta la.
///
/// `None` e ausencia de verdade. Quem lê decide o que fazer com "o registro
/// nao diz"; o que nao pode acontecer e isso virar "esta desativada".
fn ler_dword(subchave: &str, valor: &str) -> Resultado<Option<u32>> {
    use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
    use windows_sys::Win32::System::Registry::{
        HKEY_LOCAL_MACHINE, RRF_RT_REG_DWORD, RegGetValueW,
    };

    let subchave_larga = para_utf16(subchave);
    let valor_largo = para_utf16(valor);

    let mut dados: u32 = 0;
    let mut tamanho: u32 = std::mem::size_of::<u32>() as u32;

    // SEGURANCA: as duas cadeias terminam em NUL e vivem ate o fim da chamada;
    // o ponteiro de dados aponta para uma variavel desta pilha, e o tamanho
    // informado e o dela. `RRF_RT_REG_DWORD` faz a API recusar o valor se ele
    // nao for um DWORD, em vez de reinterpretar bytes.
    let estado = unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            subchave_larga.as_ptr(),
            valor_largo.as_ptr(),
            RRF_RT_REG_DWORD,
            std::ptr::null_mut(),
            (&raw mut dados).cast(),
            &mut tamanho,
        )
    };

    if estado == ERROR_SUCCESS {
        return Ok(Some(dados));
    }
    if estado == ERROR_FILE_NOT_FOUND {
        // Nem a chave nem o valor existem. E resposta, e nao falha.
        return Ok(None);
    }

    Err(Erro::Ferramenta {
        ferramenta: "registro",
        origem: std::io::Error::from_raw_os_error(estado as i32),
    })
}

#[cfg(all(test, windows))]
mod testes {
    use super::*;

    #[test]
    fn a_inicializacao_rapida_desta_maquina_responde() {
        // Nao se cobra o **valor** — ele muda de maquina para maquina e o
        // usuario pode altera-lo. Cobra-se que a leitura funcione e devolva um
        // numero, que e o que separa esta implementacao de uma que interpreta
        // frase traduzida.
        let lida = SistemaDoWindows
            .inicializacao_rapida()
            .expect("o registro responde");

        assert!(
            lida.is_some(),
            "o valor `{VALOR_DA_INICIALIZACAO_RAPIDA}` nao esta no registro desta maquina"
        );
    }

    #[test]
    fn um_valor_que_nao_existe_e_none_e_nao_erro() {
        // A distincao que o `Option` carrega: "o registro nao diz" nao pode
        // virar erro nem, pior, virar zero.
        let ausente = ler_dword(CHAVE_DA_ENERGIA, "ArcaValorQueNaoExiste").expect("nao e erro");
        assert_eq!(ausente, None);
    }

    #[test]
    fn uma_chave_que_nao_existe_tambem_e_none() {
        let ausente = ler_dword(r"SOFTWARE\ArcaChaveQueNaoExiste", "x").expect("nao e erro");
        assert_eq!(ausente, None);
    }

    #[test]
    fn o_tar_do_system32_e_o_bsdtar_e_nao_o_do_git() {
        // **A armadilha medida em 23/08/2026.** Com o Git para Windows
        // instalado, `tar` no `PATH` e o GNU tar 1.35, que nao abre zip. O
        // campo que separa os dois sem ambiguidade e o `OriginalFilename`.
        //
        // Este teste roda contra a maquina de verdade, e e o unico que pegaria
        // uma futura versao do Windows trocando a ferramenta.
        let saida = Command::new(FERRAMENTAS.bsdtar)
            .arg("--version")
            .output()
            .expect("o tar do System32 responde");

        let texto = String::from_utf8_lossy(&saida.stdout);
        assert!(
            texto.starts_with("bsdtar"),
            "o `{}` nao e o bsdtar, e sim: {texto}",
            FERRAMENTAS.bsdtar
        );
    }

    #[test]
    fn as_ferramentas_novas_sao_chamadas_por_caminho_absoluto() {
        // O que este teste guarda nao e o caminho — e a **ausencia de
        // dependencia do `PATH`**. Trocar por `Command::new("tar")` faria o
        // `arca prepare` falhar na maquina de quem tem Git instalado, e falhar
        // depois de o disco ja ter sido apagado.
        for ferramenta in [FERRAMENTAS.curl, FERRAMENTAS.bsdtar] {
            assert!(
                Path::new(ferramenta).is_absolute(),
                "`{ferramenta}` tem de ser caminho absoluto"
            );
            assert!(
                Path::new(ferramenta).exists(),
                "`{ferramenta}` nao existe nesta maquina"
            );
        }
    }

    #[test]
    fn o_curl_do_system32_e_o_da_microsoft() {
        let saida = Command::new(FERRAMENTAS.curl)
            .arg("--version")
            .output()
            .expect("o curl do System32 responde");

        let texto = String::from_utf8_lossy(&saida.stdout);
        assert!(texto.starts_with("curl "), "veio: {texto}");
    }
}
