//! O nome da imagem, e o que faz um nome ser recusado (B-2).
//!
//! O nome que o usuario digita atravessa tres mundos: vira pasta no NTFS do
//! `ARCAVAULT`, vira diretorio no Linux do Clonezilla, e vira **palavra dentro
//! de uma string de shell** que o `grub` passa adiante. Um nome que passe nos
//! dois primeiros e quebre no terceiro faz a maquina reiniciar, abrir um menu
//! em ingles tecnico e ficar parada esperando alguem que ja saiu de perto.
//!
//! # Por lista de permissao, e nao de recusa
//!
//! A tentacao e listar o que e perigoso — aspa, cifrao, crase, ponto e
//! virgula — e deixar passar o resto. Uma lista dessas so esta certa enquanto
//! ninguem esquecer um caractere, e esquecer um caractere aqui custa uma
//! execucao real. A lista de permissao erra para o outro lado: um nome
//! legitimo e recusado, o usuario troca um caractere e segue.
//!
//! O que se permite e `A-Z a-z 0-9 . _ -`, que e o que os nomes ja usados no
//! dispositivo precisam: `2026-08-21_WindowsCompleto`, `ARCA-TESTE-02`.

use std::fmt;

/// Ate onde um nome de imagem pode ir.
///
/// Quem manda aqui nao e o NTFS, cujo limite por componente e 255. E a
/// **linha de comando do kernel**: o nome aparece dez vezes na receita de
/// backup, e cada caractere a mais custa dez na linha do `grub.cfg`. Medido:
/// um nome de 13 caracteres gera 921; um de 64 gera 1431, contra os 1536 que
/// `crate::receita::TETO_DOS_PARAMETROS` orca — 105 de folga, para uma falha
/// que acontece em silencio e so aparece com a maquina parada num menu.
///
/// Com 48, a receita mais longa fica em 1271 e sobram 265 do orcamento, mais
/// os 143 que a reserva do `menuentry` ja tem de sobra. Os nomes de verdade
/// nem chegam perto: `2026-08-21_WindowsCompleto` tem 26.
///
/// Baixar isto e barato; descobrir que a linha estourou, nao. O teto de
/// verdade continua sendo cobrado em `crate::receita`, sobre a linha pronta.
pub const LIMITE: usize = 48;

/// Nomes que o Windows reserva para dispositivos, em qualquer caixa e com
/// qualquer extensao.
///
/// Do lado Windows uma pasta com um destes nomes nao chega a ser criada — mas
/// **quem cria a pasta e o Clonezilla**, do lado Linux, onde `COM1` e um nome
/// como outro qualquer. Um `arca backup COM1` gravaria a imagem sem
/// reclamacao nenhuma, e na volta todo `E:\COM1\MD5SUMS` que o Windows
/// tentasse abrir resolveria o dispositivo serial em vez do diretorio. A
/// recusa tem de acontecer aqui, antes de a receita ser montada.
///
/// `COM0` e `LPT0` estao na lista da Microsoft junto dos outros nove de cada.
const RESERVADOS: [&str; 24] = [
    "CON", "PRN", "AUX", "NUL", "COM0", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
    "COM8", "COM9", "LPT0", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Por que um nome foi recusado.
///
/// Uma variante por motivo, e nao uma mensagem so: quem digitou um nome com
/// acento precisa ouvir "acento", nao "nome invalido".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Recusa {
    Vazio,
    ComEspaco,
    ComAcento {
        caractere: char,
    },
    CaractereInvalido {
        caractere: char,
    },
    ComecaComTraco,
    ComecaComPonto,
    TerminaComPonto,
    Reservado {
        nome: String,
    },

    /// Uma das pastas de servico do `ARCAVAULT`. Distinta de
    /// [`Recusa::Reservado`], que e do Windows: esta e do dispositivo, e o
    /// estrago e outro.
    DoDispositivo {
        nome: &'static str,
    },

    LongoDemais {
        tem: usize,
        limite: usize,
    },
}

