//! O estado do job: o `estado.json` do `ARCABOOT` (§4.1, §4.3, C-11).
//!
//! Seis campos — selo, comando, nome, disco alvo, momento do armar e situacao
//! — num arquivo que mora **no dispositivo, e nunca no `C:`**. A razao esta no §4.1
//! do PRD e nao e preferencia: o `C:` e o que a restauracao substitui, e o que
//! julga a restauracao nao pode morar no disco que ela troca. Morando no
//! `ARCABOOT`, o estado sobrevive a qualquer restauracao.
//!
//! # Sem `serde`, e por que isso nao e teimosia
//!
//! O `Cargo.toml` tem tres dependencias e nenhuma delas serializa JSON.
//! Acrescentar `serde`/`serde_json` seria o caminho de sempre; escrever seis
//! campos a mao e menos codigo do que parece **porque os seis valores nao
//! podem conter nada que o JSON precise escapar** — e isso nao e sorte, e
//! propriedade de validadores que ja existem:
//!
//! | campo | quem o julga | alfabeto |
//! |---|---|---|
//! | selo | [`Selo::novo`] | 16 digitos hexadecimais minusculos |
//! | comando | [`Operacao`] | `backup` ou `restauracao` |
//! | nome | [`Nome::novo`] (B-2) | `A-Z a-z 0-9 . _ -` |
//! | disco | [`Disco::novo`] | `[a-z][a-z0-9]*` |
//! | momento | [`MomentoDoArmar`] | digitos, `-`, `:`, `T`, `+` |
//! | situacao | [`Situacao`] | `armado` ou `colhido` |
//!
//! Nenhum deles alcanca `"`, `\`, caractere de controle ou nao-ASCII. Ainda
//! assim [`campo`] confere antes de escrever, porque "ja foi validado" e
//! exatamente o que este projeto ja viu ser falso duas vezes.
//!
//! O sexto campo entrou na etapa E8, e a premissa que sustenta escrever a mao
//! continua de pe **porque ele tambem tem alfabeto fechado** — foi por isso
//! que ele e um estado e nao uma data. O ADR-0006 avisava que a discussao
//! voltaria com o campo novo na mesa, e ela voltou: ver [`Situacao`].
//!
//! Ver `docs/adr/0006-o-selo-e-o-estado-sem-dependencia-nova.md`.
//!
//! # O leitor recusa em vez de ler pela metade
//!
//! [`Estado::de_json`] nao e um parser de JSON: e o leitor do arquivo que este
//! modulo escreve. Ele recusa escape, chave desconhecida, chave repetida,
//! chave faltando e qualquer coisa depois do `}`. Um arquivo truncado no meio
//! — o desligamento que a escrita atomica existe para cobrir — cai em uma
//! dessas recusas, e ha teste que corta o arquivo em **todos** os comprimentos
//! possiveis para provar isso.

use crate::dispositivo::ARCA_LOGS;
use crate::erro::{Erro, Resultado};
use crate::nome::Nome;
use crate::portas::{Arquivos, Entropia, Relogio};
use crate::receita::{BYTES_DO_SELO, Disco, Operacao, Selo};
use std::fmt;
use std::path::{Path, PathBuf};

/// Gera um selo (C-11).
///
/// O selo nasce **ao armar**, e so ali: e o instante em que o job passa a
/// existir. Quem chama e a etapa E7.
pub fn gerar_selo(entropia: &dyn Entropia) -> Resultado<Selo> {
    let mut bytes = [0u8; BYTES_DO_SELO];
    entropia.preencher(&mut bytes)?;
    Ok(Selo::de_bytes(&bytes))
}

/// Quando o job foi armado. **Informativo, e so.**
///
/// # O tipo e a defesa, e nao o comentario
///
/// S-6 proibe comparar uma data escrita pelo Windows com outra escrita pelo
/// Linux. Uma trava construida sobre essa comparacao ja reprovou um backup
/// perfeito neste projeto (§4.3, ADR-0001), e um comentario nao teria impedido
/// — o comentario existia.
///
/// Por isso este tipo **guarda texto, e nao um `DateTime`**. Nao ha o que
/// subtrair, nao ha o que comparar com o `modificado_em` de um arquivo, e nao
/// ha acessor que devolva algo comparavel: quem quisesse violar S-6 precisaria
/// primeiro parsear a string de volta, de proposito, num `let` que apareceria
/// no diff.
///
/// Nao deriva `PartialOrd` nem `Ord`, e `tests/s6_o_tempo_nao_decide.rs` cobra
/// isso a cada build — junto de nao existir metodo daqui que devolva
/// `DateTime`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MomentoDoArmar(String);

/// Quantos caracteres tem um `2026-08-22T18:14:03-03:00`.
const LARGURA_DO_MOMENTO: usize = 25;

impl MomentoDoArmar {
    /// O momento presente, pelo relogio do Windows.
    pub fn agora(relogio: &dyn Relogio) -> MomentoDoArmar {
        // Com o deslocamento explicito: sem ele, o mesmo texto significaria
        // horas diferentes conforme quem o lê. Ele nao serve para comparar
        // nada — serve para uma pessoa reconhecer quando armou.
        MomentoDoArmar(relogio.agora().format("%Y-%m-%dT%H:%M:%S%:z").to_string())
    }

    /// O momento a partir do texto gravado, conferindo a forma.
    ///
    /// Confere a **forma**, e nao a validade da data. Nao ha por que saber se
    /// `2026-02-31` existe: nada decide nada a partir daqui. O que importa e
    /// que o texto seja ASCII reconhecivel, para que um arquivo corrompido
    /// nao passe por estado bom.
    fn de_texto(bruto: &str) -> Result<MomentoDoArmar, RecusaDoEstado> {
        let recusar = || RecusaDoEstado::MomentoInvalido {
            tem: bruto.to_string(),
        };

        let caracteres: Vec<char> = bruto.chars().collect();
        if caracteres.len() != LARGURA_DO_MOMENTO {
            return Err(recusar());
        }

        // `DDDD-DD-DDTDD:DD:DD±DD:DD`, posicao a posicao.
        const DIGITOS: [usize; 18] = [
            0, 1, 2, 3, 5, 6, 8, 9, 11, 12, 14, 15, 17, 18, 20, 21, 23, 24,
        ];
        const FIXOS: [(usize, char); 6] = [
            (4, '-'),
            (7, '-'),
            (10, 'T'),
            (13, ':'),
            (16, ':'),
            (22, ':'),
        ];

        if DIGITOS.iter().any(|i| !caracteres[*i].is_ascii_digit())
            || FIXOS
                .iter()
                .any(|(i, esperado)| caracteres[*i] != *esperado)
            || !matches!(caracteres[19], '+' | '-')
        {
            return Err(recusar());
        }

        Ok(MomentoDoArmar(bruto.to_string()))
    }

