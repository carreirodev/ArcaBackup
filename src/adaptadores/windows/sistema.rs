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
use crate::portas::{Limites, SaidaDeFerramenta, Sistema, Transferencia};
use crate::resumo::Algoritmo;
use std::ffi::OsString;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

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

/// De quanto em quanto tempo o download olha o tamanho do destino.
const VOLTA_DO_DOWNLOAD: Duration = Duration::from_millis(200);

/// De quanto em quanto tempo o andamento e avisado. PR-7 pede a linha da tela
/// reescrita ao menos a cada 2 s; um segundo deixa folga para a tela.
const INTERVALO_DO_ANDAMENTO: Duration = Duration::from_secs(1);

/// O comeco da linha que o `-w` pede ao `curl`, com o codigo HTTP da ultima
/// resposta.
const MARCA_DO_HTTP: &str = "arca-curl http=";

/// A linha de comando do `curl` para um download que retoma e nao fica mudo.
///
/// - `-C -` continua do byte em que o destino termina, e com o destino vazio
///   ou ausente baixa do comeco — medido em 04/10/2026: com 100 bytes no
///   destino o pedido sai com `Range: bytes=100-`, e sem eles, sem `Range`;
/// - `-N` tira o buffer da escrita. Sem ele, medido no mesmo dia, o destino
///   fica com 0 bytes enquanto os bytes chegam — o andamento mentiria, a
///   parada seria vista onde nao ha, e encerrar o `curl` perderia o que
///   estava no buffer;
/// - `--connect-timeout` e o unico limite que fica com o `curl`. O da fonte
///   parada e deste adaptador: ver [`Sistema::baixar`];
/// - `-f` para que uma pagina de erro HTTP nao vire um arquivo de 500 bytes
///   com cara de pacote, e `-L` porque o SourceForge redireciona.
///
/// **Sem `--retry`.** Ele faria uma fonte parada esperar tres vezes antes de
/// a seguinte ser tentada, e o que ele comprava — uma rede que cai no meio —
/// a retomada do `.parcial` compra pela proxima fonte.
fn argumentos_do_curl(url: &str, destino: &Path, limites: Limites) -> Vec<OsString> {
    let mut argumentos: Vec<OsString> = ["-sS", "-N", "-L", "-f", "-C", "-", "--connect-timeout"]
        .into_iter()
        .map(OsString::from)
        .collect();
    argumentos.push(limites.conectar.as_secs_f64().to_string().into());
    argumentos.push("-w".into());
    argumentos.push(format!("{MARCA_DO_HTTP}%{{http_code}}\\n").into());
    argumentos.push("-o".into());
    argumentos.push(destino.as_os_str().to_os_string());
    argumentos.push(url.into());
    argumentos
}

/// O codigo HTTP que o `-w` escreveu, e `None` quando ele e `000`: nenhuma
/// resposta chegou.
fn http_do_curl(stdout: &str) -> Option<u16> {
    stdout
        .lines()
        .find_map(|linha| linha.trim().strip_prefix(MARCA_DO_HTTP))
        .and_then(|codigo| codigo.trim().parse::<u16>().ok())
        .filter(|codigo| *codigo != 0)
}

/// Quantos bytes o destino tem. O que nao existe tem zero: o `curl` so cria o
/// arquivo quando o primeiro byte chega.
fn tamanho_de(destino: &Path) -> u64 {
    std::fs::metadata(destino).map(|m| m.len()).unwrap_or(0)
}