impl fmt::Display for Recusa {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Recusa::Vazio => write!(f, "nome vazio"),
            Recusa::ComEspaco => write!(
                f,
                "nome com espaco: o Clonezilla recebe o nome como palavra dentro de uma string de shell, e um espaco a reparte em duas"
            ),
            Recusa::ComAcento { caractere } => write!(
                f,
                "nome com acento (`{caractere}`): o que atravessa o grub e o live system e ASCII, e um acento chega do outro lado como outra coisa"
            ),
            Recusa::CaractereInvalido { caractere } => write!(
                f,
                "caractere `{caractere}` nao e aceito em nome de imagem: valem letras, digitos, ponto, sublinhado e traco"
            ),
            Recusa::ComecaComTraco => write!(
                f,
                "nome comecando com `-`: o `ocs-sr` o leria como opcao, e nao como nome de imagem"
            ),
            Recusa::ComecaComPonto => write!(
                f,
                "nome comecando com `.`: no Linux e pasta oculta, e `.` e `..` sao o diretorio corrente e o pai"
            ),
            Recusa::TerminaComPonto => write!(
                f,
                "nome terminando com `.`: o Windows corta o ponto final em silencio, e a pasta criada nao teria o nome pedido"
            ),
            Recusa::Reservado { nome } => write!(
                f,
                "`{nome}` e nome reservado do Windows para dispositivo: o Clonezilla criaria a pasta do lado Linux, e do lado Windows nada dentro dela abriria"
            ),
            Recusa::DoDispositivo { nome } => write!(
                f,
                "`{nome}` e uma pasta de servico do dispositivo: a imagem seria gravada dentro dela e sumiria da listagem, sem que ninguem visse por que"
            ),
            Recusa::LongoDemais { tem, limite } => {
                write!(f, "nome com {tem} caracteres, e o limite e {limite}")
            }
        }
    }
}

/// Um nome de imagem ja julgado por B-2.
///
/// So existe se passou. Quem tem um `Nome` em maos nao precisa validar de
/// novo, e nenhum caminho do codigo consegue montar receita com nome cru.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nome(String);

impl Nome {
    /// Julga um nome digitado (B-2).
    pub fn novo(bruto: &str) -> Result<Nome, Recusa> {
        if bruto.is_empty() {
            return Err(Recusa::Vazio);
        }

        // Contado em caracteres, e nao em bytes: o limite e sobre o nome que
        // se lê, e um nome com acento nem chega aqui.
        let quantos = bruto.chars().count();
        if quantos > LIMITE {
            return Err(Recusa::LongoDemais {
                tem: quantos,
                limite: LIMITE,
            });
        }

        for caractere in bruto.chars() {
            // O espaco tem recusa propria porque e o erro que quem digita
            // comete, e "caractere invalido: ` `" nao diz nada a ninguem.
            if caractere == ' ' {
                return Err(Recusa::ComEspaco);
            }
            if !caractere.is_ascii() {
                return Err(Recusa::ComAcento { caractere });
            }
            if !e_permitido(caractere) {
                return Err(Recusa::CaractereInvalido { caractere });
            }
        }

        if bruto.starts_with('-') {
            return Err(Recusa::ComecaComTraco);
        }
        if bruto.starts_with('.') {
            return Err(Recusa::ComecaComPonto);
        }
        if bruto.ends_with('.') {
            return Err(Recusa::TerminaComPonto);
        }

        // No Windows o nome reservado continua reservado com extensao: `CON`
        // e `CON.txt` sao os dois o dispositivo.
        let raiz = bruto.split('.').next().unwrap_or(bruto);
        if RESERVADOS
            .iter()
            .any(|reservado| reservado.eq_ignore_ascii_case(raiz))
        {
            return Err(Recusa::Reservado {
                nome: raiz.to_string(),
            });
        }

        // As pastas de servico do `ARCAVAULT`, pelo nome inteiro: uma imagem
        // chamada `ARCA-LOGS` seria gravada por cima da pasta de logs e
        // sumiria da enumeracao de [`crate::imagens`], que a pula. Invisivel
        // no `arca list` e invisivel para o pre-voo de B-3, que e quem
        // recusaria o nome ja usado.
        if let Some(reservada) = crate::imagens::RESERVADAS
            .iter()
            .find(|reservada| reservada.eq_ignore_ascii_case(bruto))
        {
            return Err(Recusa::DoDispositivo { nome: reservada });
        }

        Ok(Nome(bruto.to_string()))
    }

    pub fn como_texto(&self) -> &str {
        &self.0
    }