    pub fn como_texto(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MomentoDoArmar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Se o job ainda espera desfecho, ou se ele ja foi colhido.
///
/// # Por que um estado, e nao apagar o arquivo
///
/// A pergunta e da etapa E8, e ela nasce de uma contradicao que a E5 deixou
/// aberta: depois de um `arca desarmar`, o `arca status` mostrava "Boot unico:
/// nao armado" ao lado de um job pendente. Colher encerra o job — e havia tres
/// saidas para o `estado.json` ao encerrar: apagar, marcar, ou deixar e
/// distinguir por outro sinal.
///
/// **Marcar.** As outras duas custam mais:
///
/// - **Apagar** obrigaria a discutir B-10 outra vez. O [`crate::desarme`] tem
///   uma secao inteira defendendo que apagar o `bootsequence` nao fura B-10, e
///   o argumento e que a marca de boot unico e uma **intencao** que o proprio
///   ARCA gravou. O `estado.json` colhido nao e intencao, e **registro**: e o
///   unico lugar que liga um selo a um nome de imagem. Apagado ele, um
///   `arca-fim.txt` que aparecesse depois nao teria a quem pertencer, e a
///   mensagem de job fantasma passaria a ser a resposta para tudo.
/// - **Deixar e distinguir por outro sinal** — pela existencia do
///   `arca-fim.txt`, por exemplo — poria a decisao do lado do reinicio que o
///   ARCA nao escreve. E o sinal falharia justamente onde mais importa: um job
///   cujo boot nao aconteceu **nao tem** `arca-fim.txt`, e ficaria pendente
///   para sempre.
///
/// Marcar custa uma chave e nao apaga nada. O `arca status` passa a dizer
/// "ultimo job, colhido" em vez de "job pendente", e a contradicao fecha.
///
/// # Nao e uma data, e de proposito
///
/// A tentacao seria gravar `colhido_em`. Duas razoes contra: precisaria de um
/// valor sentinela enquanto o job nao foi colhido, e — a que pesa — poria mais
/// um instante ao lado do `armado_em` num arquivo cujo tipo de tempo existe
/// justamente para tornar a comparacao dificil (S-6, ADR-0006). Duas datas
/// lado a lado sao um convite a subtrai-las.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Situacao {
    /// Armado e ainda nao colhido. E o **job** do `CONTEXT.md`: existe entre o
    /// reinicio e a leitura do desfecho.
    Armado,

    /// Colhido: o ARCA ja leu o que havia no lugar do desfecho e disse o que
    /// era. Deixa de ser job pendente, qualquer que tenha sido o desfecho.
    Colhido,
}

impl Situacao {
    /// O texto que vai para o arquivo. Alfabeto fechado, como o de
    /// [`Operacao`].
    pub fn nome(self) -> &'static str {
        match self {
            Situacao::Armado => "armado",
            Situacao::Colhido => "colhido",
        }
    }
}

impl fmt::Display for Situacao {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.nome())
    }
}

/// O job armado, e se ele ja foi colhido.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Estado {
    /// O que liga este job ao desfecho que o Clonezilla escrever (§4.3).
    pub selo: Selo,
    pub comando: Operacao,

    /// A imagem sobre a qual a operacao roda, quando ha uma.
    ///
    /// # `None` na sondagem, e o sentinela e o mesmo argumento do `disco`
    ///
    /// A E12 trouxe uma operacao que **nao opera sobre imagem nenhuma**: o
    /// `lsblk` da sondagem lê os discos da maquina, e ela existe justamente
    /// para o dispositivo que ainda nao tem imagem (§4.5, P-26).
    ///
    /// A chave continua obrigatoria no JSON, pela razao do campo abaixo, e o
    /// valor ausente e a **string vazia**. O precedente e da E11 e o argumento
    /// e o mesmo, conferido antes de ser reusado: [`Nome::novo`] recusa o
    /// vazio com [`crate::nome::Recusa::Vazio`] desde a E1, entao o vazio
    /// nunca foi um nome de imagem possivel e nao pode colidir com nenhum. Um
    /// sentinela como `sondagem` colidiria — B-2 o aceita como nome de imagem.
    pub nome: Option<Nome>,

    /// O disco que a receita nomeia, com o nome que o Linux lhe da.
    ///
    /// # `None` na verificacao, e como isso cabe no arquivo
    ///
    /// A E11 trouxe uma operacao que **nao nomeia disco nenhum**: o
    /// `ocs-chkimg` opera sobre a imagem. O campo continua obrigatorio no JSON
    /// — o leitor recusa chave faltando, e afrouxar isso para um campo so
    /// tiraria a propriedade que torna o leitor confiavel —, e o valor
    /// ausente e a **string vazia**.
    ///
    /// A escolha nao e arbitraria: `Disco::novo("")` ja recusava, com
    /// [`crate::receita::RecusaDaReceita::DiscoVazio`], desde a E3. O vazio
    /// nunca foi um nome de disco possivel, entao usa-lo para dizer "nenhum"
    /// **nao pode colidir** com nome nenhum que o Linux de. Um sentinela como
    /// `nenhum` colidiria: `[a-z][a-z0-9]*` o aceitaria.
    ///
    /// E a premissa do [ADR-0006](../docs/adr/0006-o-selo-e-o-estado-sem-dependencia-nova.md)
    /// continua de pe — a string vazia nao alcanca `"`, `\`, controle nem
    /// nao-ASCII, e por isso escrever o JSON a mao continua defensavel.
    ///
    /// O leitor cobra a **coerencia** com o comando: `verificacao` exige vazio,
    /// e as outras duas exigem nome. Aceitar as quatro combinacoes deixaria
    /// passar um `estado.json` que arma `restauracao` sem dizer em que disco.
    pub disco: Option<Disco>,

    pub armado_em: MomentoDoArmar,

    /// Se ha desfecho por colher. Acrescentado na etapa E8 — ver [`Situacao`].
    pub situacao: Situacao,
}

/// As seis chaves do arquivo, na ordem em que sao escritas.
///
/// Eram cinco ate a etapa E8. A sexta e obrigatoria como as outras, e nao
/// opcional: o leitor recusa chave faltando, e afrouxar isso para um campo so
/// tiraria a propriedade que torna o leitor confiavel — ou o arquivo esta
/// inteiro, ou nao se age sobre ele. Ha exatamente um escritor, e ele esta
/// neste repositorio.
const CHAVES: [&str; 6] = ["selo", "comando", "nome", "disco", "armado_em", "situacao"];

impl Estado {
    /// Como este job se apresenta numa tela: `backup \`2026-08-22_Apps\``, ou
    /// so `sondagem`, que nao opera sobre imagem nenhuma.
    ///
    /// Existe desde a E12, quando o `nome` virou opcional e cinco telas
    /// passaram a ter de decidir o que dizer no lugar dele. Cinco decisoes
    /// separadas divergiriam, e a diferenca apareceria como uma tela dizendo
    /// ``sondagem ` ` `` — um par de crases vazio no lugar de um nome.
    pub fn descricao(&self) -> String {
        match &self.nome {
            Some(nome) => format!("{} `{nome}`", self.comando.nome()),
            None => self.comando.nome().to_string(),
        }
    }

    /// O `estado.json`, em texto.
    ///
    /// Uma chave por linha, e nao compacto: quem abrir este arquivo depois de
    /// um desligamento no meio de uma operacao esta procurando entender o que
    /// estava armado, e cinco linhas se lê de relance. Truncado, ele tambem
    /// **parece** truncado.
    pub fn como_json(&self) -> Result<String, RecusaDoEstado> {
        let valores = [
            self.selo.como_texto(),
            self.comando.nome(),
            // O vazio e o "nenhuma imagem" da sondagem — ver o campo.
            self.nome.as_ref().map_or("", Nome::como_texto),
            // O vazio e o "nenhum disco" da verificacao e da sondagem.
            self.disco.as_ref().map_or("", Disco::como_texto),
            self.armado_em.como_texto(),
            self.situacao.nome(),
        ];

        let mut linhas = Vec::with_capacity(CHAVES.len());
        for (chave, valor) in CHAVES.iter().zip(valores) {
            linhas.push(format!("  {}", campo(chave, valor)?));
        }

        Ok(format!("{{\n{}\n}}\n", linhas.join(",\n")))
    }

