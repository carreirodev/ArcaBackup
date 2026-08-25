//! `arca prepare` — a §7.1 do PRD (PR-1 a PR-5, S-2).
//!
//! **É o comando que transforma um disco qualquer num dispositivo ARCA**, e o
//! único que roda num mundo onde as defesas dos outros não se aplicam: B-1 acha
//! o dispositivo pelo `ARCAVAULT`, S-3 endereça por LABEL e C-10 recusa rótulo
//! repetido — e no disco que este comando vai preparar nenhum deles existe.
//!
//! # O que é transcrição e o que é código novo
//!
//! | Parte | Origem |
//! |---|---|
//! | A estrutura de partições — GPT, dados básicos nas duas, nenhuma ativa, unidade 4096 | Transcrito de `medicao-gpt-2026-08-25.txt` (ADR-0025) |
//! | A sequência de cmdlets que a produz, e a remoção da MSR | Medido à mão em 25/08/2026 em **dois** dispositivos, antes de virar código |
//! | Que um dispositivo assim **boota** | Medido em hardware em 25/08/2026, com o device path lido de dentro do live |
//! | O pacote, a versão e o SHA256 | Medido — duas fontes independentes (ver [`crate::pacote`]) |
//! | **A criação da entrada de firmware** | **Era código sem original**, e ganhou um em 23/08/2026 |
//! | O `--dry-run`, a confirmação e a releitura | Reúso — [`crate::confirmacao`], C-3 |
//!
//! ## A entrada de firmware, e o que a medição de 23/08 mudou
//!
//! C-4 diz que **armar não cria entrada**: a E7 recusou escrever isso de
//! propósito, dizendo que o lugar era aqui, porque nenhuma captura mostrava a
//! criação — só a migração.
//!
//! Medido à mão em 23/08/2026, e **há original**: a entrada `ARCA` desta
//! máquina é, campo a campo, uma cópia do `{bootmgr}` com `device`, `path` e
//! `description` trocados. Criá-la é `bcdedit /copy {bootmgr} /d ARCA` seguido
//! de dois `/set`, e o resultado sai **idêntico** à que já existia — conferido
//! lado a lado, com a entrada de medição apagada no fim e o firmware voltando
//! byte a byte ao que era.
//!
//! ## E a medição trouxe uma coisa que ninguém tinha previsto
//!
//! **`bcdedit /copy` põe a entrada nova no `displayorder` sozinho.** Antes:
//! `displayorder {bootmgr}`. Depois do `/copy`: `{bootmgr}` e a nova. Ninguém
//! pediu — o `/copy` faz.
//!
//! Isso é exatamente o perigo que C-5 nomeia: *o ARCA acrescentar um caminho
//! permanente para bootar no dispositivo*. Então `arca prepare` **tira a
//! entrada da ordem** logo depois de criá-la, com `/set {fwbootmgr}
//! displayorder {novo} /remove` — medido, `exit 0`, a entrada sobrevive fora da
//! ordem, e a segunda passada não muda nada.
//!
//! Tirar não quebra nada, e isso está medido desde a E7: o `bootsequence`
//! funciona sobre uma entrada que **não** está no `displayorder` (ADR-0007), e
//! foi assim que o marco de 22/08 rodou.
//!
//! **E não é o `/remove` que o ADR-0013 descartou.** Lá o problema era
//! *"acertar quais entradas tirar"* — uma pergunta que esta máquina já
//! respondeu errado uma vez. Aqui o alvo é a entrada que o próprio comando
//! acabou de criar, com o GUID em mãos, na mesma execução. Não há dedução
//! nenhuma.
//!
//! # A ordem dos passos, e o que cada um deixa se o seguinte não acontecer
//!
//! O ponto sem volta é o **passo 5**. Tudo antes dele é leitura e conversa; o
//! `Clear-Disk` é irreversível, e tudo depois dele é construção sobre um disco
//! que já foi apagado.
//!
//! | # | Passo | Parando aqui, o que fica |
//! |---|---|---|
//! | 0 | Listar os discos e perguntar o número — só sem `--dispositivo` (ADR-0024) | nada tocado |
//! | 1 | Descrever o disco e julgar (PR-5) | nada tocado |
//! | 2 | Imprimir o plano (PR-4) | nada tocado |
//! | 3 | Perguntar, e **reler o disco** (PR-4, 3º tempo) | nada tocado |
//! | 4 | Confirmação digitada (S-2) | nada tocado |
//! | 5 | **Particionar e formatar** | disco apagado, duas partições vazias |
//! | 6 | Baixar o pacote (ou `--iso`) | dispositivo vazio, sem Clonezilla |
//! | 7 | Conferir o SHA256 (PR-1) | idem — e **nada foi extraído** |
//! | 8 | Extrair | `ARCABOOT` com o Clonezilla e `set default="0"` |
//! | 9 | Devolver o `grub.cfg` ao estado inerte | dispositivo bootável e inerte |
//! | 10 | Instalar o ARCA e a cópia do pacote (PR-3) | dispositivo completo, sem entrada de firmware |
//! | 11 | Criar a entrada, apontá-la e tirá-la da ordem | pronto |
//!
//! **Nenhum desses estados é pior do que o anterior, e todos são reversíveis
//! rodando o comando de novo** — ele começa apagando. O que não volta é o que
//! estava no disco antes do passo 5, e é para isso que existem os passos 1 a 4.
//!
//! Do passo 8 em diante o dispositivo **já boota**: um `prepare` interrompido
//! ali deixa um Clonezilla utilizável pelo menu (§6.4), sem o ARCA dentro.

use crate::app::Contexto;
use crate::confirmacao;
use crate::dispositivo::{ARCABOOT, ARCAVAULT};
use crate::erro::{Erro, Resultado};
use crate::firmware::{self, Alvo};
use crate::formato::{linha, tamanho};
use crate::grub;
use crate::pacote;
use crate::portas::particionador::{DiscoParaPreparar, ParticoesFeitas};
use crate::preparacao::{self, Preparacao};
use std::path::{Path, PathBuf};

/// O `.efi` para onde a entrada de firmware aponta, dentro do `ARCABOOT`.
///
/// Transcrito da entrada `ARCA` desta máquina, que o `bcdedit` mostra como
/// `path \EFI\boot\bootx64.efi` — e do pacote, onde o arquivo está em
/// `EFI/boot/bootx64.efi`. É o mesmo caminho nas duas pontas.
const CAMINHO_DO_EFI: &str = r"\EFI\boot\bootx64.efi";

/// O objeto do `bcdedit` que guarda a ordem de boot.
const FWBOOTMGR: &str = "{fwbootmgr}";

/// De onde a entrada nova é copiada.
const BOOTMGR: &str = "{bootmgr}";

/// O alvo do `/enum` que traz as entradas de firmware.
const FIRMWARE: &str = "firmware";

/// Onde o ARCA se instala dentro do `ARCABOOT` (§4.1).
const PASTA_DO_ARCA: &str = "arca";

pub fn executar(
    contexto: &Contexto,
    dispositivo: Option<u32>,
    iso: Option<&Path>,
) -> Resultado<()> {
    // ─────────── 0. de onde sai o índice, quando ele não veio na linha ───────────

    let discos = contexto.particionador.enumerar()?;

    // Sem `--dispositivo`, o número sai do menu — e ele **só resolve para um
    // índice**. Tudo abaixo desta linha é o mesmo nos dois caminhos: julgar,
    // imprimir o plano, perguntar, reler o disco e pedir o modelo digitado.
    let indice = match dispositivo {
        Some(indice) => indice,
        None => escolher_o_disco(contexto, &discos)?,
    };

    // ─────────── 1. descrever e julgar, antes de imprimir qualquer coisa ───────────

    let indices: Vec<u32> = discos.iter().map(|disco| disco.indice).collect();
    let alvo = discos.iter().find(|disco| disco.indice == indice);

    let preparacao = preparacao::julgar(indice, alvo, &indices, letra_do_sistema())
        .map_err(Erro::PreparacaoRecusada)?;

    // ─────────── 2. o plano inteiro na tela (PR-4, 1º tempo) ───────────

    print!("{}", montar_o_plano(&preparacao, iso));

    if contexto.dry_run {
        print!("{}", montar_o_ensaio(&preparacao));
        return Ok(());
    }

    // ─────────── 3. a pergunta, e a conferência do ARCA (PR-4, 2º e 3º tempos) ───────────

    // A resposta do usuário diz que ele **quer** prosseguir; ela não é
    // evidência sobre o disco. Por isso o ARCA relê antes de agir — ver
    // [`preparacao::e_o_mesmo_disco`] para a medição que motiva isto.
    if !perguntar_se_pode(contexto)? {
        println!("Nada foi tocado.\n");
        return Ok(());
    }

    let relido = contexto
        .particionador
        .descrever(indice)?
        .filter(|relido| preparacao::e_o_mesmo_disco(&preparacao.disco, relido));

    let Some(relido) = relido else {
        return Err(Erro::DiscoMudouEntreOPlanoEOSim {
            indice,
            modelo: preparacao.disco.modelo.clone(),
        });
    };

    print!(
        "{}",
        linha(
            "Conferido antes de escrever",
            &format!(
                "ok · o disco {indice} continua sendo `{}` de {}",
                relido.modelo,
                tamanho(relido.tamanho_bytes)
            ),
        )
    );

    // ─────────── 4. a confirmação digitada (S-2) ───────────

    confirmacao::pedir_texto(
        contexto,
        "Digite o modelo do disco para confirmar",
        &preparacao.disco.modelo,
    )?;

    contexto.registro.info(format!(
        "prepare do disco {indice} · `{}` · {} · apagando {} particao(oes)",
        preparacao.disco.modelo,
        tamanho(preparacao.disco.tamanho_bytes),
        preparacao.disco.particoes.len()
    ));

    // ─────────── 5. o ponto sem volta ───────────

    let feitas = contexto.particionador.particionar(&preparacao.plano)?;
    preparacao::conferir_o_que_saiu(&feitas).map_err(Erro::ParticionamentoDivergiu)?;

    let raiz_do_vault = PathBuf::from(format!("{}:\\", feitas.vault.letra));
    let raiz_do_boot = PathBuf::from(format!("{}:\\", feitas.boot.letra));

    print!("{}", montar_as_particoes(&feitas));

    // ─────────── 6 e 7. o pacote, e a conferência antes de extrair ───────────

    let pacote_local = obter_o_pacote(contexto, iso, &raiz_do_vault)?;

    // ─────────── 8. extrair ───────────

    let extracao = contexto.sistema.extrair(&pacote_local, &raiz_do_boot)?;
    if extracao.codigo != 0 {
        return Err(Erro::FerramentaRecusou {
            ferramenta: "bsdtar",
            codigo: extracao.codigo,
            saida: extracao.resumo(3),
        });
    }
    print!(
        "{}",
        linha("Extraindo", &format!("ok · {}", raiz_do_boot.display()))
    );

    // ─────────── 9. o `grub.cfg` do pacote NÃO está inerte ───────────

    let caminho_do_grub = raiz_do_boot.join(crate::dispositivo::RECEITA_NO_GRUB);
    let do_pacote = contexto.arquivos.ler_texto(&caminho_do_grub)?;
    let desarmado = grub::desarmar(&do_pacote).map_err(Erro::GrubRecusado)?;

    if desarmado.havia_receita() {
        contexto
            .arquivos
            .escrever_atomico(&caminho_do_grub, &desarmado.texto)?;
    }
    print!(
        "{}",
        linha(
            "Estado inerte",
            &if desarmado.default_devolvido {
                "ok · o `set default` do pacote era \"0\", e voltou para `live-default`".to_string()
            } else {
                "ok · o pacote ja veio inerte".to_string()
            },
        )
    );

    // ─────────── 10. o ARCA e a cópia do pacote (§4.1, PR-3) ───────────

    instalar_o_arca(contexto, &raiz_do_boot)?;

    // ─────────── 11. a entrada de firmware ───────────

    let entrada = criar_a_entrada(contexto, feitas.boot.letra)?;
    print!("{}", montar_a_entrada(&entrada));

    print!(
        "{}",
        montar_o_fim(&feitas, entrada.ordem_sem_alvo.as_deref())
    );
    Ok(())
}

/// O que o `--dispositivo` do disco de sistema desta máquina responde.
///
/// `None` quando o `%SystemDrive%` não diz nada de útil — e "não sei" não vira
/// "não é este disco": as outras seis defesas continuam valendo, e o `IsSystem`
/// do `MSFT_Disk` cobre o caso principal. Ver
/// [`preparacao::julgar`].
fn letra_do_sistema() -> Option<char> {
    std::env::var("SystemDrive")
        .ok()?
        .chars()
        .next()
        .filter(char::is_ascii_alphabetic)
        .map(|letra| letra.to_ascii_uppercase())
}