    /// Um `Nome` que **nao passou** por B-2, para exercitar as barreiras que
    /// ficam depois dela.
    ///
    /// Existe porque a recusa por tamanho de linha de [`crate::receita`] e
    /// inalcancavel pelo caminho normal — `LIMITE` a impede —, e uma barreira
    /// que nenhum teste consegue disparar e uma barreira que ninguem sabe se
    /// funciona.
    #[cfg(test)]
    pub fn sem_julgar_para_teste(bruto: &str) -> Nome {
        Nome(bruto.to_string())
    }
}

impl fmt::Display for Nome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A lista de permissao. Tudo que nao esta aqui e recusado, inclusive o que
/// ninguem lembrou de proibir.
fn e_permitido(caractere: char) -> bool {
    caractere.is_ascii_alphanumeric() || matches!(caractere, '.' | '_' | '-')
}

#[cfg(test)]
mod testes {
    use super::*;

    /// Os nomes que existem de verdade no dispositivo, lidos em 22/08/2026.
    /// Se algum deles fosse recusado, o validador estaria errado — nao eles.
    const NOMES_REAIS: [&str; 3] = [
        "2026-08-21_WindowsCompleto",
        "ARCA-TESTE-02",
        "ARCA-TESTE-03",
    ];

    #[test]
    fn os_nomes_que_ja_estao_no_dispositivo_passam() {
        for nome in NOMES_REAIS {
            assert!(Nome::novo(nome).is_ok(), "`{nome}` foi recusado");
        }
    }

    #[test]
    fn o_nome_do_prd_passa() {
        // §5.2: `arca backup 2026-08-22_Apps`.
        assert_eq!(
            Nome::novo("2026-08-22_Apps").unwrap().como_texto(),
            "2026-08-22_Apps"
        );
    }

    #[test]
    fn nome_vazio_e_recusado() {
        assert_eq!(Nome::novo("").unwrap_err(), Recusa::Vazio);
    }

    #[test]
    fn espaco_e_recusado() {
        // O nome vira palavra dentro da string que o grub passa ao bash: um
        // espaco o reparte, e o `ocs-sr` recebe dois argumentos onde devia
        // receber um.
        assert_eq!(Nome::novo("meu backup").unwrap_err(), Recusa::ComEspaco);
        assert_eq!(Nome::novo(" backup").unwrap_err(), Recusa::ComEspaco);
        assert_eq!(Nome::novo("backup ").unwrap_err(), Recusa::ComEspaco);
    }

    #[test]
    fn acento_e_recusado_com_o_caractere_nomeado() {
        assert_eq!(
            Nome::novo("backup_do_Antônio").unwrap_err(),
            Recusa::ComAcento { caractere: 'ô' }
        );
    }

    #[test]
    fn os_metacaracteres_de_shell_sao_todos_recusados() {
        // Esta e a lista que fecha C-2 na origem: nenhum deles chega a ser
        // montado numa receita, porque o nome nao passa daqui.
        for perigoso in [
            "a|b", "a;b", "a&b", "a'b", "a\"b", "a`b", "a$b", "a(b", "a)b", "a<b", "a>b", "a*b",
            "a?b", "a!b", "a#b", "a~b", "a{b", "a}b", "a[b", "a]b", "a\\b", "a/b", "a:b", "a=b",
            "a,b", "a+b", "a%b", "a@b", "a^b",
        ] {
            assert!(
                matches!(Nome::novo(perigoso), Err(Recusa::CaractereInvalido { .. })),
                "`{perigoso}` passou pelo validador"
            );
        }
    }

    #[test]
    fn quebra_de_linha_e_caractere_de_controle_sao_recusados() {
        for bruto in ["a\nb", "a\rb", "a\tb", "a\0b"] {
            assert!(
                Nome::novo(bruto).is_err(),
                "{bruto:?} passou pelo validador"
            );
        }
    }

    #[test]
    fn nome_comecando_com_traco_e_recusado() {
        // O `ocs-sr` leria `-scs` como opcao, e nao como nome de imagem.
        assert_eq!(Nome::novo("-scs").unwrap_err(), Recusa::ComecaComTraco);
        // No meio, o traco e o separador dos nomes reais.
        assert!(Nome::novo("ARCA-TESTE-02").is_ok());
    }

    #[test]
    fn ponto_no_comeco_e_no_fim_e_recusado() {
        assert_eq!(Nome::novo(".oculto").unwrap_err(), Recusa::ComecaComPonto);
        assert_eq!(Nome::novo(".").unwrap_err(), Recusa::ComecaComPonto);
        assert_eq!(Nome::novo("..").unwrap_err(), Recusa::ComecaComPonto);
        assert_eq!(Nome::novo("backup.").unwrap_err(), Recusa::TerminaComPonto);
        // No meio, ponto e legitimo.
        assert!(Nome::novo("v1.2").is_ok());
    }