    /// O `estado.json` de volta, ou a razao de o arquivo nao servir.
    pub fn de_json(texto: &str) -> Result<Estado, RecusaDoEstado> {
        let pares = ler_objeto(texto)?;

        let mut achados: Vec<Option<String>> = vec![None; CHAVES.len()];
        for (chave, valor) in pares {
            let Some(posicao) = CHAVES.iter().position(|conhecida| *conhecida == chave) else {
                // Chave desconhecida e recusa, e nao algo a ignorar: ela veio
                // de uma versao que sabe alguma coisa que esta nao sabe, e
                // seguir em frente seria agir sobre metade de um estado.
                return Err(RecusaDoEstado::ChaveDesconhecida { chave });
            };
            if achados[posicao].is_some() {
                return Err(RecusaDoEstado::ChaveRepetida {
                    chave: CHAVES[posicao],
                });
            }
            achados[posicao] = Some(valor);
        }

        let mut tomar = |posicao: usize| -> Result<String, RecusaDoEstado> {
            achados[posicao]
                .take()
                .ok_or(RecusaDoEstado::ChaveFaltando {
                    chave: CHAVES[posicao],
                })
        };

        let selo = tomar(0)?;
        let comando = tomar(1)?;
        let nome = tomar(2)?;
        let disco = tomar(3)?;
        let armado_em = tomar(4)?;
        let situacao = tomar(5)?;

        // Cada campo volta pelo **mesmo** validador que o julgou na ida. Sem
        // isso, um `estado.json` mexido a mao entregaria um `Selo` que
        // `Selo::novo` teria recusado, e o resto do sistema confia em ter um
        // `Selo` em maos.
        let comando = operacao_de_texto(&comando)?;

        // A coerencia entre o comando e o disco, nos **dois** sentidos. Um
        // `estado.json` que dissesse `restauracao` com disco vazio armaria uma
        // operacao destrutiva sem dizer sobre o que; um que dissesse
        // `verificacao` com disco nomeado carregaria um valor que nenhuma
        // receita usa, e que ninguem conferiria.
        let disco = match (comando.nomeia_disco(), disco.is_empty()) {
            (true, false) => Some(
                Disco::novo(&disco).map_err(|_| RecusaDoEstado::DiscoInvalido { tem: disco })?,
            ),
            (false, true) => None,
            (nomeia, _) => {
                return Err(RecusaDoEstado::DiscoIncoerente {
                    comando: comando.nome(),
                    tem: disco,
                    nomeia_disco: nomeia,
                });
            }
        };

        // A mesma cobranca, no outro eixo e nos dois sentidos (E12). Um
        // `estado.json` dizendo `sondagem` com nome de imagem carregaria um
        // valor que receita nenhuma usa; um dizendo `backup` com nome vazio
        // armaria uma gravacao sem dizer com que nome — e a pasta do desfecho
        // sai do nome, entao o desfecho iria para o lugar da sondagem.
        let nome = match (comando.nomeia_imagem(), nome.is_empty()) {
            (true, false) => Some(Nome::novo(&nome).map_err(RecusaDoEstado::NomeInvalido)?),
            (false, true) => None,
            (nomeia, _) => {
                return Err(RecusaDoEstado::NomeIncoerente {
                    comando: comando.nome(),
                    tem: nome,
                    nomeia_imagem: nomeia,
                });
            }
        };

        Ok(Estado {
            selo: Selo::novo(&selo).map_err(|_| RecusaDoEstado::SeloInvalido { tem: selo })?,
            comando,
            nome,
            disco,
            armado_em: MomentoDoArmar::de_texto(&armado_em)?,
            situacao: situacao_de_texto(&situacao)?,
        })
    }
}

fn situacao_de_texto(bruto: &str) -> Result<Situacao, RecusaDoEstado> {
    for situacao in [Situacao::Armado, Situacao::Colhido] {
        if situacao.nome() == bruto {
            return Ok(situacao);
        }
    }
    Err(RecusaDoEstado::SituacaoInvalida {
        tem: bruto.to_string(),
    })
}

/// Onde o desfecho deste job vai aparecer, do lado Windows.
///
/// O caminho e montado a partir de [`crate::receita::pasta_do_log`], que e a
/// mesma funcao de que a receita monta o caminho Linux. Os dois lados do
/// reinicio nao podem divergir no nome da pasta, e a unica forma de garantir
/// isso e nao haver dois lugares onde ele se escreva.
pub fn caminho_do_desfecho(
    raiz_do_vault: &Path,
    comando: Operacao,
    nome: Option<&Nome>,
) -> PathBuf {
    raiz_do_vault
        .join(ARCA_LOGS)
        .join(crate::receita::pasta_do_log(comando, nome))
        .join(crate::receita::ARCA_FIM)
}

/// Grava o estado no `ARCABOOT`, criando `arca\` se preciso.
///
/// O diretorio nao existe num dispositivo preparado a mao — o desta mesa nao
/// o tinha —, e e por isso que ele e criado aqui em vez de se supor pronto.
///
/// A escrita e atomica porque um `estado.json` pela metade e pior do que
/// nenhum: ele diria que ha job pendente sem dizer qual, e o que decide o que
/// fazer na volta e justamente este arquivo.
pub fn gravar(arquivos: &dyn Arquivos, caminho: &Path, estado: &Estado) -> Resultado<()> {
    let json = estado.como_json().map_err(Erro::EstadoRecusado)?;

    if let Some(pasta) = caminho.parent() {
        arquivos.criar_diretorio(pasta)?;
    }

    arquivos.escrever_atomico(caminho, &json)
}

/// Lê o estado do `ARCABOOT`.
pub fn ler(arquivos: &dyn Arquivos, caminho: &Path) -> Resultado<Estado> {
    let texto = arquivos.ler_texto(caminho)?;
    Estado::de_json(&texto).map_err(Erro::EstadoRecusado)
}

/// Um par `"chave": "valor"` pronto para a linha, conferido antes de sair.
///
/// A conferencia e o que torna a escrita a mao defensavel. Nenhum dos cinco
/// valores alcanca `"`, `\`, controle ou nao-ASCII — e ainda assim se confere,
/// porque "ja foi validado antes" e a frase que produziu dois dos achados mais
/// caros deste projeto.
fn campo(chave: &str, valor: &str) -> Result<String, RecusaDoEstado> {
    for caractere in valor.chars() {
        if caractere == '"' || caractere == '\\' || caractere.is_control() || !caractere.is_ascii()
        {
            return Err(RecusaDoEstado::ValorPrecisaEscape {
                chave: chave.to_string(),
                caractere,
            });
        }
    }
    Ok(format!("\"{chave}\": \"{valor}\""))
}