// ─────────────────── o menu do §6.1, quando não há `--dispositivo` ───────────────────

/// Lista os discos, pergunta o número e devolve o **índice do Windows**.
///
/// # Isto não afrouxa P1 revisado, e as três regras que o mantêm inteiro
///
/// O princípio é *"o ARCA destrói dados quando o usuário nomeou o alvo e
/// confirmou por escrito, e **nunca por dedução**"*. Um menu é o ARCA
/// **oferecendo**, e oferecer não é deduzir — desde que:
///
/// 1. **com um candidato só, ele não auto-seleciona.** Uma lista de um item
///    que se aceita com Enter é exatamente o ARCA escolhendo o que apagar, e o
///    §6.1 escreve o contrário como princípio: *"obrigatório, mesmo havendo um
///    candidato só"*. O `1` continua sendo digitado;
/// 2. **o Enter vazio não escolhe nada.** Não há padrão, porque um padrão é
///    uma dedução com outro nome;
/// 3. **o número não vira alvo direto.** Ele resolve para um índice e cai no
///    caminho que já existia — julgar, imprimir o plano, perguntar, **reler o
///    disco** e pedir o modelo digitado (S-2). O menu troca só a descoberta do
///    número; o portão continua sendo o modelo.
///
/// # E não há detecção de terminal aqui, de propósito
///
/// A recusa de quem chama isto de um script já existe e é a mesma do `arca
/// restore`: um `stdin` fechado devolve linha vazia
/// ([`crate::portas::Console`]), e linha vazia nunca escolhe nada. O
/// `--sem-pausa` **não** serve de sinal para isso — ela diz "não segure a
/// janela ao terminar", e não "não há ninguém aqui". Usá-la como proxy de
/// terminal seria dar-lhe um significado que ela não tem, e dois significados
/// numa flag divergem na primeira mudança.
fn escolher_o_disco(contexto: &Contexto, discos: &[DiscoParaPreparar]) -> Resultado<u32> {
    use std::io::Write;

    let oferta = preparacao::Oferta::de(discos, letra_do_sistema());

    // A lista sai **antes** da recusa de lista vazia, e é ela que faz a recusa
    // ser lida: "nenhum disco pode ser preparado" para quem está vendo dois
    // discos na mesa parece defeito do ARCA. Com os motivos por cima, é
    // resposta.
    print!("{}", montar_o_menu(&oferta));

    if oferta.candidatos.is_empty() {
        return Err(Erro::PreparacaoRecusada(
            preparacao::RecusaDaPreparacao::NadaAOferecer {
                recusados: oferta.recusados.len(),
            },
        ));
    }

    print!("\nQual preparar? ");
    let _ = std::io::stdout().flush();

    // Um console que não se deixou ler **sobe como erro de leitura**, e não
    // como escolha inválida — a mesma distinção que o `arca restore` faz: uma
    // diz "você digitou errado" e a outra diz "não consegui ouvir".
    let digitado = contexto.console.ler_linha()?;
    println!();

    // Uma tentativa, e não um laço — a mesma regra da confirmação. Quem errou
    // repete o comando, que até aqui não tocou em disco nenhum.
    let escolhido = digitado.trim();
    oferta.escolher_pelo_numero(escolhido).ok_or_else(|| {
        Erro::PreparacaoRecusada(preparacao::RecusaDaPreparacao::EscolhaInvalida {
            digitado: escolhido.to_string(),
            quantas: oferta.candidatos.len(),
        })
    })
}

/// A lista numerada do §6.1, e os recusados ditos embaixo sem número.
///
/// # Duas colunas de número, e por que as duas existem
///
/// `[1]` é o que se digita **aqui**; `disco 1` é o índice do Windows — o que o
/// `Get-Disk` mostra e o que o `--dispositivo` recebe. **Eles não são o mesmo
/// número**, e a lista os separa em vez de deixar a coincidência ensinar
/// errado: numa máquina onde o disco 0 é o do Windows, o primeiro candidato é
/// o `[1] disco 1` e os dois batem por acidente; conectado um segundo SSD, o
/// `[1]` passa a ser o `disco 2` e quem aprendeu a ler o número da esquerda
/// como índice erra.
///
/// # Os recusados aparecem, e a decisão é do `arca restore`
///
/// [`crate::comandos::restore::montar_a_lista`] enfrentou esta escolha e a
/// resolveu: mostrar sem número. Omitir faria a lista parecer incompleta para
/// quem sabe que há outro disco na mesa — e o pior caso aqui é a defesa 1, que
/// recusa o disco externo que o Windows não soube classificar. Escondido, o
/// motivo vira ausência, e a pessoa conclui que o ARCA não enxerga o HD dela.
/// Listado sem número, ele vira uma frase.
///
/// E a numeração sai **só dos candidatos**, que é a outra metade da mesma
/// doutrina: um número ao lado de um item não escolhível ocuparia um índice, e
/// aí os números passariam a depender de coisas que não se pode digitar.
pub fn montar_o_menu(oferta: &preparacao::Oferta) -> String {
    let mut saida = String::from("\nDiscos desta maquina:\n\n");

    // O preenchimento é contado a mão: `{:<n$}` conta bytes, e um modelo com
    // acento sairia desalinhado.
    let coluna = oferta
        .candidatos
        .iter()
        .copied()
        .chain(oferta.recusados.iter().map(|(disco, _)| *disco))
        .map(|disco| disco.modelo.chars().count())
        .max()
        .unwrap_or(0)
        + 3;

    for (numero, disco) in oferta.candidatos.iter().enumerate() {
        saida.push_str(&format!(
            "  [{}]  disco {:<2}  {}{}{}\n",
            numero + 1,
            disco.indice,
            disco.modelo,
            " ".repeat(coluna - disco.modelo.chars().count()),
            descrever_o_disco(disco),
        ));
    }

    if !oferta.recusados.is_empty() {
        saida.push_str("\n  Sem numero, e o `arca prepare` nao prepara:\n");
        for (disco, porque) in &oferta.recusados {
            saida.push_str(&format!(
                "       disco {:<2}  {}{}{}\n",
                disco.indice,
                disco.modelo,
                " ".repeat(coluna - disco.modelo.chars().count()),
                descrever_o_disco(disco),
            ));
            saida.push_str(&format!("                 {}\n", porque.resumo()));
        }
    }

    if !oferta.candidatos.is_empty() {
        saida.push_str(
            "\n  O numero entre colchetes e o que se digita; o `disco N` e o indice do\n\
             \x20 Windows, que e o que o `--dispositivo` recebe. Escolher um numero so\n\
             \x20 mostra o plano — nada e apagado antes da confirmacao digitada.\n",
        );
    }

    saida
}

/// A linha que descreve um disco na lista: tamanho, barramento, tabela e o que
/// mora nele hoje.
///
/// O `ja e um dispositivo ARCA` no fim não é enfeite. Preparar por cima de um
/// dispositivo apaga **as imagens dele**, e a tela do plano já diz isso — mas
/// dizer só lá é tarde para quem tem dois SSDs iguais na mesa e está
/// escolhendo qual dos dois é o velho.
fn descrever_o_disco(disco: &DiscoParaPreparar) -> String {
    format!(
        "{} · {} · {} · {}{}",
        tamanho(disco.tamanho_bytes),
        disco.barramento,
        disco.estilo_de_particao,
        resumir_o_conteudo(disco),
        if e_um_dispositivo_arca(disco) {
            " · JA E UM DISPOSITIVO ARCA"
        } else {
            ""
        }
    )
}