    #[test]
    fn os_nomes_reservados_do_windows_sao_recusados_em_qualquer_caixa() {
        for bruto in ["CON", "con", "NUL", "Aux", "COM1", "lpt9"] {
            assert!(
                matches!(Nome::novo(bruto), Err(Recusa::Reservado { .. })),
                "`{bruto}` passou, e o Windows nao cria pasta com esse nome"
            );
        }
    }

    #[test]
    fn o_reservado_com_extensao_tambem_e_recusado() {
        // `CON.txt` tambem e o dispositivo, no Windows.
        assert!(matches!(
            Nome::novo("CON.txt"),
            Err(Recusa::Reservado { .. })
        ));
    }

    #[test]
    fn um_reservado_como_prefixo_nao_e_reservado() {
        assert!(Nome::novo("CONTEUDO").is_ok());
        assert!(Nome::novo("NULO").is_ok());
    }

    #[test]
    fn as_pastas_de_servico_do_dispositivo_sao_recusadas() {
        // Uma imagem chamada `ARCA-LOGS` seria gravada por cima da pasta de
        // logs — e sumiria da listagem, porque `crate::imagens` pula esse
        // nome ao enumerar. Invisivel no `arca list`, e invisivel tambem para
        // o pre-voo de B-3, que e quem recusaria o nome ja usado.
        for bruto in ["ARCA-LOGS", "arca-logs", "ARCA-DOCS"] {
            assert!(
                matches!(Nome::novo(bruto), Err(Recusa::DoDispositivo { .. })),
                "`{bruto}` passou pelo validador"
            );
        }
    }

    #[test]
    fn toda_pasta_que_a_enumeracao_pula_e_recusada_como_nome() {
        // Vale para a lista inteira, e nao para os dois que me ocorreram: o
        // que `crate::imagens` esconder da listagem, este validador tem de
        // impedir que exista. Se alguem acrescentar uma pasta de servico la,
        // este teste cobra a recusa aqui.
        for reservada in crate::imagens::RESERVADAS {
            assert!(
                Nome::novo(reservada).is_err(),
                "`{reservada}` e pulada pela enumeracao e passou como nome de imagem"
            );
        }
    }

    #[test]
    fn um_nome_que_apenas_comeca_como_pasta_de_servico_passa() {
        // A recusa e pelo nome inteiro. `ARCA-LOGS-2026` e uma pasta como
        // outra qualquer, e a enumeracao a mostra normalmente.
        assert!(Nome::novo("ARCA-LOGS-2026").is_ok());
    }

    #[test]
    fn nome_longo_demais_e_recusado_dizendo_o_limite() {
        assert_eq!(
            Nome::novo(&"a".repeat(LIMITE + 1)).unwrap_err(),
            Recusa::LongoDemais {
                tem: LIMITE + 1,
                limite: LIMITE
            }
        );
        assert!(Nome::novo(&"a".repeat(LIMITE)).is_ok());
    }

    #[test]
    fn cada_recusa_tem_mensagem_propria() {
        // Nenhum desfecho do ARCA e silencio (§5.5). Duas recusas com a mesma
        // mensagem sao uma delas sem mensagem.
        let recusas = [
            Recusa::Vazio,
            Recusa::ComEspaco,
            Recusa::ComAcento { caractere: 'ô' },
            Recusa::CaractereInvalido { caractere: '|' },
            Recusa::ComecaComTraco,
            Recusa::ComecaComPonto,
            Recusa::TerminaComPonto,
            Recusa::Reservado {
                nome: "CON".to_string(),
            },
            Recusa::DoDispositivo { nome: "ARCA-LOGS" },
            Recusa::LongoDemais {
                tem: 90,
                limite: LIMITE,
            },
        ];

        let mensagens: Vec<String> = recusas.iter().map(|r| r.to_string()).collect();
        for (i, mensagem) in mensagens.iter().enumerate() {
            assert!(!mensagem.is_empty(), "recusa {i} sem mensagem");
            assert_eq!(
                mensagens.iter().filter(|outra| *outra == mensagem).count(),
                1,
                "duas recusas com a mesma mensagem: {mensagem}"
            );
        }
    }
}