/// Lê `{ "chave": "valor", ... }` e devolve os pares, na ordem em que vieram.
///
/// Nao e um parser de JSON: e o leitor do que [`Estado::como_json`] escreve.
/// A diferenca aparece em `\`, que aqui e recusa em vez de escape — honrar
/// escapes faria este leitor aceitar textos que aquele escritor nao consegue
/// produzir, e o que se quer e o contrario.
fn ler_objeto(texto: &str) -> Result<Vec<(String, String)>, RecusaDoEstado> {
    let bytes: Vec<char> = texto.chars().collect();
    let mut i = 0usize;

    let pular_brancos = |i: &mut usize| {
        while *i < bytes.len() && bytes[*i].is_whitespace() {
            *i += 1;
        }
    };

    let exigir = |i: &mut usize, esperado: char| -> Result<(), RecusaDoEstado> {
        if *i >= bytes.len() {
            return Err(RecusaDoEstado::Truncado);
        }
        if bytes[*i] != esperado {
            return Err(RecusaDoEstado::CaractereInesperado {
                posicao: *i,
                tem: bytes[*i],
                esperado,
            });
        }
        *i += 1;
        Ok(())
    };

    let ler_texto_entre_aspas = |i: &mut usize| -> Result<String, RecusaDoEstado> {
        exigir(i, '"')?;
        let mut saida = String::new();
        loop {
            if *i >= bytes.len() {
                // Aspa que nunca fecha e a assinatura de um arquivo cortado no
                // meio de um valor.
                return Err(RecusaDoEstado::Truncado);
            }
            let caractere = bytes[*i];
            *i += 1;
            match caractere {
                '"' => return Ok(saida),
                '\\' => return Err(RecusaDoEstado::EscapeNoTexto),
                outro if outro.is_control() => {
                    return Err(RecusaDoEstado::CaractereDeControle {
                        codigo: outro as u32,
                    });
                }
                outro => saida.push(outro),
            }
        }
    };

    pular_brancos(&mut i);
    exigir(&mut i, '{')?;

    let mut pares = Vec::new();
    loop {
        pular_brancos(&mut i);
        if i >= bytes.len() {
            return Err(RecusaDoEstado::Truncado);
        }
        if bytes[i] == '}' {
            i += 1;
            break;
        }

        let chave = ler_texto_entre_aspas(&mut i)?;
        pular_brancos(&mut i);
        exigir(&mut i, ':')?;
        pular_brancos(&mut i);
        let valor = ler_texto_entre_aspas(&mut i)?;
        pares.push((chave, valor));

        pular_brancos(&mut i);
        if i >= bytes.len() {
            return Err(RecusaDoEstado::Truncado);
        }
        match bytes[i] {
            ',' => i += 1,
            '}' => {
                i += 1;
                break;
            }
            tem => {
                return Err(RecusaDoEstado::CaractereInesperado {
                    posicao: i,
                    tem,
                    esperado: '}',
                });
            }
        }
    }

    // Depois do `}` so pode haver branco. Um segundo objeto colado no fim e
    // um arquivo que duas gravacoes concorrentes produziriam, e ele nao pode
    // passar por estado bom so porque a primeira metade se lê.
    pular_brancos(&mut i);
    if i < bytes.len() {
        return Err(RecusaDoEstado::SobrouTextoDepoisDoFim {
            resto: bytes[i..].iter().collect(),
        });
    }

    Ok(pares)
}

fn operacao_de_texto(bruto: &str) -> Result<Operacao, RecusaDoEstado> {
    for operacao in [
        Operacao::Backup,
        Operacao::Restauracao,
        Operacao::Verificacao,
        Operacao::Sondagem,
    ] {
        if operacao.nome() == bruto {
            return Ok(operacao);
        }
    }
    Err(RecusaDoEstado::ComandoDesconhecido {
        tem: bruto.to_string(),
    })
}

/// Por que um `estado.json` nao serve.
///
/// Uma variante por motivo, como em toda recusa deste projeto: quem abrir um
/// dispositivo com o estado ilegivel precisa saber se foi desligamento no meio
/// da gravacao ou arquivo de outra versao.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecusaDoEstado {
    /// O arquivo acaba antes de o objeto fechar. E o desligamento no meio.
    Truncado,
    CaractereInesperado {
        posicao: usize,
        tem: char,
        esperado: char,
    },
    EscapeNoTexto,
    CaractereDeControle {
        codigo: u32,
    },
    SobrouTextoDepoisDoFim {
        resto: String,
    },
    ChaveDesconhecida {
        chave: String,
    },
    ChaveRepetida {
        chave: &'static str,
    },
    ChaveFaltando {
        chave: &'static str,
    },
    SeloInvalido {
        tem: String,
    },
    ComandoDesconhecido {
        tem: String,
    },
    NomeInvalido(crate::nome::Recusa),
    DiscoInvalido {
        tem: String,
    },

    /// O comando e o disco nao combinam: uma `verificacao` com disco nomeado,
    /// ou um `backup`/`restauracao` sem. Ver o campo
    /// [`Estado::disco`].
    DiscoIncoerente {
        comando: &'static str,
        tem: String,
        nomeia_disco: bool,
    },

    /// O comando e o nome da imagem nao combinam. A gemea de
    /// [`RecusaDoEstado::DiscoIncoerente`], no outro eixo (E12).
    NomeIncoerente {
        comando: &'static str,
        tem: String,
        nomeia_imagem: bool,
    },

    MomentoInvalido {
        tem: String,
    },
    SituacaoInvalida {
        tem: String,
    },

    /// Um valor precisaria de escape para caber no JSON. Nao deveria acontecer
    /// — os seis campos passam por validadores que o impedem —, e por isso
    /// mesmo e erro alto em vez de escape silencioso.
    ValorPrecisaEscape {
        chave: String,
        caractere: char,
    },
}

impl fmt::Display for RecusaDoEstado {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecusaDoEstado::Truncado => write!(
                f,
                "o arquivo termina no meio: e o rastro de um desligamento durante a gravacao. Nao da para saber o que estava armado, e o ARCA nao lê estado pela metade"
            ),
            RecusaDoEstado::CaractereInesperado {
                posicao,
                tem,
                esperado,
            } => write!(
                f,
                "caractere `{tem}` na posicao {posicao}, onde se esperava `{esperado}`"
            ),
            RecusaDoEstado::EscapeNoTexto => write!(
                f,
                "ha uma barra invertida num valor, e o ARCA nunca escreve uma: os cinco campos deste arquivo nao alcancam caractere que precise de escape. Este arquivo nao foi escrito pelo ARCA"
            ),
            RecusaDoEstado::CaractereDeControle { codigo } => write!(
                f,
                "ha um caractere de controle (U+{codigo:04X}) dentro de um valor"
            ),
            RecusaDoEstado::SobrouTextoDepoisDoFim { resto } => write!(
                f,
                "sobrou texto depois do fim do objeto: `{}`",
                resto.chars().take(40).collect::<String>()
            ),
            RecusaDoEstado::ChaveDesconhecida { chave } => write!(
                f,
                "o arquivo traz a chave `{chave}`, que esta versao do ARCA nao conhece. Agir sobre metade de um estado seria pior do que recusar"
            ),
            RecusaDoEstado::ChaveRepetida { chave } => {
                write!(f, "a chave `{chave}` aparece mais de uma vez")
            }
            RecusaDoEstado::ChaveFaltando { chave } => {
                write!(f, "falta a chave `{chave}`")
            }
            RecusaDoEstado::SeloInvalido { tem } => write!(
                f,
                "`{tem}` nao e um selo: sao 16 digitos hexadecimais minusculos"
            ),
            RecusaDoEstado::ComandoDesconhecido { tem } => write!(
                f,
                "`{tem}` nao e um comando do ARCA: valem `backup`, `restauracao`, `verificacao` e `sondagem`"
            ),
            RecusaDoEstado::DiscoIncoerente {
                comando,
                nomeia_disco: true,
                ..
            } => write!(
                f,
                "o estado diz `{comando}`, que nomeia um disco na receita, e o campo `disco` esta vazio. Nao se arma uma operacao sobre um disco que o arquivo nao diz qual e"
            ),
            RecusaDoEstado::DiscoIncoerente { comando, tem, .. } => write!(
                f,
                "o estado diz `{comando}`, que nao nomeia disco nenhum na receita, e o campo `disco` tras `{tem}`. Um disco guardado por uma operacao que nao o usa e um valor que ninguem confere"
            ),
            RecusaDoEstado::NomeIncoerente {
                comando,
                nomeia_imagem: true,
                ..
            } => write!(
                f,
                "o estado diz `{comando}`, que opera sobre uma imagem, e o campo `nome` esta vazio. E do nome que sai a pasta do desfecho, e sem ele o desfecho iria para o lugar de outra operacao"
            ),
            RecusaDoEstado::NomeIncoerente { comando, tem, .. } => write!(
                f,
                "o estado diz `{comando}`, que nao opera sobre imagem nenhuma — ela lê os discos da maquina —, e o campo `nome` tras `{tem}`. Um nome guardado por uma operacao que nao o usa e um valor que ninguem confere"
            ),
            RecusaDoEstado::NomeInvalido(recusa) => {
                write!(f, "o nome gravado nao passa por B-2: {recusa}")
            }
            RecusaDoEstado::DiscoInvalido { tem } => {
                write!(f, "`{tem}` nao e nome de disco do Linux")
            }
            RecusaDoEstado::MomentoInvalido { tem } => write!(
                f,
                "`{tem}` nao tem a forma de um momento (`2026-08-22T18:14:03-03:00`)"
            ),
            RecusaDoEstado::SituacaoInvalida { tem } => write!(
                f,
                "`{tem}` nao e situacao de job: so `{}` e `{}`",
                Situacao::Armado.nome(),
                Situacao::Colhido.nome()
            ),
            RecusaDoEstado::ValorPrecisaEscape { chave, caractere } => write!(
                f,
                "o valor de `{chave}` tem `{caractere}`, que precisaria de escape no JSON. Nenhum dos seis campos deveria alcancar esse caractere, e escrever o arquivo assim produziria um estado que nem o proprio ARCA leria"
            ),
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::duplos::{ArquivosEmMemoria, EntropiaDeMentira, RelogioParado};