/// Le tudo o que um processo escrever num cano, numa thread propria — um cano
/// cheio que ninguem lê para o `curl` no meio do download.
fn ler_tudo(cano: Option<impl Read + Send + 'static>) -> std::thread::JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(mut cano) = cano {
            let _ = cano.read_to_end(&mut bytes);
        }
        bytes
    })
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

    fn baixar(
        &self,
        url: &str,
        destino: &Path,
        limites: Limites,
        andamento: &mut dyn FnMut(u64),
    ) -> Resultado<Transferencia> {
        let falhou = |origem| Erro::Ferramenta {
            ferramenta: "curl",
            origem,
        };

        // `-sS` cala a barra de progresso e mantem a mensagem de erro: o
        // andamento e impresso pelo ARCA, e a barra do `curl` no meio da tela
        // do §7.1 atrapalharia mais do que ajuda.
        let mut curl = Command::new(FERRAMENTAS.curl)
            .args(argumentos_do_curl(url, destino, limites))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(falhou)?;
        let stdout = ler_tudo(curl.stdout.take());
        let stderr = ler_tudo(curl.stderr.take());

        // A fonte parada e julgada pelo destino que nao cresce, e nao pelo
        // `--speed-time`: o `curl` mede velocidade por media e, medido em
        // 04/10/2026, abandona `T + 6 s` depois do ultimo byte. O relogio
        // aqui so mede o intervalo entre duas leituras deste mesmo processo.
        // O primeiro aviso vem depois do primeiro intervalo, e nao agora: a
        // linha que quem chamou acabou de escrever — de quanto retoma — fica
        // na tela esse segundo.
        let mut tamanho = tamanho_de(destino);
        let mut cresceu_em = Instant::now();
        let mut avisado_em = Instant::now();

        let mut parou = false;
        let terminou = loop {
            if let Some(estado) = curl.try_wait().map_err(falhou)? {
                break Some(estado);
            }
            std::thread::sleep(VOLTA_DO_DOWNLOAD);

            let agora = tamanho_de(destino);
            if agora != tamanho {
                tamanho = agora;
                cresceu_em = Instant::now();
            }
            if avisado_em.elapsed() >= INTERVALO_DO_ANDAMENTO {
                andamento(tamanho);
                avisado_em = Instant::now();
            }
            if cresceu_em.elapsed() >= limites.parado {
                // Encerrar e o que um Ctrl+C faz, e o `.parcial` fica com o
                // que veio: com `-N`, nada esta no buffer.
                let _ = curl.kill();
                let _ = curl.wait();
                parou = true;
                break None;
            }
        };

        let pagina = pagina_do_console();
        let escrito = de_pagina_de_codigo(&stdout.join().unwrap_or_default(), pagina);
        let texto = de_pagina_de_codigo(&stderr.join().unwrap_or_default(), pagina);

        let tamanho_no_destino = tamanho_de(destino);
        andamento(tamanho_no_destino);

        Ok(Transferencia {
            codigo: terminou.and_then(|estado| estado.code()).unwrap_or(-1),
            texto,
            http: http_do_curl(&escrito),
            parou,
            tamanho_no_destino,
        })
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

// Dois atributos, e nao `cfg(all(test, windows))`: a varredura de
// `tests/b10_nada_e_apagado.rs` reconhece o modulo de testes pelo
// `#[cfg(test)]`, e os testes do download (04/10/2026) limpam o proprio
// diretorio temporario — que nao e o que B-10 governa.
#[cfg(test)]
#[cfg(windows)]
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

    // ─────────── o download contra servidores desta maquina (PR-7) ───────────
    //
    // O `curl` de verdade, contra um servidor HTTP em `127.0.0.1` que faz o
    // que cada teste pede. E o que mede o adaptador sem depender da internet:
    // as quatro formas de uma fonte responder foram medidas assim em
    // 04/10/2026, antes de este codigo existir.

    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;
    use std::sync::mpsc;

    /// O pacote de mentira: 1000 bytes que nao se repetem em 251, para que
    /// uma retomada no byte errado nao passe por certa.
    fn corpo() -> Vec<u8> {
        (0..1000u32).map(|i| (i % 251) as u8).collect()
    }

    #[derive(Clone, Copy)]
    enum Servidor {
        /// `200` com os 100 primeiros bytes, e depois mais nada.
        Parar,
        /// `404`.
        NaoAchou,
        /// Honra o `Range`: `206` com o que falta.
        Retomar,
        /// `200`, 100 bytes a cada 400 ms.
        Devagar,
    }

    /// Sobe o servidor e devolve a URL e o canal por onde chegam os pedidos.
    fn servir(como: Servidor) -> (String, mpsc::Receiver<String>) {
        let ouvinte = TcpListener::bind("127.0.0.1:0").unwrap();
        let porta = ouvinte.local_addr().unwrap().port();
        let (pedidos, recebidos) = mpsc::channel();

        std::thread::spawn(move || {
            for conexao in ouvinte.incoming() {
                let mut conexao = conexao.unwrap();
                let mut leitor = BufReader::new(conexao.try_clone().unwrap());
                let mut pedido = String::new();
                loop {
                    let mut linha = String::new();
                    if leitor.read_line(&mut linha).unwrap_or(0) == 0 || linha == "\r\n" {
                        break;
                    }
                    pedido.push_str(&linha);
                }
                let _ = pedidos.send(pedido.clone());

                let corpo = corpo();
                match como {
                    Servidor::Parar => {
                        let _ = write!(conexao, "HTTP/1.1 200 OK\r\nContent-Length: 1000\r\n\r\n");
                        let _ = conexao.write_all(&corpo[..100]);
                        let _ = conexao.flush();
                        std::thread::sleep(Duration::from_secs(30));
                    }
                    Servidor::NaoAchou => {
                        let _ = write!(
                            conexao,
                            "HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\n\r\nnot found"
                        );
                    }
                    Servidor::Retomar => {
                        let de = pedido
                            .lines()
                            .find_map(|l| l.strip_prefix("Range: bytes="))
                            .and_then(|r| r.trim_end_matches('-').parse::<usize>().ok())
                            .unwrap_or(0);
                        let _ = write!(
                            conexao,
                            "HTTP/1.1 206 Partial Content\r\nContent-Range: bytes {de}-999/1000\r\nContent-Length: {}\r\n\r\n",
                            1000 - de
                        );
                        let _ = conexao.write_all(&corpo[de..]);
                    }
                    Servidor::Devagar => {
                        let _ = write!(conexao, "HTTP/1.1 200 OK\r\nContent-Length: 1000\r\n\r\n");
                        for pedaco in corpo.chunks(100) {
                            let _ = conexao.write_all(pedaco);
                            let _ = conexao.flush();
                            std::thread::sleep(Duration::from_millis(400));
                        }
                    }
                }
            }
        });

        (format!("http://127.0.0.1:{porta}/pacote.zip"), recebidos)
    }

    /// Um `.parcial` novo num diretorio so deste teste.
    fn parcial(etiqueta: &str) -> std::path::PathBuf {
        let pasta =
            std::env::temp_dir().join(format!("arca-curl-{etiqueta}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&pasta);
        std::fs::create_dir_all(&pasta).unwrap();
        pasta.join("clonezilla-live-3.3.3-15-amd64.zip.parcial")
    }

    /// Limites curtos: os de verdade sao 30 e 60 s, e o que se prova aqui e
    /// o mecanismo, nao o numero.
    const CURTOS: Limites = Limites {
        conectar: Duration::from_secs(5),
        parado: Duration::from_secs(2),
    };

    #[test]
    fn o_curl_recebe_os_limites_e_retoma() {
        let argumentos: Vec<String> = argumentos_do_curl(
            "https://exemplo/pacote.zip",
            Path::new(r"C:\ARCA\pacote.zip.parcial"),
            crate::pacote::LIMITES,
        )
        .into_iter()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();

        let depois = |opcao: &str| {
            argumentos
                .iter()
                .position(|a| a == opcao)
                .and_then(|i| argumentos.get(i + 1))
                .map(String::as_str)
        };

        assert_eq!(depois("--connect-timeout"), Some("30"), "{argumentos:?}");
        assert_eq!(depois("-C"), Some("-"), "{argumentos:?}");
        assert_eq!(depois("-o"), Some(r"C:\ARCA\pacote.zip.parcial"));
        for presente in ["-N", "-f", "-L"] {
            assert!(argumentos.iter().any(|a| a == presente), "falta {presente}");
        }
        for ausente in [
            "--retry",
            "--speed-time",
            "--speed-limit",
            "--remove-on-error",
        ] {
            assert!(
                !argumentos.iter().any(|a| a == ausente),
                "{ausente} esta la: {argumentos:?}"
            );
        }
    }

    #[test]
    fn uma_fonte_parada_e_abandonada_e_o_parcial_fica_com_o_que_veio() {
        let (url, _pedidos) = servir(Servidor::Parar);
        let destino = parcial("parada");

        let inicio = Instant::now();
        let transferencia = SistemaDoWindows
            .baixar(&url, &destino, CURTOS, &mut |_| {})
            .expect("o curl roda");
        let levou = inicio.elapsed();

        assert!(transferencia.parou, "{transferencia:?}");
        assert!(
            levou < Duration::from_secs(5),
            "uma fonte parada levou {levou:?} para ser abandonada, com limite de 2 s"
        );
        assert_eq!(transferencia.tamanho_no_destino, 100);
        assert_eq!(std::fs::read(&destino).unwrap(), &corpo()[..100]);

        let _ = std::fs::remove_dir_all(destino.parent().unwrap());
    }

    #[test]
    fn o_curl_retoma_do_byte_em_que_o_parcial_termina() {
        let (url, pedidos) = servir(Servidor::Retomar);
        let destino = parcial("retomada");
        std::fs::write(&destino, &corpo()[..100]).unwrap();

        let transferencia = SistemaDoWindows
            .baixar(&url, &destino, CURTOS, &mut |_| {})
            .expect("o curl roda");

        let pedido = pedidos.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(pedido.contains("Range: bytes=100-"), "{pedido}");
        assert_eq!(transferencia.codigo, 0, "{transferencia:?}");
        assert_eq!(transferencia.http, Some(206));
        assert_eq!(transferencia.tamanho_no_destino, 1000);
        assert_eq!(std::fs::read(&destino).unwrap(), corpo());

        let _ = std::fs::remove_dir_all(destino.parent().unwrap());
    }

    #[test]
    fn um_erro_http_volta_com_o_codigo() {
        let (url, _pedidos) = servir(Servidor::NaoAchou);
        let destino = parcial("404");

        let transferencia = SistemaDoWindows
            .baixar(&url, &destino, CURTOS, &mut |_| {})
            .expect("o curl roda");

        assert_eq!(transferencia.codigo, 22, "{transferencia:?}");
        assert_eq!(transferencia.http, Some(404));
        assert!(!transferencia.parou);
        assert_eq!(transferencia.tamanho_no_destino, 0);

        let _ = std::fs::remove_dir_all(destino.parent().unwrap());
    }

    #[test]
    fn o_andamento_chega_ao_menos_a_cada_2_s() {
        // Uns 4 s de download. O que se cobra e o intervalo entre dois
        // avisos, que e o que mantem a linha da tela viva (PR-7).
        let (url, _pedidos) = servir(Servidor::Devagar);
        let destino = parcial("andamento");

        let mut avisos: Vec<(Instant, u64)> = Vec::new();
        let transferencia = SistemaDoWindows
            .baixar(&url, &destino, CURTOS, &mut |bytes| {
                avisos.push((Instant::now(), bytes))
            })
            .expect("o curl roda");

        assert_eq!(transferencia.codigo, 0, "{transferencia:?}");
        assert!(avisos.len() >= 3, "so {} avisos", avisos.len());
        for par in avisos.windows(2) {
            assert!(
                par[1].0 - par[0].0 <= Duration::from_secs(2),
                "{:?} entre dois avisos",
                par[1].0 - par[0].0
            );
            assert!(par[1].1 >= par[0].1, "o andamento voltou: {avisos:?}");
        }
        assert_eq!(avisos.last().map(|(_, bytes)| *bytes), Some(1000));

        let _ = std::fs::remove_dir_all(destino.parent().unwrap());
    }
}