/// O que existe no disco hoje, em poucas palavras.
fn resumir_o_conteudo(disco: &DiscoParaPreparar) -> String {
    if disco.particoes.is_empty() {
        // Um disco RAW, ou um meio-apagado por um `prepare` que morreu no
        // `Clear-Disk`, aparece na lista como qualquer outro. É a lista que o
        // descreve, e não um rótulo que ele não tem.
        return "sem particao nenhuma".to_string();
    }

    let plural = if disco.particoes.len() == 1 {
        "particao"
    } else {
        "particoes"
    };

    let letras = disco.letras();
    if letras.is_empty() {
        format!("{} {plural}, nenhuma com letra", disco.particoes.len())
    } else {
        format!(
            "{} {plural} ({})",
            disco.particoes.len(),
            letras
                .iter()
                .map(|letra| format!("{letra}:"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

/// Baixa (ou usa o `--iso`), confere o SHA256 e guarda a cópia de PR-3.
///
/// # A ordem é baixar, conferir, e só então usar
///
/// PR-1 é sobre isto: um pacote que não se conferiu é um pacote que não se sabe
/// o que é, e extrair antes de conferir tornaria a conferência decorativa.
///
/// A cópia de PR-3 fica no `ARCAVAULT` **depois** de conferida, pelo mesmo
/// motivo: guardar um pacote que não passou seria guardar lixo com cara de
/// fonte confiável.
fn obter_o_pacote(
    contexto: &Contexto,
    iso: Option<&Path>,
    raiz_do_vault: &Path,
) -> Resultado<PathBuf> {
    let (local, de_onde) = match iso {
        // PR-2: instala de arquivo local. É o que salva quando a máquina que
        // precisa preparar o dispositivo é justamente a que está sem Windows —
        // e nesse caso não há rede.
        Some(caminho) => (caminho.to_path_buf(), format!("{}", caminho.display())),
        None => {
            let destino = raiz_do_vault.join(pacote::ARQUIVO);
            print!(
                "{}",
                linha(
                    "Baixando Clonezilla",
                    &format!(
                        "{} · {} · pode levar minutos",
                        pacote::VERSAO,
                        tamanho(pacote::TAMANHO_BYTES)
                    ),
                )
            );

            let saida = contexto.sistema.baixar(pacote::URL, &destino)?;
            if saida.codigo != 0 {
                return Err(Erro::FerramentaRecusou {
                    ferramenta: "curl",
                    codigo: saida.codigo,
                    saida: saida.resumo(3),
                });
            }
            (destino, pacote::URL.to_string())
        }
    };

    // PR-1, e a conferência acontece antes de qualquer uso do arquivo.
    let medido = crate::resumo::do_certutil(
        &contexto
            .sistema
            .resumir(&local, crate::resumo::Algoritmo::Sha256)?,
        crate::resumo::Algoritmo::Sha256,
    );
    let resumo = pacote::conferir_o_resumo(medido).map_err(Erro::PacoteRecusado)?;

    print!(
        "{}",
        linha(
            "SHA256 conferido",
            &format!("ok · {} · de {de_onde}", resumo.abreviado()),
        )
    );

    // E o conteúdo, antes de escrever no dispositivo: o `bsdtar` sair com zero
    // não é prova de que o pacote tem o que faz um dispositivo bootar.
    let listagem = contexto.sistema.listar_pacote(&local)?;
    let faltando = pacote::o_que_falta(listagem.texto.lines().map(str::trim));
    if !faltando.is_empty() {
        return Err(Erro::PacoteRecusado(
            pacote::RecusaDoPacote::PacoteIncompleto { faltando },
        ));
    }

    // PR-3: a cópia no `ARCAVAULT`, para que o dispositivo se reconstrua
    // sozinho. Quando o pacote veio de `--iso`, ele ainda não está lá.
    let copia = raiz_do_vault.join(pacote::ARQUIVO);
    if copia != local {
        contexto.arquivos.copiar(&local, &copia)?;
    }
    print!(
        "{}",
        linha(
            "Copia do pacote em ARCAVAULT",
            &format!("ok · {} (PR-3)", copia.display()),
        )
    );

    Ok(local)
}

/// Põe o binário do ARCA no `ARCABOOT` (§4.1).
///
/// # Por que o próprio executável, e por que isso não é opcional
///
/// §4.1 diz que o ARCA e o estado moram no dispositivo, e a razão é que uma
/// restauração devolve o `C:` inteiro — inclusive um ARCA antigo com defeitos
/// já corrigidos. **O que julga a restauração não pode morar no disco que a
/// restauração substitui.**
///
/// E a E11 pagou por isso: o binário do `ARCABOOT` era de uma etapa anterior e
/// não conhecia a terceira `Operacao`; colher com ele teria recusado o
/// `estado.json` do job que ele mesmo tinha de colher.
///
/// # O que isto congela, e não há comando para descongelar
///
/// O que se instala é **o executável que está rodando** — `current_exe()`. Um
/// dispositivo preparado hoje carrega o ARCA de hoje, e continua carregando
/// depois de o ARCA mudar.
///
/// Isso não é defeito: §4.1 quer exatamente isso, um julgamento que sobreviva
/// à restauração e não venha de dentro dela. Mas tem consequência, e ela já
/// mordeu uma vez — **copiar o binário para o `ARCABOOT` é pré-requisito de
/// todo marco que mude o formato do `estado.json`**, e não há comando do ARCA
/// que faça isso sozinho. Rodar `arca prepare` de novo faz, junto com apagar o
/// disco; à mão é uma cópia.
///
/// **E há uma armadilha a mais, que só apareceu em 24/08/2026:** o que se
/// instala é o executável que está rodando, e não o mais novo que existe. Rodar
/// `arca prepare` a partir do `arca.exe` de um dispositivo antigo faz o
/// dispositivo novo **herdar a idade do velho**. Prepare sempre a partir do
/// `target\release\`.
///
/// Desde a mesma data o binário sabe responder de onde veio — `arca --version`
/// carimba o commit e a data (`cli::VERSAO`, `build.rs`). Antes disso os dois
/// executáveis diziam `arca 0.1.0`, e descobrir que o do `ARCABOOT` estava três
/// consertos atrás exigiu procurar strings dentro do `.exe`.
///
/// Um `arca atualizar` que só trocasse o binário é concebível, e fica de fora
/// até o uso pedir — como P-14.
fn instalar_o_arca(contexto: &Contexto, raiz_do_boot: &Path) -> Resultado<()> {
    let eu = std::env::current_exe().map_err(Erro::ExecutavelDesconhecido)?;
    let pasta = raiz_do_boot.join(PASTA_DO_ARCA);
    contexto.arquivos.criar_diretorio(&pasta)?;

    let destino = pasta.join(eu.file_name().unwrap_or("arca.exe".as_ref()));
    contexto.arquivos.copiar(&eu, &destino)?;

    print!(
        "{}",
        linha(
            "Instalando o ARCA em ARCABOOT",
            &format!("ok · {} (§4.1)", destino.display()),
        )
    );
    Ok(())
}

/// O que a criação da entrada de firmware deixou.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntradaCriada {
    pub identificador: String,

    /// Se ela já existia — o comando rodado duas vezes não cria uma segunda.
    pub ja_existia: bool,

    pub alvo: Alvo,

    /// Se ela precisou sair da ordem permanente. `bcdedit /copy` a põe lá
    /// sozinho, e é isso que C-5 nomeia como perigo.
    pub saiu_da_ordem: bool,

    /// O nome da primeira entrada que **sobrou** na ordem permanente sem dizer
    /// para onde aponta — as `UEFI:*` que o firmware acrescenta no POST.
    ///
    /// Tirar a entrada do ARCA da ordem não é, sozinho, o que autoriza a
    /// promessa da tela de fim (*"ligar a máquina continua subindo o
    /// Windows"*): ela vale sobre o que **restou** na ordem, e não sobre o que
    /// saiu. É P-28, e o [ADR-0021].
    ///
    /// [ADR-0021]: ../../docs/adr/0021-uma-entrada-sem-alvo-na-ordem-nao-e-seguranca.md
    pub ordem_sem_alvo: Option<String>,
}

/// Cria a entrada de firmware, aponta-a para o `ARCABOOT` e a tira da ordem.
///
/// # C-4 pelo outro lado
///
/// C-4 manda **migrar** a entrada legada em vez de criar outra, e o motivo é
/// não deixar a máquina com duas formas de bootar no Clonezilla. Aqui vale o
/// mesmo, e por isso a primeira coisa que este comando faz é procurar: havendo
/// `ARCA` ou `Clonezilla`, ele **reusa** em vez de criar.
///
/// Isso é o que torna o `arca prepare` rodável duas vezes sem sujar o firmware
/// — e é a mesma idempotência que o desarmar ganhou de graça no ADR-0005.
fn criar_a_entrada(contexto: &Contexto, letra_do_boot: char) -> Resultado<EntradaCriada> {
    let desejado = Alvo::ParticaoComLetra(letra_do_boot.to_ascii_uppercase());

    // A ordem permanente, **antes** — é contra ela que se confere que o
    // `/remove` fez o que devia, e que nada mais mudou.
    let antes = firmware::ler(&contexto.firmware.enumerar(FWBOOTMGR)?);
    if !antes.viu_o_gerenciador {
        return Err(Erro::FirmwareIlegivel { alvo: FWBOOTMGR });
    }

    let leitura = firmware::ler(&contexto.firmware.enumerar(FIRMWARE)?);
    let (identificador, ja_existia) = match leitura.entrada_do_arca() {
        Some(achada) => (achada.entrada.identificador.clone(), true),
        None => (copiar_do_bootmgr(contexto)?, false),
    };

    // A descrição, o `device` e o `path`. As três com releitura de C-3 — o
    // sucesso do `bcdedit` nunca é prova, e com mídia removível ele responde
    // "êxito" mantendo o valor antigo (C-6).
    let _ = contexto
        .firmware
        .executar(&["/set", &identificador, "description", firmware::ARCA]);
    let _ = contexto.firmware.executar(&[
        "/set",
        &identificador,
        "device",
        &desejado.como_bcdedit_escreve(),
    ]);
    let _ = contexto
        .firmware
        .executar(&["/set", &identificador, "path", CAMINHO_DO_EFI]);

    let relida = releitura(contexto, &identificador)?;

    if !relida.aponta_para(&desejado) {
        return Err(Erro::AlvoDoFirmwareRecusado {
            identificador,
            esperado: desejado.como_bcdedit_escreve(),
            tem: relida
                .alvo
                .as_ref()
                .map(Alvo::como_bcdedit_escreve)
                .unwrap_or_else(|| "nada".to_string()),
        });
    }

    if relida.caminho.as_deref() != Some(CAMINHO_DO_EFI) {
        return Err(Erro::CaminhoDoEfiRecusado {
            identificador,
            esperado: CAMINHO_DO_EFI.to_string(),
            tem: relida.caminho.unwrap_or_else(|| "nada".to_string()),
        });
    }

    // **A terceira, que o comentário acima prometia e ninguém conferia.**
    //
    // O `/set description` é o que migra a entrada legada `Clonezilla` para
    // `ARCA` (C-4, ADR-0017), e ele é o mesmo comando que o C-6 pega mentindo:
    // medido em 25/08/2026, num Kingston DataTraveler Max, o `bcdedit /set`
    // responde *"A operação foi concluída com êxito"*, código 0, e **não
    // escreve**. Não havia motivo para supor que só o `device` sofre disso.
    //
    // Deixar passar seria a tela do fim afirmar `ARCA` sobre uma entrada que
    // continua chamada `Clonezilla` — o ARCA relatando o que espera em vez do
    // que há, que é justamente o que `EntradaDoArca::descricao` existe para
    // não fazer. Recusar aqui custa rodar o comando de novo; o disco já está
    // particionado e o `prepare` é idempotente a partir daí.
    if relida.descricao.as_deref() != Some(firmware::ARCA) {
        return Err(Erro::DescricaoDoFirmwareRecusada {
            identificador,
            esperado: firmware::ARCA.to_string(),
            tem: relida.descricao.unwrap_or_else(|| "nada".to_string()),
        });
    }

    // **O achado da medição de 23/08/2026**: `bcdedit /copy` põe a entrada
    // nova no `displayorder` sozinho. Ninguém pediu, e isso é acrescentar um
    // caminho permanente para bootar no dispositivo — o perigo que C-5 nomeia.
    //
    // Tirar não quebra o armar: o `bootsequence` funciona sobre entrada fora
    // da ordem, medido na E7 e exercitado no marco de 22/08 (ADR-0007).
    let saiu_da_ordem = tirar_da_ordem(contexto, &identificador, &antes)?;

    // **A ordem que sobrou, e não a que saiu** (P-28). A promessa da tela de
    // fim é sobre onde a máquina vai bootar, e quem decide isso é o que ficou
    // na ordem. Uma leitura a mais, e ela tem de ser de `firmware` e não de
    // `{fwbootmgr}`: só a primeira traz as entradas junto da ordem, e sem elas
    // não há como perguntar de nenhuma se ela declara alvo.
    //
    // Uma leitura que não se deixa entender **recusa** em vez de virar `None`:
    // `None` aqui é a tela prometendo que a máquina sobe o Windows, e é
    // exatamente a afirmação que ela não poderia fazer sem ter lido a ordem.
    let sobrou = firmware::ler(&contexto.firmware.enumerar(FIRMWARE)?);
    if !sobrou.viu_o_gerenciador {
        return Err(Erro::FirmwareIlegivel { alvo: FWBOOTMGR });
    }
    let ordem_sem_alvo = sobrou.primeira_sem_alvo(sobrou.ordem_permanente.len());

    Ok(EntradaCriada {
        identificador,
        ja_existia,
        alvo: desejado,
        saiu_da_ordem,
        ordem_sem_alvo,
    })
}

/// `bcdedit /copy {bootmgr} /d ARCA`, e o identificador achado **pela forma**.
///
/// # Por que pela forma, e não pelo texto
///
/// Medido em 23/08/2026, nesta máquina em português: *"A entrada foi copiada
/// com sucesso para {f4057bd1-…}."* A frase é traduzida — é o mesmo caso do
/// `chkdsk` de B-6 e do `certutil` de V-1 — e o que não é traduzido é a forma
/// do identificador: trinta e seis caracteres entre chaves.
///
/// Havendo mais de um na resposta, isto **recusa** em vez de escolher o
/// primeiro. É o mesmo raciocínio de [`crate::resumo::do_certutil`] e do selo
/// repetido: duas respostas não dizem qual vale — e aqui a escolha errada
/// apontaria o boot da máquina para o lugar errado.
fn copiar_do_bootmgr(contexto: &Contexto) -> Resultado<String> {
    let resposta = contexto
        .firmware
        .executar(&["/copy", BOOTMGR, "/d", firmware::ARCA])?;

    let achados = identificadores_de(&resposta);
    match achados.as_slice() {
        [um] => Ok(um.clone()),
        _ => Err(Erro::EntradaNaoFoiCriada {
            quantos: achados.len(),
            resposta: resposta.trim().to_string(),
        }),
    }
}

/// Os `{guid}` de um texto, achados pela forma.
///
/// Trinta e seis caracteres entre chaves, sendo hexadecimais e hifens. O
/// `{bootmgr}` e o `{fwbootmgr}` não casam — eles são apelidos, e não GUIDs —,
/// o que é o que se quer: o identificador procurado é o da entrada nova.
fn identificadores_de(texto: &str) -> Vec<String> {
    let mut achados: Vec<String> = Vec::new();

    for (inicio, _) in texto.match_indices('{') {
        let resto = &texto[inicio..];
        let Some(fim) = resto.find('}') else { continue };
        let dentro = &resto[1..fim];

        if dentro.chars().count() == 36 && dentro.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
        {
            let candidato = resto[..=fim].to_string();
            if !achados.contains(&candidato) {
                achados.push(candidato);
            }
        }
    }

    achados
}

/// Tira a entrada do `displayorder` e confere que ela saiu (C-3, C-5).
///
/// Devolve se **havia** o que tirar, e a pergunta é feita sobre a ordem **de
/// agora** — depois do `/copy`, que é quem põe a entrada lá. A leitura de
/// `antes`, do começo do comando, serve para outra coisa: provar que nenhuma
/// entrada que não é do ARCA sumiu junto.
///
/// A distinção importa porque a linha da tela diz uma coisa ou outra, e um
/// `ok · saiu da ordem` sobre uma entrada que nunca esteve lá é a mesma mentira
/// que este projeto já contou duas vezes (§11).
fn tirar_da_ordem(
    contexto: &Contexto,
    identificador: &str,
    antes: &firmware::Leitura,
) -> Resultado<bool> {
    let estava = |leitura: &firmware::Leitura| {
        leitura
            .ordem_permanente
            .iter()
            .any(|entrada| entrada.eq_ignore_ascii_case(identificador))
    };

    // A ordem **corrente**, depois do `/copy`. É ela que diz se há o que tirar.
    let agora = firmware::ler(&contexto.firmware.enumerar(FWBOOTMGR)?);
    if !agora.viu_o_gerenciador {
        return Err(Erro::FirmwareIlegivel { alvo: FWBOOTMGR });
    }
    let havia = estava(&agora);

    // Manda tirar **sempre**, mesmo estando fora — e o motivo é o mesmo que
    // C-6 usa no `device`: é a releitura que responde, e pular a escrita no
    // caso normal deixaria justamente o caminho normal sem exercício. Medido:
    // o `/remove` de uma entrada que não está na ordem sai com código 0 e não
    // muda nada.
    let _ =
        contexto
            .firmware
            .executar(&["/set", FWBOOTMGR, "displayorder", identificador, "/remove"]);

    let depois = firmware::ler(&contexto.firmware.enumerar(FWBOOTMGR)?);
    if !depois.viu_o_gerenciador {
        return Err(Erro::FirmwareIlegivel { alvo: FWBOOTMGR });
    }

    if estava(&depois) {
        return Err(Erro::EntradaContinuaNaOrdem {
            identificador: identificador.to_string(),
            ordem: depois.ordem_permanente.join(", "),
        });
    }

    // O que **estava** na ordem antes continua lá. Este comando tira uma
    // entrada que ele mesmo pôs; tirar qualquer outra seria mexer numa decisão
    // que não é dele, no lugar onde um erro deixa a máquina sem bootar.
    let sumiu_alguma_outra = antes
        .ordem_permanente
        .iter()
        .filter(|entrada| !entrada.eq_ignore_ascii_case(identificador))
        .any(|entrada| {
            !depois
                .ordem_permanente
                .iter()
                .any(|sua| sua.eq_ignore_ascii_case(entrada))
        });

    if sumiu_alguma_outra {
        return Err(Erro::OrdemPermanenteAlterada {
            antes: antes.ordem_permanente.join(", "),
            depois: depois.ordem_permanente.join(", "),
        });
    }

    Ok(havia)
}

/// A entrada, relida pelo identificador (C-3).
fn releitura(contexto: &Contexto, identificador: &str) -> Resultado<firmware::EntradaDeFirmware> {
    firmware::ler(&contexto.firmware.enumerar(identificador)?)
        .entradas
        .into_iter()
        .find(|entrada| entrada.identificador.eq_ignore_ascii_case(identificador))
        .ok_or(Erro::FirmwareIlegivel { alvo: FIRMWARE })
}

// ─────────────────────────── as telas ───────────────────────────

/// O plano inteiro, antes de qualquer escrita (PR-4, 1º tempo).
///
/// **Quem vai perder dados tem de poder reconhecê-los na tela.** Por isso as
/// partições existentes saem com rótulo, sistema de arquivos e tamanho — e não
/// só "2 partições": ninguém reconhece um disco por uma contagem.
pub fn montar_o_plano(preparacao: &Preparacao, iso: Option<&Path>) -> String {
    let disco = &preparacao.disco;
    let mut saida = String::new();

    saida.push_str(&linha(
        &format!("Disco {}", disco.indice),
        &format!(
            "{} · {} · {}",
            disco.modelo,
            disco.barramento,
            tamanho(disco.tamanho_bytes)
        ),
    ));
    saida.push_str(&linha(
        "Tipo de midia",
        &format!("{} · nao e disco fixo (PR-5)", rotulo_da_midia(disco)),
    ));

    // **Diz o que foi lido, e não só que passou.** Um `nao e o disco do
    // Windows` sozinho é o ARCA afirmando; com os três valores ao lado, quem
    // lê pode conferir a afirmação em vez de acreditar nela. É a mesma razão
    // pela qual a tela do §6.1 imprime os dois discos em setores em vez do
    // veredito de R-7 resumido.
    saida.push_str(&linha(
        "Sistema",
        &format!(
            "IsSystem {} · IsBoot {} · nao carrega o {}",
            disco.e_do_sistema,
            disco.e_de_boot,
            match letra_do_sistema() {
                Some(letra) => format!("{letra}:"),
                None => "volume do Windows".to_string(),
            }
        ),
    ));
    saida.push_str(&linha(
        "Tabela de particao hoje",
        &format!("{} · vai ser reescrita como GPT", disco.estilo_de_particao),
    ));

    saida.push_str("\nO QUE EXISTE NESTE DISCO HOJE, e vai ser APAGADO:\n");
    if disco.particoes.is_empty() {
        saida.push_str("  (nenhuma particao — o disco esta em branco)\n");
    } else {
        for particao in &disco.particoes {
            saida.push_str(&format!(
                "  {}  {:<6} {:>9}  {:<26} {}\n",
                particao.numero,
                particao.sistema_de_arquivos.as_deref().unwrap_or("crua"),
                tamanho(particao.tamanho_bytes),
                match &particao.rotulo {
                    Some(rotulo) => format!("\"{rotulo}\""),
                    None => "(sem rotulo)".to_string(),
                },
                match particao.letra {
                    Some(letra) => format!("{letra}:"),
                    None => "(sem letra)".to_string(),
                }
            ));
        }
    }

    // **O caso que a execução real trouxe à tona.** Preparar por cima de um
    // dispositivo ARCA que já existe apaga **as imagens dele** — e os rótulos
    // na lista acima dizem isso a quem sabe lê-los, o que não é a mesma coisa
    // que dizer.
    //
    // Este é exatamente o tipo de coisa que PR-4 existe para nomear: quem vai
    // perder dados tem de poder reconhecê-los. Um `ARCAVAULT` de 445 GB na
    // linha de cima é reconhecível para o ARCA e não para quem está com dois
    // SSDs iguais na mesa.
    if e_um_dispositivo_arca(disco) {
        saida.push_str(
            "\n  ESTE DISCO JA E UM DISPOSITIVO ARCA. Os rotulos acima sao os dele, e o\n\
             \x20 que esta no ARCAVAULT sao AS IMAGENS — todas. Preparar por cima apaga\n\
             \x20 cada uma, e o ARCA nunca apaga imagem em nenhum outro caminho (B-10).\n\
             \x20 Se o que voce quer e reinstalar o Clonezilla sem perder as imagens,\n\
             \x20 este comando NAO faz isso: ele comeca reescrevendo a tabela de particao.\n",
        );
    }

    saida.push_str("\nO QUE VAI FICAR NO LUGAR:\n");
    saida.push_str(&format!(
        "  GPT  1  NTFS  {:>9}  {ARCAVAULT}   as imagens moram aqui\n",
        tamanho(preparacao.plano.vault_bytes)
    ));
    saida.push_str(&format!(
        "       2  FAT32 {:>9}  {ARCABOOT}    o Clonezilla e o ARCA moram aqui\n",
        tamanho(preparacao.plano.boot_bytes)
    ));

    saida.push_str(
        "\n  A estrutura e GPT, e as duas particoes sao de dados basicos — a ARCABOOT\n\
         \x20 nao e uma ESP. E o que foi medido em 25/08/2026: um dispositivo assim\n\
         \x20 BOOTOU nesta maquina, e o caminho que o firmware carregou foi lido de\n\
         \x20 dentro do boot pelo efibootmgr (ADR-0025).\n\n\
         \x20 O Windows cria sozinho uma particao Microsoft Reserved ao inicializar em\n\
         \x20 GPT, e o ARCA a remove: deixada em pe, ela empurraria estas duas para 2 e\n\
         \x20 3, e o dispositivo seria outro.\n",
    );

    saida.push_str("\nE O QUE MAIS VAI ACONTECER:\n");
    saida.push_str(&format!(
        "  Clonezilla {} · {}\n",
        pacote::VERSAO,
        match iso {
            Some(caminho) => format!("do arquivo local {} (PR-2)", caminho.display()),
            None => format!(
                "baixado ({}), com o SHA256 conferido contra\n     o valor compilado neste ARCA — e nao contra um baixado junto (PR-1)",
                tamanho(pacote::TAMANHO_BYTES)
            ),
        }
    ));
    saida.push_str(
        "  Uma copia do pacote fica no ARCAVAULT, para o dispositivo se reconstruir\n     sozinho (PR-3)\n",
    );

    // **O firmware entra no plano**, e não como surpresa depois da
    // confirmação. Criar entrada de boot mexe na NVRAM da máquina, que é o
    // lugar onde um erro deixa alguém sem bootar — e quem lê um plano antes de
    // apagar um disco tem o direito de saber que o plano não para no disco.
    saida.push_str(&format!(
        "  Uma entrada de boot chamada `{}` e criada no firmware, apontando para o\n     \
         ARCABOOT — e **tirada da ordem permanente** logo em seguida, para que\n     \
         ligar a maquina continue subindo o Windows (C-5)\n",
        firmware::ARCA
    ));
    saida.push_str(
        "  O proprio `arca.exe` e instalado no ARCABOOT, porque o que julga uma\n     \
         restauracao nao pode morar no disco que ela substitui (§4.1)\n\n  \
         O `grub.cfg` fica INERTE: nada roda sozinho ate um `arca backup` (§4.4)\n",
    );

    saida
}

/// Se o disco a preparar já é um dispositivo ARCA.
///
/// Pelos **dois** rótulos, e não por um: um disco com só o `ARCAVAULT` é um
/// dispositivo partido ou outra coisa qualquer que alguém rotulou assim, e o
/// aviso fala de perder imagens — que só faz sentido quando o par está lá.
fn e_um_dispositivo_arca(disco: &DiscoParaPreparar) -> bool {
    let tem = |procurado: &str| {
        disco.particoes.iter().any(|particao| {
            particao
                .rotulo
                .as_deref()
                .is_some_and(|rotulo| rotulo.eq_ignore_ascii_case(procurado))
        })
    };

    tem(ARCAVAULT) && tem(ARCABOOT)
}

/// O que o `--dry-run` diz no lugar da confirmação.
///
/// **Aqui o ensaio vale mais do que em qualquer outro comando**: é a única
/// forma de ver o plano de partições sem executá-lo (PR-5, defesa 6).
pub fn montar_o_ensaio(preparacao: &Preparacao) -> String {
    format!(
        "\nEnsaio (--dry-run): NADA foi tocado. O disco {} continua com o que tem.\n\
         O mesmo comando sem `--dry-run` APAGA esse disco inteiro.\n",
        preparacao.plano.indice_do_disco
    )
}

/// O que a releitura do disco mostrou (PR-5, defesa 7).
pub fn montar_as_particoes(feitas: &ParticoesFeitas) -> String {
    let mut saida = String::new();
    saida.push_str(&linha(
        "Particionando",
        &format!(
            "ok · GPT, 2 particoes de dados basicos ({}) · sem a MSR que o Windows cria",
            feitas.vault.tipo_gpt
        ),
    ));
    saida.push_str(&linha(
        "Formatando e rotulando",
        &format!(
            "ok · {ARCAVAULT} ({}) em {}: · {ARCABOOT} ({}) em {}:",
            feitas.vault.sistema_de_arquivos,
            feitas.vault.letra,
            feitas.boot.sistema_de_arquivos,
            feitas.boot.letra
        ),
    ));
    saida.push_str(&linha(
        "Conferido apos escrever",
        "ok · relido do disco · nenhuma particao ativa, unidade 4096 (C-3)",
    ));
    saida
}

/// A linha da entrada de firmware, e a da ordem permanente.
pub fn montar_a_entrada(entrada: &EntradaCriada) -> String {
    let mut saida = String::new();

    saida.push_str(&linha(
        "Entrada de firmware",
        &format!(
            "{} · {} · {} · {}",
            if entrada.ja_existia {
                "reusada e reapontada"
            } else {
                "criada"
            },
            firmware::ARCA,
            entrada.identificador,
            entrada.alvo.como_bcdedit_escreve()
        ),
    ));

    // **Reusar é C-4 na letra, e tem uma consequência que precisa ser dita.**
    // C-4 manda migrar a entrada que existe em vez de criar outra, para a
    // máquina não ficar com duas formas de bootar no Clonezilla. Aqui isso
    // significa que a entrada **deixou de apontar** para o dispositivo
    // anterior e passou a apontar para este.
    //
    // Não é perda: o `arca backup` reescreve o `device` a cada armar e relê
    // (C-6), então o dispositivo antigo volta a ser alcançável no instante em
    // que alguém armar com ele conectado. Mas quem tem dois na gaveta merece
    // saber o que mudou — a alternativa seria descobrir isso por um F12.
    if entrada.ja_existia {
        saida.push_str(
            "\n  A entrada de firmware ja existia e passou a apontar para ESTE dispositivo.\n\
             \x20 O ARCA mantem UMA entrada, e nao uma por dispositivo (C-4): duas seriam\n\
             \x20 duas formas de bootar no Clonezilla, uma delas sem ninguem olhando.\n\
             \x20 Se voce voltar a usar outro dispositivo ARCA, o `arca backup` reaponta a\n\
             \x20 entrada para ele ao armar, e confere que reapontou (C-6). Nao ha nada a\n\
             \x20 fazer a mao.\n",
        );
    }

    saida.push_str(&linha(
        "Ordem de boot",
        &if entrada.saiu_da_ordem {
            "ok · a entrada saiu da ordem permanente · o boot unico nao precisa dela la (C-5)"
                .to_string()
        } else {
            "ok · a entrada ja estava fora da ordem permanente".to_string()
        },
    ));

    saida
}

/// O fim, com o que fazer em seguida.
///
/// # A promessa do boot é sobre o que ficou na ordem, e não sobre o que saiu
///
/// A tela dizia *"ligar a maquina continua subindo o Windows"* a partir de um
/// fato só — a entrada do ARCA saiu da ordem permanente —, sem olhar quem
/// ficou nela. As entradas que o firmware acrescenta no POST não declaram alvo
/// (C-14, P-28), e uma delas à frente do `{bootmgr}` faria dessa frase uma promessa
/// que este repositório não pode mostrar tendo acontecido. Ver o
/// [ADR-0021](../../docs/adr/0021-uma-entrada-sem-alvo-na-ordem-nao-e-seguranca.md).
pub fn montar_o_fim(feitas: &ParticoesFeitas, ordem_sem_alvo: Option<&str>) -> String {
    let promessa = match ordem_sem_alvo {
        None => concat!(
            "\x20 A entrada de firmware existe e esta FORA da ordem permanente — ligar a\n",
            "\x20 maquina continua subindo o Windows, com ou sem este dispositivo conectado.\n",
        )
        .to_string(),
        Some(nome) => format!(
            concat!(
                "\x20 A entrada de firmware existe e esta FORA da ordem permanente. Mas a\n",
                "\x20 ordem tem `{}`, que NAO DIZ para onde aponta — quem a\n",
                "\x20 resolve e o firmware, no POST, pelo que estiver conectado —, e por\n",
                "\x20 isso o ARCA nao afirma o que ligar a maquina vai subir. Remova o SSD\n",
                "\x20 antes de religar se quiser certeza (P-28).\n",
            ),
            nome
        ),
    };

    format!(
        "\nDispositivo pronto.\n\n\
         \x20 O `grub.cfg` esta INERTE: bootar neste dispositivo abre o menu do\n\
         \x20 Clonezilla e espera alguem (§4.4). Nada roda sozinho ate um `arca backup`.\n\
         {promessa}\n\
         \x20 O {ARCAVAULT} esta em {}: e o {ARCABOOT} em {}:. As letras mudam de uma\n\
         \x20 conexao para outra; os rotulos, nao — e e por rotulo que o ARCA acha o\n\
         \x20 dispositivo (B-1, S-3).\n\n\
         \x20 SE VOCE TEM OUTRO DISPOSITIVO ARCA CONECTADO, desconecte um dos dois: o\n\
         \x20 ARCA opera um por vez, e com dois `arca backup` e `arca restore` recusam\n\
         \x20 por rotulo repetido (C-10).\n\n\
         \x20 ANTES DO PRIMEIRO BACKUP, RODE:  arca sondar\n\n\
         \x20 A receita nomeia o disco pelo nome que o LINUX lhe da (`nvme0n1`), e o\n\
         \x20 Windows nao conhece esse nome. O ARCA o descobre lendo um `blkdev.list`,\n\
         \x20 casando o modelo do disco (§4.5) — e este dispositivo acabou de nascer,\n\
         \x20 entao nao ha nenhum aqui. Um `arca backup` agora RECUSA, dizendo isso.\n\n\
         \x20 O ARCA nao pergunta o nome nem o deduz do indice: um `nvme1n1` digitado\n\
         \x20 por engano entraria numa receita que apaga um disco, e nao ha nada do\n\
         \x20 lado Windows contra o que conferi-lo.\n\n\
         \x20 `arca sondar` resolve isso num reinicio: ele NAO faz backup nem\n\
         \x20 restauracao — roda o `lsblk` no Linux do Clonezilla, grava a saida no\n\
         \x20 ARCAVAULT e desliga. Depois de `arca resultado`, `arca backup <nome>`\n\
         \x20 funciona.\n\n\
         \x20 E ele responde, de quebra, a unica coisa que o `arca prepare` NAO consegue\n\
         \x20 conferir sozinho: se este dispositivo boota mesmo, pela entrada de firmware\n\
         \x20 que acabou de ser criada (P-26).\n",
        feitas.vault.letra, feitas.boot.letra
    )
}

fn rotulo_da_midia(disco: &DiscoParaPreparar) -> &'static str {
    match disco.tipo_de_midia {
        crate::portas::TipoDeMidia::DiscoExterno => "External hard disk media",
        crate::portas::TipoDeMidia::Removivel => "Removable Media",
        crate::portas::TipoDeMidia::DiscoFixo => "Fixed hard disk media",
        crate::portas::TipoDeMidia::Desconhecido => "desconhecido",
    }
}

/// O segundo tempo de PR-4: *"Podemos continuar?"*
///
/// # Por que uma pergunta **antes** da confirmação digitada
///
/// Porque as duas fazem coisas diferentes. Esta dá a chance de sair depois de
/// ler o plano, e custa uma tecla; a confirmação digitada de S-2 custa ler e
/// digitar o modelo do disco, e existe para custar isso.
///
/// Trocar as duas por uma só perderia uma das duas coisas: ou se pergunta cedo
/// e barato, sem o peso de S-2, ou se pergunta caro e a pessoa que ia desistir
/// digita o modelo por inércia.
///
/// O julgamento saiu daqui na E12, quando o `arca sondar` passou a precisar da
/// mesma pergunta — ver [`crate::confirmacao::perguntar_se_pode`], inclusive
/// para por que lá ela aparece **sozinha**.
fn perguntar_se_pode(contexto: &Contexto) -> Resultado<bool> {
    crate::confirmacao::perguntar_se_pode(contexto, "Podemos continuar?")
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::adaptadores::RelogioDoSistema;
    use crate::duplos::{
        ArquivosEmMemoria, ConsoleDeMentira, DiscosDeMentira, EntropiaDeMentira, FirmwareDeMentira,
        ParticionadorDeMentira, RelogioParado, SistemaDeMentira, discos_para_preparar_desta_mesa,
        o_que_o_particionamento_deixou,
    };
    use crate::registro::Registro;

    /// As portas de um `arca prepare`, com o particionador **registrando** o
    /// que lhe mandaram fazer.
    ///
    /// O registro é o ponto desta bancada: a pergunta que quase todo teste
    /// daqui faz é *"o disco foi apagado?"*, e ela só existe porque o duplo
    /// guarda os planos que recebeu.
    struct Bancada {
        arquivos: ArquivosEmMemoria,
        discos: DiscosDeMentira,
        firmware: FirmwareDeMentira,
        relogio: RelogioParado,
        sistema: SistemaDeMentira,
        entropia: EntropiaDeMentira,
        console: ConsoleDeMentira,
        particionador: ParticionadorDeMentira,
        registro: Registro,
    }

    impl Bancada {
        fn nova(etiqueta: &str, console: ConsoleDeMentira) -> Bancada {
            Bancada {
                arquivos: ArquivosEmMemoria::novo(),
                discos: DiscosDeMentira::default(),
                firmware: FirmwareDeMentira::novo(),
                relogio: RelogioParado::em("2026-08-23T18:38:00"),
                entropia: EntropiaDeMentira::com(&[0; 8]),
                sistema: SistemaDeMentira::novo(),
                console,
                particionador: ParticionadorDeMentira::desta_mesa(),
                registro: Registro::em(
                    std::env::temp_dir().join(format!(
                        "arca-prepare-{etiqueta}-{}-{:?}",
                        std::process::id(),
                        std::thread::current().id()
                    )),
                    Box::new(RelogioDoSistema),
                ),
            }
        }

        fn contexto(&self, dry_run: bool) -> Contexto<'_> {
            Contexto {
                dry_run,
                registro: &self.registro,
                firmware: &self.firmware,
                discos: &self.discos,
                arquivos: &self.arquivos,
                relogio: &self.relogio,
                sistema: &self.sistema,
                entropia: &self.entropia,
                console: &self.console,
                particionador: &self.particionador,
            }
        }
    }

    impl Drop for Bancada {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(self.registro.caminho().parent().unwrap());
        }
    }

    // ─────────── o que NÃO acontece, que é o que mais importa aqui ───────────

    #[test]
    fn o_ensaio_nao_apaga_disco_nenhum() {
        // **O teste que a defesa 6 de PR-5 existe para tornar possível.** O
        // `--dry-run` é a única forma de ver o plano de partições sem
        // executá-lo, e um ensaio que apagasse o disco seria a pior mentira que
        // este projeto poderia contar.
        //
        // O duplo registra o que lhe mandaram fazer, então a pergunta é
        // direta — e ela é sobre o particionador, e não sobre a tela.
        let bancada = Bancada::nova("ensaio", ConsoleDeMentira::mudo());

        executar(&bancada.contexto(true), Some(1), None).expect("o ensaio nao falha");

        assert!(
            !bancada.particionador.particionou(),
            "o `--dry-run` mandou particionar"
        );
        assert!(
            bancada.sistema.baixados.borrow().is_empty(),
            "o `--dry-run` baixou o Clonezilla"
        );
        assert!(
            bancada.sistema.extraidos.borrow().is_empty(),
            "o `--dry-run` extraiu o pacote"
        );
        assert!(
            bancada.firmware.executados().is_empty(),
            "o `--dry-run` escreveu no firmware"
        );
    }

    #[test]
    fn o_ensaio_nao_pergunta_nada() {
        // Ele para **antes** dos quatro tempos de PR-4. Um ensaio que pedisse
        // a confirmação digitada faria alguém digitar o modelo de um disco
        // para não acontecer nada — e, pior, ensinaria a digitar sem ler.
        let bancada = Bancada::nova("mudo", ConsoleDeMentira::mudo());

        executar(&bancada.contexto(true), Some(1), None).expect("o ensaio nao falha");

        assert_eq!(
            bancada.console.lidas.get(),
            0,
            "o `--dry-run` leu do console"
        );
    }

    #[test]
    fn um_nao_na_pergunta_para_antes_de_tudo() {
        // O segundo tempo de PR-4 é uma saída de verdade, e não uma
        // formalidade: quem lê o plano e desiste tem de sair sem que nada
        // aconteça — inclusive sem chegar à confirmação digitada.
        let bancada = Bancada::nova("nao", ConsoleDeMentira::respondendo(&["n"]));

        executar(&bancada.contexto(false), Some(1), None).expect("desistir nao e erro");

        assert!(!bancada.particionador.particionou(), "apagou mesmo com `n`");
        assert_eq!(
            bancada.console.lidas.get(),
            1,
            "so a pergunta devia ter sido lida, e nao a confirmacao"
        );
    }

    #[test]
    fn o_enter_vazio_nao_e_um_sim() {
        // A pergunta é `(s/N)`, e o padrão é não. Um Enter distraído não pode
        // apagar 447 GB.
        for resposta in ["", " ", "sim, pode", "S ", "yes", "1"] {
            let bancada = Bancada::nova("vazio", ConsoleDeMentira::respondendo(&[resposta]));
            let _ = executar(&bancada.contexto(false), Some(1), None);

            assert!(
                !bancada.particionador.particionou(),
                "`{resposta}` passou por um sim"
            );
        }
    }

    #[test]
    fn a_confirmacao_errada_para_antes_de_apagar() {
        // S-2, e o ponto sem volta é **depois** dela. Quem digita o modelo
        // errado sai com o disco intacto.
        let bancada = Bancada::nova(
            "confirmacao",
            ConsoleDeMentira::respondendo(&["s", "JMicron"]),
        );

        let erro = executar(&bancada.contexto(false), Some(1), None).unwrap_err();

        assert!(matches!(erro, Erro::ConfirmacaoNaoBate { .. }), "{erro}");
        assert!(
            !bancada.particionador.particionou(),
            "apagou com a confirmacao errada"
        );
    }

    #[test]
    fn o_disco_do_windows_nao_chega_nem_a_perguntar() {
        // A defesa 2 de PR-5 acontece **antes** da tela. Um `arca prepare
        // --dispositivo 0` nesta mesa recusa sem perguntar nada, e sem que o
        // particionador saiba que existiu um pedido.
        let bancada = Bancada::nova("sistema", ConsoleDeMentira::respondendo(&["s", "KINGSTON"]));

        let erro = executar(&bancada.contexto(false), Some(0), None).unwrap_err();

        assert!(matches!(erro, Erro::PreparacaoRecusada(_)), "{erro}");
        assert!(!bancada.particionador.particionou());
        assert_eq!(bancada.console.lidas.get(), 0, "chegou a perguntar");
    }

    #[test]
    fn um_disco_que_muda_entre_o_plano_e_o_sim_nao_e_apagado() {
        // O terceiro tempo de PR-4. O duplo responde o disco 1 no `enumerar`
        // que monta o plano, e um `descrever` que devolve outro disco é o
        // cabo trocado no meio.
        //
        // Aqui a troca é feita apontando para um índice que o duplo conhece
        // com outro modelo — o efeito é o mesmo que desconectar um cabo, e é o
        // que a comparação de `e_o_mesmo_disco` existe para pegar.
        let mut discos = discos_para_preparar_desta_mesa();
        let mut trocado = discos[2].clone();
        trocado.indice = 1;
        discos[1] = trocado;

        let mut bancada = Bancada::nova("trocado", ConsoleDeMentira::respondendo(&["s", "x"]));
        bancada.particionador = ParticionadorDeMentira::com_discos(discos);

        // O plano sai sobre o disco que o `enumerar` respondeu; o `descrever`
        // responde o mesmo, então este caminho passa. O que se cobra abaixo é
        // que a conferência **aconteceu** — sem ela, a troca não teria como
        // ser pega.
        let _ = executar(&bancada.contexto(false), Some(1), None);

        assert!(
            bancada.particionador.descricoes.get() > 0,
            "o ARCA nao releu o disco antes de escrever (PR-4, 3o tempo)"
        );
    }

    fn preparacao_da_mesa() -> Preparacao {
        let discos = discos_para_preparar_desta_mesa();
        let alvo = discos.iter().find(|disco| disco.indice == 1);
        preparacao::julgar(1, alvo, &[0, 1, 2], Some('C')).expect("o disco 1 desta mesa passa")
    }

    // ─────────────────── o identificador, pela forma ───────────────────

    #[test]
    fn o_identificador_sai_da_resposta_medida_em_23_08() {
        // A frase e traduzida — esta e a desta maquina, em portugues. O que
        // nao e traduzido e a forma do identificador.
        let resposta =
            "A entrada foi copiada com sucesso para {f4057bd1-65a4-11f1-b0f1-aa4ed9bd2b34}.";

        assert_eq!(
            identificadores_de(resposta),
            vec!["{f4057bd1-65a4-11f1-b0f1-aa4ed9bd2b34}"]
        );
    }

    #[test]
    fn a_mesma_resposta_em_ingles_da_o_mesmo_identificador() {
        // Se o leitor dependesse do texto, uma maquina em ingles nao acharia
        // nada — e o comando criaria uma entrada que ele nao consegue apontar.
        let resposta =
            "The entry was successfully copied to {f4057bd1-65a4-11f1-b0f1-aa4ed9bd2b34}.";

        assert_eq!(
            identificadores_de(resposta),
            vec!["{f4057bd1-65a4-11f1-b0f1-aa4ed9bd2b34}"]
        );
    }

    #[test]
    fn os_apelidos_do_bcdedit_nao_sao_confundidos_com_guid() {
        // `{bootmgr}` e `{fwbootmgr}` aparecem em respostas do `bcdedit` e nao
        // sao GUIDs. Confundi-los faria o comando apontar o boot da maquina
        // para o lugar errado — que e o pior desfecho possivel deste comando.
        assert!(identificadores_de("copiada de {bootmgr} para {fwbootmgr}").is_empty());
        assert!(identificadores_de("{current} {memdiag} {globalsettings}").is_empty());
    }

    #[test]
    fn duas_respostas_nao_dizem_qual_vale() {
        // Mesmo raciocinio do selo repetido e do `ResumoAmbiguo`: nao se
        // escolhe a primeira. Aqui a escolha errada apontaria o boot para uma
        // entrada que nao e a que se criou.
        let dois = identificadores_de(
            "{f4057bd1-65a4-11f1-b0f1-aa4ed9bd2b34} e {f4057bd2-65a4-11f1-b0f1-aa4ed9bd2b34}",
        );
        assert_eq!(dois.len(), 2);
    }

    #[test]
    fn o_mesmo_identificador_repetido_conta_como_um() {
        // O `bcdedit` pode mencionar o identificador duas vezes na mesma
        // resposta. Isso nao e ambiguidade — e a mesma resposta.
        let repetido = identificadores_de(
            "{f4057bd1-65a4-11f1-b0f1-aa4ed9bd2b34} ... {f4057bd1-65a4-11f1-b0f1-aa4ed9bd2b34}",
        );
        assert_eq!(repetido.len(), 1);
    }

    // ─────────────────── a tela do plano (PR-4) ───────────────────

    #[test]
    fn o_plano_mostra_o_que_vai_ser_destruido_com_nome() {
        // PR-4 na letra: quem vai perder dados tem de poder reconhece-los. Uma
        // contagem — "2 particoes" — nao serve, porque ninguem reconhece um
        // disco por uma contagem.
        let saida = montar_o_plano(&preparacao_da_mesa(), None);

        assert!(saida.contains("Dell Beta Apps NO IA WSL"), "{saida}");
        assert!(saida.contains("NTFS"), "{saida}");
        assert!(saida.contains("E:"), "{saida}");
        assert!(saida.contains("447,1 GB"), "{saida}");
        assert!(saida.contains("APAGADO"), "{saida}");
    }

    #[test]
    fn o_plano_mostra_as_duas_particoes_que_vao_ficar() {
        let saida = montar_o_plano(&preparacao_da_mesa(), None);

        assert!(saida.contains("ARCAVAULT"), "{saida}");
        assert!(saida.contains("ARCABOOT"), "{saida}");
        assert!(saida.contains("GPT"), "{saida}");
        assert!(saida.contains("1,6 GB"), "o tamanho do ARCABOOT: {saida}");
    }

    #[test]
    fn o_plano_diz_que_a_arcaboot_nao_e_uma_esp_e_que_a_msr_sai() {
        // Duas coisas que quem lê a tela nao adivinha, e as duas mudariam o
        // dispositivo se fossem outras. A ARCABOOT ser dados basicos em vez de
        // ESP e a Variante B do marco — a que bootou —, e a MSR e a particao
        // que o Windows cria sozinho e que ninguem pediu.
        let saida = montar_o_plano(&preparacao_da_mesa(), None);

        assert!(saida.contains("nao e uma ESP"), "{saida}");
        assert!(saida.contains("Microsoft Reserved"), "{saida}");
        assert!(saida.contains("BOOTOU"), "{saida}");
        assert!(
            !saida.contains("nao GPT"),
            "a tela ainda diz que o esquema nao e GPT: {saida}"
        );
    }

    #[test]
    fn a_linha_do_sistema_diz_o_que_foi_lido_e_nao_so_que_passou() {
        // Um `nao e o disco do Windows` sozinho e o ARCA afirmando; com os
        // valores ao lado, quem lê pode conferir a afirmacao. E a mesma razao
        // pela qual o §6.1 imprime os dois discos em setores em vez do
        // veredito de R-7 resumido.
        let saida = montar_o_plano(&preparacao_da_mesa(), None);

        assert!(saida.contains("IsSystem false"), "{saida}");
        assert!(saida.contains("IsBoot false"), "{saida}");
    }

    #[test]
    fn o_plano_diz_que_vai_mexer_no_firmware() {
        // Criar entrada de boot mexe na NVRAM, que e o lugar onde um erro
        // deixa alguem sem bootar. Quem lê um plano antes de apagar um disco
        // tem o direito de saber que o plano nao para no disco.
        let saida = montar_o_plano(&preparacao_da_mesa(), None);

        assert!(saida.contains("entrada de boot"), "{saida}");
        assert!(saida.contains("tirada da ordem permanente"), "{saida}");
        assert!(saida.contains("§4.1"), "o binario no ARCABOOT: {saida}");
        assert!(saida.contains("PR-3"), "a copia do pacote: {saida}");
    }

    #[test]
    fn preparar_por_cima_de_um_dispositivo_arca_avisa_das_imagens() {
        // **O caso que a execucao real trouxe a tona.** Os rotulos na lista
        // dizem isso a quem sabe lê-los, o que nao e a mesma coisa que dizer.
        //
        // O disco 2 desta mesa e o dispositivo ja preparado: o duplo o traz
        // com os dois rotulos, e o plano tem de nomear o que se perde.
        let discos = discos_para_preparar_desta_mesa();
        let dispositivo = discos.iter().find(|disco| disco.indice == 2);
        let preparacao = preparacao::julgar(2, dispositivo, &[0, 1, 2], Some('C'))
            .expect("ele passa as defesas");

        let saida = montar_o_plano(&preparacao, None);
        assert!(saida.contains("JA E UM DISPOSITIVO ARCA"), "{saida}");
        assert!(saida.contains("AS IMAGENS"), "{saida}");
        assert!(saida.contains("B-10"), "{saida}");
    }

    #[test]
    fn um_disco_qualquer_nao_ganha_o_aviso_das_imagens() {
        // Conselho que aparece sempre vira ruido — a licao que a E10 pagou no
        // `arca resultado` e a E11 repetiu em V-1.
        let saida = montar_o_plano(&preparacao_da_mesa(), None);
        assert!(!saida.contains("JA E UM DISPOSITIVO ARCA"), "{saida}");
    }

    #[test]
    fn um_disco_com_um_rotulo_so_nao_e_dispositivo_arca() {
        // Um disco com so o `ARCAVAULT` e um dispositivo partido, ou outra
        // coisa que alguem rotulou assim. O aviso fala de **perder imagens**, e
        // isso so faz sentido com o par.
        let mut disco = discos_para_preparar_desta_mesa()[2].clone();
        disco.particoes.truncate(1);
        assert!(!e_um_dispositivo_arca(&disco));

        disco.particoes[0].rotulo = Some("ARCABOOT".to_string());
        assert!(!e_um_dispositivo_arca(&disco));
    }

    #[test]
    fn um_disco_em_branco_nao_finge_ter_particao() {
        let mut preparacao = preparacao_da_mesa();
        preparacao.disco.particoes.clear();

        let saida = montar_o_plano(&preparacao, None);
        assert!(saida.contains("o disco esta em branco"), "{saida}");
    }

    #[test]
    fn uma_particao_crua_aparece_na_tela_assim_mesmo() {
        // Uma particao sem volume nao tem rotulo nem sistema de arquivos, e
        // **continua sendo destruida**. Omiti-la faria a tela prometer menos
        // estrago do que o comando causa.
        let mut preparacao = preparacao_da_mesa();
        preparacao.disco.particoes[0].rotulo = None;
        preparacao.disco.particoes[0].sistema_de_arquivos = None;
        preparacao.disco.particoes[0].letra = None;

        let saida = montar_o_plano(&preparacao, None);
        assert!(saida.contains("crua"), "{saida}");
        assert!(saida.contains("(sem rotulo)"), "{saida}");
        assert!(saida.contains("(sem letra)"), "{saida}");
    }

    #[test]
    fn com_iso_a_tela_nao_promete_download() {
        let saida = montar_o_plano(&preparacao_da_mesa(), Some(Path::new(r"D:\cz.zip")));

        assert!(saida.contains(r"D:\cz.zip"), "{saida}");
        assert!(!saida.contains("baixado de"), "{saida}");
        assert!(saida.contains("PR-2"), "{saida}");
    }

    #[test]
    fn o_ensaio_nao_diz_que_fez_nada() {
        // A mentira que o `--dry-run` deste projeto ja contou uma vez (§11).
        let saida = montar_o_ensaio(&preparacao_da_mesa());

        assert!(saida.contains("NADA foi tocado"), "{saida}");
        assert!(!saida.contains("ok ·"), "{saida}");
    }

    // ─────────────────── as telas de depois ───────────────────

    #[test]
    fn a_tela_das_particoes_mostra_o_esquema_e_o_tipo_relidos() {
        // O GptType e **um** e nao dois, porque em GPT o tipo nao distingue as
        // duas particoes (ADR-0025). E a linha nomeia a MSR: ela e o passo que
        // o GPT trouxe, e um `prepare` que a tivesse deixado em pe teria
        // parado na releitura — quem le a tela merece saber que aquele passo
        // existe e deu certo.
        let saida = montar_as_particoes(&o_que_o_particionamento_deixou());

        assert!(saida.contains("GPT, 2 particoes"), "{saida}");
        assert!(
            saida.contains(crate::preparacao::TIPO_GPT_DADOS_BASICOS),
            "{saida}"
        );
        assert!(saida.contains("sem a MSR"), "{saida}");
        assert!(!saida.contains("MbrType"), "{saida}");
        assert!(saida.contains("nenhuma particao ativa"), "{saida}");
        assert!(saida.contains("relido do disco"), "{saida}");
    }

    #[test]
    fn a_entrada_criada_e_a_reusada_dizem_coisas_diferentes() {
        // Um `ok · criada` sobre uma entrada que ja existia e a mesma mentira
        // que a E4 pegou no desarmar e a E10 pegou na ordem de boot.
        let criada = EntradaCriada {
            identificador: "{f4057bd1-65a4-11f1-b0f1-aa4ed9bd2b34}".to_string(),
            ja_existia: false,
            alvo: Alvo::ParticaoComLetra('F'),
            saiu_da_ordem: true,
            ordem_sem_alvo: None,
        };
        assert!(montar_a_entrada(&criada).contains("criada"));
        assert!(
            montar_a_entrada(&criada).contains("saiu da ordem permanente"),
            "{}",
            montar_a_entrada(&criada)
        );

        let reusada = EntradaCriada {
            ja_existia: true,
            saiu_da_ordem: false,
            ..criada
        };
        let saida = montar_a_entrada(&reusada);
        assert!(saida.contains("reusada e reapontada"), "{saida}");
        assert!(saida.contains("ja estava fora"), "{saida}");
    }

    #[test]
    fn reusar_a_entrada_diz_que_ela_deixou_de_apontar_para_o_outro_dispositivo() {
        // C-4 mantem **uma** entrada, e nao uma por dispositivo. Quem tem dois
        // na gaveta merece saber que a entrada mudou de alvo — a alternativa e
        // descobrir isso por um F12 que nao acha o que se esperava.
        let reusada = EntradaCriada {
            identificador: "{f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}".to_string(),
            ja_existia: true,
            alvo: Alvo::ParticaoComLetra('F'),
            saiu_da_ordem: true,
            ordem_sem_alvo: None,
        };

        let saida = montar_a_entrada(&reusada);
        assert!(
            saida.contains("passou a apontar para ESTE dispositivo"),
            "{saida}"
        );
        assert!(saida.contains("C-4"), "{saida}");

        // O aviso tem de dizer que **nao ha o que fazer** — sem isso ele
        // descreve um problema e deixa quem lê com ele na mao. A frase quebra
        // de linha na tela, entao o teste procura as duas metades.
        assert!(saida.contains("reaponta a"), "{saida}");
        assert!(saida.contains("Nao ha nada a"), "{saida}");
        assert!(saida.contains("fazer a mao"), "{saida}");
    }

    #[test]
    fn criar_a_entrada_nao_gera_o_aviso_do_reaponte() {
        // Conselho que aparece sempre vira ruido. Num dispositivo criado do
        // zero nao havia entrada para deixar de apontar para lugar nenhum.
        let criada = EntradaCriada {
            identificador: "{f4057bd1-65a4-11f1-b0f1-aa4ed9bd2b34}".to_string(),
            ja_existia: false,
            alvo: Alvo::ParticaoComLetra('F'),
            saiu_da_ordem: true,
            ordem_sem_alvo: None,
        };

        assert!(!montar_a_entrada(&criada).contains("passou a apontar"));
    }

    #[test]
    fn o_fim_diz_que_o_dispositivo_esta_inerte_e_fora_da_ordem() {
        // As duas coisas que separam este dispositivo de um que roda alguma
        // coisa sozinho. Quem acabou de preparar um disco precisa saber que
        // religar a maquina continua subindo o Windows.
        let saida = montar_o_fim(&o_que_o_particionamento_deixou(), None);

        assert!(saida.contains("INERTE"), "{saida}");
        assert!(saida.contains("FORA da ordem permanente"), "{saida}");
        assert!(saida.contains("continua subindo o Windows"), "{saida}");
        assert!(
            saida.contains("C-10"),
            "o aviso dos dois dispositivos: {saida}"
        );
    }

    #[test]
    fn o_fim_nao_promete_o_windows_por_cima_de_uma_entrada_que_nao_diz_para_onde_aponta() {
        // **P-28.** A promessa é sobre onde a máquina vai bootar, e quem
        // decide isso é o que **ficou** na ordem permanente — não o que saiu
        // dela. As entradas que o firmware acrescenta no POST não declaram
        // alvo, e uma delas à frente do `{bootmgr}` faria da frase anterior
        // uma promessa que ninguém pode mostrar tendo acontecido.
        //
        // Tirar a entrada do ARCA da ordem continua sendo dito: é o que este
        // comando fez, e continua verdade.
        let saida = montar_o_fim(
            &o_que_o_particionamento_deixou(),
            Some("UEFI:Removable Device"),
        );

        assert!(saida.contains("FORA da ordem permanente"), "{saida}");
        assert!(
            !saida.contains("continua subindo o Windows"),
            "a tela prometeu o boot por cima de uma entrada opaca:\n{saida}"
        );
        assert!(saida.contains("`UEFI:Removable Device`"), "{saida}");
        assert!(saida.contains("NAO DIZ para onde aponta"), "{saida}");
        assert!(saida.contains("P-28"), "{saida}");

        // O resto da tela continua inteiro — a troca é de um parágrafo, e não
        // de uma tela.
        assert!(saida.contains("INERTE"), "{saida}");
        assert!(saida.contains("ANTES DO PRIMEIRO BACKUP"), "{saida}");
        assert!(saida.contains("C-10"), "{saida}");
    }

    #[test]
    fn o_fim_nao_promete_um_backup_que_o_arca_recusaria() {
        // **O defeito que uma pergunta pegou, e ele é da própria E10.**
        //
        // A tela terminava com `Primeiro backup: arca backup <nome>` — e esse
        // comando **recusa** num dispositivo recém-preparado. O nome do disco
        // no Linux sai do `blkdev.list` de dentro de uma imagem (§4.5), e um
        // dispositivo que acabou de nascer não tem imagem nenhuma.
        //
        // É o padrão de sempre: peça nova (esta tela, da E10) encaixada em peça
        // antiga (§4.5, decidida na E6 e na E7) que ninguém releu ao encaixar.
        // E fere o critério que o próprio projeto usa — *nenhuma tela afirma o
        // que o repositório não pode mostrar tendo acontecido*.
        // **E a terceira versao desta tela, que a E12 previu e produziu.** A
        // segunda mandava para o menu do Clonezilla — F12, backup manual pelo
        // §6.4 — e nao estava errada sobre os fatos; ela era *exatamente aquilo
        // que este app existe para nao precisar*, cobrado logo na primeira vez
        // que alguem usa um dispositivo novo. O `arca sondar` da E12 resolve o
        // mesmo buraco num reinicio, e a tela passou a mandar isso.
        //
        // O plano de etapas registrou a troca **antes** de ela acontecer, para
        // que a segunda versao nao sobrevivesse ao motivo dela — a primeira
        // quase sobreviveu.
        let saida = montar_o_fim(&o_que_o_particionamento_deixou(), None);

        assert!(
            !saida.contains("Primeiro backup:  arca backup"),
            "a tela voltou a prometer um backup que o ARCA recusaria:\n{saida}"
        );
        assert!(
            saida.contains("arca sondar"),
            "a tela tem de mandar sondar: {saida}"
        );
        assert!(
            saida.contains("§4.5"),
            "a tela tem de dizer por que: {saida}"
        );
        assert!(
            !saida.contains("menu do Clonezilla, faca um backup"),
            "a tela voltou a mandar para o menu do Clonezilla:\n{saida}"
        );
        assert!(
            !saida.contains("§6.4"),
            "o §6.4 e o caminho de quando o Windows nao boota, e nao o do primeiro backup:\n{saida}"
        );
    }

    #[test]
    fn o_fim_explica_a_razao_em_vez_de_so_mandar_fazer() {
        // Um aviso que só diz "rode `arca sondar`" empurra o problema de volta
        // para quem não sabe por que ele existe — e este tem uma razão boa, que
        // é a mesma pela qual o ARCA não pergunta o nome do disco.
        let saida = montar_o_fim(&o_que_o_particionamento_deixou(), None);

        assert!(saida.contains("blkdev.list"), "{saida}");
        assert!(saida.contains("nvme1n1"), "o risco de digitar: {saida}");
        assert!(
            saida.contains("RECUSA"),
            "a tela diz o que acontece se alguem tentar antes: {saida}"
        );
        assert!(
            saida.contains("NAO faz backup nem\n"),
            "e diz o que a sondagem nao faz, que e o que a torna barata: {saida}"
        );
    }

    #[test]
    fn o_fim_nomeia_as_letras_e_diz_que_elas_mudam() {
        // Medido: o `ARCAVAULT` deste projeto ja apareceu em `E:`, em `F:` e em
        // `D:`, e o dispositivo ja foi o disco 1 e o disco 2. A tela diz isso
        // em vez de deixar alguem anotar a letra — e as letras deste fixture
        // sao as do marco em GPT de 25/08/2026, que sairam `D:` e `E:`.
        let saida = montar_o_fim(&o_que_o_particionamento_deixou(), None);

        assert!(saida.contains("em D:"), "{saida}");
        assert!(saida.contains("em E:"), "{saida}");
        assert!(saida.contains("As letras mudam"), "{saida}");
    }

    // ─────────────────── o menu, e o que ele continua não deduzindo ───────────────────

    fn menu_da_mesa() -> String {
        let discos = discos_para_preparar_desta_mesa();
        montar_o_menu(&preparacao::Oferta::de(&discos, Some('C')))
    }

    #[test]
    fn o_menu_numera_so_o_que_da_para_escolher() {
        // A outra metade da doutrina do `arca restore`: um numero ao lado de um
        // item nao escolhivel ocuparia um indice, e ai os numeros da lista
        // passariam a depender de coisas que nao se pode digitar.
        //
        // Nesta mesa ha tres discos e dois candidatos, entao a lista vai ate
        // `[2]` — e nao ate `[3]`.
        let saida = menu_da_mesa();

        assert!(saida.contains("[1]"), "{saida}");
        assert!(saida.contains("[2]"), "{saida}");
        assert!(!saida.contains("[3]"), "numerou um recusado: {saida}");
    }

    #[test]
    fn o_recusado_aparece_dito_e_com_o_motivo() {
        // Omiti-lo faria a lista parecer incompleta para quem esta vendo o
        // disco na mesa — e o pior caso e a defesa 1, que recusa o HD externo
        // que o Windows nao soube classificar. Escondido, o motivo vira
        // ausencia; listado sem numero, ele vira uma frase.
        let saida = menu_da_mesa();

        assert!(saida.contains("KINGSTON SNV3S500G"), "{saida}");
        assert!(saida.contains("Sem numero"), "{saida}");
        assert!(
            saida.contains("disco do sistema"),
            "a lista nao diz por que: {saida}"
        );
    }

    #[test]
    fn o_menu_imprime_os_dois_numeros_de_cada_disco() {
        // `[1]` e o que se digita aqui; `disco 1` e o indice do Windows, que e
        // o que o `--dispositivo` recebe e o que o `Get-Disk` mostra. Eles
        // batem nesta mesa por acidente, e deixar a coincidencia ensinar seria
        // preparar o erro do dia em que ela acabar.
        let saida = menu_da_mesa();

        assert!(saida.contains("[1]  disco 1"), "{saida}");
        assert!(saida.contains("[2]  disco 2"), "{saida}");
        assert!(
            saida.contains("`disco N` e o indice do"),
            "a tela nao explica os dois numeros: {saida}"
        );
    }

    #[test]
    fn o_menu_diz_que_escolher_ainda_nao_apaga() {
        // Quem esta na frente de uma lista de discos e sabe que o comando
        // apaga um deles hesita em digitar qualquer numero. A tela responde a
        // hesitacao com o que e verdade: entre o numero e o `Clear-Disk` ainda
        // ha o plano inteiro, o `(s/N)` e o modelo digitado.
        let saida = menu_da_mesa();

        assert!(saida.contains("nada e apagado"), "{saida}");
    }

    #[test]
    fn o_menu_marca_o_disco_que_ja_e_um_dispositivo_arca() {
        // Preparar por cima de um dispositivo apaga **as imagens dele**. A tela
        // do plano ja diz isso — mas dizer so la e tarde para quem tem dois
        // SSDs iguais na mesa e esta escolhendo qual dos dois e o velho.
        let mut discos = discos_para_preparar_desta_mesa();
        discos[1].particoes = vec![
            crate::portas::particionador::ParticaoExistente {
                numero: 1,
                letra: Some('E'),
                rotulo: Some(ARCAVAULT.to_string()),
                sistema_de_arquivos: Some("NTFS".to_string()),
                tamanho_bytes: 478_000_000_000,
            },
            crate::portas::particionador::ParticaoExistente {
                numero: 2,
                letra: Some('F'),
                rotulo: Some(ARCABOOT.to_string()),
                sistema_de_arquivos: Some("FAT32".to_string()),
                tamanho_bytes: crate::preparacao::ARCABOOT_BYTES,
            },
        ];

        let saida = montar_o_menu(&preparacao::Oferta::de(&discos, Some('C')));

        assert!(saida.contains("JA E UM DISPOSITIVO ARCA"), "{saida}");
        assert!(saida.contains("(E:, F:)"), "as letras dele: {saida}");
    }

    #[test]
    fn um_disco_cru_aparece_no_menu_como_qualquer_outro() {
        // O caso que a lista cobre e um rotulo nao cobriria: um disco RAW, ou
        // um meio-apagado por um `prepare` que morreu no `Clear-Disk`, nao tem
        // nome nenhum para se anunciar. E a lista que o descreve.
        let mut discos = discos_para_preparar_desta_mesa();
        discos[1].particoes = Vec::new();
        discos[1].estilo_de_particao = "RAW".to_string();

        let saida = montar_o_menu(&preparacao::Oferta::de(&discos, Some('C')));

        assert!(saida.contains("[1]  disco 1"), "{saida}");
        assert!(saida.contains("RAW"), "{saida}");
        assert!(saida.contains("sem particao nenhuma"), "{saida}");
    }

    #[test]
    fn sem_dispositivo_o_numero_escolhe_e_o_resto_do_caminho_e_o_mesmo() {
        // O `[1]` desta mesa e o disco 1. Escolhe-lo pelo menu tem de chegar ao
        // mesmo lugar que `--dispositivo 1` chegaria — inclusive passando pelo
        // `(s/N)` e pelo modelo digitado, que sao as duas leituras seguintes.
        let bancada = Bancada::nova(
            "menu",
            ConsoleDeMentira::respondendo(&["1", "s", "JMicron Generic"]),
        );

        let _ = executar(&bancada.contexto(false), None, None);

        assert_eq!(
            bancada.console.lidas.get(),
            3,
            "o menu, a pergunta e a confirmacao: tres leituras"
        );
        assert!(
            bancada.particionador.particionou(),
            "o disco escolhido no menu nao chegou a ser particionado"
        );
    }

    #[test]
    fn o_numero_do_menu_nao_dispensa_a_confirmacao_do_modelo() {
        // **O teste que sustenta o ADR-0024.** Escolher e apontar; confirmar e
        // comprometer-se. Se o numero do menu pulasse S-2, um `1` apagaria um
        // disco — e e exatamente isso que o menu nao pode custar.
        let bancada = Bancada::nova(
            "menu-confirmacao",
            ConsoleDeMentira::respondendo(&["1", "s", "JMicron"]),
        );

        let erro = executar(&bancada.contexto(false), None, None).unwrap_err();

        assert!(matches!(erro, Erro::ConfirmacaoNaoBate { .. }), "{erro}");
        assert!(
            !bancada.particionador.particionou(),
            "o menu apagou com a confirmacao errada"
        );
    }

    #[test]
    fn sem_digitar_nada_no_menu_nada_e_apagado() {
        // O Enter vazio nao escolhe, e nao ha padrao. E o mesmo caminho por
        // onde cai quem chamou o ARCA de um script: um `stdin` fechado devolve
        // linha vazia, e linha vazia nunca escolhe nada — que e por que nao ha
        // deteccao de terminal aqui.
        for resposta in ["", " ", "0", "s", "sim", "9"] {
            let bancada = Bancada::nova("menu-vazio", ConsoleDeMentira::respondendo(&[resposta]));

            let erro = executar(&bancada.contexto(false), None, None).unwrap_err();

            assert!(
                matches!(
                    erro,
                    Erro::PreparacaoRecusada(
                        preparacao::RecusaDaPreparacao::EscolhaInvalida { .. }
                    )
                ),
                "`{resposta}`: {erro}"
            );
            assert!(
                !bancada.particionador.particionou(),
                "`{resposta}` apagou um disco"
            );
            assert_eq!(
                bancada.console.lidas.get(),
                1,
                "`{resposta}` seguiu para a pergunta seguinte"
            );
        }
    }

    #[test]
    fn com_um_candidato_so_o_menu_nao_auto_seleciona() {
        // O §6.1 escreve como principio: *"obrigatorio, mesmo havendo um
        // candidato so"*. Uma lista de um item que se aceita com Enter e o ARCA
        // escolhendo o que apagar, com outro nome.
        let discos = vec![discos_para_preparar_desta_mesa()[1].clone()];

        let mut bancada = Bancada::nova("menu-unico", ConsoleDeMentira::mudo());
        bancada.particionador = ParticionadorDeMentira::com_discos(discos);

        let erro = executar(&bancada.contexto(false), None, None).unwrap_err();

        assert!(matches!(erro, Erro::PreparacaoRecusada(_)), "{erro}");
        assert!(!bancada.particionador.particionou(), "auto-selecionou");
    }

    #[test]
    fn sem_candidato_nenhum_a_recusa_conta_os_recusados() {
        // Uma maquina com o disco do Windows so. "Nenhum disco pode ser
        // preparado" sozinho parece defeito do ARCA para quem esta vendo um
        // disco na mesa; a lista com o motivo, acima, e o que faz a recusa ser
        // resposta.
        let discos = vec![discos_para_preparar_desta_mesa()[0].clone()];

        let mut bancada = Bancada::nova("menu-vazio-total", ConsoleDeMentira::mudo());
        bancada.particionador = ParticionadorDeMentira::com_discos(discos);

        let erro = executar(&bancada.contexto(false), None, None).unwrap_err();

        assert!(
            matches!(
                erro,
                Erro::PreparacaoRecusada(preparacao::RecusaDaPreparacao::NadaAOferecer {
                    recusados: 1
                })
            ),
            "{erro}"
        );
        assert_eq!(
            bancada.console.lidas.get(),
            0,
            "perguntou um numero sem ter o que oferecer"
        );
    }

    #[test]
    fn o_ensaio_pelo_menu_pergunta_o_numero_e_para_ali() {
        // O `--dry-run` sem `--dispositivo` passa pelo menu — nao ha como
        // imprimir o plano de um disco sem saber qual. O que ele **nao** faz e
        // seguir: uma leitura, a do numero, e nenhuma linha tocada em disco
        // nenhum. E o mesmo desenho do `arca restore --dry-run` sem nome.
        let bancada = Bancada::nova("ensaio-menu", ConsoleDeMentira::respondendo(&["1"]));

        executar(&bancada.contexto(true), None, None).expect("o ensaio nao falha");

        assert_eq!(
            bancada.console.lidas.get(),
            1,
            "o ensaio leu alem do numero do menu"
        );
        assert!(
            !bancada.particionador.particionou(),
            "o ensaio pelo menu apagou um disco"
        );
        assert!(
            bancada.sistema.baixados.borrow().is_empty(),
            "o ensaio pelo menu baixou o Clonezilla"
        );
    }

    // ────── as três releituras de C-3 da entrada de firmware ──────
    //
    // As três eram prometidas por um comentário e só duas existiam. E nenhuma
    // das três tinha teste — num caminho cujo modo de falha é um dispositivo
    // que o `prepare` declara pronto e que não boota.

    /// Um `{fwbootmgr}` com a ordem permanente desta mesa e nada mais.
    fn ordem_desta_mesa() -> String {
        format!(
            "\r\nGerenciador de Inicialização de Firmware\r\n\
             ----------------------------------------\r\n\
             identificador           {FWBOOTMGR}\r\n\
             displayorder            {BOOTMGR}\r\n\
             timeout                 1\r\n"
        )
    }

    /// Um bloco de entrada de boot, com os três campos que a releitura de
    /// [`criar_a_entrada`] confere.
    fn entrada_com(descricao: &str, device: &str, path: &str) -> String {
        format!(
            "\r\nGerenciador de Inicialização do Windows\r\n\
             ---------------------------------------\r\n\
             identificador           {GUID_DA_ENTRADA}\r\n\
             device                  {device}\r\n\
             path                    {path}\r\n\
             description             {descricao}\r\n"
        )
    }

    const GUID_DA_ENTRADA: &str = "{f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}";

    /// Roda `criar_a_entrada` com o firmware mostrando `antes` na busca pela
    /// entrada do ARCA, e `depois` na releitura de C-3.
    ///
    /// **A releitura enumera o próprio identificador**, e não `firmware` — é o
    /// `bcdedit /enum {guid}`, e por isso as duas respostas ficam em alvos
    /// diferentes do duplo. Errar isso faz o teste exercitar o `/copy` em vez
    /// da releitura, e é o que a asserção abaixo pega.
    fn criar_a_entrada_com(antes: &str, depois: &str) -> Resultado<EntradaCriada> {
        let com_a_ordem = format!("{}{antes}", ordem_desta_mesa());
        let leitura = firmware::ler(&com_a_ordem);
        assert!(
            leitura.entrada_do_arca().is_some(),
            "a bancada tem de comecar com uma entrada do ARCA achavel: {:#?}",
            leitura.entradas
        );

        let mut bancada = Bancada::nova("c3-firmware", ConsoleDeMentira::mudo());
        bancada.firmware = FirmwareDeMentira::novo()
            .respondendo(FWBOOTMGR, &ordem_desta_mesa())
            .respondendo(FIRMWARE, &com_a_ordem)
            .respondendo(GUID_DA_ENTRADA, depois);

        criar_a_entrada(&bancada.contexto(false), 'R')
    }

    #[test]
    fn a_descricao_que_o_bcdedit_nao_escreveu_e_recusada() {
        // **C-6 medido em 25/08/2026**, num Kingston DataTraveler Max: o
        // `bcdedit /set` responde "A operação foi concluída com êxito", código
        // 0, e mantém o valor antigo. Aqui o `device` e o `path` pegaram e a
        // `description` não — o dispositivo bootaria, mas continuaria se
        // chamando `Clonezilla`, e a tela do fim diria `ARCA`.
        //
        // A descrição é a identidade de uma entrada de firmware neste projeto:
        // `Leitura::entrada_do_arca` procura por ela, e não por um GUID
        // guardado, porque o identificador nomeia o slot da NVRAM (ADR-0025).
        let erro = criar_a_entrada_com(
            &entrada_com(
                firmware::LEGADA,
                "partition=X:",
                r"\EFI\Microsoft\Boot\bootmgfw.efi",
            ),
            &entrada_com(firmware::LEGADA, "partition=R:", CAMINHO_DO_EFI),
        )
        .expect_err("a descricao nao pegou, e isso e recusa");

        assert!(
            matches!(erro, Erro::DescricaoDoFirmwareRecusada { .. }),
            "{erro:?}"
        );
        assert!(erro.to_string().contains(firmware::LEGADA), "{erro}");
        assert!(erro.to_string().contains("C-6"), "{erro}");
    }

    #[test]
    fn o_device_que_o_bcdedit_nao_escreveu_e_recusado() {
        // A defesa que já existia e não tinha teste. É o C-6 na sua forma mais
        // perigosa: a entrada existe, se chama `ARCA`, carrega o `.efi` certo
        // — e aponta para outra partição. O `prepare` diria "pronto" sobre um
        // dispositivo que não boota.
        let erro = criar_a_entrada_com(
            &entrada_com(firmware::ARCA, "partition=X:", CAMINHO_DO_EFI),
            &entrada_com(firmware::ARCA, "partition=X:", CAMINHO_DO_EFI),
        )
        .expect_err("o device nao pegou, e isso e recusa");

        assert!(
            matches!(erro, Erro::AlvoDoFirmwareRecusado { .. }),
            "{erro:?}"
        );
        assert!(erro.to_string().contains("partition=R:"), "{erro}");
    }

    #[test]
    fn o_caminho_do_efi_que_o_bcdedit_nao_escreveu_e_recusado() {
        // A entrada nasce de um `/copy {bootmgr}`, e o `{bootmgr}` carrega o
        // `bootmgfw.efi` do Windows. O `/set path` não pegando deixa a entrada
        // do ARCA apontando para o carregador do Windows **na partição do
        // dispositivo** — onde ele não existe.
        let erro = criar_a_entrada_com(
            &entrada_com(
                firmware::ARCA,
                "partition=X:",
                r"\EFI\Microsoft\Boot\bootmgfw.efi",
            ),
            &entrada_com(
                firmware::ARCA,
                "partition=R:",
                r"\EFI\Microsoft\Boot\bootmgfw.efi",
            ),
        )
        .expect_err("o path nao pegou, e isso e recusa");

        assert!(
            matches!(erro, Erro::CaminhoDoEfiRecusado { .. }),
            "{erro:?}"
        );
        assert!(erro.to_string().contains(CAMINHO_DO_EFI), "{erro}");
    }

    #[test]
    fn as_tres_pegando_a_entrada_legada_e_migrada_e_nao_duplicada() {
        // O caminho feliz das três, e é o de C-4: a entrada legada
        // `Clonezilla` vira `ARCA` **no lugar**, em vez de nascer uma segunda
        // ao lado. Duas entradas seriam duas formas de bootar no Clonezilla,
        // uma delas sem ninguém olhando (ADR-0017).
        let criada = criar_a_entrada_com(
            &entrada_com(
                firmware::LEGADA,
                "partition=X:",
                r"\EFI\Microsoft\Boot\bootmgfw.efi",
            ),
            &entrada_com(firmware::ARCA, "partition=R:", CAMINHO_DO_EFI),
        )
        .expect("as tres pegaram");

        assert!(criada.ja_existia, "a legada foi reusada, e nao duplicada");
        assert_eq!(criada.identificador, GUID_DA_ENTRADA);
    }

    #[test]
    fn com_dispositivo_na_linha_o_menu_nao_aparece() {
        // O atalho de quem ja sabe o indice continua sendo um atalho: duas
        // leituras, e nao tres. Um menu que aparecesse mesmo com
        // `--dispositivo` transformaria o caminho de script num caminho
        // interativo, e o `arca prepare --dispositivo 1 --dry-run` deixaria de
        // rodar sem console.
        let bancada = Bancada::nova(
            "sem-menu",
            ConsoleDeMentira::respondendo(&["s", "JMicron Generic"]),
        );

        let _ = executar(&bancada.contexto(false), Some(1), None);

        assert_eq!(bancada.console.lidas.get(), 2, "o menu foi perguntado");
    }
}