    const CAMINHO: &str = r"R:\arca\estado.json";

    fn estado() -> Estado {
        Estado {
            selo: Selo::novo("a3f1c9e07b2d4856").unwrap(),
            comando: Operacao::Backup,
            nome: Some(Nome::novo("2026-08-22_Apps").unwrap()),
            disco: Some(Disco::novo("nvme0n1").unwrap()),
            armado_em: MomentoDoArmar::agora(&RelogioParado::em("2026-08-22T18:14:03")),
            situacao: Situacao::Armado,
        }
    }

    /// O estado de uma verificacao armada (E11): sem disco, porque o
    /// `ocs-chkimg` opera sobre a imagem.
    fn estado_de_verificacao() -> Estado {
        Estado {
            comando: Operacao::Verificacao,
            disco: None,
            ..estado()
        }
    }

    // ────────── a terceira operacao, e a coerencia com o disco ──────────

    #[test]
    fn a_verificacao_da_a_volta_com_o_disco_vazio() {
        let original = estado_de_verificacao();
        let volta = Estado::de_json(&original.como_json().unwrap()).unwrap();

        assert_eq!(volta, original);
        assert_eq!(volta.comando, Operacao::Verificacao);
        assert_eq!(volta.disco, None);
    }

    #[test]
    fn o_disco_vazio_e_a_string_vazia_no_arquivo() {
        // A escolha nao e arbitraria: `Disco::novo("")` ja recusava desde a
        // E3, entao o vazio nunca foi um nome de disco possivel e nao pode
        // colidir com nenhum. Um sentinela como `nenhum` colidiria —
        // `[a-z][a-z0-9]*` o aceitaria.
        let json = estado_de_verificacao().como_json().unwrap();

        assert!(json.contains("\"comando\": \"verificacao\""), "{json}");
        assert!(json.contains("\"disco\": \"\""), "{json}");
        assert!(
            Disco::novo("").is_err(),
            "se `Disco::novo` passar a aceitar vazio, o sentinela colide"
        );
    }

    #[test]
    fn um_estado_de_verificacao_com_disco_e_recusado() {
        // **A mutacao que a falsificacao pegou faltando.** Um `estado.json`
        // que dissesse `verificacao` com disco nomeado carregaria um valor que
        // nenhuma receita usa, e que ninguem conferiria.
        let json = estado_de_verificacao()
            .como_json()
            .unwrap()
            .replace("\"disco\": \"\"", "\"disco\": \"nvme0n1\"");

        match Estado::de_json(&json).unwrap_err() {
            RecusaDoEstado::DiscoIncoerente {
                comando,
                tem,
                nomeia_disco,
            } => {
                assert_eq!(comando, "verificacao");
                assert_eq!(tem, "nvme0n1");
                assert!(!nomeia_disco);
            }
            outro => panic!("esperava disco incoerente, veio {outro}"),
        }
    }

    #[test]
    fn um_backup_ou_restauracao_sem_disco_e_recusado() {
        // O outro sentido, e o que mais dói: um `estado.json` que dissesse
        // `restauracao` com disco vazio armaria uma operacao destrutiva sem
        // dizer sobre que disco. "Nao ha disco" nunca pode virar "tanto faz".
        for operacao in [Operacao::Backup, Operacao::Restauracao] {
            let json = Estado {
                comando: operacao,
                ..estado()
            }
            .como_json()
            .unwrap()
            .replace("\"disco\": \"nvme0n1\"", "\"disco\": \"\"");

            match Estado::de_json(&json).unwrap_err() {
                RecusaDoEstado::DiscoIncoerente {
                    comando,
                    nomeia_disco,
                    ..
                } => {
                    assert_eq!(comando, operacao.nome());
                    assert!(nomeia_disco, "{} nomeia disco", operacao.nome());
                }
                outro => panic!("{}: esperava incoerencia, veio {outro}", operacao.nome()),
            }
        }
    }

    #[test]
    fn as_quatro_operacoes_dao_a_volta_pelo_nome() {
        // O leitor conhece as quatro, e nao duas. Um `estado.json` escrito por
        // esta versao e lido por ela tem de voltar igual — inclusive o
        // `verificacao`, que a E11 acrescentou, e o `sondagem`, da E12.
        for operacao in [
            Operacao::Backup,
            Operacao::Restauracao,
            Operacao::Verificacao,
            Operacao::Sondagem,
        ] {
            let original = Estado {
                comando: operacao,
                nome: operacao
                    .nomeia_imagem()
                    .then(|| Nome::novo("2026-08-22_Apps").unwrap()),
                disco: operacao
                    .nomeia_disco()
                    .then(|| Disco::novo("nvme0n1").unwrap()),
                ..estado()
            };
            let volta = Estado::de_json(&original.como_json().unwrap())
                .unwrap_or_else(|erro| panic!("{}: {erro}", operacao.nome()));

            assert_eq!(volta, original, "{}", operacao.nome());
        }
    }

    #[test]
    fn a_recusa_de_comando_desconhecido_lista_as_quatro() {
        // A mensagem tem de acompanhar o enum: um usuario que abra o arquivo e
        // veja `valem backup e restauracao` concluiria que `verificacao` e
        // corrupcao.
        let recusa = RecusaDoEstado::ComandoDesconhecido {
            tem: "arrumacao".to_string(),
        }
        .to_string();

        for nome in ["backup", "restauracao", "verificacao", "sondagem"] {
            assert!(
                recusa.contains(nome),
                "a mensagem nao cita `{nome}`: {recusa}"
            );
        }
    }

    // ────────── a quarta operacao, e a coerencia com o nome ──────────

    fn estado_de_sondagem() -> Estado {
        Estado {
            comando: Operacao::Sondagem,
            nome: None,
            disco: None,
            ..estado()
        }
    }

    #[test]
    fn o_nome_vazio_e_a_string_vazia_no_arquivo() {
        // O mesmo argumento do `disco` da E11, conferido antes de ser reusado:
        // `Nome::novo("")` recusa desde a E1, entao o vazio nunca foi um nome
        // de imagem possivel e nao pode colidir com nenhum. Um sentinela como
        // `sondagem` colidiria — B-2 o aceita como nome de imagem.
        let json = estado_de_sondagem().como_json().unwrap();

        assert!(json.contains("\"comando\": \"sondagem\""), "{json}");
        assert!(json.contains("\"nome\": \"\""), "{json}");
        assert!(
            Nome::novo("").is_err(),
            "se `Nome::novo` passar a aceitar vazio, o sentinela colide"
        );
        assert!(
            Nome::novo("sondagem").is_ok(),
            "e por isso `sondagem` nao serviria de sentinela: B-2 o aceita"
        );
    }

    #[test]
    fn uma_sondagem_com_nome_de_imagem_e_recusada() {
        // Um nome carregado ate uma receita que nao o usa e um valor que
        // ninguem confere — o mesmo defeito que o `disco` da E11 fecha, no
        // outro eixo.
        let json = estado_de_sondagem()
            .como_json()
            .unwrap()
            .replace("\"nome\": \"\"", "\"nome\": \"2026-08-22_Apps\"");

        match Estado::de_json(&json).unwrap_err() {
            RecusaDoEstado::NomeIncoerente {
                comando,
                tem,
                nomeia_imagem,
            } => {
                assert_eq!(comando, "sondagem");
                assert_eq!(tem, "2026-08-22_Apps");
                assert!(!nomeia_imagem);
            }
            outro => panic!("esperava nome incoerente, veio {outro}"),
        }
    }

    #[test]
    fn as_tres_que_operam_sobre_imagem_sem_nome_sao_recusadas() {
        // **O sentido que dói**, e ele e pior do que o do disco: a pasta do
        // desfecho sai do nome (`pasta_do_log`), entao um `backup` com nome
        // vazio procuraria o desfecho na pasta `backup-`, que nao e a de
        // ninguem — e um `sondagem` colheria o desfecho errado.
        for operacao in [
            Operacao::Backup,
            Operacao::Restauracao,
            Operacao::Verificacao,
        ] {
            let json = Estado {
                comando: operacao,
                disco: operacao
                    .nomeia_disco()
                    .then(|| Disco::novo("nvme0n1").unwrap()),
                ..estado()
            }
            .como_json()
            .unwrap()
            .replace("\"nome\": \"2026-08-22_Apps\"", "\"nome\": \"\"");

            match Estado::de_json(&json).unwrap_err() {
                RecusaDoEstado::NomeIncoerente {
                    comando,
                    nomeia_imagem,
                    ..
                } => {
                    assert_eq!(comando, operacao.nome());
                    assert!(nomeia_imagem, "{} opera sobre imagem", operacao.nome());
                }
                outro => panic!("{}: esperava incoerencia, veio {outro}", operacao.nome()),
            }
        }
    }

    #[test]
    fn a_sondagem_procura_o_desfecho_na_pasta_fixa() {
        // Os dois lados do reinicio pelo mesmo caminho: a receita escreve em
        // `/home/partimag/ARCA-LOGS/sondagem/arca-fim.txt`, e a colheita
        // procura em `E:\ARCA-LOGS\sondagem\arca-fim.txt`. Sao a mesma funcao
        // — [`crate::receita::pasta_do_log`] — vista dos dois lados.
        assert_eq!(
            caminho_do_desfecho(Path::new(r"E:\"), Operacao::Sondagem, None),
            PathBuf::from(r"E:\ARCA-LOGS\sondagem\arca-fim.txt")
        );
    }

    #[test]
    fn a_descricao_de_um_job_sem_imagem_nao_tem_crase_vazia() {
        // Cinco telas imprimem o job, e o `nome` opcional as obrigou a decidir
        // o que dizer no lugar dele. A decisao e uma so, e mora aqui: sem
        // imagem, sobra a operacao — e nunca ``sondagem ` ` ``.
        assert_eq!(estado_de_sondagem().descricao(), "sondagem");
        assert_eq!(estado().descricao(), "backup `2026-08-22_Apps`");
    }

    // ───────────────────────────── o selo ─────────────────────────────

    #[test]
    fn o_selo_gerado_passa_pelo_proprio_validador() {
        // Nao ha caminho por onde um selo recem-gerado saia recusado por
        // `Selo::novo`. Se houvesse, ele seria embutido na receita e nunca
        // casaria com nada — e a falha apareceria depois do reinicio.
        for byte in 0u8..=255 {
            let bytes = [byte; BYTES_DO_SELO];
            let selo = Selo::de_bytes(&bytes);

            assert_eq!(
                Selo::novo(selo.como_texto()),
                Ok(selo.clone()),
                "byte {byte:#04x} produziu um selo que o validador recusa: {selo}"
            );
        }
    }

    #[test]
    fn o_selo_tem_dezesseis_digitos_mesmo_com_bytes_pequenos() {
        // Sem o `02` no formato, `0x0a` sairia como `a` e o selo teria quinze
        // digitos. O validador pegaria, mas depois de a receita ja existir.
        let selo = Selo::de_bytes(&[0x00, 0x01, 0x0a, 0x0f, 0x10, 0xff, 0x7f, 0x80]);
        assert_eq!(selo.como_texto(), "00010a0f10ff7f80");
    }

    #[test]
    fn o_selo_vem_dos_bytes_que_a_porta_entregou() {
        let entropia = EntropiaDeMentira::com(&[0xa3, 0xf1, 0xc9, 0xe0, 0x7b, 0x2d, 0x48, 0x56]);
        let selo = gerar_selo(&entropia).expect("a porta responde");

        assert_eq!(selo.como_texto(), "a3f1c9e07b2d4856");
    }

    #[test]
    fn a_entropia_que_falha_nao_vira_selo_de_zeros() {
        // O modo de falha que importa: um gerador que recusasse e um selo de
        // zeros passariam por `Selo::novo`, e zeros sao o selo de ensaio.
        assert!(gerar_selo(&EntropiaDeMentira::recusando()).is_err());
    }

    // ────────────────────────── o momento ──────────────────────────

    #[test]
    fn o_momento_leva_o_deslocamento_e_volta_igual() {
        let momento = MomentoDoArmar::agora(&RelogioParado::em("2026-08-22T18:14:03"));
        let texto = momento.como_texto().to_string();

        assert_eq!(texto.chars().count(), LARGURA_DO_MOMENTO, "veio {texto}");
        assert!(texto.starts_with("2026-08-22T18:14:03"), "veio {texto}");
        assert_eq!(MomentoDoArmar::de_texto(&texto), Ok(momento));
    }

    #[test]
    fn momento_com_forma_errada_e_recusado() {
        for bruto in [
            "",
            "2026-08-22",
            "2026-08-22T18:14:03",
            "2026-08-22 18:14:03-03:00",
            "2026-08-22T18:14:03-0300",
            "20x6-08-22T18:14:03-03:00",
            "2026-08-22T18:14:03Z0:00",
        ] {
            assert!(
                MomentoDoArmar::de_texto(bruto).is_err(),
                "`{bruto}` passou por momento"
            );
        }
    }

    // ────────────────────────── ida e volta ──────────────────────────

    #[test]
    fn os_seis_campos_voltam_como_foram() {
        // O teste que importa, e o unico que prova que escrever a mao nao
        // perdeu nada pelo caminho.
        let original = estado();
        let json = original.como_json().expect("os seis campos cabem no JSON");
        let volta = Estado::de_json(&json).expect("o proprio arquivo se lê");

        assert_eq!(volta, original);
        assert_eq!(volta.selo.como_texto(), "a3f1c9e07b2d4856");
        assert_eq!(volta.comando, Operacao::Backup);
        assert_eq!(
            volta.nome.as_ref().map(Nome::como_texto),
            Some("2026-08-22_Apps")
        );
        assert_eq!(volta.disco.as_ref().map(Disco::como_texto), Some("nvme0n1"));
    }

    #[test]
    fn a_restauracao_tambem_da_a_volta() {
        let original = Estado {
            comando: Operacao::Restauracao,
            ..estado()
        };
        let volta = Estado::de_json(&original.como_json().unwrap()).unwrap();
        assert_eq!(volta, original);
    }

    #[test]
    fn o_json_tem_as_seis_chaves_e_nada_mais() {
        let json = estado().como_json().unwrap();
        for chave in CHAVES {
            assert!(
                json.contains(&format!("\"{chave}\"")),
                "faltou {chave}:\n{json}"
            );
        }

        // Contado pelos pares que o leitor devolve, e nao pelos dois-pontos do
        // texto: o momento tem tres deles dentro do proprio valor.
        assert_eq!(ler_objeto(&json).unwrap().len(), CHAVES.len());
    }

    // ───────────────────── o arquivo truncado ─────────────────────

    #[test]
    fn nenhum_corte_do_arquivo_e_lido_pela_metade() {
        // O caso construido tem de ser o caso dificil, e nao o que era facil
        // de montar: em vez de escolher um ponto de corte, corta-se em **todos**
        // os comprimentos possiveis. Um so que passasse seria um `estado.json`
        // meio gravado sendo tomado por bom, e ele decide o que fazer na volta.
        let json = estado().como_json().unwrap();
        let caracteres: Vec<char> = json.chars().collect();

        // O corte tem de tirar **conteudo**, e nao so o branco do fim. Um
        // arquivo sem a quebra de linha final e um objeto completo, e o leitor
        // o aceita de proposito: nada garante que quem gravou terminou com
        // `\n`. Este limite saiu do proprio teste, que reprovou no corte 150
        // de 151 — o que ele tirava era exatamente essa quebra.
        let ate_o_conteudo = json.trim_end().chars().count();

        for corte in 0..ate_o_conteudo {
            let pedaco: String = caracteres[..corte].iter().collect();
            assert!(
                Estado::de_json(&pedaco).is_err(),
                "o arquivo cortado em {corte} de {ate_o_conteudo} passou por estado bom:\n{pedaco}"
            );
        }

        // E o arquivo inteiro continua sendo lido, senao o teste acima estaria
        // provando so que o leitor recusa tudo.
        assert!(Estado::de_json(&json).is_ok());
        assert!(Estado::de_json(json.trim_end()).is_ok());
    }

    #[test]
    fn o_arquivo_vazio_e_recusado_como_truncado() {
        assert_eq!(Estado::de_json(""), Err(RecusaDoEstado::Truncado));
    }

    // ───────────────────── o que mais nao passa ─────────────────────

    #[test]
    fn chave_desconhecida_e_recusa_e_nao_algo_a_ignorar() {
        let json = estado()
            .como_json()
            .unwrap()
            .replace("\"disco\"", "\"disco_alvo\"");

        match Estado::de_json(&json).unwrap_err() {
            RecusaDoEstado::ChaveDesconhecida { chave } => assert_eq!(chave, "disco_alvo"),
            outro => panic!("esperava chave desconhecida, veio {outro}"),
        }
    }

    #[test]
    fn chave_faltando_e_recusa() {
        let json = "{\n  \"selo\": \"a3f1c9e07b2d4856\",\n  \"comando\": \"backup\"\n}\n";
        match Estado::de_json(json).unwrap_err() {
            RecusaDoEstado::ChaveFaltando { chave } => assert_eq!(chave, "nome"),
            outro => panic!("esperava chave faltando, veio {outro}"),
        }
    }

    #[test]
    fn chave_repetida_e_recusa() {
        let json = estado().como_json().unwrap();
        let repetida = json.replace(
            "\"comando\": \"backup\"",
            "\"comando\": \"backup\",\n  \"comando\": \"restauracao\"",
        );

        assert!(matches!(
            Estado::de_json(&repetida).unwrap_err(),
            RecusaDoEstado::ChaveRepetida { .. }
        ));
    }

    #[test]
    fn dois_objetos_colados_nao_passam_pelo_primeiro() {
        let json = estado().como_json().unwrap();
        let dobrado = format!("{json}{json}");

        assert!(matches!(
            Estado::de_json(&dobrado).unwrap_err(),
            RecusaDoEstado::SobrouTextoDepoisDoFim { .. }
        ));
    }

    #[test]
    fn escape_no_valor_e_recusa_porque_o_arca_nunca_escreve_um() {
        let json = "{\"selo\": \"a3f1c9e0\\u0037b2d4856\", \"comando\": \"backup\", \"nome\": \"x\", \"disco\": \"sda\", \"armado_em\": \"2026-08-22T18:14:03-03:00\"}";
        assert_eq!(Estado::de_json(json), Err(RecusaDoEstado::EscapeNoTexto));
    }

    #[test]
    fn cada_campo_volta_pelo_validador_que_o_julgou_na_ida() {
        // Um `estado.json` mexido a mao nao pode produzir um `Selo` que
        // `Selo::novo` recusaria: o resto do sistema confia em ter um em maos.
        let base = estado().como_json().unwrap();

        let casos: [(&str, &str); 4] = [
            ("\"a3f1c9e07b2d4856\"", "\"A3F1C9E07B2D4856\""),
            ("\"backup\"", "\"formatar\""),
            ("\"2026-08-22_Apps\"", "\"ARCA-LOGS\""),
            ("\"nvme0n1\"", "\"NVME0N1\""),
        ];

        for (de, para) in casos {
            let json = base.replace(de, para);
            assert!(
                Estado::de_json(&json).is_err(),
                "`{para}` passou onde o validador da ida teria recusado"
            );
        }
    }

    #[test]
    fn valor_que_precisaria_de_escape_nao_e_escrito() {
        // Nenhum dos cinco campos alcanca isto, e e por isso que a conferencia
        // existe: "ja foi validado antes" e a frase que este projeto ja viu
        // ser falsa duas vezes.
        assert!(matches!(
            campo("nome", "com \"aspa\""),
            Err(RecusaDoEstado::ValorPrecisaEscape { .. })
        ));
        assert!(matches!(
            campo("nome", "com\\barra"),
            Err(RecusaDoEstado::ValorPrecisaEscape { .. })
        ));
        assert!(matches!(
            campo("nome", "com\nquebra"),
            Err(RecusaDoEstado::ValorPrecisaEscape { .. })
        ));
        assert!(matches!(
            campo("nome", "acentuação"),
            Err(RecusaDoEstado::ValorPrecisaEscape { .. })
        ));
    }

    #[test]
    fn os_seis_campos_de_verdade_atravessam_sem_escape() {
        // O outro lado: os alfabetos que B-2, `Selo` e `Disco` permitem nao
        // alcancam nada que o JSON precise escapar. E o que torna escrever a
        // mao defensavel, e nao so curto.
        for valor in [
            "a3f1c9e07b2d4856",
            "backup",
            "restauracao",
            "2026-08-21_WindowsCompleto",
            "ARCA-TESTE-03",
            "nome.com.ponto",
            "nvme0n1",
            "sda",
            "2026-08-22T18:14:03-03:00",
            "2026-01-01T00:00:00+00:00",
        ] {
            assert!(campo("x", valor).is_ok(), "`{valor}` precisou de escape");
        }
    }

    // ─────────────────────── a situacao (E8) ───────────────────────

    #[test]
    fn as_duas_situacoes_dao_a_volta() {
        for situacao in [Situacao::Armado, Situacao::Colhido] {
            let original = Estado {
                situacao,
                ..estado()
            };
            let volta = Estado::de_json(&original.como_json().unwrap()).unwrap();
            assert_eq!(volta.situacao, situacao);
            assert_eq!(volta, original);
        }
    }

    #[test]
    fn situacao_que_nao_existe_e_recusa_e_nao_um_padrao() {
        // Um `estado.json` com situacao que este binario nao conhece veio de
        // uma versao que sabe alguma coisa que esta nao sabe. Cair num padrao
        // — "nao entendi, entao esta armado" — faria o ARCA agir sobre metade
        // de um estado, que e o que o leitor inteiro existe para nao fazer.
        let json = estado().como_json().unwrap().replace(
            "\"situacao\": \"armado\"",
            "\"situacao\": \"colhido-pela-metade\"",
        );

        match Estado::de_json(&json).unwrap_err() {
            RecusaDoEstado::SituacaoInvalida { tem } => assert_eq!(tem, "colhido-pela-metade"),
            outro => panic!("esperava a situacao invalida, veio {outro}"),
        }
    }

    #[test]
    fn um_estado_json_de_cinco_campos_e_recusado_por_chave_faltando() {
        // O formato mudou na E8, e a mudanca e visivel em vez de silenciosa.
        // Um arquivo da versao anterior nao vira "job armado por suposicao":
        // ele e recusado nomeando a chave que falta, e quem lê decide.
        let de_antes = concat!(
            "{\n",
            "  \"selo\": \"a3f1c9e07b2d4856\",\n",
            "  \"comando\": \"backup\",\n",
            "  \"nome\": \"2026-08-22_Apps\",\n",
            "  \"disco\": \"nvme0n1\",\n",
            "  \"armado_em\": \"2026-08-22T18:14:03-03:00\"\n",
            "}\n"
        );

        assert_eq!(
            Estado::de_json(de_antes),
            Err(RecusaDoEstado::ChaveFaltando { chave: "situacao" })
        );
    }

    #[test]
    fn brancos_a_mais_nao_atrapalham_a_leitura() {
        let frouxo = "  {  \"selo\" : \"a3f1c9e07b2d4856\" ,\n\t\"comando\":\"backup\", \"nome\":\"x\", \"disco\":\"sda\", \"armado_em\":\"2026-08-22T18:14:03-03:00\", \"situacao\" : \"colhido\" }  \n";
        assert!(Estado::de_json(frouxo).is_ok());
    }

    // ──────────────────────── gravar e lê ────────────────────────

    #[test]
    fn gravar_cria_a_pasta_e_o_arquivo_da_a_volta() {
        // `R:\arca\` nao existe num dispositivo preparado a mao — o desta mesa
        // nao o tinha. Supor a pasta pronta faria a primeira gravacao real
        // falhar, que e justamente a que arma.
        let arquivos = ArquivosEmMemoria::novo();
        let original = estado();

        gravar(&arquivos, Path::new(CAMINHO), &original).expect("grava");

        assert!(arquivos.conteudo_de(CAMINHO).is_some());
        assert_eq!(ler(&arquivos, Path::new(CAMINHO)).unwrap(), original);
    }

    #[test]
    fn gravar_por_cima_substitui_em_vez_de_concatenar() {
        let arquivos = ArquivosEmMemoria::novo();
        let primeiro = estado();
        let segundo = Estado {
            selo: Selo::novo("ffffffffffffffff").unwrap(),
            comando: Operacao::Restauracao,
            ..estado()
        };

        gravar(&arquivos, Path::new(CAMINHO), &primeiro).unwrap();
        gravar(&arquivos, Path::new(CAMINHO), &segundo).unwrap();

        assert_eq!(ler(&arquivos, Path::new(CAMINHO)).unwrap(), segundo);
    }

    #[test]
    fn lê_um_arquivo_que_nao_existe_e_erro_e_nao_estado_vazio() {
        let arquivos = ArquivosEmMemoria::novo();
        assert!(ler(&arquivos, Path::new(CAMINHO)).is_err());
    }

    #[test]
    fn o_estado_ilegivel_chega_com_o_motivo_e_nao_como_ausencia() {
        // "Nao entendi o arquivo" nao pode virar "nao ha job": um dispositivo
        // com job armado e estado corrompido continua armado.
        let arquivos = ArquivosEmMemoria::novo().com(CAMINHO, "{\"selo\": \"a3f1");
        match ler(&arquivos, Path::new(CAMINHO)).unwrap_err() {
            Erro::EstadoRecusado(RecusaDoEstado::Truncado) => {}
            outro => panic!("esperava o truncado, veio {outro}"),
        }
    }

    // ───────────────── o caminho do desfecho ─────────────────

    #[test]
    fn o_caminho_do_desfecho_leva_a_operacao_no_nome_da_pasta() {
        // A mesma pasta que a receita monta do lado Linux. Se os dois lados
        // divergissem, o ARCA procuraria o desfecho onde ele nao esta — e o
        // §5.5 leria um backup bem-sucedido como desfecho ausente.
        let nome = Nome::novo("2026-08-22_Apps").unwrap();

        assert_eq!(
            caminho_do_desfecho(Path::new(r"E:\"), Operacao::Backup, Some(&nome)),
            PathBuf::from(r"E:\ARCA-LOGS\backup-2026-08-22_Apps\arca-fim.txt")
        );
        assert_eq!(
            caminho_do_desfecho(Path::new(r"E:\"), Operacao::Restauracao, Some(&nome)),
            PathBuf::from(r"E:\ARCA-LOGS\restauracao-2026-08-22_Apps\arca-fim.txt")
        );
    }

    #[test]
    fn o_caminho_do_desfecho_bate_com_o_que_a_receita_escreve() {
        // O oraculo e a propria receita: o caminho Linux que ela grava tem de
        // terminar com a mesma pasta e o mesmo arquivo que o ARCA procura do
        // lado Windows.
        use crate::receita::{Pedido, Receita};

        let nome = Nome::novo("2026-08-22_Apps").unwrap();
        let receita = Receita::montar(&Pedido {
            operacao: Operacao::Backup,
            nome: Some(nome.clone()),
            disco: Some(Disco::novo("nvme0n1").unwrap()),
            selo: Selo::de_ensaio(),
        })
        .unwrap();

        let windows = caminho_do_desfecho(Path::new(r"E:\"), Operacao::Backup, Some(&nome));
        let cauda = windows
            .to_string_lossy()
            .trim_start_matches(r"E:\")
            .replace('\\', "/");

        assert!(
            receita.comando().contains(&cauda),
            "a receita nao escreve em `{cauda}`:\n{}",
            receita.comando()
        );
    }
}
