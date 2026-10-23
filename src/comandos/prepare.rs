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
//! | pré-voo | Obter e conferir o pacote: o do `--iso`, ou o pacote guardado neste PC — reusado ou baixado (PR-1, PR-2, PR-7) | nada tocado — e o download pela metade fica no `.parcial`, para a próxima vez |
//! | 0 | Listar os discos e perguntar o número — só sem `--dispositivo` (ADR-0024) | nada tocado |
//! | 1 | Descrever o disco e julgar (PR-5) | nada tocado |
//! | 1b | **Ler o firmware** — a ordem permanente e a entrada a reusar (PR-6, C-15) | nada tocado |
//! | 2 | Imprimir o plano (PR-4) | nada tocado |
//! | 3 | Perguntar, e **reler o disco** (PR-4, 3º tempo) | nada tocado |
//! | 4 | Confirmação digitada (S-2) | nada tocado |
//! | 5 | **Particionar e formatar** | disco apagado, duas partições vazias — e a entrada de firmware, se apontava para ele, **pendurada** |
//! | 6 | O pacote já é local: o do `--iso` ou o guardado (PR-7) | dispositivo vazio, sem Clonezilla |
//! | 7 | Conferir o SHA256 (PR-1) | idem — e **nada foi extraído** |
//! | 8 | Extrair | `ARCABOOT` com o Clonezilla e `set default="0"` |
//! | 9 | Devolver o `grub.cfg` ao estado inerte | dispositivo bootável e inerte |
//! | 10 | Instalar o ARCA e a cópia do pacote (PR-3) | dispositivo completo, sem entrada de firmware |
//! | 11 | Reapontar a entrada (ou criá-la) e tirá-la da ordem — o `/set device` primeiro | pronto |
//!
//! **Nenhum desses estados é pior do que o anterior, e todos são reversíveis
//! rodando o comando de novo** — ele começa apagando. O que não volta é o que
//! estava no disco antes do passo 5, e é para isso que existem os passos 1 a 4.
//!
//! Do passo 8 em diante o dispositivo **já boota**: um `prepare` interrompido
//! ali deixa um Clonezilla utilizável pelo menu (§6.4), sem o ARCA dentro.
//!
//! ## O passo 1b existe por causa de 27/08/2026
//!
//! O passo 5, apagando um dispositivo ARCA que já existia, deixa a entrada de
//! firmware apontando para uma partição que não existe mais — e o `bcdedit`,
//! nesse estado, **listava tudo e saía com código 1** em todo `/enum`. O passo
//! 11 começava lendo, e morria antes do `/set device` que o consertaria; o
//! `arca sondar` morria na mesma leitura. Ver `ler_o_firmware_antes`,
//! `criar_a_entrada` e o ADR-0026.
//!
//! # O pré-voo do pacote, e por que ele não é um passo
//!
//! O `--iso` é a única entrada deste comando que dá para julgar inteira sem
//! tocar em disco nenhum: o arquivo está lá, o SHA256 é o do pacote fixado, e
//! dentro dele estão os quatro caminhos que fazem um dispositivo bootar. As
//! três perguntas custam segundos e não escrevem nada.
//!
//! Medido em 27/08/2026: elas eram feitas **no passo 7**, e a única diferença
//! é que ali o disco já não existe mais. Um caminho digitado errado, um
//! download de outra versão, um zip truncado — os três custavam o disco antes
//! de serem ditos, e a tentativa seguinte custava o disco de novo, porque a
//! mensagem chegava depois de o comando ter feito a parte irreversível do
//! trabalho.
//!
//! Por isso o pré-voo não recebeu número, nem letra como o passo 1b: o 1b é um
//! passo do `prepare`, que lê o firmware depois de o disco ter sido escolhido;
//! o pré-voo é a recusa que acontece **antes** de o comando começar — o mesmo
//! lugar de [`crate::prevoo`] nos outros comandos.
//!
//! O passo 7 continua conferindo, e conferindo a mesma coisa. Não é
//! desperdício: entre o pré-voo e ele há uma pessoa lendo o plano e digitando
//! o modelo do disco, e é exatamente o intervalo em que o terceiro tempo de
//! PR-4 relê o disco. O que vale para o disco vale para o arquivo.
//!
//! ## E desde 04/10/2026 o pré-voo vale sem `--iso` também (PR-7)
//!
//! Sem `--iso`, o pacote era baixado no passo 6, com o disco já apagado, e o
//! download não tinha limite para uma transferência parada. Em 03/10 e
//! 04/10/2026 ele ficou parado calado até o usuário cancelar, e cada
//! tentativa custou o disco antes de falhar.
//!
//! Agora o pré-voo obtém o pacote antes do passo 0 — do pacote guardado no
//! PC, conferido de novo a cada uso, ou baixando-o das [`pacote::FONTES`] —,
//! e dali em diante o comando segue o caminho que o `--iso` já seguia. Nenhum
//! download acontece depois do ponto sem volta. Ver o
//! [ADR-0028](../../docs/adr/0028-nenhum-download-depois-do-ponto-sem-volta.md).
//!
//! ## E desde 23/10/2026 a fonte lenta perde a vez, e o `--iso` fica guardado
//!
//! O download não terminava em todo PC: uma fonte a 64 KB/s ainda manda
//! bytes, e o limite de 60 s nunca a largava. A saída era baixar pelo
//! navegador e passar o `--iso`, e ela não ficava — o preparo seguinte baixava
//! de novo. Agora uma fonte lenta perde a vez para a seguinte, e se todas
//! forem lentas a mais rápida medida volta e vai até o fim; a tela mostra a
//! barra, a velocidade e quanto falta; e o `--iso` que passa no SHA256 vira o
//! pacote guardado deste PC (WPC-220).

use crate::andamento::{self, Medicao};
use crate::app::Contexto;
use crate::confirmacao;
use crate::dispositivo::{self, ARCABOOT, ARCAVAULT};
use crate::erro::{Erro, Resultado};
use crate::firmware::{self, Alvo};
use crate::formato::{linha, tamanho};
use crate::grub;
use crate::pacote;
use crate::portas::OQueHa;
use crate::portas::particionador::{DiscoParaPreparar, ParticoesFeitas};
use crate::preparacao::{self, Preparacao};
use std::ops::ControlFlow;
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
    // ─────────── pré-voo: o pacote, com o disco ainda intacto (PR-2, PR-7) ───────────

    // Antes do passo 0, e não junto do passo 6: ver a seção sobre o pré-voo no
    // cabeçalho deste módulo para o que essa ordem custou.
    let pacote = obter_o_pacote_no_pre_voo(contexto, iso, &mut escrever_na_tela)?;

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

    // ─────────── 1b. o firmware, lido ANTES do ponto sem volta (C-15, PR-6) ───────────

    // É daqui que saem a ordem permanente que C-5 protege e a entrada que o
    // passo 11 vai reusar — lidas com o firmware ainda coerente. Ver
    // [`ler_o_firmware_antes`] para o que custou descobrir isto.
    let firmware_antes = ler_o_firmware_antes(contexto)?;

    // ─────────── 2. o plano inteiro na tela (PR-4, 1º tempo) ───────────

    print!("{}", montar_o_plano(&preparacao, &pacote, &firmware_antes));

    if contexto.dry_run {
        print!("{}", montar_o_ensaio(&preparacao));
        return Ok(());
    }

    // O pré-voo só volta sem pacote no ensaio, que acabou de parar acima.
    let origem = pacote
        .local()
        .expect("fora do ensaio, o pre-voo so volta com o pacote local")
        .to_path_buf();

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

    let pacote_local = obter_o_pacote(contexto, &origem, &raiz_do_vault)?;

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

    let entrada = criar_a_entrada(contexto, feitas.boot.letra, &firmware_antes)?;
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
///
/// A marca sozinha não resolvia isso: os dois SSDs iguais saíam na mesma
/// linha. Desde C-16 (27/09/2026), quando o `ARCABOOT` tem nome, ele vai
/// entre parênteses no fim — o mesmo rótulo que o Explorer mostra.
fn descrever_o_disco(disco: &DiscoParaPreparar) -> String {
    let marca = if !e_um_dispositivo_arca(disco) {
        String::new()
    } else {
        let nome = disco
            .particoes
            .iter()
            .filter_map(|particao| particao.rotulo.as_deref())
            .find_map(dispositivo::nome_do_boot);
        match nome {
            Some(nome) => format!(" · JA E UM DISPOSITIVO ARCA ({nome})"),
            None => " · JA E UM DISPOSITIVO ARCA".to_string(),
        }
    };

    format!(
        "{} · {} · {} · {}{marca}",
        tamanho(disco.tamanho_bytes),
        disco.barramento,
        disco.estilo_de_particao,
        resumir_o_conteudo(disco),
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

/// O pré-voo de PR-2: o pacote do `--iso` julgado inteiro com o disco intacto.
///
/// # Por que as perguntas "o arquivo está lá?" e "é uma pasta?" moram aqui, e
/// não em [`conferir_o_pacote`]
///
/// Porque [`pacote::RecusaDoPacote::NaoEstaLa`],
/// [`pacote::RecusaDoPacote::EUmaPasta`] e
/// [`pacote::RecusaDoPacote::NaoDeuParaOlhar`] prometem que *nada foi
/// apagado*, e só esta função pode fazer essa promessa — ela roda antes do
/// passo 0. A conferência do passo 7 chama a mesma [`conferir_o_pacote`] com
/// o disco já particionado, e levantar as mesmas recusas de lá faria o ARCA
/// prometer o que não é verdade.
///
/// Um arquivo que existe no pré-voo e some — ou vira pasta — antes do passo 7
/// continua caindo onde caía: no `certutil`, que responde
/// `ERROR_FILE_NOT_FOUND`. É o desfecho certo para esse caso — ele **é** uma
/// surpresa, e a mensagem crua diz isso melhor do que uma frase escrita para
/// quem digitou um caminho errado.
fn conferir_o_pacote_local(
    contexto: &Contexto,
    caminho: &Path,
) -> Resultado<crate::resumo::Resumo> {
    match o_que_ha_no_pre_voo(contexto, caminho)? {
        OQueHa::Arquivo => conferir_o_pacote(contexto, caminho),
        OQueHa::Nada => Err(Erro::PacoteRecusado(pacote::RecusaDoPacote::NaoEstaLa {
            caminho: caminho.to_path_buf(),
        })),

        // Uma pasta passava por um "há alguma coisa aqui", e o `certutil`
        // responde para ela o mesmo `0x80070002` de um arquivo ausente
        // (WPC-64, 28/09/2026). O pacote de dentro dela só é apontado: ver
        // `EUmaPasta` para por que não é usado.
        OQueHa::Pasta => {
            let dentro = caminho.join(pacote::ARQUIVO);
            let o_pacote_esta_dentro = o_que_ha_no_pre_voo(contexto, &dentro)? == OQueHa::Arquivo;
            Err(Erro::PacoteRecusado(pacote::RecusaDoPacote::EUmaPasta {
                caminho: caminho.to_path_buf(),
                dentro: o_pacote_esta_dentro.then_some(dentro),
            }))
        }
    }
}

/// A pergunta do pré-voo sobre um caminho, com o "não sei" virando a recusa
/// que diz isso.
///
/// Até 29/09/2026 (WPC-68) o "não sei" virava `NaoEstaLa`, e, dentro de uma
/// pasta, `EUmaPasta` sem `dentro`, que diz que o pacote não está nela.
fn o_que_ha_no_pre_voo(contexto: &Contexto, caminho: &Path) -> Resultado<OQueHa> {
    contexto.arquivos.o_que_ha(caminho).map_err(|erro| {
        Erro::PacoteRecusado(pacote::RecusaDoPacote::NaoDeuParaOlhar {
            caminho: caminho.to_path_buf(),
            motivo: erro.motivo(),
        })
    })
}

/// As duas perguntas de PR-1 sobre um arquivo que está lá: o SHA256 bate, e
/// dentro dele está o que faz um dispositivo bootar.
///
/// Não imprime nada. Quem chama diz de onde veio o pacote e em que momento — e
/// os dois momentos em que ela roda dizem coisas diferentes na tela.
fn conferir_o_pacote(contexto: &Contexto, local: &Path) -> Resultado<crate::resumo::Resumo> {
    let medido = crate::resumo::do_certutil(
        &contexto
            .sistema
            .resumir(local, crate::resumo::Algoritmo::Sha256)?,
        crate::resumo::Algoritmo::Sha256,
    );
    let resumo = pacote::conferir_o_resumo(medido).map_err(Erro::PacoteRecusado)?;

    // O conteúdo, e não só o resumo: o `bsdtar` sair com zero não é prova de
    // que o pacote tem o que faz um dispositivo bootar.
    let listagem = contexto.sistema.listar_pacote(local)?;
    let faltando = pacote::o_que_falta(listagem.texto.lines().map(str::trim));
    if !faltando.is_empty() {
        return Err(Erro::PacoteRecusado(
            pacote::RecusaDoPacote::PacoteIncompleto { faltando },
        ));
    }

    Ok(resumo)
}

/// De onde vem o pacote, decidido no pré-voo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PacoteDoPreVoo {
    /// O arquivo do `--iso`, conferido (PR-2).
    DoIso(PathBuf),

    /// O pacote guardado neste PC, conferido — reusado ou recém-baixado
    /// (PR-7).
    Guardado(PathBuf),

    /// Nada guardado, e o ensaio não baixa. Só o `--dry-run` chega aqui.
    PorBaixar,
}

impl PacoteDoPreVoo {
    /// O arquivo que os passos 6 e 7 usam, quando já há um.
    pub fn local(&self) -> Option<&Path> {
        match self {
            PacoteDoPreVoo::DoIso(caminho) | PacoteDoPreVoo::Guardado(caminho) => Some(caminho),
            PacoteDoPreVoo::PorBaixar => None,
        }
    }
}

/// A tela do `arca prepare`, para quem não é teste: escreve e descarrega na
/// hora, porque a linha do andamento é reescrita no lugar e não termina em
/// quebra de linha.
fn escrever_na_tela(texto: &str) {
    use std::io::Write;

    print!("{texto}");
    let _ = std::io::stdout().flush();
}

/// O pré-voo do pacote: o do `--iso`, ou o pacote guardado neste PC — e o
/// guardado ou se reusa, ou se baixa (PR-1, PR-2, PR-7).
///
/// Escreve na `tela` em vez de imprimir, para que o texto que alguém lê
/// durante um download de minutos seja o que os testes leem.
///
/// # Por que o guardado é conferido a cada uso
///
/// Guardado não é confiado. Um arquivo que ficou meses no `%LOCALAPPDATA%`
/// pode ter sido trocado, truncado por um disco cheio ou restaurado de outro
/// lugar, e o SHA256 compilado custa segundos. É a mesma razão pela qual o
/// `--iso` é conferido de novo no passo 7.
fn obter_o_pacote_no_pre_voo(
    contexto: &Contexto,
    iso: Option<&Path>,
    tela: &mut dyn FnMut(&str),
) -> Resultado<PacoteDoPreVoo> {
    if let Some(caminho) = iso {
        // Até 23/10/2026 o `--iso` não lia nem escrevia o pacote guardado, e
        // cada preparo sem `--iso` daquele PC baixava de novo — e falhava de
        // novo, onde o download falhava. Agora o que passa aqui vira o pacote
        // guardado, e a cópia manual acontece uma vez por PC (WPC-220). O
        // preparo continua saindo do arquivo do `--iso`, e uma cópia que não
        // se faz é aviso, e não recusa: este preparo nunca dependeu dela.
        let resumo = conferir_o_pacote_local(contexto, caminho)?;
        tela(&linha(
            "Pacote conferido",
            &format!(
                "ok · {} · {} · nada tocado ainda (PR-2)",
                resumo.abreviado(),
                caminho.display()
            ),
        ));
        guardar_o_iso(contexto, caminho, tela);
        return Ok(PacoteDoPreVoo::DoIso(caminho.to_path_buf()));
    }

    let pasta = contexto.registro.pasta();
    let guardado = pacote::guardado_em(pasta);

    let o_guardado_nao_bateu = match contexto.arquivos.o_que_ha(&guardado)? {
        OQueHa::Nada => false,
        OQueHa::Arquivo => match conferir_o_pacote(contexto, &guardado) {
            Ok(resumo) => {
                contexto.registro.info(format!(
                    "pacote guardado usado sem rede · {}",
                    guardado.display()
                ));
                tela(&montar_o_pacote_guardado(&resumo, &guardado));
                return Ok(PacoteDoPreVoo::Guardado(guardado));
            }
            // Só a divergência vira download: "não consegui medir" continua
            // sendo a recusa dele, e não um motivo para baixar meio giga.
            Err(Erro::PacoteRecusado(pacote::RecusaDoPacote::ResumoDivergente { .. })) => true,
            Err(outro) => return Err(outro),
        },
        // Uma pasta com o nome do pacote faria o `renomear` falhar depois de
        // um download inteiro. Melhor dizer agora.
        OQueHa::Pasta => {
            return Err(Erro::PacoteRecusado(
                pacote::RecusaDoPacote::NaoDeuParaGuardar {
                    pasta: pasta.to_path_buf(),
                    motivo: format!("`{}` e uma pasta", pacote::ARQUIVO),
                },
            ));
        }
    };

    // O ensaio não baixa: ele diz o que o preparo de verdade faria. Nada é
    // escrito na pasta, nem o `.parcial`.
    if contexto.dry_run {
        tela(&if o_guardado_nao_bateu {
            linha(
                "Pacote guardado",
                "nao bate com o SHA256 deste ARCA · o preparo de verdade baixa de novo, do zero; o ensaio nao baixa",
            )
        } else {
            linha(
                "Pacote",
                &format!(
                    "nao esta neste PC · o preparo de verdade baixa {} antes de qualquer pergunta; o ensaio nao baixa",
                    tamanho(pacote::TAMANHO_BYTES)
                ),
            )
        });
        return Ok(PacoteDoPreVoo::PorBaixar);
    }

    if o_guardado_nao_bateu {
        tela(&linha(
            "Pacote guardado",
            "nao bate com o SHA256 deste ARCA · baixando de novo, do zero",
        ));
    }

    let resumo = baixar_o_pacote_guardado(contexto, pasta, o_guardado_nao_bateu, tela)?;
    tela(&montar_o_pacote_guardado(&resumo, &guardado));
    Ok(PacoteDoPreVoo::Guardado(guardado))
}

/// Baixa o pacote para o `.parcial`, confere e o renomeia para o pacote
/// guardado (PR-7).
///
/// # A ordem das recusas, e por que todas vêm antes do `curl`
///
/// A pasta tem de se deixar criar, o `.parcial` tem de se deixar criar ou
/// zerar, e o volume tem de ter o que falta baixar. As três perguntas custam
/// milissegundos, e cada uma, respondida depois, custaria um download de
/// minutos que não teria onde ficar.
///
/// # Nenhuma fonte é confiável, e por isso elas são intercambiáveis
///
/// Toda troca de fonte continua do byte em que o `.parcial` está: ele é um
/// só. Quem decide se o que veio é o pacote é o SHA256 compilado, depois da
/// fonte que entregou.
///
/// # As duas voltas, e o que as fez nascer
///
/// Até 23/10/2026 cada fonte ia uma vez, e só saía a que ficava 60 s sem
/// mandar um byte. Uma fonte a 64 KB/s manda: naquele dia o navegador baixava
/// este pacote a 63,9 KB/s numa conexão de 800 Mb, com duas horas pela
/// frente, e o mirror do projeto, medido desta mesa no mesmo dia, o entregava
/// a 5,1 MB/s. O ARCA ficaria mais de duas horas na fonte lenta, com a
/// rápida esperando.
///
/// Agora, na primeira volta, cada fonte vai uma vez, na ordem de
/// [`pacote::FONTES`], e a lenta perde a vez
/// ([`crate::andamento::Medicao::lentidao`]). Na segunda voltam só as que
/// saíram por lentas, da mais rápida medida para a mais lenta, e nenhuma é
/// julgada lenta: a primeira vai até o fim, e só a parada, a falta de conexão
/// ou um erro a tiram. Nada é recusado por ser lento. Ver o ADR-0029.
fn baixar_o_pacote_guardado(
    contexto: &Contexto,
    pasta: &Path,
    do_zero: bool,
    tela: &mut dyn FnMut(&str),
) -> Resultado<crate::resumo::Resumo> {
    let parcial = pacote::parcial_em(pasta);

    contexto
        .arquivos
        .criar_diretorio(pasta)
        .map_err(|erro| nao_deu_para_guardar(pasta, erro))?;

    let mut ja_veio = match (
        do_zero,
        pacote::como_comecar(tamanho_do_parcial(contexto, pasta, &parcial)?),
    ) {
        (false, pacote::Comeco::Retomando(bytes)) => bytes,
        _ => {
            zerar_o_parcial(contexto, pasta)?;
            0
        }
    };

    let falta = pacote::TAMANHO_BYTES.saturating_sub(ja_veio);
    let livre = contexto.arquivos.espaco_livre(pasta)?;
    if livre < falta {
        return Err(Erro::PacoteRecusado(pacote::RecusaDoPacote::FaltaEspaco {
            pasta: pasta.to_path_buf(),
            falta_bytes: falta,
            livre_bytes: livre,
        }));
    }

    // O que cada fonte respondeu por último, na ordem de `FONTES`: é o que a
    // recusa lista, uma linha por fonte, se nenhuma entregar.
    let mut ultimas: Vec<Option<pacote::Resposta>> = vec![None; pacote::FONTES.len()];
    let mut lentas: Vec<(usize, u64)> = Vec::new();

    for (indice, fonte) in pacote::FONTES.iter().enumerate() {
        match tentar_a_fonte(contexto, pasta, fonte, Volta::Primeira, &mut ja_veio, tela)? {
            Tentativa::Entregou(resumo) => return Ok(resumo),
            Tentativa::Abandonada(resposta) => {
                if let pacote::Resposta::Lenta(lentidao) = &resposta {
                    lentas.push((indice, lentidao.velocidade()));
                }
                ultimas[indice] = Some(resposta);
            }
        }
    }

    // Da mais rápida medida para a mais lenta. O `sort_by_key` é estável: as
    // empatadas ficam na ordem de `FONTES`.
    lentas.sort_by_key(|&(_, velocidade)| std::cmp::Reverse(velocidade));
    for (indice, _) in lentas {
        let fonte = &pacote::FONTES[indice];
        match tentar_a_fonte(contexto, pasta, fonte, Volta::Segunda, &mut ja_veio, tela)? {
            Tentativa::Entregou(resumo) => return Ok(resumo),
            Tentativa::Abandonada(resposta) => ultimas[indice] = Some(resposta),
        }
    }

    let tentativas = pacote::FONTES
        .iter()
        .zip(ultimas)
        .map(|(fonte, resposta)| {
            (
                fonte.rotulo,
                resposta.expect("toda fonte vai ao menos na primeira volta"),
            )
        })
        .collect();
    Err(Erro::PacoteRecusado(
        pacote::RecusaDoPacote::NenhumaFonteEntregou { tentativas },
    ))
}

/// Em qual volta uma fonte está.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Volta {
    /// Cada fonte, uma vez, em ordem, e a lenta perde a vez.
    Primeira,

    /// As que saíram por lentas, sem julgar lentidão.
    Segunda,
}

/// Como uma fonte saiu.
enum Tentativa {
    /// O pacote veio, passou e já é o pacote guardado.
    Entregou(crate::resumo::Resumo),

    /// A fonte saiu sem entregar, e a próxima continua do mesmo `.parcial`.
    Abandonada(pacote::Resposta),
}

/// Uma fonte, numa volta: a linha `Fonte`, a linha viva enquanto ela corre, e
/// o que ela respondeu.
///
/// A versão, o rótulo e a retomada ficam na linha `Fonte`, e não na que se
/// reescreve: com eles dentro, como a linha foi mostrada ao usuário em
/// 23/10/2026, ela passaria de 120 colunas, e uma linha que quebra não se
/// reescreve no lugar (Unresolved 2 da WPC-220).
fn tentar_a_fonte(
    contexto: &Contexto,
    pasta: &Path,
    fonte: &pacote::Fonte,
    volta: Volta,
    ja_veio: &mut u64,
    tela: &mut dyn FnMut(&str),
) -> Resultado<Tentativa> {
    let parcial = pacote::parcial_em(pasta);

    // Uma fonte que deixou o `.parcial` do tamanho do pacote ou maior e não
    // entregou deixou um arquivo que não se retoma: retomar pediria um byte
    // que o pacote não tem.
    if pacote::como_comecar(Some(*ja_veio)) == pacote::Comeco::DoZero {
        zerar_o_parcial(contexto, pasta)?;
        *ja_veio = 0;
    }

    let antes = *ja_veio;
    let mut valor = format!("{} · {}", pacote::VERSAO, fonte.rotulo);
    if volta == Volta::Segunda {
        valor.push_str(" · segunda volta");
    }
    if antes > 0 {
        valor.push_str(&format!(" · retomando de {}", tamanho(antes)));
    }
    tela(&linha("Fonte", &valor));

    let mut medicao = Medicao::comecando_com(antes);
    let mut linha_viva = LinhaViva::default();
    linha_viva.escrever(tela, &medicao.linha_viva());

    // Quem julga a fonte lenta é código puro, sobre as amostras do
    // adaptador; o adaptador só encerra o `curl` (ADR-0029). A tela e o
    // julgamento saem da mesma `medicao`, e por isso a linha que diz que a
    // fonte saiu traz os números que a linha viva mostrava.
    let mut lentidao = None;
    let transferencia =
        contexto
            .sistema
            .baixar(fonte.url, &parcial, pacote::LIMITES, &mut |amostra| {
                // Decidido, nada muda: uma amostra depois da decisão mudaria
                // os números que a linha `Fonte abandonada` vai dizer.
                if lentidao.is_some() {
                    return ControlFlow::Break(());
                }
                medicao.anotar(amostra);
                linha_viva.escrever(tela, &medicao.linha_viva());
                if volta == Volta::Primeira {
                    lentidao = medicao.lentidao();
                }
                match lentidao {
                    Some(_) => ControlFlow::Break(()),
                    None => ControlFlow::Continue(()),
                }
            });
    let transferencia = match transferencia {
        Ok(transferencia) => transferencia,
        Err(erro) => {
            tela("\n");
            return Err(erro);
        }
    };

    let resposta = match lentidao {
        Some(lentidao) if transferencia.encerrada => pacote::Resposta::Lenta(lentidao),
        _ => pacote::o_que_a_fonte_respondeu(&transferencia),
    };
    if resposta == pacote::Resposta::Entregou {
        linha_viva.escrever(
            tela,
            &medicao.linha_de_entrega(transferencia.tamanho_no_destino),
        );
    }
    tela("\n");

    *ja_veio = transferencia.tamanho_no_destino;
    contexto.registro.info(format!(
        "fonte {} · {resposta} · {} bytes desta fonte",
        fonte.url,
        ja_veio.saturating_sub(antes)
    ));

    if resposta == pacote::Resposta::Entregou {
        // PR-1 antes do nome: o `.parcial` só vira o pacote guardado depois
        // de passar.
        let resumo = match conferir_o_pacote(contexto, &parcial) {
            Ok(resumo) => resumo,

            // O que não bate é zerado, e a próxima execução baixa do zero.
            // Deixá-lo prenderia o ARCA nele: uma página de login de Wi-Fi
            // entregue no lugar do zip é menor que o pacote, e seria retomada
            // para sempre. Decidido pelo usuário em 04/10/2026 (WPC-134).
            // "Não consegui medir" não cai aqui, e não zera nada.
            Err(
                divergente @ Erro::PacoteRecusado(pacote::RecusaDoPacote::ResumoDivergente {
                    ..
                }),
            ) => {
                if let Err(nao_zerou) = zerar_o_parcial(contexto, pasta) {
                    contexto.registro.aviso(format!(
                        "o .parcial que nao bate nao foi zerado: {nao_zerou}"
                    ));
                }
                return Err(divergente);
            }
            Err(outro) => return Err(outro),
        };
        contexto
            .arquivos
            .renomear(&parcial, &pacote::guardado_em(pasta))?;
        return Ok(Tentativa::Entregou(resumo));
    }

    // Uma escrita que falhou não é resposta da fonte: as outras cairiam no
    // mesmo `.parcial` (decidido pelo usuário em 04/10/2026).
    if let pacote::Resposta::NaoConseguiuEscrever(frase) = resposta {
        return Err(Erro::PacoteRecusado(
            pacote::RecusaDoPacote::NaoDeuParaGuardar {
                pasta: pasta.to_path_buf(),
                motivo: frase,
            },
        ));
    }

    tela(&linha(
        "Fonte abandonada",
        &format!("{} · {resposta}", fonte.rotulo),
    ));
    Ok(Tentativa::Abandonada(resposta))
}

/// A recusa de uma pasta que não aceita escrita, com o que o Windows
/// respondeu.
fn nao_deu_para_guardar(pasta: &Path, erro: Erro) -> Erro {
    Erro::PacoteRecusado(pacote::RecusaDoPacote::NaoDeuParaGuardar {
        pasta: pasta.to_path_buf(),
        motivo: erro.motivo(),
    })
}

/// Zera o `.parcial`, escrevendo-o vazio por cima.
///
/// É a única substituição que o download faz, e é também a prova de que a
/// pasta aceita escrita.
fn zerar_o_parcial(contexto: &Contexto, pasta: &Path) -> Resultado<()> {
    contexto
        .arquivos
        .escrever_atomico(&pacote::parcial_em(pasta), "")
        .map_err(|erro| nao_deu_para_guardar(pasta, erro))
}

/// O tamanho do `.parcial`, ou `None` quando ele não está lá.
fn tamanho_do_parcial(contexto: &Contexto, pasta: &Path, parcial: &Path) -> Resultado<Option<u64>> {
    Ok(contexto
        .arquivos
        .listar(pasta)?
        .into_iter()
        .find(|entrada| entrada.caminho == parcial && !entrada.diretorio)
        .map(|entrada| entrada.tamanho_bytes))
}

/// A linha `Baixando Clonezilla`, que o andamento reescreve no lugar.
///
/// Reescrever é voltar ao começo da linha com `\r` e escrever por cima — e o
/// que sobrar da escrita anterior, se ela foi mais comprida, é coberto com
/// espaços. Por isso a linha cabe em 120 colunas: uma linha que quebra não se
/// reescreve, ela se repete.
#[derive(Default)]
struct LinhaViva {
    largura: usize,
}

impl LinhaViva {
    fn escrever(&mut self, tela: &mut dyn FnMut(&str), valor: &str) {
        let texto = andamento::com_o_rotulo(valor);
        let largura = texto.chars().count();

        // A primeira escrita começa a linha; as seguintes voltam ao começo dela.
        let volta = if self.largura == 0 { "" } else { "\r" };
        let sobra = self.largura.saturating_sub(largura);
        self.largura = self.largura.max(largura);

        tela(&format!("{volta}{texto}{}", " ".repeat(sobra)));
    }
}

/// O `--iso` que passou vira o pacote guardado deste PC, por cópia (PR-2,
/// WPC-220).
///
/// # Por que cópia, pelo `.parcial`, e nunca recusa
///
/// O arquivo do `--iso` é de quem chamou, e fica onde está. A cópia passa
/// pelo `.parcial`, é conferida e só então renomeada: o pacote guardado
/// continua só existindo sob o nome final depois de passar no SHA256. A cópia
/// de PR-3 vai direto para o nome final no `ARCAVAULT`, e esse precedente não
/// serve aqui — ali o disco acabou de ser particionado, e aqui mora um
/// pacote que o próximo preparo usa sem rede.
///
/// Nada aqui recusa o preparo: ele sai do arquivo do `--iso` de qualquer
/// jeito, e uma cópia que não se faz vira uma linha de aviso na tela e no
/// `arca.log`. Um guardado que não se deixou medir não é tocado — não ter
/// conseguido medir não é o mesmo que não bater.
fn guardar_o_iso(contexto: &Contexto, iso: &Path, tela: &mut dyn FnMut(&str)) {
    let pasta = contexto.registro.pasta();
    let guardado = pacote::guardado_em(pasta);
    let parcial = pacote::parcial_em(pasta);

    let o_guardado_nao_batia = match contexto.arquivos.o_que_ha(&guardado) {
        Err(erro) => return nao_deu_para_medir(tela, &erro),
        Ok(OQueHa::Pasta) => {
            return nao_guardou(contexto, tela, format!("`{}` e uma pasta", pacote::ARQUIVO));
        }
        Ok(OQueHa::Nada) => false,
        Ok(OQueHa::Arquivo) => match conferir_o_pacote(contexto, &guardado) {
            // O guardado que bate não é reescrito, e a tela é a de antes.
            Ok(_) => return,
            Err(Erro::PacoteRecusado(pacote::RecusaDoPacote::ResumoDivergente { .. })) => true,
            Err(erro) => return nao_deu_para_medir(tela, &erro),
        },
    };

    // O ensaio diz o que o preparo de verdade faria, e não escreve nada.
    if contexto.dry_run {
        tela(&linha(
            "Pacote guardado",
            "o preparo de verdade copia o --iso para este PC; o ensaio nao escreve",
        ));
        return;
    }

    if let Err(erro) = contexto.arquivos.criar_diretorio(pasta) {
        return nao_guardou(contexto, tela, erro.motivo());
    }
    match contexto.arquivos.espaco_livre(pasta) {
        Err(erro) => return nao_guardou(contexto, tela, erro.motivo()),
        Ok(livre) if livre < pacote::TAMANHO_BYTES => {
            return nao_guardou(
                contexto,
                tela,
                format!(
                    "precisa de {} MB livres, e ha {} MB",
                    pacote::TAMANHO_BYTES.div_ceil(pacote::MEGABYTE),
                    livre / pacote::MEGABYTE
                ),
            );
        }
        Ok(_) => {}
    }
    if let Err(erro) = contexto.arquivos.copiar(iso, &parcial) {
        return nao_guardou(contexto, tela, erro.motivo());
    }

    match conferir_o_pacote(contexto, &parcial) {
        Ok(_) => {}
        // A cópia que não bate é zerada, como o download que não bate: um
        // `.parcial` que a próxima execução retomasse seria o pacote errado
        // pela metade.
        Err(Erro::PacoteRecusado(pacote::RecusaDoPacote::ResumoDivergente { .. })) => {
            if let Err(nao_zerou) = zerar_o_parcial(contexto, pasta) {
                contexto.registro.aviso(format!(
                    "o .parcial copiado do --iso que nao bate nao foi zerado: {nao_zerou}"
                ));
            }
            contexto
                .registro
                .aviso("a copia do --iso nao bateu com o SHA256 · este preparo segue do --iso");
            tela(&linha(
                "Pacote guardado",
                "a copia nao bateu com o SHA256 · este preparo segue do --iso",
            ));
            return;
        }
        Err(erro) => return nao_guardou(contexto, tela, motivo_da_medida(&erro)),
    }
    if let Err(erro) = contexto.arquivos.renomear(&parcial, &guardado) {
        return nao_guardou(contexto, tela, erro.motivo());
    }

    contexto.registro.info(format!(
        "pacote guardado copiado do --iso · {} · {}",
        iso.display(),
        guardado.display()
    ));
    tela(&linha(
        "Pacote guardado",
        &if o_guardado_nao_batia {
            "nao batia com o SHA256 deste ARCA · substituido pela copia do --iso".to_string()
        } else {
            format!(
                "copiado do --iso para {} · o proximo preparo nao baixa (PR-2)",
                guardado.display()
            )
        },
    ));
}

/// A linha do guardado que não se deixou medir, e que fica como está.
fn nao_deu_para_medir(tela: &mut dyn FnMut(&str), erro: &Erro) {
    tela(&linha(
        "Pacote guardado",
        &format!(
            "nao deu para medir o que ja esta guardado ({}) · deixado como esta",
            motivo_da_medida(erro)
        ),
    ));
}

/// A linha da cópia que não se fez, e o aviso no `arca.log` — que o ensaio
/// não escreve, porque ele só diz o que o preparo de verdade faria.
fn nao_guardou(contexto: &Contexto, tela: &mut dyn FnMut(&str), motivo: String) {
    let pasta = contexto.registro.pasta();
    if !contexto.dry_run {
        contexto.registro.aviso(format!(
            "o --iso nao foi guardado em {} ({motivo}) · este preparo segue do --iso",
            pasta.display()
        ));
    }
    tela(&linha(
        "Pacote guardado",
        &format!(
            "nao deu para guardar em {} ({motivo}) · este preparo segue do --iso",
            pasta.display()
        ),
    ));
}

/// O motivo de uma medida que não se fez: o que o `certutil` respondeu, sem a
/// frase da recusa de PR-1 em volta; a recusa do pacote, sem o "o pacote do
/// Clonezilla foi recusado" que o `erro:` põe na frente; ou o que o Windows
/// respondeu. Ele vai entre parênteses no meio de uma linha que já diz de que
/// pacote se trata.
fn motivo_da_medida(erro: &Erro) -> String {
    match erro {
        Erro::PacoteRecusado(pacote::RecusaDoPacote::NaoDeuParaResumir(porque)) => {
            porque.to_string()
        }
        Erro::PacoteRecusado(recusa) => recusa.to_string(),
        outro => outro.motivo(),
    }
}

/// A linha do pacote guardado que passou na conferência (PR-7).
pub fn montar_o_pacote_guardado(resumo: &crate::resumo::Resumo, guardado: &Path) -> String {
    linha(
        "Pacote conferido",
        &format!(
            "ok · {} · guardado neste PC em {} · nada tocado ainda (PR-7)",
            resumo.abreviado(),
            guardado.display()
        ),
    )
}

/// Confere o pacote de novo e guarda a cópia de PR-3 — os passos 6 e 7.
///
/// O pacote já é local desde o pré-voo: o do `--iso` ou o guardado neste PC
/// (PR-7). Nenhum download acontece aqui, depois do ponto sem volta.
///
/// # A ordem é conferir, e só então usar
///
/// PR-1 é sobre isto: um pacote que não se conferiu é um pacote que não se sabe
/// o que é, e extrair antes de conferir tornaria a conferência decorativa.
///
/// A cópia de PR-3 fica no `ARCAVAULT` **depois** de conferida, pelo mesmo
/// motivo: guardar um pacote que não passou seria guardar lixo com cara de
/// fonte confiável.
fn obter_o_pacote(contexto: &Contexto, local: &Path, raiz_do_vault: &Path) -> Resultado<PathBuf> {
    // PR-1, e esta é a **segunda** conferência: a primeira foi o pré-voo, e o
    // que separa as duas é uma pessoa lendo o plano e digitando o modelo do
    // disco. É o mesmo intervalo que o terceiro tempo de PR-4 cobre relendo o
    // disco.
    let resumo = conferir_o_pacote(contexto, local)?;

    print!(
        "{}",
        linha(
            "SHA256 conferido",
            &format!("ok · {} · de {}", resumo.abreviado(), local.display()),
        )
    );

    // PR-3: a cópia no `ARCAVAULT`, para que o dispositivo se reconstrua
    // sozinho. O pacote mora fora dele — no `--iso` ou no PC —, e o passo 5
    // acabou de apagar o que havia lá.
    let copia = raiz_do_vault.join(pacote::ARQUIVO);
    contexto.arquivos.copiar(local, &copia)?;
    print!(
        "{}",
        linha(
            "Copia do pacote em ARCAVAULT",
            &format!("ok · {} (PR-3)", copia.display()),
        )
    );

    Ok(local.to_path_buf())
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
///
/// # A leitura vem de antes do passo 5, e o `device` é a primeira escrita
///
/// **Medido em 27/08/2026.** Este comando apagou um dispositivo ARCA existente
/// e morreu aqui, no primeiro `/enum`: a entrada `ARCA` apontava para o GUID
/// da `ARCABOOT` que o passo 5 tinha acabado de apagar, e nesse estado
/// **todo** `bcdedit /enum` desta máquina lista tudo e sai com código 1 —
/// *"Foi especificado um dispositivo inexistente."*. O comando que conserta o
/// estado é o `/set device` três linhas abaixo, e ele nunca era alcançado. O
/// `arca sondar` morria na mesma leitura.
///
/// Duas coisas mudaram por isso. A entrada a reusar e a ordem permanente são
/// **lidas antes do ponto sem volta** ([`ler_o_firmware_antes`]), quando o
/// firmware ainda está coerente — e é lá que C-4 decide se pode haver `/copy`.
/// E o primeiro `bcdedit` depois do apagar é o **`/set device`** para a
/// partição nova, que é o que devolve o código 0 aos `/enum` seguintes
/// (medido à mão no mesmo dia, sobre o estado quebrado). Os `/enum` daqui em
/// diante passam por [`firmware::enumerar`], que aceita a listagem mesmo com
/// código (C-15) — mas com o `device` já apontado eles nem precisam disso.
fn criar_a_entrada(
    contexto: &Contexto,
    letra_do_boot: char,
    antes: &firmware::Leitura,
) -> Resultado<EntradaCriada> {
    let desejado = Alvo::ParticaoComLetra(letra_do_boot.to_ascii_uppercase());

    let existente = antes
        .entrada_do_arca()
        .map(|achada| achada.entrada.identificador.clone());

    // C-15 com C-4 já foi decidido em `ler_o_firmware_antes`: chegar aqui sem
    // entrada é chegar com uma leitura limpa, e o `/copy` pode acontecer.
    let (identificador, ja_existia) = match existente {
        Some(identificador) => (identificador, true),
        None => (copiar_do_bootmgr(contexto)?, false),
    };

    // O `device`, a descrição e o `path` — **o `device` primeiro**, ver acima.
    // As três com releitura de C-3: o sucesso do `bcdedit` nunca é prova, e
    // com mídia removível ele responde "êxito" mantendo o valor antigo (C-6).
    let _ = contexto.firmware.executar(&[
        "/set",
        &identificador,
        "device",
        &desejado.como_bcdedit_escreve(),
    ]);
    let _ = contexto
        .firmware
        .executar(&["/set", &identificador, "description", firmware::ARCA]);
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
    let saiu_da_ordem = tirar_da_ordem(contexto, &identificador, antes)?;

    // **A ordem que sobrou, e não a que saiu** (P-28). A promessa da tela de
    // fim é sobre onde a máquina vai bootar, e quem decide isso é o que ficou
    // na ordem. Uma leitura a mais, e ela tem de ser de `firmware` e não de
    // `{fwbootmgr}`: só a primeira traz as entradas junto da ordem, e sem elas
    // não há como perguntar de nenhuma se ela declara alvo.
    //
    // Uma leitura que não se deixa entender **recusa** em vez de virar `None`:
    // `None` aqui é a tela prometendo que a máquina sobe o Windows, e é
    // exatamente a afirmação que ela não poderia fazer sem ter lido a ordem.
    let sobrou = firmware::enumerar(contexto.firmware, FIRMWARE)?;
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

/// O firmware, lido **antes** do ponto sem volta — a ordem permanente e a
/// entrada a reusar (PR-6).
///
/// # Por que antes, e não no passo 11
///
/// O passo 5 apaga o disco. Se ele era um dispositivo ARCA, a entrada de
/// firmware passa a apontar para uma partição que não existe mais — e é o
/// **próprio `prepare`** que a deixa assim. Medido em 27/08/2026: nesse
/// estado, todo `bcdedit /enum` lista tudo e sai com código 1, e o passo 11,
/// que começava lendo, morria antes de reapontar. Lido aqui, o firmware ainda
/// está coerente, e o que se lê é o que vale: a entrada não muda de
/// identificador porque uma partição sumiu.
///
/// # C-15 com C-4, e a decisão é daqui
///
/// Uma leitura que veio com código é aceita para **reusar** — a entrada está
/// na listagem, com identificador — e recusada para **criar**: um `/copy`
/// sobre uma listagem que o `bcdedit` disse ter problema é apostar que ela
/// está completa, e a aposta errada é uma segunda entrada, que é exatamente o
/// que C-4 proíbe. A recusa acontece aqui, com o disco intacto, e custa rodar
/// de novo depois de olhar o `bcdedit`.
///
/// Uma chamada só, ao `firmware`, que traz o gerenciador e as entradas
/// juntos — duas chamadas dariam duas leituras de momentos diferentes coladas
/// numa só (§11 do PRD).
fn ler_o_firmware_antes(contexto: &Contexto) -> Resultado<firmware::Leitura> {
    let leitura = firmware::enumerar(contexto.firmware, FIRMWARE)?;
    if !leitura.viu_o_gerenciador {
        return Err(Erro::FirmwareIlegivel { alvo: FIRMWARE });
    }

    if let Some(codigo) = leitura.codigo_da_recusa {
        let Some(achada) = leitura.entrada_do_arca() else {
            return Err(Erro::EntradaNaoNasceDeLeituraRecusada { codigo });
        };

        // Fica no registro, e não na tela: é diagnóstico de projeto, e a tela
        // do fim já diz o que a entrada passou a ser.
        contexto.registro.info(format!(
            "o bcdedit listou o firmware e saiu com codigo {codigo} · a entrada {} `{}` aponta para {} · sera reusada (C-15)",
            achada.entrada.identificador,
            achada.descricao,
            achada
                .entrada
                .alvo
                .as_ref()
                .map(Alvo::como_bcdedit_escreve)
                .unwrap_or_else(|| "nada".to_string()),
        ));
    }

    Ok(leitura)
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
    let agora = firmware::enumerar(contexto.firmware, FWBOOTMGR)?;
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

    let depois = firmware::enumerar(contexto.firmware, FWBOOTMGR)?;
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
    firmware::enumerar(contexto.firmware, identificador)?
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
pub fn montar_o_plano(
    preparacao: &Preparacao,
    pacote_do_pre_voo: &PacoteDoPreVoo,
    firmware_antes: &firmware::Leitura,
) -> String {
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

    // **Não há parágrafo explicando o esquema aqui, e a ausência é decisão.**
    //
    // Até o ADR-0025 havia um, e ele existia por uma razão que o próprio
    // ADR-0014 escreveu: *"quem lê `MBR` num dispositivo novo em 2026 vai achar
    // que e engano"*. Era texto para desfazer uma estranheza — e a estranheza
    // era do MBR.
    //
    // Com GPT ela some. Ninguém para diante de `GPT` numa tela de 2026 achando
    // que houve erro, e o que sobrava do parágrafo era a data de uma medição, o
    // número de um ADR e a menção a uma partição que o comando remove sozinho.
    // Nada disso é o que quem está prestes a apagar um disco precisa ler: essa
    // pessoa quer saber **o que vai acontecer com o disco dela**, e as duas
    // linhas acima já dizem.
    //
    // O porquê do esquema é registro de projeto, e o lugar dele é o
    // [ADR-0025](../../docs/adr/0025-o-arca-particiona-em-gpt.md).
    saida.push_str("\nE O QUE MAIS VAI ACONTECER:\n");

    // O plano não promete download: desde PR-7 o pacote já está no PC quando
    // ele é impresso. Só o ensaio sem pacote guardado chega aqui sem ele, e aí
    // a linha diz o que o preparo de verdade faria, e quando.
    saida.push_str(&format!(
        "  Clonezilla {} · {}\n",
        pacote::VERSAO,
        match pacote_do_pre_voo {
            PacoteDoPreVoo::DoIso(caminho) => format!("do arquivo local {} (PR-2)", caminho.display()),
            PacoteDoPreVoo::Guardado(_) => "do pacote guardado neste PC, conferido contra o valor\n     compilado neste ARCA (PR-1)".to_string(),
            PacoteDoPreVoo::PorBaixar => format!(
                "o preparo de verdade o baixa antes do menu ({}) e o guarda\n     neste PC, com o SHA256 conferido contra o valor compilado neste ARCA (PR-1)",
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
    //
    // E o plano diz **qual** dos dois vai acontecer — criar ou reusar —,
    // porque o firmware ja foi lido (PR-6). Um plano que dissesse "criada"
    // sobre uma entrada que existe seria a tela afirmando o que o comando nao
    // vai fazer.
    match firmware_antes.entrada_do_arca() {
        Some(achada) => saida.push_str(&format!(
            "  A entrada de boot `{}` que ja existe no firmware ({}) e REAPONTADA\n     \
             para o ARCABOOT deste disco — o ARCA mantem uma entrada, e nao uma por\n     \
             dispositivo (C-4) —, e fica **fora da ordem permanente**, para que\n     \
             ligar a maquina continue subindo o Windows (C-5)\n",
            achada.descricao, achada.entrada.identificador
        )),
        None => saida.push_str(&format!(
            "  Uma entrada de boot chamada `{}` e criada no firmware, apontando para o\n     \
             ARCABOOT — e **tirada da ordem permanente** logo em seguida, para que\n     \
             ligar a maquina continue subindo o Windows (C-5)\n",
            firmware::ARCA
        )),
    }
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
///
/// O boot se reconhece pelo mesmo casamento do `dispositivo::encontrar`, e
/// não pela constante: desde C-16 (27/09/2026) ele pode se chamar
/// `ARCA-<texto>`, e um dispositivo nomeado perde as imagens do mesmo jeito.
fn e_um_dispositivo_arca(disco: &DiscoParaPreparar) -> bool {
    let tem = |casa: &dyn Fn(&str) -> bool| {
        disco
            .particoes
            .iter()
            .any(|particao| particao.rotulo.as_deref().is_some_and(casa))
    };

    tem(&|rotulo| rotulo.eq_ignore_ascii_case(ARCAVAULT)) && tem(&dispositivo::e_rotulo_de_boot)
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
         \x20 Ele reinicia a maquina, roda no Clonezilla e desliga — nao faz backup\n\
         \x20 nem restauracao. Depois dele, `arca resultado`. Sem isso, `arca backup`\n\
         \x20 recusa.\n",
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
        ArquivosEmMemoria, ArquivosQueRecusam, ConsoleDeMentira, DiscosDeMentira,
        EntropiaDeMentira, FirmwareDeMentira, ParticionadorDeMentira, RelogioParado,
        SistemaDeMentira, amostra, discos_para_preparar_desta_mesa, o_que_o_particionamento_deixou,
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

    /// O espaço livre de um volume que não é o assunto do teste.
    const MUITO_ESPACO: u64 = 1 << 40;

    /// O que o pacote guardado da bancada tem dentro.
    const O_PACOTE_GUARDADO: &str = "o pacote guardado, de mentira";

    impl Bancada {
        /// A bancada nasce com o pacote guardado que bate, que é o estado
        /// normal de um PC depois do primeiro preparo (PR-7): o preparo segue
        /// sem rede, e os testes que não são sobre o pacote continuam sobre o
        /// que eram. [`Bancada::sem_pacote_guardado`] tira o pacote.
        fn nova(etiqueta: &str, console: ConsoleDeMentira) -> Bancada {
            let registro = Registro::em(
                std::env::temp_dir().join(format!(
                    "arca-prepare-{etiqueta}-{}-{:?}",
                    std::process::id(),
                    std::thread::current().id()
                )),
                Box::new(RelogioDoSistema),
            );
            let guardado = pacote::guardado_em(registro.pasta());

            let mut arquivos = ArquivosEmMemoria::novo().com(&guardado, O_PACOTE_GUARDADO);
            arquivos.espaco_livre = MUITO_ESPACO;

            Bancada {
                arquivos,
                discos: DiscosDeMentira::default(),
                // O firmware é lido antes do plano (PR-6), então a bancada
                // nasce com um gerenciador legível e sem entrada do ARCA — o
                // caso "disco qualquer, entrada por criar".
                firmware: FirmwareDeMentira::novo().respondendo(FIRMWARE, &ordem_desta_mesa()),
                relogio: RelogioParado::em("2026-08-23T18:38:00"),
                entropia: EntropiaDeMentira::com(&[0; 8]),
                sistema: SistemaDeMentira::novo()
                    .com_resumo(&guardado.to_string_lossy(), pacote::SHA256),
                console,
                particionador: ParticionadorDeMentira::desta_mesa(),
                registro,
            }
        }

        /// A mesma bancada num PC que nunca preparou nada: sem pacote
        /// guardado, e o `.parcial` que o download vai deixar passando no
        /// SHA256 e virando o guardado.
        fn sem_pacote_guardado(mut self) -> Bancada {
            let mut arquivos = ArquivosEmMemoria::novo();
            arquivos.espaco_livre = MUITO_ESPACO;
            self.arquivos = arquivos;
            self.sistema = SistemaDeMentira::novo()
                .com_resumo(&self.parcial().to_string_lossy(), pacote::SHA256)
                .com_resumo(&self.guardado().to_string_lossy(), pacote::SHA256);
            self
        }

        fn guardado(&self) -> PathBuf {
            pacote::guardado_em(self.registro.pasta())
        }

        fn parcial(&self) -> PathBuf {
            pacote::parcial_em(self.registro.pasta())
        }

        /// O texto do `arca.log` desta bancada.
        fn registrado(&self) -> String {
            std::fs::read_to_string(self.registro.caminho()).unwrap_or_default()
        }

        /// O pré-voo do pacote, com a tela que ele escreveu.
        fn pre_voo(
            &self,
            dry_run: bool,
            iso: Option<&Path>,
        ) -> (Resultado<PacoteDoPreVoo>, String) {
            let (pacote, escritas) = self.pre_voo_escrita_a_escrita(dry_run, iso);
            (pacote, escritas.concat())
        }

        /// O mesmo pré-voo, com cada escrita na tela separada das outras: a
        /// linha viva é reescrita no lugar, e o que um teste dela afirma é
        /// uma escrita, e não o texto junto (WPC-220).
        fn pre_voo_escrita_a_escrita(
            &self,
            dry_run: bool,
            iso: Option<&Path>,
        ) -> (Resultado<PacoteDoPreVoo>, Vec<String>) {
            let mut escritas = Vec::new();
            let pacote = obter_o_pacote_no_pre_voo(&self.contexto(dry_run), iso, &mut |texto| {
                escritas.push(texto.to_string())
            });
            (pacote, escritas)
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

    // ─────────── o pré-voo do `--iso`, e o disco que ele salva ───────────

    /// Onde um pacote baixado costuma estar de verdade — e o caminho tem
    /// espaço, que é metade das formas de digitá-lo errado.
    const NA_PASTA_DE_DOWNLOADS: &str =
        r"C:\Users\Ana Paula\Downloads\clonezilla-live-3.3.3-15-amd64.zip";

    /// Uma bancada onde o arquivo do `--iso` **está lá**, e o `certutil`
    /// responde o resumo que se pedir sobre ele.
    ///
    /// O resumo entra por parâmetro porque é o que separa os três casos: o
    /// pacote certo, o de outra versão e o que nem chega a ser medido.
    fn com_o_pacote_em(
        etiqueta: &str,
        caminho: &str,
        sha256: &str,
        console: ConsoleDeMentira,
    ) -> Bancada {
        let mut bancada = Bancada::nova(etiqueta, console);
        bancada.arquivos = ArquivosEmMemoria::novo().com(caminho, "o zip, de mentira");
        // Desde a WPC-220 o `--iso` que passa é copiado para o pacote
        // guardado, e a cópia pergunta pelo espaço: um volume sem nenhum
        // byte livre tiraria a cópia de todo teste do `--iso` sem dizer.
        bancada.arquivos.espaco_livre = MUITO_ESPACO;
        bancada.sistema = SistemaDeMentira::novo().com_resumo(caminho, sha256);
        bancada
    }

    #[test]
    fn um_iso_que_nao_esta_la_recusa_com_o_disco_intacto() {
        // **O teste que motivou o pré-voo.** Um caminho digitado errado é o
        // erro mais barato de cometer neste comando e era o mais caro de
        // pagar: medido em 27/08/2026, a recusa nascia no passo 7, com o disco
        // já apagado, e quem tentasse de novo pagava outro disco.
        //
        // O duplo não precisa ser ensinado a falhar: o `resumo_padrao` do
        // `SistemaDeMentira` **é** a resposta medida do `certutil` para um
        // arquivo ausente. Se o pré-voo sumisse, este teste passaria a falhar
        // pelo `certutil`, e não pelo caminho — e a asserção do
        // particionamento pegaria isso.
        let bancada = Bancada::nova("iso-ausente", ConsoleDeMentira::mudo());

        let erro = executar(
            &bancada.contexto(false),
            Some(1),
            Some(Path::new(NA_PASTA_DE_DOWNLOADS)),
        )
        .unwrap_err();

        assert!(
            matches!(
                erro,
                Erro::PacoteRecusado(pacote::RecusaDoPacote::NaoEstaLa { .. })
            ),
            "{erro}"
        );
        assert!(
            !bancada.particionador.particionou(),
            "o caminho errado do `--iso` apagou o disco"
        );
        assert_eq!(
            bancada.console.lidas.get(),
            0,
            "chegou a perguntar antes de olhar o pacote"
        );

        // E a mensagem fala do caminho, que é o problema — e não do
        // `0x80070002`, que é o sintoma.
        let dito = erro.to_string();
        assert!(dito.contains(NA_PASTA_DE_DOWNLOADS), "{dito}");
        assert!(dito.contains("Nada foi apagado"), "{dito}");
    }

    #[test]
    fn um_iso_com_o_sha256_errado_recusa_com_o_disco_intacto() {
        // O caso de PR-1, e o mais provável depois de um caminho errado: o
        // SourceForge publica versões novas, e quem baixa "o Clonezilla" baixa
        // a de hoje. O ARCA fixa a `3.3.3-15`, então o pacote certo do site
        // errado é o pacote errado — e descobrir isso não pode custar o disco.
        let outra_versao = "1d76439d27aa20ec74a8e22c486dcfab67473b6fd6bbc7376a806fede0293b10";
        let bancada = com_o_pacote_em(
            "iso-divergente",
            NA_PASTA_DE_DOWNLOADS,
            outra_versao,
            ConsoleDeMentira::mudo(),
        );

        let erro = executar(
            &bancada.contexto(false),
            Some(1),
            Some(Path::new(NA_PASTA_DE_DOWNLOADS)),
        )
        .unwrap_err();

        assert!(
            matches!(
                erro,
                Erro::PacoteRecusado(pacote::RecusaDoPacote::ResumoDivergente { .. })
            ),
            "{erro}"
        );
        assert!(
            !bancada.particionador.particionou(),
            "o pacote de outra versao apagou o disco"
        );
    }

    #[test]
    fn um_iso_sem_o_que_boota_recusa_com_o_disco_intacto() {
        // A terceira pergunta do pré-voo, e a que o `bsdtar` sair com zero não
        // responde. Um zip sem o `bootx64.efi` produz um dispositivo que não
        // boota — e isso só se descobriria quando alguém precisasse dele.
        let mut bancada = Bancada::nova("iso-incompleto", ConsoleDeMentira::mudo());
        bancada.arquivos =
            ArquivosEmMemoria::novo().com(NA_PASTA_DE_DOWNLOADS, "o zip, de mentira");
        bancada.sistema = SistemaDeMentira::novo()
            .com_resumo(NA_PASTA_DE_DOWNLOADS, pacote::SHA256)
            .listando(0, "boot/grub/grub.cfg\nlive/vmlinuz\nlive/initrd.img\n");

        let erro = executar(
            &bancada.contexto(false),
            Some(1),
            Some(Path::new(NA_PASTA_DE_DOWNLOADS)),
        )
        .unwrap_err();

        assert!(
            matches!(
                erro,
                Erro::PacoteRecusado(pacote::RecusaDoPacote::PacoteIncompleto { .. })
            ),
            "{erro}"
        );
        assert!(
            !bancada.particionador.particionou(),
            "o pacote sem o `.efi` apagou o disco"
        );
    }

    #[test]
    fn o_pacote_local_bom_e_conferido_de_novo_depois_do_ponto_sem_volta() {
        // O pré-voo não substitui o passo 7, e a razão é a mesma do terceiro
        // tempo de PR-4: entre os dois há uma pessoa lendo o plano e digitando
        // o modelo do disco, e nesse intervalo cabe trocar o arquivo. O que
        // vale para o disco vale para o pacote.
        let bancada = com_o_pacote_em(
            "iso-bom",
            NA_PASTA_DE_DOWNLOADS,
            pacote::SHA256,
            ConsoleDeMentira::respondendo(&["s", "JMicron Generic"]),
        );

        // O comando segue além do passo 8 e morre no `grub.cfg` que nenhum
        // duplo escreveu — o `bsdtar` de mentira não extrai nada. Não importa:
        // o que se cobra aqui acontece antes disso, e a extração registrada é
        // a prova de que o passo 7 foi atravessado.
        let _ = executar(
            &bancada.contexto(false),
            Some(1),
            Some(Path::new(NA_PASTA_DE_DOWNLOADS)),
        );

        assert!(
            !bancada.sistema.extraidos.borrow().is_empty(),
            "o pacote certo nao chegou a ser extraido"
        );
        // Entre as duas conferências do `--iso` está, desde a WPC-220, a da
        // cópia dele no `.parcial`, que vai virar o pacote guardado. A
        // igualdade continua exata: uma conferência do `--iso` a mais ou a
        // menos aparece aqui.
        assert_eq!(
            bancada.sistema.resumidos(),
            vec![
                PathBuf::from(NA_PASTA_DE_DOWNLOADS),
                bancada.parcial(),
                PathBuf::from(NA_PASTA_DE_DOWNLOADS)
            ],
            "o pacote tem de ser conferido duas vezes: antes e depois do ponto sem volta"
        );
        assert!(
            bancada.particionador.particionou(),
            "o caminho feliz do `--iso` nao chegou a particionar"
        );
        assert!(
            bancada.sistema.baixados.borrow().is_empty(),
            "o `--iso` baixou o Clonezilla"
        );
    }

    #[test]
    fn sem_iso_o_pacote_guardado_e_conferido_antes_do_menu_e_de_novo_no_passo_7() {
        // **A premissa deste teste mudou por decisão, e não para ficar
        // verde.** Até 04/10/2026 ele se chamava
        // `sem_iso_o_pre_voo_nao_tem_o_que_conferir` e exigia que, sem `--iso`,
        // o pacote fosse conferido "uma vez so, depois do ponto sem volta" —
        // baixado no passo 6, para o `ARCAVAULT`. PR-7 revogou isso: nenhum
        // download depois do passo 5, e o pacote mora no PC (WPC-134,
        // ADR-0028).
        //
        // O que ele cobra agora é o caminho do `--iso` valendo para o pacote
        // guardado: conferido no pré-voo e de novo no passo 7, extraído dele,
        // e a cópia de PR-3 feita a partir dele. A igualdade exata de
        // `resumidos` pega uma conferência a mais ou a menos.
        let bancada = Bancada::nova(
            "sem-iso",
            ConsoleDeMentira::respondendo(&["s", "JMicron Generic"]),
        );
        let guardado = bancada.guardado();
        let copia = PathBuf::from(format!(
            "{}:\\",
            o_que_o_particionamento_deixou().vault.letra
        ))
        .join(pacote::ARQUIVO);

        // O resultado é descartado: o comando morre no `grub.cfg` que o
        // `bsdtar` de mentira não extraiu. A extração registrada é a prova de
        // que ele atravessou os passos 6 e 7.
        let _ = executar(&bancada.contexto(false), Some(1), None);

        assert!(
            bancada.particionador.particionou(),
            "sem `--iso` o comando nao chegou a particionar"
        );
        assert_eq!(
            bancada.sistema.resumidos(),
            vec![guardado.clone(), guardado.clone()],
            "o pacote guardado tem de ser conferido no pre-voo e de novo no passo 7"
        );
        assert_eq!(
            bancada
                .sistema
                .extraidos
                .borrow()
                .first()
                .map(|(de, _)| de.clone()),
            Some(guardado),
            "a extracao nao saiu do pacote guardado"
        );
        assert_eq!(
            bancada.arquivos.conteudo_de(&copia).as_deref(),
            Some(O_PACOTE_GUARDADO),
            "a copia de PR-3 nao saiu do pacote guardado"
        );
        assert!(
            bancada.sistema.baixados.borrow().is_empty(),
            "com o pacote guardado, o preparo usou a rede"
        );
    }

    #[test]
    fn um_pacote_guardado_que_muda_antes_do_passo_7_nao_e_copiado() {
        // A segunda conferência existe para isto: entre o pré-voo e o passo 7
        // há uma pessoa lendo o plano, e o arquivo pode mudar. A cópia de PR-3
        // vem depois dela, e um pacote que não passa não é copiado.
        let mut bancada = Bancada::nova(
            "guardado-muda",
            ConsoleDeMentira::respondendo(&["s", "JMicron Generic"]),
        );
        let outra_versao = "1d76439d27aa20ec74a8e22c486dcfab67473b6fd6bbc7376a806fede0293b10";
        bancada.sistema = SistemaDeMentira::novo()
            .com_resumo(&bancada.guardado().to_string_lossy(), pacote::SHA256)
            .com_resumo_depois(&bancada.guardado().to_string_lossy(), outra_versao);
        let copia = PathBuf::from(format!(
            "{}:\\",
            o_que_o_particionamento_deixou().vault.letra
        ))
        .join(pacote::ARQUIVO);

        let erro = executar(&bancada.contexto(false), Some(1), None).unwrap_err();

        assert!(
            matches!(
                erro,
                Erro::PacoteRecusado(pacote::RecusaDoPacote::ResumoDivergente { .. })
            ),
            "{erro}"
        );
        assert!(
            bancada.particionador.particionou(),
            "a recusa veio antes do passo 5"
        );
        assert_eq!(
            bancada.arquivos.conteudo_de(&copia),
            None,
            "o pacote que nao passou foi copiado para o ARCAVAULT"
        );
    }

    // ─────────── o pacote guardado (PR-7) ───────────

    /// A linha que o pré-voo escreve quando o pacote guardado passa.
    fn linha_do_guardado(guardado: &Path) -> String {
        format!(
            "  Pacote conferido ................ ok · 00cee7700433 · guardado neste PC em {} · nada tocado ainda (PR-7)\n",
            guardado.display()
        )
    }

    /// 120,0 MB, o que já tinha vindo nos testes de retomada.
    const CENTO_E_VINTE_MB: u64 = 125_829_120;

    /// Uma fonte que não completou a conexão: o que o `curl` respondeu em
    /// 04/10/2026 contra `192.0.2.1`.
    fn nao_respondeu(no_destino: u64) -> crate::portas::Transferencia {
        crate::portas::Transferencia {
            codigo: 28,
            texto: "curl: (28) Connection timed out after 30002 milliseconds".to_string(),
            http: None,
            parou: false,
            encerrada: false,
            tamanho_no_destino: no_destino,
        }
    }

    /// Uma fonte que parou de mandar, encerrada pelo adaptador.
    fn parou(no_destino: u64) -> crate::portas::Transferencia {
        crate::portas::Transferencia {
            codigo: 1,
            texto: String::new(),
            http: Some(200),
            parou: true,
            encerrada: false,
            tamanho_no_destino: no_destino,
        }
    }

    /// Uma fonte que respondeu 404.
    fn nao_achou(no_destino: u64) -> crate::portas::Transferencia {
        crate::portas::Transferencia {
            codigo: 22,
            texto: "curl: (22) The requested URL returned error: 404".to_string(),
            http: Some(404),
            parou: false,
            encerrada: false,
            tamanho_no_destino: no_destino,
        }
    }

    #[test]
    fn sem_pacote_guardado_ele_e_baixado_conferido_e_renomeado_antes_dos_discos() {
        let bancada = Bancada::nova(
            "sem-guardado",
            ConsoleDeMentira::respondendo(&["s", "JMicron Generic"]),
        )
        .sem_pacote_guardado();
        let (guardado, parcial) = (bancada.guardado(), bancada.parcial());

        // O pré-voo sozinho, para ver a ordem: nada de disco antes de ele
        // terminar.
        let (pacote, _) = bancada.pre_voo(false, None);
        assert_eq!(pacote.unwrap(), PacoteDoPreVoo::Guardado(guardado.clone()));
        assert_eq!(bancada.particionador.enumeracoes.get(), 0);
        assert_eq!(
            *bancada.sistema.baixados.borrow(),
            vec![pacote::FONTES[0].url]
        );
        assert_eq!(*bancada.sistema.destinos.borrow(), vec![parcial.clone()]);
        assert_eq!(
            bancada.sistema.resumidos().first(),
            Some(&parcial),
            "o `.parcial` tem de ser conferido antes de virar o pacote"
        );
        assert!(bancada.arquivos.conteudo_de(&guardado).is_some());
        assert_eq!(
            bancada.arquivos.conteudo_de(&parcial),
            None,
            "o `.parcial` nao foi renomeado"
        );

        // E o comando inteiro, que só então chega aos discos.
        let bancada = Bancada::nova(
            "sem-guardado-inteiro",
            ConsoleDeMentira::respondendo(&["s", "JMicron Generic"]),
        )
        .sem_pacote_guardado();
        let _ = executar(&bancada.contexto(false), Some(1), None);
        assert_eq!(bancada.sistema.baixados.borrow().len(), 1);
        assert!(bancada.particionador.enumeracoes.get() >= 1);
        assert!(bancada.particionador.particionou());
    }

    #[test]
    fn um_pacote_baixado_sem_o_que_boota_nao_chega_aos_discos() {
        let mut bancada =
            Bancada::nova("baixado-incompleto", ConsoleDeMentira::mudo()).sem_pacote_guardado();
        bancada.sistema = SistemaDeMentira::novo()
            .com_resumo(&bancada.parcial().to_string_lossy(), pacote::SHA256)
            .listando(0, "boot/grub/grub.cfg\nlive/vmlinuz\nlive/initrd.img\n");

        let erro = executar(&bancada.contexto(false), Some(1), None).unwrap_err();

        assert!(
            matches!(
                erro,
                Erro::PacoteRecusado(pacote::RecusaDoPacote::PacoteIncompleto { .. })
            ),
            "{erro}"
        );
        assert_eq!(bancada.particionador.enumeracoes.get(), 0);
        assert!(!bancada.particionador.particionou());
        assert_eq!(bancada.arquivos.conteudo_de(bancada.guardado()), None);
    }

    #[test]
    fn um_pacote_guardado_que_bate_e_usado_sem_rede() {
        let bancada = Bancada::nova("guardado-bom", ConsoleDeMentira::mudo());

        let (pacote, tela) = bancada.pre_voo(false, None);

        assert_eq!(
            pacote.unwrap(),
            PacoteDoPreVoo::Guardado(bancada.guardado())
        );
        assert!(bancada.sistema.baixados.borrow().is_empty(), "usou a rede");
        assert_eq!(tela, linha_do_guardado(&bancada.guardado()));
        assert!(
            bancada.registrado().contains(&format!(
                "pacote guardado usado sem rede · {}",
                bancada.guardado().display()
            )),
            "{}",
            bancada.registrado()
        );
    }

    #[test]
    fn um_pacote_guardado_que_nao_bate_e_baixado_de_novo_do_zero() {
        let mut bancada = Bancada::nova("guardado-ruim", ConsoleDeMentira::mudo());
        let (guardado, parcial) = (bancada.guardado(), bancada.parcial());
        bancada.arquivos = ArquivosEmMemoria::novo()
            .com(&guardado, "o pacote que nao bate")
            .com_tamanho(&parcial, "o que veio da ultima vez", CENTO_E_VINTE_MB);
        bancada.arquivos.espaco_livre = MUITO_ESPACO;
        let outra_versao = "1d76439d27aa20ec74a8e22c486dcfab67473b6fd6bbc7376a806fede0293b10";
        bancada.sistema = SistemaDeMentira::novo()
            .com_resumo(&guardado.to_string_lossy(), outra_versao)
            .com_resumo(&parcial.to_string_lossy(), pacote::SHA256);

        let (pacote, tela) = bancada.pre_voo(false, None);

        assert_eq!(pacote.unwrap(), PacoteDoPreVoo::Guardado(guardado.clone()));
        let aviso = "  Pacote guardado ................. nao bate com o SHA256 deste ARCA · baixando de novo, do zero\n";
        let baixando = tela.find("  Baixando Clonezilla").expect("nao baixou");
        assert_eq!(tela.find(aviso), Some(0), "{tela}");
        assert!(aviso.len() <= baixando, "{tela}");
        // Do zero: a primeira linha do download não retoma, e o que tinha
        // vindo da última vez foi zerado — o guardado termina vazio, que é o
        // `.parcial` zerado e renomeado.
        assert!(!tela.contains("retomando"), "{tela}");
        assert_eq!(bancada.arquivos.conteudo_de(&guardado).as_deref(), Some(""));

        // E a ordem das escritas: a pasta, o `.parcial` zerado, e só então o
        // `.parcial` renomeado por cima do guardado que não bateu.
        let pasta = bancada.registro.pasta().to_path_buf();
        assert_eq!(
            bancada.arquivos.escritos(),
            vec![pasta, parcial.clone(), parcial, guardado]
        );
    }

    #[test]
    fn um_pacote_guardado_que_nao_bate_so_e_substituido_quando_o_novo_passa() {
        let mut bancada = Bancada::nova("guardado-ruim-de-novo", ConsoleDeMentira::mudo());
        let (guardado, parcial) = (bancada.guardado(), bancada.parcial());
        bancada.arquivos = ArquivosEmMemoria::novo().com(&guardado, "o pacote que nao bate");
        bancada.arquivos.espaco_livre = MUITO_ESPACO;
        let outra_versao = "1d76439d27aa20ec74a8e22c486dcfab67473b6fd6bbc7376a806fede0293b10";
        bancada.sistema = SistemaDeMentira::novo()
            .com_resumo(&guardado.to_string_lossy(), outra_versao)
            .com_resumo(&parcial.to_string_lossy(), outra_versao);

        let (pacote, _) = bancada.pre_voo(false, None);

        assert!(
            matches!(
                pacote,
                Err(Erro::PacoteRecusado(
                    pacote::RecusaDoPacote::ResumoDivergente { .. }
                ))
            ),
            "{pacote:?}"
        );
        assert_eq!(
            bancada.arquivos.conteudo_de(&guardado).as_deref(),
            Some("o pacote que nao bate"),
            "o guardado foi substituido por um pacote que tambem nao passou"
        );
    }

    #[test]
    fn um_parcial_menor_que_o_pacote_e_retomado_de_onde_parou() {
        let mut bancada =
            Bancada::nova("parcial-menor", ConsoleDeMentira::mudo()).sem_pacote_guardado();
        let parcial = bancada.parcial();
        bancada.arquivos = ArquivosEmMemoria::novo().com_tamanho(
            &parcial,
            "o que veio da ultima vez",
            CENTO_E_VINTE_MB,
        );
        bancada.arquivos.espaco_livre = MUITO_ESPACO;

        let (pacote, escritas) = bancada.pre_voo_escrita_a_escrita(false, None);

        // O critério 2 da WPC-220. Até ela, a retomada ia dentro da linha
        // viva, `3.3.3-15 · retomando de 120,0 MB · <rotulo>`; a versão, o
        // rótulo e a retomada passaram para a linha `Fonte`, e a linha viva
        // começa na barra, no ponto em que o `.parcial` está.
        assert!(pacote.is_ok(), "{pacote:?}");
        assert_eq!(
            escritas[..2],
            [
                "  Fonte ........................... 3.3.3-15 · free.nchc.org.tw (oldstable) · retomando de 120,0 MB\n",
                "  Baixando Clonezilla ............. [████░░░░░░░░░░░░░░░░]  22% · 120,0 de 535,5 MB · medindo",
            ],
            "{escritas:?}"
        );
        assert_eq!(
            bancada.arquivos.conteudo_de(bancada.guardado()).as_deref(),
            Some("o que veio da ultima vez"),
            "o `.parcial` foi zerado em vez de retomado"
        );
        // Nenhuma escrita no `.parcial` além da renomeação: ele não foi zerado.
        assert_eq!(
            bancada.arquivos.escritos(),
            vec![
                bancada.registro.pasta().to_path_buf(),
                parcial,
                bancada.guardado()
            ]
        );
    }

    #[test]
    fn um_parcial_do_tamanho_do_pacote_ou_maior_recomeca_do_zero() {
        for tamanho_do_parcial in [pacote::TAMANHO_BYTES, pacote::TAMANHO_BYTES + 1] {
            let mut bancada =
                Bancada::nova("parcial-inteiro", ConsoleDeMentira::mudo()).sem_pacote_guardado();
            let parcial = bancada.parcial();
            bancada.arquivos = ArquivosEmMemoria::novo().com_tamanho(
                &parcial,
                "um pacote inteiro que nao bateu",
                tamanho_do_parcial,
            );
            bancada.arquivos.espaco_livre = MUITO_ESPACO;

            let (pacote, tela) = bancada.pre_voo(false, None);

            assert!(pacote.is_ok(), "{tamanho_do_parcial}: {pacote:?}");
            assert!(!tela.contains("retomando"), "{tamanho_do_parcial}: {tela}");
            assert_eq!(
                bancada.arquivos.conteudo_de(bancada.guardado()).as_deref(),
                Some(""),
                "{tamanho_do_parcial}: o `.parcial` nao foi zerado"
            );
            assert_eq!(
                bancada.arquivos.escritos(),
                vec![
                    bancada.registro.pasta().to_path_buf(),
                    parcial.clone(),
                    parcial,
                    bancada.guardado()
                ],
                "{tamanho_do_parcial}: o `.parcial` nao foi zerado antes de renomeado"
            );
        }
    }

    #[test]
    fn nenhuma_fonte_entregou_recusa_antes_dos_discos_e_guarda_o_que_veio() {
        let mut bancada =
            Bancada::nova("nenhuma-fonte", ConsoleDeMentira::mudo()).sem_pacote_guardado();
        let [primeira, segunda, terceira] = pacote::FONTES;
        bancada.sistema = SistemaDeMentira::novo()
            .baixando_de(primeira.url, nao_respondeu(0), &[])
            .baixando_de(
                segunda.url,
                parou(CENTO_E_VINTE_MB),
                &[amostra(1.0, CENTO_E_VINTE_MB)],
            )
            .baixando_de(terceira.url, nao_achou(CENTO_E_VINTE_MB), &[]);

        let erro = executar(&bancada.contexto(false), Some(1), None).unwrap_err();

        assert!(
            matches!(
                erro,
                Erro::PacoteRecusado(pacote::RecusaDoPacote::NenhumaFonteEntregou { .. })
            ),
            "{erro}"
        );
        assert_eq!(
            erro.to_string(),
            "nao deu para baixar o Clonezilla 3.3.3-15 de nenhuma das 3 fontes:\n  \
             free.nchc.org.tw (oldstable) — nao respondeu em 30 s\n  \
             free.nchc.org.tw (stable) — parou de mandar por 60 s, com 120,0 MB de 535,5 MB\n  \
             downloads.sourceforge.net — HTTP 404\n\
             O que ja veio fica guardado, e a proxima tentativa continua de onde parou. Para nao depender do download do ARCA, baixe pelo navegador\n  \
             https://free.nchc.org.tw/clonezilla-live/oldstable/3.3.3-15/clonezilla-live-3.3.3-15-amd64.zip\n\
             e passe o arquivo com --iso. Nada foi apagado (PR-7)"
        );
        assert_eq!(
            *bancada.sistema.baixados.borrow(),
            vec![primeira.url, segunda.url, terceira.url],
            "cada fonte, uma vez, nesta ordem"
        );
        assert_eq!(bancada.particionador.enumeracoes.get(), 0);
        assert!(!bancada.particionador.particionou());
        assert!(
            bancada.arquivos.conteudo_de(bancada.parcial()).is_some(),
            "o `.parcial` sumiu"
        );
        assert_eq!(bancada.arquivos.conteudo_de(bancada.guardado()), None);
    }

    #[test]
    fn um_pacote_recem_baixado_que_nao_bate_recusa_antes_dos_discos() {
        // Até 04/10/2026 o `.parcial` que não bate ficava; o usuário decidiu
        // que ele é zerado ("Zerar o que não bate", WPC-134). O `.parcial`
        // começa com o que veio da última vez, para que a única escrita nele
        // seja a de depois da conferência.
        let mut bancada =
            Bancada::nova("baixado-divergente", ConsoleDeMentira::mudo()).sem_pacote_guardado();
        let parcial = bancada.parcial();
        bancada.arquivos =
            ArquivosEmMemoria::novo().com_tamanho(&parcial, "o que veio", CENTO_E_VINTE_MB);
        bancada.arquivos.espaco_livre = MUITO_ESPACO;
        let outra_versao = "1d76439d27aa20ec74a8e22c486dcfab67473b6fd6bbc7376a806fede0293b10";
        bancada.sistema =
            SistemaDeMentira::novo().com_resumo(&parcial.to_string_lossy(), outra_versao);

        let erro = executar(&bancada.contexto(false), Some(1), None).unwrap_err();

        assert!(
            matches!(
                erro,
                Erro::PacoteRecusado(pacote::RecusaDoPacote::ResumoDivergente { .. })
            ),
            "{erro}"
        );
        assert_eq!(bancada.particionador.enumeracoes.get(), 0);
        assert!(!bancada.particionador.particionou());
        assert_eq!(bancada.sistema.resumidos(), vec![parcial.clone()]);
        assert_eq!(
            bancada.arquivos.conteudo_de(&parcial).as_deref(),
            Some(""),
            "o `.parcial` que nao bate nao foi zerado"
        );
        assert_eq!(
            bancada.arquivos.escritos(),
            vec![bancada.registro.pasta().to_path_buf(), parcial]
        );
        assert_eq!(bancada.arquivos.conteudo_de(bancada.guardado()), None);
    }

    #[test]
    fn uma_pagina_no_lugar_do_pacote_e_zerada_e_a_proxima_execucao_baixa_do_zero() {
        // O caso que o Verifier achou: uma página de login de Wi-Fi entregue
        // com sucesso no lugar do zip é menor que o pacote, e sem o zero ela
        // seria retomada em toda execução seguinte. O `.parcial` começa com a
        // página — o que o `curl` teria escrito —, e a fonte diz que entregou.
        let mut bancada =
            Bancada::nova("pagina-de-login", ConsoleDeMentira::mudo()).sem_pacote_guardado();
        let parcial = bancada.parcial();
        bancada.arquivos =
            ArquivosEmMemoria::novo().com_tamanho(&parcial, "<html>entre no Wi-Fi</html>", 4096);
        bancada.arquivos.espaco_livre = MUITO_ESPACO;
        let pagina = "2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae";
        bancada.sistema = SistemaDeMentira::novo()
            .com_resumo(&parcial.to_string_lossy(), pagina)
            .baixando_de(
                pacote::FONTES[0].url,
                crate::portas::Transferencia {
                    tamanho_no_destino: 8192,
                    ..crate::duplos::entregou()
                },
                &[],
            );

        let (primeira, _) = bancada.pre_voo(false, None);
        assert!(
            matches!(
                primeira,
                Err(Erro::PacoteRecusado(
                    pacote::RecusaDoPacote::ResumoDivergente { .. }
                ))
            ),
            "{primeira:?}"
        );

        // Desde a WPC-220 a retomada é dita na linha `Fonte`, e a linha viva
        // começa no ponto em que o `.parcial` está: do zero, as duas dizem.
        let (_, tela) = bancada.pre_voo(false, None);
        assert!(
            tela.starts_with(
                "  Fonte ........................... 3.3.3-15 · free.nchc.org.tw (oldstable)\n  \
                 Baixando Clonezilla ............. [░░░░░░░░░░░░░░░░░░░░]   0% · 0,0 de 535,5 MB · medindo"
            ),
            "a execucao seguinte retomou a pagina: {tela}"
        );
    }

    #[test]
    fn um_parcial_que_nao_se_deixou_medir_nao_e_zerado() {
        // "Não consegui olhar" nunca vira "não bate": sem resumo, o `certutil`
        // do duplo responde o arquivo ausente, e o `.parcial` fica como está.
        let mut bancada =
            Bancada::nova("parcial-sem-medida", ConsoleDeMentira::mudo()).sem_pacote_guardado();
        let parcial = bancada.parcial();
        bancada.arquivos =
            ArquivosEmMemoria::novo().com_tamanho(&parcial, "o que veio", CENTO_E_VINTE_MB);
        bancada.arquivos.espaco_livre = MUITO_ESPACO;
        bancada.sistema = SistemaDeMentira::novo();

        let (pacote, _) = bancada.pre_voo(false, None);

        assert!(
            matches!(
                pacote,
                Err(Erro::PacoteRecusado(
                    pacote::RecusaDoPacote::NaoDeuParaResumir(_)
                ))
            ),
            "{pacote:?}"
        );
        assert_eq!(
            bancada.arquivos.conteudo_de(&parcial).as_deref(),
            Some("o que veio")
        );
        assert_eq!(
            bancada.arquivos.escritos(),
            vec![bancada.registro.pasta().to_path_buf()],
            "o `.parcial` que nao se deixou medir foi escrito"
        );
    }

    #[test]
    fn um_parcial_que_o_curl_nao_consegue_escrever_recusa_na_primeira_fonte() {
        // Decidido pelo usuário em 04/10/2026 ("Reconhecer a falha de
        // escrita"). A resposta do `curl` é a medida nesse dia contra um
        // `.parcial` somente-leitura.
        let mut bancada =
            Bancada::nova("curl-nao-escreve", ConsoleDeMentira::mudo()).sem_pacote_guardado();
        let parcial = bancada.parcial();
        bancada.arquivos =
            ArquivosEmMemoria::novo().com_tamanho(&parcial, "o que veio", CENTO_E_VINTE_MB);
        bancada.arquivos.espaco_livre = MUITO_ESPACO;
        bancada.sistema = SistemaDeMentira::novo().baixando_de(
            pacote::FONTES[0].url,
            crate::portas::Transferencia {
                codigo: 23,
                texto: "curl: cannot open \r\ncurl: 'C:\\ARCA\\clonezilla\r\ncurl: .parcial'\r\ncurl: (23) Failed writing received data to disk/application\r\n".to_string(),
                http: None,
                parou: false,
                encerrada: false,
                tamanho_no_destino: CENTO_E_VINTE_MB,
            },
            &[],
        );

        let erro = executar(&bancada.contexto(false), Some(1), None).unwrap_err();

        assert_eq!(
            erro.to_string(),
            format!(
                "nao deu para guardar o pacote em {} (curl: (23) Failed writing received data to disk/application). Nada foi apagado (PR-7)",
                bancada.registro.pasta().display()
            )
        );
        assert_eq!(
            *bancada.sistema.baixados.borrow(),
            vec![pacote::FONTES[0].url],
            "outra fonte foi tentada sobre o mesmo arquivo"
        );
        assert_eq!(bancada.particionador.enumeracoes.get(), 0);
        assert!(!bancada.particionador.particionou());
    }

    #[test]
    fn uma_pasta_que_nao_aceita_escrita_recusa_antes_de_baixar() {
        // Duas formas de "não aceita escrita": a pasta que não se deixa
        // criar, e o `.parcial` que não se deixa criar.
        let bancada = Bancada::nova("sem-escrita", ConsoleDeMentira::mudo());
        let (pasta, parcial) = (bancada.registro.pasta().to_path_buf(), bancada.parcial());
        drop(bancada);

        for recusado in [pasta.clone(), parcial] {
            let bancada =
                Bancada::nova("sem-escrita", ConsoleDeMentira::mudo()).sem_pacote_guardado();
            let arquivos = ArquivosQueRecusam::com(
                recusado.clone(),
                std::io::ErrorKind::PermissionDenied,
                "Acesso negado. (os error 5)",
            );
            let contexto = Contexto {
                arquivos: &arquivos,
                ..bancada.contexto(false)
            };

            let erro = obter_o_pacote_no_pre_voo(&contexto, None, &mut |_| {}).unwrap_err();

            assert!(
                matches!(
                    &erro,
                    Erro::PacoteRecusado(pacote::RecusaDoPacote::NaoDeuParaGuardar { pasta: p, .. })
                        if *p == pasta
                ),
                "{}: {erro}",
                recusado.display()
            );
            assert_eq!(
                erro.to_string(),
                format!(
                    "nao deu para guardar o pacote em {} (Acesso negado. (os error 5)). Nada foi apagado (PR-7)",
                    pasta.display()
                )
            );
            assert!(
                bancada.sistema.baixados.borrow().is_empty(),
                "{}: baixou sem ter onde guardar",
                recusado.display()
            );
        }
    }

    #[test]
    fn o_ensaio_com_pacote_guardado_que_nao_bate_nao_baixa() {
        // Decidido na construção (WPC-134, Handoff): o ensaio diz o que o
        // preparo de verdade faria com o guardado que não bate, e não baixa.
        let mut bancada = Bancada::nova("ensaio-guardado-ruim", ConsoleDeMentira::mudo());
        let outra_versao = "1d76439d27aa20ec74a8e22c486dcfab67473b6fd6bbc7376a806fede0293b10";
        bancada.sistema = SistemaDeMentira::novo()
            .com_resumo(&bancada.guardado().to_string_lossy(), outra_versao);

        let (pacote, tela) = bancada.pre_voo(true, None);

        assert_eq!(pacote.unwrap(), PacoteDoPreVoo::PorBaixar);
        assert_eq!(
            tela,
            "  Pacote guardado ................. nao bate com o SHA256 deste ARCA · o preparo de verdade baixa de novo, do zero; o ensaio nao baixa\n"
        );
        assert!(
            bancada.sistema.baixados.borrow().is_empty(),
            "o ensaio baixou"
        );
        assert_eq!(
            bancada.arquivos.escritos(),
            Vec::<PathBuf>::new(),
            "o ensaio escreveu"
        );
    }

    #[test]
    fn uma_pasta_com_o_nome_do_pacote_recusa_antes_de_baixar() {
        // Decidido na construção (WPC-134, Handoff): o `renomear` falharia
        // sobre a pasta depois de um download inteiro.
        let mut bancada = Bancada::nova("pasta-com-o-nome", ConsoleDeMentira::mudo());
        let guardado = bancada.guardado();
        bancada.arquivos = ArquivosEmMemoria::novo().com_pasta_vazia(&guardado);
        bancada.arquivos.espaco_livre = MUITO_ESPACO;

        let (pacote, _) = bancada.pre_voo(false, None);

        let erro = pacote.unwrap_err();
        assert_eq!(
            erro.to_string(),
            format!(
                "nao deu para guardar o pacote em {} (`clonezilla-live-3.3.3-15-amd64.zip` e uma pasta). Nada foi apagado (PR-7)",
                bancada.registro.pasta().display()
            )
        );
        assert!(bancada.sistema.baixados.borrow().is_empty());
    }

    #[test]
    fn sem_espaco_para_o_que_falta_recusa_antes_de_baixar() {
        let mut bancada =
            Bancada::nova("sem-espaco", ConsoleDeMentira::mudo()).sem_pacote_guardado();
        bancada.arquivos.espaco_livre = 100 * 1024 * 1024;

        let (pacote, _) = bancada.pre_voo(false, None);

        // O texto inteiro, como o `erro:` o mostra: sem o prefixo das recusas
        // de PR-1 e PR-2.
        let erro = pacote.unwrap_err();
        assert!(
            matches!(
                erro,
                Erro::PacoteRecusado(pacote::RecusaDoPacote::FaltaEspaco { .. })
            ),
            "{erro}"
        );
        assert_eq!(
            erro.to_string(),
            format!(
                "faltam 536 MB para baixar o pacote, e {} tem 100 MB livres. Nada foi apagado (PR-7)",
                bancada.registro.pasta().display()
            )
        );
        assert!(bancada.sistema.baixados.borrow().is_empty());

        // O que conta é o que falta: com 500 MB já no `.parcial`, faltam
        // 35,5 MB, e 40 MB livres bastam.
        let mut bancada =
            Bancada::nova("sem-espaco-retomando", ConsoleDeMentira::mudo()).sem_pacote_guardado();
        let parcial = bancada.parcial();
        bancada.arquivos =
            ArquivosEmMemoria::novo().com_tamanho(&parcial, "o que ja veio", 500 * 1024 * 1024);
        bancada.arquivos.espaco_livre = 40 * 1024 * 1024;

        let (pacote, _) = bancada.pre_voo(false, None);
        assert!(pacote.is_ok(), "{pacote:?}");
    }

    #[test]
    fn o_ensaio_sem_pacote_guardado_nao_baixa_e_diz_que_o_preparo_baixa() {
        let bancada =
            Bancada::nova("ensaio-sem-guardado", ConsoleDeMentira::mudo()).sem_pacote_guardado();

        let (pacote, tela) = bancada.pre_voo(true, None);

        assert_eq!(pacote.unwrap(), PacoteDoPreVoo::PorBaixar);
        assert!(
            bancada.sistema.baixados.borrow().is_empty(),
            "o ensaio baixou"
        );
        assert_eq!(
            bancada.arquivos.conteudo_de(bancada.parcial()),
            None,
            "o ensaio escreveu o `.parcial`"
        );
        assert_eq!(
            bancada.arquivos.escritos(),
            Vec::<PathBuf>::new(),
            "o ensaio escreveu"
        );
        assert_eq!(
            tela,
            "  Pacote .......................... nao esta neste PC · o preparo de verdade baixa 535,5 MB antes de qualquer pergunta; o ensaio nao baixa\n"
        );

        let plano = montar_o_plano(
            &preparacao_da_mesa(),
            &PacoteDoPreVoo::PorBaixar,
            &sem_entrada(),
        );
        assert!(
            plano.contains(
                "  Clonezilla 3.3.3-15 · o preparo de verdade o baixa antes do menu (535,5 MB) e o guarda\n     \
                 neste PC, com o SHA256 conferido contra o valor compilado neste ARCA (PR-1)\n"
            ),
            "{plano}"
        );
    }

    #[test]
    fn o_ensaio_com_pacote_guardado_confere_e_nao_particiona() {
        let bancada = Bancada::nova("ensaio-com-guardado", ConsoleDeMentira::mudo());

        let (pacote, tela) = bancada.pre_voo(true, None);
        assert_eq!(
            pacote.unwrap(),
            PacoteDoPreVoo::Guardado(bancada.guardado())
        );
        assert_eq!(tela, linha_do_guardado(&bancada.guardado()));
        assert_eq!(bancada.sistema.resumidos(), vec![bancada.guardado()]);

        let bancada = Bancada::nova("ensaio-com-guardado-inteiro", ConsoleDeMentira::mudo());
        executar(&bancada.contexto(true), Some(1), None).expect("o ensaio nao falha");
        assert!(!bancada.particionador.particionou());
        assert!(bancada.sistema.baixados.borrow().is_empty());
    }

    #[test]
    fn o_plano_com_pacote_guardado_nao_promete_download() {
        let plano = montar_o_plano(&preparacao_da_mesa(), &guardado_da_mesa(), &sem_entrada());

        assert!(
            plano.contains(
                "  Clonezilla 3.3.3-15 · do pacote guardado neste PC, conferido contra o valor\n     \
                 compilado neste ARCA (PR-1)\n"
            ),
            "{plano}"
        );
        assert!(!plano.contains("baixa"), "{plano}");
    }

    #[test]
    fn um_iso_que_passa_com_o_guardado_que_bate_nao_escreve_nada() {
        // **A premissa deste teste mudou por decisão, e não para ficar
        // verde.** Até 23/10/2026 ele se chamava
        // `com_iso_o_pacote_guardado_nem_e_olhado` e cobrava o critério 12 da
        // WPC-134: o `--iso` nem olhava o pacote guardado. A WPC-220 revogou
        // isso — o `--iso` que passa vira o pacote guardado, e para saber se
        // copia ele tem de medir o que já está lá. O que continua valendo, e
        // é o critério 19 dela: um guardado que bate não é reescrito, nada é
        // escrito na pasta, e a tela é a de antes.
        let mut bancada = com_o_pacote_em(
            "iso-com-guardado",
            NA_PASTA_DE_DOWNLOADS,
            pacote::SHA256,
            ConsoleDeMentira::respondendo(&["s", "JMicron Generic"]),
        );
        let (pasta, guardado, parcial) = (
            bancada.registro.pasta().to_path_buf(),
            bancada.guardado(),
            bancada.parcial(),
        );
        bancada.arquivos = ArquivosEmMemoria::novo()
            .com(NA_PASTA_DE_DOWNLOADS, "o zip, de mentira")
            .com(&guardado, O_PACOTE_GUARDADO);
        bancada.arquivos.espaco_livre = MUITO_ESPACO;
        bancada.sistema = SistemaDeMentira::novo()
            .com_resumo(NA_PASTA_DE_DOWNLOADS, pacote::SHA256)
            .com_resumo(&guardado.to_string_lossy(), pacote::SHA256);

        let (pacote, tela) = bancada.pre_voo(false, Some(Path::new(NA_PASTA_DE_DOWNLOADS)));
        let _ = executar(
            &bancada.contexto(false),
            Some(1),
            Some(Path::new(NA_PASTA_DE_DOWNLOADS)),
        );

        assert!(
            bancada.particionador.particionou(),
            "o caminho do `--iso` nao chegou ao fim"
        );
        assert!(
            bancada.sistema.resumidos().contains(&guardado),
            "o guardado nao foi medido"
        );
        for caminho in [&pasta, &guardado, &parcial] {
            assert!(
                !bancada.arquivos.foi_escrito(caminho),
                "o `--iso` escreveu {}",
                caminho.display()
            );
        }
        assert!(
            bancada
                .arquivos
                .escritos()
                .iter()
                .all(|escrito| !escrito.starts_with(&pasta)),
            "o `--iso` escreveu na pasta do pacote guardado: {:?}",
            bancada.arquivos.escritos()
        );
        assert!(bancada.sistema.baixados.borrow().is_empty());
        assert_eq!(
            tela,
            format!(
                "  Pacote conferido ................ ok · 00cee7700433 · {NA_PASTA_DE_DOWNLOADS} · nada tocado ainda (PR-2)\n"
            )
        );

        let plano = montar_o_plano(&preparacao_da_mesa(), &pacote.unwrap(), &sem_entrada());
        assert!(
            plano.contains(&format!(
                "  Clonezilla 3.3.3-15 · do arquivo local {NA_PASTA_DE_DOWNLOADS} (PR-2)\n"
            )),
            "{plano}"
        );
    }

    // ─────────── o `--iso` que passa vira o pacote guardado (WPC-220) ───────────

    /// A outra versão, para o guardado e a cópia que não batem.
    const OUTRA_VERSAO: &str = "1d76439d27aa20ec74a8e22c486dcfab67473b6fd6bbc7376a806fede0293b10";

    /// A linha `Pacote conferido` do `--iso`, a de antes da WPC-220.
    fn linha_do_iso() -> String {
        format!(
            "  Pacote conferido ................ ok · 00cee7700433 · {NA_PASTA_DE_DOWNLOADS} · nada tocado ainda (PR-2)\n"
        )
    }

    /// Uma bancada com o `--iso` que passa, sem pacote guardado, e o
    /// `certutil` respondendo `parcial` sobre a cópia no `.parcial`.
    fn com_um_iso_que_passa(etiqueta: &str, parcial: &str) -> Bancada {
        let mut bancada = com_o_pacote_em(
            etiqueta,
            NA_PASTA_DE_DOWNLOADS,
            pacote::SHA256,
            ConsoleDeMentira::mudo(),
        );
        bancada.sistema = SistemaDeMentira::novo()
            .com_resumo(NA_PASTA_DE_DOWNLOADS, pacote::SHA256)
            .com_resumo(&bancada.parcial().to_string_lossy(), parcial);
        bancada
    }

    /// Os caminhos escritos dentro da pasta do pacote guardado.
    fn escritos_na_pasta(escritos: Vec<PathBuf>, pasta: &Path) -> Vec<PathBuf> {
        escritos
            .into_iter()
            .filter(|escrito| escrito.starts_with(pasta))
            .collect()
    }

    #[test]
    fn um_iso_que_passa_vira_o_pacote_guardado() {
        let bancada = com_um_iso_que_passa("iso-vira-guardado", pacote::SHA256);
        let (pasta, guardado, parcial) = (
            bancada.registro.pasta().to_path_buf(),
            bancada.guardado(),
            bancada.parcial(),
        );
        let iso = PathBuf::from(NA_PASTA_DE_DOWNLOADS);

        let (pacote, tela) = bancada.pre_voo(false, Some(&iso));

        assert_eq!(pacote.unwrap(), PacoteDoPreVoo::DoIso(iso.clone()));
        assert_eq!(bancada.particionador.enumeracoes.get(), 0);
        assert_eq!(
            bancada.arquivos.escritos(),
            vec![pasta, parcial.clone(), parcial.clone(), guardado.clone()],
            "criar a pasta, copiar para o `.parcial` e renomea-lo"
        );
        assert_eq!(
            bancada.arquivos.copiados(),
            vec![(iso.clone(), parcial.clone())]
        );
        assert_eq!(
            bancada.sistema.resumidos(),
            vec![iso.clone(), parcial.clone()]
        );
        // O SHA256 e os quatro caminhos: a cópia é listada como o `--iso` foi.
        assert_eq!(
            *bancada.sistema.listados.borrow(),
            vec![iso.clone(), parcial.clone()],
            "a copia nao foi conferida pelos quatro caminhos"
        );
        assert_eq!(
            bancada.arquivos.conteudo_de(&guardado).as_deref(),
            Some("o zip, de mentira")
        );
        assert_eq!(bancada.arquivos.conteudo_de(&parcial), None);
        assert!(!bancada.arquivos.foi_escrito(&iso), "o `--iso` foi escrito");
        assert_eq!(
            bancada.arquivos.conteudo_de(&iso).as_deref(),
            Some("o zip, de mentira"),
            "o `--iso` mudou"
        );
        assert!(bancada.sistema.baixados.borrow().is_empty());
        assert_eq!(
            tela,
            format!(
                "{}  Pacote guardado ................. copiado do --iso para {} · o proximo preparo nao baixa (PR-2)\n",
                linha_do_iso(),
                guardado.display()
            )
        );
    }

    #[test]
    fn um_iso_que_passa_substitui_o_guardado_que_nao_bate_depois_de_conferido() {
        let iso = PathBuf::from(NA_PASTA_DE_DOWNLOADS);
        for (parcial_resume, guardado_termina) in [
            (pacote::SHA256, "o zip, de mentira"),
            (OUTRA_VERSAO, "o pacote que nao bate"),
        ] {
            let mut bancada = com_um_iso_que_passa("iso-substitui", parcial_resume);
            let (pasta, guardado, parcial) = (
                bancada.registro.pasta().to_path_buf(),
                bancada.guardado(),
                bancada.parcial(),
            );
            bancada.arquivos = ArquivosEmMemoria::novo()
                .com(&iso, "o zip, de mentira")
                .com(&guardado, "o pacote que nao bate");
            bancada.arquivos.espaco_livre = MUITO_ESPACO;
            bancada.sistema = SistemaDeMentira::novo()
                .com_resumo(NA_PASTA_DE_DOWNLOADS, pacote::SHA256)
                .com_resumo(&guardado.to_string_lossy(), OUTRA_VERSAO)
                .com_resumo(&parcial.to_string_lossy(), parcial_resume);

            let (pacote, tela) = bancada.pre_voo(false, Some(&iso));

            assert_eq!(pacote.unwrap(), PacoteDoPreVoo::DoIso(iso.clone()));
            assert_eq!(
                bancada.arquivos.conteudo_de(&guardado).as_deref(),
                Some(guardado_termina),
                "{parcial_resume}"
            );
            if parcial_resume == pacote::SHA256 {
                assert_eq!(
                    bancada.sistema.resumidos(),
                    vec![iso.clone(), guardado.clone(), parcial.clone()]
                );
                // O guardado só é escrito pela renomeação, depois de o
                // `.parcial` passar.
                assert_eq!(
                    bancada.arquivos.escritos(),
                    vec![pasta, parcial.clone(), parcial, guardado]
                );
                assert_eq!(
                    tela,
                    format!(
                        "{}  Pacote guardado ................. nao batia com o SHA256 deste ARCA · substituido pela copia do --iso\n",
                        linha_do_iso()
                    )
                );
            }
        }
    }

    #[test]
    fn um_guardado_que_nao_se_deixa_medir_nao_e_tocado_pelo_iso() {
        let iso = PathBuf::from(NA_PASTA_DE_DOWNLOADS);

        // O `certutil` não resume o guardado.
        let mut bancada = com_um_iso_que_passa("guardado-sem-medida", pacote::SHA256);
        let (pasta, guardado) = (bancada.registro.pasta().to_path_buf(), bancada.guardado());
        bancada.arquivos = ArquivosEmMemoria::novo()
            .com(&iso, "o zip, de mentira")
            .com(&guardado, O_PACOTE_GUARDADO);
        bancada.arquivos.espaco_livre = MUITO_ESPACO;
        let recusa =
            "CertUtil: -hashfile comando FALHOU: 0x80070020 (WIN32: 32 ERROR_SHARING_VIOLATION)";
        bancada.sistema = SistemaDeMentira::novo()
            .com_resumo(NA_PASTA_DE_DOWNLOADS, pacote::SHA256)
            .com_resposta_do_certutil(&guardado.to_string_lossy(), -2147024864, recusa);
        let motivo = "o `certutil` saiu com codigo -2147024864 e nao resumiu o arquivo: \
                      CertUtil: -hashfile comando FALHOU: 0x80070020 (WIN32: 32 ERROR_SHARING_VIOLATION)";

        let (pacote, tela) = bancada.pre_voo(false, Some(&iso));

        assert_eq!(pacote.unwrap(), PacoteDoPreVoo::DoIso(iso.clone()));
        assert_eq!(
            escritos_na_pasta(bancada.arquivos.escritos(), &pasta),
            Vec::<PathBuf>::new()
        );
        assert!(
            bancada.arquivos.copiados().is_empty(),
            "o `--iso` foi copiado"
        );
        assert_eq!(
            tela,
            format!(
                "{}  Pacote guardado ................. nao deu para medir o que ja esta guardado ({motivo}) · deixado como esta\n",
                linha_do_iso()
            )
        );

        // O Windows não deixa olhar o guardado.
        let bancada = com_um_iso_que_passa("guardado-sem-olhar", pacote::SHA256);
        let arquivos = ArquivosQueRecusam::com(
            bancada.guardado(),
            std::io::ErrorKind::PermissionDenied,
            "Acesso negado. (os error 5)",
        )
        .com_arquivo(&iso, "o zip, de mentira")
        .com_espaco_livre(MUITO_ESPACO);
        let contexto = Contexto {
            arquivos: &arquivos,
            ..bancada.contexto(false)
        };
        let mut tela = String::new();

        let pacote =
            obter_o_pacote_no_pre_voo(&contexto, Some(&iso), &mut |texto| tela.push_str(texto));

        assert_eq!(pacote.unwrap(), PacoteDoPreVoo::DoIso(iso.clone()));
        assert_eq!(arquivos.escritos(), Vec::<PathBuf>::new());
        assert_eq!(
            tela,
            format!(
                "{}  Pacote guardado ................. nao deu para medir o que ja esta guardado (Acesso negado. (os error 5)) · deixado como esta\n",
                linha_do_iso()
            )
        );
    }

    #[test]
    fn uma_copia_do_iso_que_nao_se_faz_vira_aviso_e_o_preparo_segue() {
        let iso = PathBuf::from(NA_PASTA_DE_DOWNLOADS);
        let linha_de = |pasta: &Path, motivo: &str| {
            format!(
                "{}  Pacote guardado ................. nao deu para guardar em {} ({motivo}) · este preparo segue do --iso\n",
                linha_do_iso(),
                pasta.display()
            )
        };
        let cobrar = |bancada: &Bancada, tela: &str, motivo: &str| {
            let pasta = bancada.registro.pasta();
            assert_eq!(tela, linha_de(pasta, motivo));
            let registrado = bancada.registrado();
            assert!(
                registrado
                    .lines()
                    .any(|linha| linha.contains("AVISO") && linha.contains(motivo)),
                "{motivo}: sem aviso no arca.log: {registrado}"
            );
            assert!(
                !bancada.arquivos.foi_escrito(bancada.guardado()),
                "{motivo}: o guardado foi escrito"
            );
        };

        // (1) e (2): a pasta e o `.parcial` que não aceitam escrita.
        for recusado in ["pasta", "parcial"] {
            let bancada = com_um_iso_que_passa("copia-recusada", pacote::SHA256);
            let caminho = match recusado {
                "pasta" => bancada.registro.pasta().to_path_buf(),
                _ => bancada.parcial(),
            };
            let arquivos = ArquivosQueRecusam::com(
                caminho,
                std::io::ErrorKind::PermissionDenied,
                "Acesso negado. (os error 5)",
            )
            .com_arquivo(&iso, "o zip, de mentira")
            .com_espaco_livre(MUITO_ESPACO);
            let contexto = Contexto {
                arquivos: &arquivos,
                ..bancada.contexto(false)
            };
            let mut tela = String::new();

            let pacote =
                obter_o_pacote_no_pre_voo(&contexto, Some(&iso), &mut |texto| tela.push_str(texto));

            assert_eq!(
                pacote.unwrap(),
                PacoteDoPreVoo::DoIso(iso.clone()),
                "{recusado}"
            );
            cobrar(&bancada, &tela, "Acesso negado. (os error 5)");
            assert!(
                !arquivos.escritos().contains(&bancada.guardado()),
                "{recusado}: o guardado foi escrito"
            );
        }

        // (3) Um byte a menos que o pacote, e nada é copiado.
        let mut bancada = com_um_iso_que_passa("copia-sem-espaco", pacote::SHA256);
        bancada.arquivos.espaco_livre = pacote::TAMANHO_BYTES - 1;
        let (pacote, tela) = bancada.pre_voo(false, Some(&iso));
        assert_eq!(pacote.unwrap(), PacoteDoPreVoo::DoIso(iso.clone()));
        cobrar(&bancada, &tela, "precisa de 536 MB livres, e ha 535 MB");
        assert!(bancada.arquivos.copiados().is_empty(), "copiou sem espaco");

        // E com o tamanho exato do pacote, a cópia acontece.
        let mut bancada = com_um_iso_que_passa("copia-no-limite", pacote::SHA256);
        bancada.arquivos.espaco_livre = pacote::TAMANHO_BYTES;
        let (_, tela) = bancada.pre_voo(false, Some(&iso));
        assert!(tela.contains("copiado do --iso para"), "{tela}");

        // (4) Uma pasta com o nome do pacote, e nada é copiado.
        let mut bancada = com_um_iso_que_passa("copia-sobre-pasta", pacote::SHA256);
        bancada.arquivos = ArquivosEmMemoria::novo()
            .com(&iso, "o zip, de mentira")
            .com_pasta_vazia(bancada.guardado());
        bancada.arquivos.espaco_livre = MUITO_ESPACO;
        let (pacote, tela) = bancada.pre_voo(false, Some(&iso));
        assert_eq!(pacote.unwrap(), PacoteDoPreVoo::DoIso(iso.clone()));
        cobrar(
            &bancada,
            &tela,
            "`clonezilla-live-3.3.3-15-amd64.zip` e uma pasta",
        );
        assert!(
            bancada.arquivos.copiados().is_empty(),
            "copiou sobre a pasta"
        );

        // (5) O `certutil` não resume a cópia (borda 5 do checklist): ela não
        // é zerada, porque não ter conseguido medir não é não bater.
        let mut bancada = com_um_iso_que_passa("copia-sem-medida", pacote::SHA256);
        bancada.sistema =
            SistemaDeMentira::novo().com_resumo(NA_PASTA_DE_DOWNLOADS, pacote::SHA256);
        let (pacote, tela) = bancada.pre_voo(false, Some(&iso));
        assert_eq!(pacote.unwrap(), PacoteDoPreVoo::DoIso(iso.clone()));
        cobrar(&bancada, &tela, CERTUTIL_SEM_O_ARQUIVO);
        assert_eq!(
            bancada.arquivos.conteudo_de(bancada.parcial()).as_deref(),
            Some("o zip, de mentira"),
            "a copia que nao se mediu foi zerada"
        );

        // (6) e (7): o espaço que não se deixa medir, e a renomeação
        // recusada depois de a cópia passar. Nenhum dos outros duplos chega
        // a esses dois braços, e o guardado continua não existindo.
        for falha in [Falha::Espaco, Falha::Renomear] {
            let bancada = com_um_iso_que_passa("copia-falha-tarde", pacote::SHA256);
            let mut dentro = ArquivosEmMemoria::novo().com(&iso, "o zip, de mentira");
            dentro.espaco_livre = MUITO_ESPACO;
            let arquivos = ArquivosQueFalhamEm { dentro, falha };
            let contexto = Contexto {
                arquivos: &arquivos,
                ..bancada.contexto(false)
            };
            let mut tela = String::new();

            let pacote =
                obter_o_pacote_no_pre_voo(&contexto, Some(&iso), &mut |texto| tela.push_str(texto));

            assert_eq!(
                pacote.unwrap(),
                PacoteDoPreVoo::DoIso(iso.clone()),
                "{falha:?}"
            );
            cobrar(&bancada, &tela, "Acesso negado. (os error 5)");
            assert_eq!(
                arquivos.dentro.conteudo_de(bancada.guardado()),
                None,
                "{falha:?}: o guardado existe"
            );
        }

        // (8) A cópia bate o SHA256 e não tem os quatro caminhos. Não
        // acontece de verdade — a cópia tem os bytes do `--iso`, que acabou
        // de passar na mesma listagem —, e o código não a renomeia: um
        // pacote guardado sem o que boota seria pior que nenhum. Não é a
        // divergência de C23, e por isso a cópia não é zerada.
        let mut bancada = com_um_iso_que_passa("copia-incompleta", pacote::SHA256);
        bancada.sistema = SistemaDeMentira::novo()
            .com_resumo(NA_PASTA_DE_DOWNLOADS, pacote::SHA256)
            .com_resumo(&bancada.parcial().to_string_lossy(), pacote::SHA256)
            .listando_em(
                &bancada.parcial().to_string_lossy(),
                "boot/grub/grub.cfg\nlive/vmlinuz\nlive/initrd.img\n",
            );
        let (pacote, tela) = bancada.pre_voo(false, Some(&iso));
        assert_eq!(pacote.unwrap(), PacoteDoPreVoo::DoIso(iso.clone()));
        cobrar(
            &bancada,
            &tela,
            "o pacote foi extraido e nao tem o que faz um dispositivo bootar: falta EFI/boot/bootx64.efi. \
             O `bsdtar` ter saido com codigo zero nao e prova de que o conteudo esta la",
        );
        assert_eq!(
            bancada.arquivos.conteudo_de(bancada.parcial()).as_deref(),
            Some("o zip, de mentira"),
            "a copia incompleta foi zerada"
        );
    }

    /// O que o `certutil` responde, pelo `resumo_padrao` do duplo, sobre um
    /// arquivo que ele não acha — o texto medido em 23/08/2026 —, como a
    /// recusa de PR-1 o cita.
    const CERTUTIL_SEM_O_ARQUIVO: &str = "o `certutil` saiu com codigo -2147024894 e nao resumiu o arquivo: \
         CertUtil: -hashfile comando FALHOU: 0x80070002 (WIN32: 2 ERROR_FILE_NOT_FOUND) · \
         CertUtil: O sistema nao pode encontrar o arquivo especificado.";

    /// A operação que falha em [`ArquivosQueFalhamEm`].
    #[derive(Debug, Clone, Copy)]
    enum Falha {
        Espaco,
        Renomear,
    }

    /// Um sistema de arquivos em memória em que uma operação só falha, com o
    /// `Acesso negado` do Windows. Existe para os dois braços da cópia do
    /// `--iso` (WPC-220) que vêm depois de a pasta aceitar escrita: o
    /// `ArquivosQueRecusam` recusa por caminho, e a pasta e o `.parcial`
    /// recusados param a cópia antes deles.
    struct ArquivosQueFalhamEm {
        dentro: ArquivosEmMemoria,
        falha: Falha,
    }

    impl ArquivosQueFalhamEm {
        fn recusa(&self, operacao: &'static str, caminho: &Path) -> Erro {
            Erro::Arquivo {
                operacao,
                caminho: caminho.to_path_buf(),
                origem: std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "Acesso negado. (os error 5)",
                ),
            }
        }
    }

    impl crate::portas::Arquivos for ArquivosQueFalhamEm {
        fn existe(&self, caminho: &Path) -> bool {
            self.dentro.existe(caminho)
        }
        fn o_que_ha(&self, caminho: &Path) -> Resultado<OQueHa> {
            self.dentro.o_que_ha(caminho)
        }
        fn ler_texto(&self, caminho: &Path) -> Resultado<String> {
            self.dentro.ler_texto(caminho)
        }
        fn ler_texto_alheio(&self, caminho: &Path) -> Resultado<String> {
            self.dentro.ler_texto_alheio(caminho)
        }
        fn escrever_atomico(&self, caminho: &Path, conteudo: &str) -> Resultado<()> {
            self.dentro.escrever_atomico(caminho, conteudo)
        }
        fn criar_diretorio(&self, caminho: &Path) -> Resultado<()> {
            self.dentro.criar_diretorio(caminho)
        }
        fn listar(&self, caminho: &Path) -> Resultado<Vec<crate::portas::Entrada>> {
            self.dentro.listar(caminho)
        }
        fn copiar(&self, origem: &Path, destino: &Path) -> Resultado<()> {
            self.dentro.copiar(origem, destino)
        }
        fn renomear(&self, origem: &Path, destino: &Path) -> Resultado<()> {
            match self.falha {
                Falha::Renomear => Err(self.recusa("renomeacao", destino)),
                Falha::Espaco => self.dentro.renomear(origem, destino),
            }
        }
        fn espaco_livre(&self, caminho: &Path) -> Resultado<u64> {
            match self.falha {
                Falha::Espaco => Err(self.recusa("espaco livre", caminho)),
                Falha::Renomear => self.dentro.espaco_livre(caminho),
            }
        }
    }

    #[test]
    fn uma_copia_do_iso_que_nao_bate_e_zerada_e_o_preparo_segue() {
        let bancada = com_um_iso_que_passa("copia-divergente", OUTRA_VERSAO);
        let (pasta, guardado, parcial) = (
            bancada.registro.pasta().to_path_buf(),
            bancada.guardado(),
            bancada.parcial(),
        );
        let iso = PathBuf::from(NA_PASTA_DE_DOWNLOADS);

        let (pacote, tela) = bancada.pre_voo(false, Some(&iso));

        assert_eq!(pacote.unwrap(), PacoteDoPreVoo::DoIso(iso.clone()));
        assert_eq!(bancada.arquivos.conteudo_de(&parcial).as_deref(), Some(""));
        // As escritas no `.parcial` são a cópia e o zero, e mais nenhuma. Que
        // o zero só acontece no braço da divergência é C18 quem mostra: lá a
        // cópia chega intacta ao guardado.
        assert_eq!(bancada.sistema.resumidos(), vec![iso, parcial.clone()]);
        assert_eq!(
            bancada.arquivos.escritos(),
            vec![pasta, parcial.clone(), parcial]
        );
        assert_eq!(bancada.arquivos.conteudo_de(&guardado), None);
        assert_eq!(
            tela,
            format!(
                "{}  Pacote guardado ................. a copia nao bateu com o SHA256 · este preparo segue do --iso\n",
                linha_do_iso()
            )
        );
    }

    #[test]
    fn com_iso_o_passo_7_a_extracao_e_a_copia_saem_do_iso() {
        let mut bancada = com_um_iso_que_passa("iso-ate-o-fim", pacote::SHA256);
        bancada.console = ConsoleDeMentira::respondendo(&["s", "JMicron Generic"]);
        let (guardado, parcial) = (bancada.guardado(), bancada.parcial());
        let iso = PathBuf::from(NA_PASTA_DE_DOWNLOADS);
        let copia = PathBuf::from(format!(
            "{}:\\",
            o_que_o_particionamento_deixou().vault.letra
        ))
        .join(pacote::ARQUIVO);

        // O comando morre no `grub.cfg` que o `bsdtar` de mentira não
        // extraiu; a extração registrada é a prova de que ele passou do 7.
        let _ = executar(&bancada.contexto(false), Some(1), Some(&iso));

        assert!(bancada.particionador.particionou());
        assert_eq!(
            bancada.sistema.resumidos(),
            vec![iso.clone(), parcial.clone(), iso.clone()],
            "o pre-voo, a copia e o passo 7"
        );
        assert_eq!(
            bancada
                .sistema
                .extraidos
                .borrow()
                .first()
                .map(|(de, _)| de.clone()),
            Some(iso.clone()),
            "a extracao nao saiu do `--iso`"
        );
        let copiados = bancada.arquivos.copiados();
        assert!(
            copiados.contains(&(iso.clone(), copia.clone())),
            "a copia de PR-3 nao saiu do `--iso`: {copiados:?}"
        );
        assert!(
            !copiados.contains(&(guardado, copia)),
            "a copia de PR-3 saiu do guardado: {copiados:?}"
        );
        assert!(bancada.sistema.baixados.borrow().is_empty());
    }

    #[test]
    fn um_iso_recusado_no_pre_voo_nao_escreve_na_pasta_do_arca() {
        let iso = PathBuf::from(NA_PASTA_DE_DOWNLOADS);
        let pasta_do_iso = PathBuf::from(A_PASTA_DE_DOWNLOADS);

        // NaoEstaLa, ResumoDivergente e PacoteIncompleto, num ArquivosEmMemoria.
        let ausente = Bancada::nova("recusa-ausente", ConsoleDeMentira::mudo());
        let divergente = com_o_pacote_em(
            "recusa-divergente",
            NA_PASTA_DE_DOWNLOADS,
            OUTRA_VERSAO,
            ConsoleDeMentira::mudo(),
        );
        let mut incompleto = com_o_pacote_em(
            "recusa-incompleto",
            NA_PASTA_DE_DOWNLOADS,
            pacote::SHA256,
            ConsoleDeMentira::mudo(),
        );
        incompleto.sistema = SistemaDeMentira::novo()
            .com_resumo(NA_PASTA_DE_DOWNLOADS, pacote::SHA256)
            .listando(0, "boot/grub/grub.cfg\nlive/vmlinuz\nlive/initrd.img\n");
        let mut pasta = Bancada::nova("recusa-pasta", ConsoleDeMentira::mudo());
        pasta.arquivos = ArquivosEmMemoria::novo().com_pasta_vazia(&pasta_do_iso);
        pasta.arquivos.espaco_livre = MUITO_ESPACO;

        for (bancada, caminho, esperada) in [
            (&ausente, &iso, "NaoEstaLa"),
            (&pasta, &pasta_do_iso, "EUmaPasta"),
            (&divergente, &iso, "ResumoDivergente"),
            (&incompleto, &iso, "PacoteIncompleto"),
        ] {
            let (pacote, _) = bancada.pre_voo(false, Some(caminho));
            let recusa = match pacote {
                Err(Erro::PacoteRecusado(recusa)) => recusa,
                outro => panic!("{esperada}: {outro:?}"),
            };
            assert!(
                format!("{recusa:?}").starts_with(esperada),
                "{esperada}: {recusa:?}"
            );
            assert_eq!(
                escritos_na_pasta(bancada.arquivos.escritos(), bancada.registro.pasta()),
                Vec::<PathBuf>::new(),
                "{esperada}"
            );
        }

        // NaoDeuParaOlhar, num ArquivosQueRecusam.
        let bancada = Bancada::nova("recusa-sem-olhar", ConsoleDeMentira::mudo());
        let arquivos = ArquivosQueRecusam::com(
            &iso,
            std::io::ErrorKind::PermissionDenied,
            "Acesso negado. (os error 5)",
        )
        .com_espaco_livre(MUITO_ESPACO);
        let contexto = Contexto {
            arquivos: &arquivos,
            ..bancada.contexto(false)
        };
        let pacote = obter_o_pacote_no_pre_voo(&contexto, Some(&iso), &mut |_| {});
        assert!(
            matches!(
                pacote,
                Err(Erro::PacoteRecusado(
                    pacote::RecusaDoPacote::NaoDeuParaOlhar { .. }
                ))
            ),
            "{pacote:?}"
        );
        assert_eq!(arquivos.escritos(), Vec::<PathBuf>::new());
    }

    #[test]
    fn o_ensaio_com_iso_nao_copia_e_diz_que_o_preparo_copia() {
        let iso = PathBuf::from(NA_PASTA_DE_DOWNLOADS);
        let copia_no_preparo = format!(
            "{}  Pacote guardado ................. o preparo de verdade copia o --iso para este PC; o ensaio nao escreve\n",
            linha_do_iso()
        );

        // Sem guardado, e com um que não bate: o preparo de verdade copiaria.
        for guardado_resume in [None, Some(OUTRA_VERSAO)] {
            let mut bancada = com_um_iso_que_passa("ensaio-iso", pacote::SHA256);
            if let Some(resumo) = guardado_resume {
                let guardado = bancada.guardado();
                bancada.arquivos = ArquivosEmMemoria::novo()
                    .com(&iso, "o zip, de mentira")
                    .com(&guardado, "o pacote que nao bate");
                bancada.arquivos.espaco_livre = MUITO_ESPACO;
                bancada.sistema = SistemaDeMentira::novo()
                    .com_resumo(NA_PASTA_DE_DOWNLOADS, pacote::SHA256)
                    .com_resumo(&guardado.to_string_lossy(), resumo);
            }

            let (pacote, tela) = bancada.pre_voo(true, Some(&iso));

            assert_eq!(pacote.unwrap(), PacoteDoPreVoo::DoIso(iso.clone()));
            assert_eq!(tela, copia_no_preparo, "{guardado_resume:?}");
            assert_eq!(
                bancada.arquivos.escritos(),
                Vec::<PathBuf>::new(),
                "{guardado_resume:?}: o ensaio escreveu"
            );
        }

        // Com um guardado que bate, a tela é a de antes.
        let mut bancada = com_um_iso_que_passa("ensaio-iso-bate", pacote::SHA256);
        let guardado = bancada.guardado();
        bancada.arquivos = ArquivosEmMemoria::novo()
            .com(&iso, "o zip, de mentira")
            .com(&guardado, O_PACOTE_GUARDADO);
        bancada.arquivos.espaco_livre = MUITO_ESPACO;
        bancada.sistema = SistemaDeMentira::novo()
            .com_resumo(NA_PASTA_DE_DOWNLOADS, pacote::SHA256)
            .com_resumo(&guardado.to_string_lossy(), pacote::SHA256);
        let (_, tela) = bancada.pre_voo(true, Some(&iso));
        assert_eq!(tela, linha_do_iso());
        assert_eq!(bancada.arquivos.escritos(), Vec::<PathBuf>::new());

        // Borda 6: onde o preparo de verdade não copiaria, o ensaio diz o
        // mesmo que ele diria — e não escreve aviso no `arca.log`.
        let mut bancada = com_um_iso_que_passa("ensaio-iso-sem-medida", pacote::SHA256);
        let guardado = bancada.guardado();
        bancada.arquivos = ArquivosEmMemoria::novo()
            .com(&iso, "o zip, de mentira")
            .com(&guardado, O_PACOTE_GUARDADO);
        bancada.arquivos.espaco_livre = MUITO_ESPACO;
        let (_, tela) = bancada.pre_voo(true, Some(&iso));
        assert_eq!(
            tela,
            format!(
                "{}  Pacote guardado ................. nao deu para medir o que ja esta guardado ({CERTUTIL_SEM_O_ARQUIVO}) · deixado como esta\n",
                linha_do_iso()
            )
        );
        assert_eq!(bancada.arquivos.escritos(), Vec::<PathBuf>::new());

        let mut bancada = com_um_iso_que_passa("ensaio-iso-pasta", pacote::SHA256);
        bancada.arquivos = ArquivosEmMemoria::novo()
            .com(&iso, "o zip, de mentira")
            .com_pasta_vazia(bancada.guardado());
        bancada.arquivos.espaco_livre = MUITO_ESPACO;
        let (_, tela) = bancada.pre_voo(true, Some(&iso));
        assert_eq!(
            tela,
            format!(
                "{}  Pacote guardado ................. nao deu para guardar em {} (`clonezilla-live-3.3.3-15-amd64.zip` e uma pasta) · este preparo segue do --iso\n",
                linha_do_iso(),
                bancada.registro.pasta().display()
            )
        );
        assert_eq!(bancada.arquivos.escritos(), Vec::<PathBuf>::new());
        assert!(
            !bancada.registrado().contains("nao foi guardado"),
            "{}",
            bancada.registrado()
        );
    }

    #[test]
    fn a_copia_do_iso_nao_mexe_nos_outros_arquivos_da_pasta() {
        use crate::portas::Arquivos as _;

        let mut bancada = com_um_iso_que_passa("iso-outros-arquivos", pacote::SHA256);
        let (pasta, guardado, parcial) = (
            bancada.registro.pasta().to_path_buf(),
            bancada.guardado(),
            bancada.parcial(),
        );
        let iso = PathBuf::from(NA_PASTA_DE_DOWNLOADS);
        let antigo = pasta.join("clonezilla-live-3.3.2-1-amd64.zip");
        let log = pasta.join("arca.log");
        bancada.arquivos = ArquivosEmMemoria::novo()
            .com(&iso, "o zip, de mentira")
            .com(&guardado, "o pacote que nao bate")
            .com(&antigo, "a versao anterior")
            .com(&log, "o registro");
        bancada.arquivos.espaco_livre = MUITO_ESPACO;
        bancada.sistema = SistemaDeMentira::novo()
            .com_resumo(NA_PASTA_DE_DOWNLOADS, pacote::SHA256)
            .com_resumo(&guardado.to_string_lossy(), OUTRA_VERSAO)
            .com_resumo(&parcial.to_string_lossy(), pacote::SHA256);

        let (pacote, _) = bancada.pre_voo(false, Some(&iso));

        assert!(pacote.is_ok(), "{pacote:?}");
        assert_eq!(
            bancada.arquivos.conteudo_de(&antigo).as_deref(),
            Some("a versao anterior")
        );
        assert_eq!(
            bancada.arquivos.conteudo_de(&log).as_deref(),
            Some("o registro")
        );
        let chaves: Vec<PathBuf> = bancada
            .arquivos
            .listar(&pasta)
            .unwrap()
            .into_iter()
            .map(|entrada| entrada.caminho)
            .collect();
        assert_eq!(
            chaves,
            vec![log, antigo, guardado.clone()],
            "sobrou ou sumiu arquivo na pasta"
        );
        assert!(
            escritos_na_pasta(bancada.arquivos.escritos(), &pasta)
                .iter()
                .all(|escrito| [&pasta, &parcial, &guardado].contains(&escrito)),
            "{:?}",
            bancada.arquivos.escritos()
        );
    }

    #[test]
    fn o_download_nao_mexe_nos_outros_arquivos_da_pasta() {
        use crate::portas::Arquivos as _;

        let mut bancada = Bancada::nova("outros-arquivos", ConsoleDeMentira::mudo());
        let (pasta, guardado, parcial) = (
            bancada.registro.pasta().to_path_buf(),
            bancada.guardado(),
            bancada.parcial(),
        );
        let antigo = pasta.join("clonezilla-live-3.3.2-1-amd64.zip");
        let log = pasta.join("arca.log");
        bancada.arquivos = ArquivosEmMemoria::novo()
            .com(&guardado, "o pacote que nao bate")
            .com(&antigo, "a versao anterior")
            .com(&log, "o registro");
        bancada.arquivos.espaco_livre = MUITO_ESPACO;
        let outra_versao = "1d76439d27aa20ec74a8e22c486dcfab67473b6fd6bbc7376a806fede0293b10";
        bancada.sistema = SistemaDeMentira::novo()
            .com_resumo(&guardado.to_string_lossy(), outra_versao)
            .com_resumo(&parcial.to_string_lossy(), pacote::SHA256);

        let (pacote, _) = bancada.pre_voo(false, None);

        assert!(pacote.is_ok(), "{pacote:?}");
        assert_eq!(
            bancada.arquivos.conteudo_de(&antigo).as_deref(),
            Some("a versao anterior")
        );
        assert_eq!(
            bancada.arquivos.conteudo_de(&log).as_deref(),
            Some("o registro")
        );
        let chaves: Vec<PathBuf> = bancada
            .arquivos
            .listar(&pasta)
            .unwrap()
            .into_iter()
            .map(|entrada| entrada.caminho)
            .collect();
        assert_eq!(
            chaves,
            vec![log, antigo, guardado],
            "sobrou ou sumiu arquivo na pasta"
        );
    }

    #[test]
    fn um_download_interrompido_nao_chega_ao_particionador() {
        let mut bancada =
            Bancada::nova("interrompido", ConsoleDeMentira::mudo()).sem_pacote_guardado();
        bancada.sistema = SistemaDeMentira::novo().recusando_o_download(
            pacote::FONTES[0].url,
            Erro::Ferramenta {
                ferramenta: "curl",
                origem: std::io::Error::other("interrompido"),
            },
        );

        let erro = executar(&bancada.contexto(false), Some(1), None).unwrap_err();

        // O duplo devolve uma cópia da recusa, e não a mesma variante; o que
        // importa é que a falha do `curl` sobe, e não uma recusa do pacote.
        assert!(erro.to_string().contains("interrompido"), "{erro}");
        assert!(!matches!(erro, Erro::PacoteRecusado(_)), "{erro}");
        assert_eq!(bancada.particionador.enumeracoes.get(), 0);
        assert!(!bancada.particionador.particionou());
        assert!(bancada.arquivos.conteudo_de(bancada.parcial()).is_some());
    }

    // ─────────── as fontes do pacote (PR-7) ───────────

    /// As escritas da linha viva da `n`-ésima fonte tentada: as que vêm
    /// depois da linha `Fonte` dela e antes do `\n` que a termina.
    fn linha_viva_da_fonte(escritas: &[String], n: usize) -> Vec<String> {
        let inicio = escritas
            .iter()
            .enumerate()
            .filter(|(_, escrita)| escrita.starts_with("  Fonte ."))
            .nth(n)
            .unwrap_or_else(|| panic!("a fonte {n} nao foi tentada: {escritas:?}"))
            .0
            + 1;
        let fim = inicio
            + escritas[inicio..]
                .iter()
                .position(|escrita| escrita == "\n")
                .expect("a linha viva termina num `\\n`");
        escritas[inicio..fim].to_vec()
    }

    /// O que uma escrita da linha viva mostra: ela sem os espaços do fim,
    /// cobrando que eles são exatamente os que cobrem a sobra da mais longa
    /// das anteriores. É o mecanismo da `LinhaViva` da WPC-134, e a borda 1
    /// do checklist da WPC-220: o literal de um critério é o que o console
    /// mostra.
    fn sem_a_sobra(linha_viva: &[String], indice: usize) -> String {
        let largura = |escrita: &str| {
            escrita
                .trim_start_matches('\r')
                .trim_end_matches(' ')
                .chars()
                .count()
        };
        let escrita = &linha_viva[indice];
        let visivel = escrita.trim_end_matches(' ');
        let maior_antes = linha_viva[..indice]
            .iter()
            .map(|anterior| largura(anterior))
            .max()
            .unwrap_or(0);
        assert_eq!(
            escrita.len() - visivel.len(),
            maior_antes.saturating_sub(largura(escrita)),
            "a sobra de {escrita:?} nao e a que cobre {linha_viva:?}"
        );
        visivel.to_string()
    }

    #[test]
    fn uma_fonte_que_comeca_do_zero_ganha_a_linha_fonte_e_a_barra_vazia() {
        let bancada =
            Bancada::nova("fonte-do-zero", ConsoleDeMentira::mudo()).sem_pacote_guardado();

        let (pacote, escritas) = bancada.pre_voo_escrita_a_escrita(false, None);

        assert!(pacote.is_ok(), "{pacote:?}");
        assert_eq!(
            escritas[..2],
            [
                "  Fonte ........................... 3.3.3-15 · free.nchc.org.tw (oldstable)\n",
                "  Baixando Clonezilla ............. [░░░░░░░░░░░░░░░░░░░░]   0% · 0,0 de 535,5 MB · medindo",
            ],
            "{escritas:?}"
        );
    }

    #[test]
    fn o_andamento_reescreve_a_linha_no_lugar() {
        // **A premissa deste teste mudou por decisão, e não para ficar
        // verde.** Até 23/10/2026 ele cobrava a linha
        // `3.3.3-15 · <recebido> de 535,5 MB · <rotulo>`, reescrita a cada
        // tamanho. Desde a WPC-220 a linha é a barra, a velocidade dos
        // últimos 20 s e quanto falta, e o duplo ensina amostras com tempo.
        // As amostras e o literal são os do critério 3.
        let mut bancada =
            Bancada::nova("andamento", ConsoleDeMentira::mudo()).sem_pacote_guardado();
        bancada.sistema = SistemaDeMentira::novo()
            .com_resumo(&bancada.parcial().to_string_lossy(), pacote::SHA256)
            .baixando_de(
                pacote::FONTES[0].url,
                crate::duplos::entregou(),
                &[
                    amostra(0.0, 0),
                    amostra(10.0, 10_485_760),
                    amostra(30.0, 115_343_360),
                ],
            );

        let (_, escritas) = bancada.pre_voo_escrita_a_escrita(false, None);
        let viva = linha_viva_da_fonte(&escritas, 0);

        // Uma antes das amostras, uma por amostra e a da entrega, todas na
        // mesma linha: só a primeira não volta ao começo dela.
        assert_eq!(viva.len(), 5, "{viva:?}");
        assert!(!viva[0].starts_with('\r'), "{viva:?}");
        assert!(
            viva[1..].iter().all(|escrita| escrita.starts_with('\r')),
            "{viva:?}"
        );
        assert!(
            viva.iter().all(|escrita| !escrita.contains('\n')),
            "{viva:?}"
        );
        assert_eq!(
            viva[3],
            "\r  Baixando Clonezilla ............. [████░░░░░░░░░░░░░░░░]  20% · 110,0 de 535,5 MB · 5,0 MB/s · faltam 2 min"
        );
    }

    #[test]
    fn a_linha_viva_diz_ha_quanto_tempo_a_fonte_parou() {
        let mut bancada =
            Bancada::nova("parada-ha", ConsoleDeMentira::mudo()).sem_pacote_guardado();
        bancada.sistema = SistemaDeMentira::novo()
            .com_resumo(&bancada.parcial().to_string_lossy(), pacote::SHA256)
            .baixando_de(
                pacote::FONTES[0].url,
                crate::duplos::entregou(),
                &[
                    amostra(0.0, 0),
                    amostra(10.0, 10_485_760),
                    amostra(15.0, 10_485_760),
                ],
            );

        let (_, escritas) = bancada.pre_voo_escrita_a_escrita(false, None);
        let viva = linha_viva_da_fonte(&escritas, 0);

        assert_eq!(
            sem_a_sobra(&viva, 3),
            "\r  Baixando Clonezilla ............. [░░░░░░░░░░░░░░░░░░░░]   1% · 10,0 de 535,5 MB · 0 B/s · parada ha 5 s"
        );
    }

    #[test]
    fn a_fonte_que_entrega_deixa_a_linha_em_100_por_cento() {
        let mut bancada =
            Bancada::nova("entrega-100", ConsoleDeMentira::mudo()).sem_pacote_guardado();
        bancada.sistema = SistemaDeMentira::novo()
            .com_resumo(&bancada.parcial().to_string_lossy(), pacote::SHA256)
            .baixando_de(
                pacote::FONTES[0].url,
                crate::duplos::entregou(),
                &[
                    amostra(0.0, 0),
                    amostra(90.0, 456_621_048),
                    amostra(110.0, pacote::TAMANHO_BYTES),
                ],
            );

        let (pacote, escritas) = bancada.pre_voo_escrita_a_escrita(false, None);
        assert!(pacote.is_ok(), "{pacote:?}");
        let viva = linha_viva_da_fonte(&escritas, 0);

        let ultima = sem_a_sobra(&viva, viva.len() - 1);
        assert_eq!(
            ultima,
            "\r  Baixando Clonezilla ............. [████████████████████] 100% · 535,5 de 535,5 MB · 5,0 MB/s"
        );
        assert!(!viva.last().unwrap().contains("faltam"), "{viva:?}");
        // Logo depois da última escrita da linha viva vem o `\n` que a
        // termina, e nenhuma escrita dela tem outro.
        assert!(
            viva.iter().all(|escrita| !escrita.contains('\n')),
            "{viva:?}"
        );
        let fonte = escritas
            .iter()
            .position(|escrita| escrita.starts_with("  Fonte ."))
            .unwrap();
        assert_eq!(escritas[fonte + 1 + viva.len()], "\n");
    }

    #[test]
    fn uma_fonte_que_nao_responde_e_abandonada_e_a_proxima_e_tentada() {
        let mut bancada =
            Bancada::nova("nao-responde", ConsoleDeMentira::mudo()).sem_pacote_guardado();
        bancada.sistema = SistemaDeMentira::novo()
            .com_resumo(&bancada.parcial().to_string_lossy(), pacote::SHA256)
            .baixando_de(pacote::FONTES[0].url, nao_respondeu(0), &[]);

        let (pacote, tela) = bancada.pre_voo(false, None);

        assert!(pacote.is_ok(), "{pacote:?}");
        assert!(
            tela.contains(
                "\n  Fonte abandonada ................ free.nchc.org.tw (oldstable) · nao respondeu em 30 s\n"
            ),
            "{tela}"
        );
        assert_eq!(
            *bancada.sistema.baixados.borrow(),
            vec![pacote::FONTES[0].url, pacote::FONTES[1].url]
        );
    }

    #[test]
    fn uma_fonte_que_para_e_abandonada_e_a_proxima_continua_do_mesmo_parcial() {
        let mut bancada = Bancada::nova("para", ConsoleDeMentira::mudo()).sem_pacote_guardado();
        let parcial = bancada.parcial();
        bancada.arquivos =
            ArquivosEmMemoria::novo().com_tamanho(&parcial, "o que ja tinha vindo", 1024);
        bancada.arquivos.espaco_livre = MUITO_ESPACO;
        bancada.sistema = SistemaDeMentira::novo()
            .com_resumo(&parcial.to_string_lossy(), pacote::SHA256)
            .baixando_de(pacote::FONTES[0].url, parou(CENTO_E_VINTE_MB), &[]);

        let (pacote, tela) = bancada.pre_voo(false, None);

        assert!(pacote.is_ok(), "{pacote:?}");
        assert!(
            tela.contains(
                "\n  Fonte abandonada ................ free.nchc.org.tw (oldstable) · parou de mandar por 60 s, com 120,0 MB de 535,5 MB\n"
            ),
            "{tela}"
        );
        // A retomada saiu da linha viva para a linha `Fonte` (WPC-220).
        assert!(
            tela.contains(
                "  Fonte ........................... 3.3.3-15 · free.nchc.org.tw (stable) · retomando de 120,0 MB\n"
            ),
            "{tela}"
        );
        assert_eq!(
            *bancada.sistema.destinos.borrow(),
            vec![parcial.clone(), parcial]
        );
        assert_eq!(
            bancada.arquivos.conteudo_de(bancada.guardado()).as_deref(),
            Some("o que ja tinha vindo"),
            "o `.parcial` foi zerado entre as duas fontes"
        );
    }

    #[test]
    fn uma_fonte_com_erro_http_e_abandonada_e_a_proxima_e_tentada() {
        let mut bancada = Bancada::nova("http", ConsoleDeMentira::mudo()).sem_pacote_guardado();
        bancada.sistema = SistemaDeMentira::novo()
            .com_resumo(&bancada.parcial().to_string_lossy(), pacote::SHA256)
            .baixando_de(pacote::FONTES[0].url, nao_achou(0), &[]);

        let (pacote, tela) = bancada.pre_voo(false, None);

        assert!(pacote.is_ok(), "{pacote:?}");
        assert!(
            tela.contains(
                "\n  Fonte abandonada ................ free.nchc.org.tw (oldstable) · HTTP 404\n"
            ),
            "{tela}"
        );
        assert_eq!(bancada.sistema.baixados.borrow()[1], pacote::FONTES[1].url);
    }

    #[test]
    fn cada_fonte_tentada_fica_no_arca_log() {
        let mut bancada = Bancada::nova("log", ConsoleDeMentira::mudo()).sem_pacote_guardado();
        let [primeira, segunda, terceira] = pacote::FONTES;
        bancada.sistema = SistemaDeMentira::novo()
            .com_resumo(&bancada.parcial().to_string_lossy(), pacote::SHA256)
            .baixando_de(primeira.url, nao_respondeu(0), &[])
            .baixando_de(segunda.url, parou(CENTO_E_VINTE_MB), &[])
            .baixando_de(terceira.url, crate::duplos::entregou(), &[]);

        let (pacote, _) = bancada.pre_voo(false, None);
        assert!(pacote.is_ok(), "{pacote:?}");

        let registrado = bancada.registrado();
        let linhas: Vec<&str> = registrado
            .lines()
            .filter(|l| l.contains("fonte https://"))
            .collect();
        let esperadas = [
            format!(
                "fonte {} · nao respondeu em 30 s · 0 bytes desta fonte",
                primeira.url
            ),
            format!(
                "fonte {} · parou de mandar por 60 s, com 120,0 MB de 535,5 MB · 125829120 bytes desta fonte",
                segunda.url
            ),
            format!(
                "fonte {} · entregou · 435649528 bytes desta fonte",
                terceira.url
            ),
        ];
        assert_eq!(linhas.len(), 3, "{registrado}");
        for (linha, esperada) in linhas.iter().zip(&esperadas) {
            assert!(
                linha.ends_with(esperada.as_str()),
                "{linha}\nesperava: {esperada}"
            );
        }
    }

    // ─────────── a fonte lenta e a segunda volta (WPC-220) ───────────

    /// 1.310.720 bytes: o que uma fonte a 64,0 KB/s manda em 20 s.
    const VINTE_SEGUNDOS_A_64_KB: u64 = 1_310_720;

    /// Uma fonte que saiu com o texto do `curl`: sem nome que resolva.
    fn nao_resolveu(no_destino: u64) -> crate::portas::Transferencia {
        crate::portas::Transferencia {
            codigo: 6,
            texto: "curl: (6) Could not resolve host: downloads.sourceforge.net".to_string(),
            http: None,
            parou: false,
            encerrada: false,
            tamanho_no_destino: no_destino,
        }
    }

    /// A primeira volta em que as três saem por lentas: oldstable a 64,0
    /// KB/s, stable sem nada nos últimos 20 s, e o SourceForge a 128,0 KB/s.
    /// O `.parcial` termina a volta com 3.932.160 bytes. O que se ensina é a
    /// segunda volta de cada uma, na ordem de `FONTES`: a resposta e as
    /// amostras.
    fn tres_lentas(
        sistema: SistemaDeMentira,
        segunda_volta: [(crate::portas::Transferencia, &[crate::portas::Amostra]); 3],
    ) -> SistemaDeMentira {
        let [oldstable, stable, sourceforge] = pacote::FONTES;
        let [
            (de_oldstable, amostras_de_oldstable),
            (de_stable, amostras_de_stable),
            (de_sourceforge, amostras_de_sourceforge),
        ] = segunda_volta;
        sistema
            .baixando_de(
                oldstable.url,
                crate::duplos::entregou(),
                &[amostra(20.0, VINTE_SEGUNDOS_A_64_KB)],
            )
            .baixando_de(oldstable.url, de_oldstable, amostras_de_oldstable)
            .baixando_de(
                stable.url,
                crate::duplos::entregou(),
                &[amostra(20.0, VINTE_SEGUNDOS_A_64_KB)],
            )
            .baixando_de(stable.url, de_stable, amostras_de_stable)
            .baixando_de(
                sourceforge.url,
                crate::duplos::entregou(),
                &[amostra(20.0, DEPOIS_DAS_TRES_LENTAS)],
            )
            .baixando_de(sourceforge.url, de_sourceforge, amostras_de_sourceforge)
    }

    /// O que o `.parcial` tem depois da primeira volta de [`tres_lentas`].
    const DEPOIS_DAS_TRES_LENTAS: u64 = 3 * VINTE_SEGUNDOS_A_64_KB;

    #[test]
    fn uma_fonte_lenta_perde_a_vez_e_a_seguinte_continua_do_mesmo_parcial() {
        let mut bancada = Bancada::nova("lenta", ConsoleDeMentira::mudo()).sem_pacote_guardado();
        let (pasta, parcial, guardado) = (
            bancada.registro.pasta().to_path_buf(),
            bancada.parcial(),
            bancada.guardado(),
        );
        let [oldstable, stable, _] = pacote::FONTES;
        bancada.sistema = SistemaDeMentira::novo()
            .com_resumo(&parcial.to_string_lossy(), pacote::SHA256)
            .baixando_de(
                oldstable.url,
                crate::duplos::entregou(),
                &[amostra(0.0, 0), amostra(20.0, VINTE_SEGUNDOS_A_64_KB)],
            );

        let (pacote, tela) = bancada.pre_voo(false, None);

        assert!(pacote.is_ok(), "{pacote:?}");
        assert_eq!(*bancada.sistema.encerrados.borrow(), vec![oldstable.url]);
        assert!(
            tela.contains(
                "\n  Fonte abandonada ................ free.nchc.org.tw (oldstable) · lenta: faltariam 2 h 23 min a 64,0 KB/s\n"
            ),
            "{tela}"
        );
        // A tela mostra o mesmo número que decide (Key decision 3).
        assert!(tela.contains(" · 64,0 KB/s · faltam 2 h 23 min"), "{tela}");
        assert!(
            tela.contains(
                "  Fonte ........................... 3.3.3-15 · free.nchc.org.tw (stable) · retomando de 1,2 MB\n"
            ),
            "{tela}"
        );
        assert_eq!(
            *bancada.sistema.baixados.borrow(),
            vec![oldstable.url, stable.url]
        );
        assert_eq!(
            *bancada.sistema.destinos.borrow(),
            vec![parcial.clone(), parcial.clone()]
        );
        // Zerado só no começo, porque não existia; entre as duas fontes,
        // nada o toca.
        assert_eq!(
            bancada.arquivos.escritos(),
            vec![pasta, parcial.clone(), parcial, guardado]
        );
    }

    #[test]
    fn uma_fonte_com_menos_de_20_s_nao_sai_por_lenta() {
        let mut bancada =
            Bancada::nova("menos-de-20", ConsoleDeMentira::mudo()).sem_pacote_guardado();
        bancada.sistema = SistemaDeMentira::novo()
            .com_resumo(&bancada.parcial().to_string_lossy(), pacote::SHA256)
            .baixando_de(
                pacote::FONTES[0].url,
                crate::duplos::entregou(),
                &[amostra(5.0, 1), amostra(10.0, 2), amostra(19.0, 3)],
            );

        let (pacote, tela) = bancada.pre_voo(false, None);

        assert!(pacote.is_ok(), "{pacote:?}");
        assert!(bancada.sistema.encerrados.borrow().is_empty());
        assert!(!tela.contains("Fonte abandonada"), "{tela}");
        assert_eq!(
            *bancada.sistema.baixados.borrow(),
            vec![pacote::FONTES[0].url]
        );
    }

    #[test]
    fn uma_fonte_lenta_perto_do_fim_vai_ate_o_fim() {
        let mut bancada =
            Bancada::nova("perto-do-fim", ConsoleDeMentira::mudo()).sem_pacote_guardado();
        let parcial = bancada.parcial();
        bancada.arquivos =
            ArquivosEmMemoria::novo().com_tamanho(&parcial, "quase tudo", 522_977_280);
        bancada.arquivos.espaco_livre = MUITO_ESPACO;
        bancada.sistema = SistemaDeMentira::novo()
            .com_resumo(&parcial.to_string_lossy(), pacote::SHA256)
            .baixando_de(
                pacote::FONTES[0].url,
                crate::duplos::entregou(),
                &[amostra(20.0, 524_288_000)],
            );

        let (pacote, tela) = bancada.pre_voo(false, None);

        assert!(pacote.is_ok(), "{pacote:?}");
        assert!(!tela.contains("Fonte abandonada"), "{tela}");
        assert!(tela.contains(" · 64,0 KB/s · faltam 10 min"), "{tela}");
        assert_eq!(
            *bancada.sistema.baixados.borrow(),
            vec![pacote::FONTES[0].url]
        );
    }

    #[test]
    fn uma_fonte_que_nada_mandou_em_20_s_sai_por_lenta() {
        let mut bancada =
            Bancada::nova("nada-em-20", ConsoleDeMentira::mudo()).sem_pacote_guardado();
        bancada.sistema = SistemaDeMentira::novo()
            .com_resumo(&bancada.parcial().to_string_lossy(), pacote::SHA256)
            .baixando_de(
                pacote::FONTES[0].url,
                crate::duplos::entregou(),
                &[amostra(10.0, 0), amostra(20.0, 0)],
            );

        let (pacote, tela) = bancada.pre_voo(false, None);

        assert!(pacote.is_ok(), "{pacote:?}");
        assert!(
            tela.contains(
                "\n  Fonte abandonada ................ free.nchc.org.tw (oldstable) · lenta: nada veio nos ultimos 20 s\n"
            ),
            "{tela}"
        );
        assert_eq!(bancada.sistema.baixados.borrow()[1], pacote::FONTES[1].url);
    }

    #[test]
    fn todas_lentas_a_mais_rapida_medida_volta_na_segunda_volta() {
        let mut bancada =
            Bancada::nova("segunda-volta", ConsoleDeMentira::mudo()).sem_pacote_guardado();
        let [oldstable, stable, sourceforge] = pacote::FONTES;
        // Na segunda volta, o SourceForge manda a 64 KB/s, o que na primeira
        // o tiraria aos 20 s, e vai até o fim.
        bancada.sistema = tres_lentas(
            SistemaDeMentira::novo()
                .com_resumo(&bancada.parcial().to_string_lossy(), pacote::SHA256),
            [
                (nao_respondeu(DEPOIS_DAS_TRES_LENTAS), &[]),
                (nao_respondeu(DEPOIS_DAS_TRES_LENTAS), &[]),
                (
                    crate::duplos::entregou(),
                    &[
                        amostra(20.0, DEPOIS_DAS_TRES_LENTAS + VINTE_SEGUNDOS_A_64_KB),
                        amostra(40.0, DEPOIS_DAS_TRES_LENTAS + 2 * VINTE_SEGUNDOS_A_64_KB),
                    ],
                ),
            ],
        );

        let (pacote, tela) = bancada.pre_voo(false, None);

        assert!(pacote.is_ok(), "{pacote:?}");
        assert_eq!(
            *bancada.sistema.baixados.borrow(),
            vec![oldstable.url, stable.url, sourceforge.url, sourceforge.url]
        );
        assert_eq!(
            *bancada.sistema.encerrados.borrow(),
            vec![oldstable.url, stable.url, sourceforge.url],
            "a segunda volta pediu para encerrar"
        );
        assert!(
            tela.contains(
                "  Fonte ........................... 3.3.3-15 · downloads.sourceforge.net · segunda volta · retomando de 3,8 MB\n"
            ),
            "{tela}"
        );
    }

    #[test]
    fn na_segunda_volta_o_empate_segue_a_ordem_das_fontes() {
        let mut bancada = Bancada::nova("empate", ConsoleDeMentira::mudo()).sem_pacote_guardado();
        let [oldstable, stable, sourceforge] = pacote::FONTES;
        bancada.sistema = SistemaDeMentira::novo()
            .com_resumo(&bancada.parcial().to_string_lossy(), pacote::SHA256)
            .baixando_de(
                oldstable.url,
                crate::duplos::entregou(),
                &[amostra(20.0, VINTE_SEGUNDOS_A_64_KB)],
            )
            .baixando_de(oldstable.url, parou(2 * VINTE_SEGUNDOS_A_64_KB), &[])
            .baixando_de(stable.url, nao_achou(VINTE_SEGUNDOS_A_64_KB), &[])
            .baixando_de(
                sourceforge.url,
                crate::duplos::entregou(),
                &[amostra(20.0, 2 * VINTE_SEGUNDOS_A_64_KB)],
            )
            .baixando_de(sourceforge.url, parou(2 * VINTE_SEGUNDOS_A_64_KB), &[]);

        let (pacote, _) = bancada.pre_voo(false, None);

        assert!(pacote.is_err(), "{pacote:?}");
        assert_eq!(
            *bancada.sistema.baixados.borrow(),
            vec![
                oldstable.url,
                stable.url,
                sourceforge.url,
                oldstable.url,
                sourceforge.url
            ]
        );
    }

    #[test]
    fn na_segunda_volta_a_fonte_que_falha_da_a_vez_a_seguinte() {
        let mut bancada =
            Bancada::nova("segunda-falha", ConsoleDeMentira::mudo()).sem_pacote_guardado();
        let [oldstable, stable, sourceforge] = pacote::FONTES;
        bancada.sistema = tres_lentas(
            SistemaDeMentira::novo(),
            [
                (nao_respondeu(DEPOIS_DAS_TRES_LENTAS), &[]),
                (nao_achou(DEPOIS_DAS_TRES_LENTAS), &[]),
                (parou(DEPOIS_DAS_TRES_LENTAS), &[]),
            ],
        );

        let (pacote, tela) = bancada.pre_voo(false, None);

        assert!(
            matches!(
                pacote,
                Err(Erro::PacoteRecusado(
                    pacote::RecusaDoPacote::NenhumaFonteEntregou { .. }
                ))
            ),
            "{pacote:?}"
        );
        assert_eq!(
            *bancada.sistema.baixados.borrow(),
            vec![
                oldstable.url,
                stable.url,
                sourceforge.url,
                sourceforge.url,
                oldstable.url,
                stable.url
            ]
        );
        let segunda_volta = &tela[tela.find("segunda volta").expect("nao houve segunda volta")..];
        let linhas = [
            "  Fonte abandonada ................ downloads.sourceforge.net · parou de mandar por 60 s, com 3,8 MB de 535,5 MB\n".to_string(),
            "  Fonte abandonada ................ free.nchc.org.tw (oldstable) · nao respondeu em 30 s\n".to_string(),
            "  Fonte abandonada ................ free.nchc.org.tw (stable) · HTTP 404\n".to_string(),
        ];
        let posicoes: Vec<usize> = linhas
            .iter()
            .map(|linha| {
                segunda_volta
                    .find(linha.as_str())
                    .unwrap_or_else(|| panic!("falta {linha:?} em {segunda_volta}"))
            })
            .collect();
        assert!(posicoes.windows(2).all(|par| par[0] < par[1]), "{tela}");
    }

    #[test]
    fn a_fonte_que_saiu_sem_ser_lenta_nao_volta() {
        let [oldstable, stable, sourceforge] = pacote::FONTES;

        // Nenhuma saiu por lenta: não há segunda volta.
        let mut bancada =
            Bancada::nova("sem-segunda-volta", ConsoleDeMentira::mudo()).sem_pacote_guardado();
        bancada.sistema = SistemaDeMentira::novo()
            .baixando_de(oldstable.url, nao_respondeu(0), &[])
            .baixando_de(stable.url, nao_achou(0), &[])
            .baixando_de(sourceforge.url, nao_resolveu(0), &[]);

        let (pacote, _) = bancada.pre_voo(false, None);

        assert!(pacote.is_err(), "{pacote:?}");
        assert_eq!(
            *bancada.sistema.baixados.borrow(),
            vec![oldstable.url, stable.url, sourceforge.url]
        );

        // Só a lenta volta.
        let mut bancada =
            Bancada::nova("so-a-lenta-volta", ConsoleDeMentira::mudo()).sem_pacote_guardado();
        bancada.sistema = SistemaDeMentira::novo()
            .baixando_de(
                oldstable.url,
                crate::duplos::entregou(),
                &[amostra(20.0, VINTE_SEGUNDOS_A_64_KB)],
            )
            .baixando_de(oldstable.url, parou(VINTE_SEGUNDOS_A_64_KB), &[])
            .baixando_de(stable.url, nao_achou(VINTE_SEGUNDOS_A_64_KB), &[])
            .baixando_de(sourceforge.url, nao_respondeu(VINTE_SEGUNDOS_A_64_KB), &[]);

        let (pacote, _) = bancada.pre_voo(false, None);

        assert!(pacote.is_err(), "{pacote:?}");
        assert_eq!(
            *bancada.sistema.baixados.borrow(),
            vec![oldstable.url, stable.url, sourceforge.url, oldstable.url]
        );
    }

    #[test]
    fn a_recusa_lista_cada_fonte_com_a_ultima_resposta() {
        let mut bancada =
            Bancada::nova("recusa-por-fonte", ConsoleDeMentira::mudo()).sem_pacote_guardado();
        bancada.sistema = tres_lentas(
            SistemaDeMentira::novo(),
            [
                (nao_respondeu(DEPOIS_DAS_TRES_LENTAS), &[]),
                (nao_achou(DEPOIS_DAS_TRES_LENTAS), &[]),
                (parou(DEPOIS_DAS_TRES_LENTAS), &[]),
            ],
        );

        let erro = executar(&bancada.contexto(false), Some(1), None).unwrap_err();

        assert!(
            matches!(
                erro,
                Erro::PacoteRecusado(pacote::RecusaDoPacote::NenhumaFonteEntregou { .. })
            ),
            "{erro}"
        );
        // Uma linha por fonte, na ordem de `FONTES`, com a resposta da
        // segunda volta — e não uma por tentativa, que seriam seis.
        assert_eq!(
            erro.to_string(),
            "nao deu para baixar o Clonezilla 3.3.3-15 de nenhuma das 3 fontes:\n  \
             free.nchc.org.tw (oldstable) — nao respondeu em 30 s\n  \
             free.nchc.org.tw (stable) — HTTP 404\n  \
             downloads.sourceforge.net — parou de mandar por 60 s, com 3,8 MB de 535,5 MB\n\
             O que ja veio fica guardado, e a proxima tentativa continua de onde parou. Para nao depender do download do ARCA, baixe pelo navegador\n  \
             https://free.nchc.org.tw/clonezilla-live/oldstable/3.3.3-15/clonezilla-live-3.3.3-15-amd64.zip\n\
             e passe o arquivo com --iso. Nada foi apagado (PR-7)"
        );
        assert_eq!(bancada.particionador.enumeracoes.get(), 0);
    }

    #[test]
    fn a_fonte_lenta_fica_no_arca_log_nas_duas_voltas() {
        let mut bancada =
            Bancada::nova("log-das-voltas", ConsoleDeMentira::mudo()).sem_pacote_guardado();
        let [oldstable, stable, sourceforge] = pacote::FONTES;
        bancada.sistema = SistemaDeMentira::novo()
            .com_resumo(&bancada.parcial().to_string_lossy(), pacote::SHA256)
            .baixando_de(
                oldstable.url,
                crate::duplos::entregou(),
                &[amostra(20.0, VINTE_SEGUNDOS_A_64_KB)],
            )
            .baixando_de(oldstable.url, crate::duplos::entregou(), &[])
            .baixando_de(
                stable.url,
                crate::duplos::entregou(),
                &[amostra(20.0, VINTE_SEGUNDOS_A_64_KB)],
            )
            .baixando_de(sourceforge.url, nao_respondeu(VINTE_SEGUNDOS_A_64_KB), &[]);

        let (pacote, _) = bancada.pre_voo(false, None);
        assert!(pacote.is_ok(), "{pacote:?}");

        let registrado = bancada.registrado();
        let linhas: Vec<&str> = registrado
            .lines()
            .filter(|linha| linha.contains("fonte https://"))
            .collect();
        let esperadas = [
            format!(
                "fonte {} · lenta: faltariam 2 h 23 min a 64,0 KB/s · 1310720 bytes desta fonte",
                oldstable.url
            ),
            format!(
                "fonte {} · lenta: nada veio nos ultimos 20 s · 0 bytes desta fonte",
                stable.url
            ),
            format!(
                "fonte {} · nao respondeu em 30 s · 0 bytes desta fonte",
                sourceforge.url
            ),
            format!(
                "fonte {} · entregou · 560167928 bytes desta fonte",
                oldstable.url
            ),
        ];
        assert_eq!(linhas.len(), 4, "{registrado}");
        for (linha, esperada) in linhas.iter().zip(&esperadas) {
            assert!(
                linha.ends_with(esperada.as_str()),
                "{linha}\nesperava: {esperada}"
            );
        }
    }

    #[test]
    fn nenhuma_fonte_e_tentada_mais_de_duas_vezes() {
        let mut bancada =
            Bancada::nova("duas-vezes", ConsoleDeMentira::mudo()).sem_pacote_guardado();
        bancada.sistema = tres_lentas(
            SistemaDeMentira::novo(),
            [
                (nao_resolveu(DEPOIS_DAS_TRES_LENTAS), &[]),
                (nao_resolveu(DEPOIS_DAS_TRES_LENTAS), &[]),
                (nao_resolveu(DEPOIS_DAS_TRES_LENTAS), &[]),
            ],
        );

        let (pacote, _) = bancada.pre_voo(false, None);

        assert!(pacote.is_err(), "{pacote:?}");
        let baixados = bancada.sistema.baixados.borrow();
        assert_eq!(baixados.len(), 6, "{baixados:?}");
        let primeira_volta: Vec<&str> = pacote::FONTES.iter().map(|fonte| fonte.url).collect();
        assert_eq!(baixados[..3], primeira_volta[..]);
        for fonte in pacote::FONTES {
            assert_eq!(
                baixados.iter().filter(|url| **url == fonte.url).count(),
                2,
                "{}: {baixados:?}",
                fonte.rotulo
            );
        }
    }

    // ─────────── uma pasta no `--iso`, e o pacote que ela pode ter dentro ───────────

    /// A pasta onde o pacote costuma estar — e o erro mais provável de quem o
    /// baixou é passá-la no lugar dele.
    const A_PASTA_DE_DOWNLOADS: &str = r"C:\Users\Ana Paula\Downloads";

    #[test]
    fn uma_pasta_no_iso_sem_o_pacote_dentro_recusa_no_pre_voo() {
        // Medido em 28/09/2026 (WPC-64): a pasta passava pelo `existe` do
        // pré-voo e chegava ao `certutil`, que responde para ela o mesmo
        // `0x80070002` de um arquivo ausente. Aqui a pasta tem o ISO, e não o
        // zip — o nome da flag convida ao engano —, e a recusa diz que o
        // arquivo que o `--iso` nomeia não está ali.
        let mut bancada = Bancada::nova("iso-pasta-sem-pacote", ConsoleDeMentira::mudo());
        bancada.arquivos = ArquivosEmMemoria::novo().com(
            r"C:\Users\Ana Paula\Downloads\clonezilla-live-3.3.3-15-amd64.iso",
            "o iso, de mentira",
        );

        let erro = executar(
            &bancada.contexto(false),
            Some(1),
            Some(Path::new(A_PASTA_DE_DOWNLOADS)),
        )
        .unwrap_err();

        assert!(
            matches!(
                erro,
                Erro::PacoteRecusado(pacote::RecusaDoPacote::EUmaPasta { dentro: None, .. })
            ),
            "{erro}"
        );
        let dito = erro.to_string();
        for trecho in [
            r"C:\Users\Ana Paula\Downloads",
            "e uma pasta",
            "clonezilla-live-3.3.3-15-amd64.zip",
            "nao esta nesta pasta",
            "Nada foi apagado",
        ] {
            assert!(dito.contains(trecho), "falta `{trecho}`: {dito}");
        }

        // O `certutil` não é chamado, e o sintoma dele não chega à tela.
        assert!(
            bancada.sistema.resumidos().is_empty(),
            "a pasta foi mandada ao certutil: {:?}",
            bancada.sistema.resumidos()
        );
        assert!(!dito.contains("0x80070002"), "{dito}");
        assert!(
            !bancada.particionador.particionou(),
            "a pasta no `--iso` apagou o disco"
        );
    }

    #[test]
    fn uma_pasta_no_iso_com_o_pacote_dentro_aponta_o_arquivo_e_para() {
        // O pacote certo está dentro da pasta, e o ARCA sabe disso. Ele dá o
        // caminho e para: o `--iso` nomeia o arquivo (PR-2), e seguir com o de
        // dentro seria deduzir (ADR-0024). A bancada está montada para que
        // seguir custasse o disco — o `certutil` responde o SHA256 certo para
        // o zip, e o console responde a pergunta e o modelo.
        let bancada = com_o_pacote_em(
            "iso-pasta-com-pacote",
            NA_PASTA_DE_DOWNLOADS,
            pacote::SHA256,
            ConsoleDeMentira::respondendo(&["s", "JMicron Generic"]),
        );

        let erro = executar(
            &bancada.contexto(false),
            Some(1),
            Some(Path::new(A_PASTA_DE_DOWNLOADS)),
        )
        .unwrap_err();

        assert!(
            matches!(
                erro,
                Erro::PacoteRecusado(pacote::RecusaDoPacote::EUmaPasta {
                    dentro: Some(_),
                    ..
                })
            ),
            "{erro}"
        );
        let dito = erro.to_string();
        for trecho in [
            "e uma pasta",
            r"C:\Users\Ana Paula\Downloads\clonezilla-live-3.3.3-15-amd64.zip",
            "Nada foi apagado",
        ] {
            assert!(dito.contains(trecho), "falta `{trecho}`: {dito}");
        }

        // Nem o de dentro vai ao `certutil`: apontar não é conferir.
        assert!(
            bancada.sistema.resumidos().is_empty(),
            "o pacote apontado foi mandado ao certutil: {:?}",
            bancada.sistema.resumidos()
        );
        assert!(!dito.contains("0x80070002"), "{dito}");

        // E o comando termina na recusa.
        assert_eq!(bancada.console.lidas.get(), 0, "seguiu para a pergunta");
        assert!(
            !bancada.particionador.particionou(),
            "seguiu com o pacote de dentro e apagou o disco"
        );
        assert!(
            bancada.sistema.extraidos.borrow().is_empty(),
            "extraiu o pacote de dentro"
        );
    }

    #[test]
    fn uma_pasta_com_o_nome_do_pacote_dentro_nao_e_apontada() {
        // "Como arquivo" é metade da regra. Uma pasta que tem o nome do pacote
        // não é o pacote, e apontá-la mandaria a pessoa passar outra pasta ao
        // `--iso` — a recusa seguinte seria esta mesma, e o conselho andaria em
        // círculo.
        let mut bancada = Bancada::nova("iso-pasta-com-pasta", ConsoleDeMentira::mudo());
        bancada.arquivos = ArquivosEmMemoria::novo().com(
            r"C:\Users\Ana Paula\Downloads\clonezilla-live-3.3.3-15-amd64.zip\live\vmlinuz",
            "o kernel, de mentira",
        );

        let erro = executar(
            &bancada.contexto(false),
            Some(1),
            Some(Path::new(A_PASTA_DE_DOWNLOADS)),
        )
        .unwrap_err();

        assert!(
            matches!(
                erro,
                Erro::PacoteRecusado(pacote::RecusaDoPacote::EUmaPasta { dentro: None, .. })
            ),
            "{erro}"
        );
        assert!(erro.to_string().contains("nao esta nesta pasta"), "{erro}");
    }

    #[test]
    fn uma_pasta_no_iso_sem_dispositivo_recusa_antes_do_menu() {
        // O menu do passo 0 é a primeira leitura do console, e a pasta tem de
        // ser recusada antes dela. Este console responderia o menu, a pergunta
        // e o modelo até apagar o disco 1.
        let console = || ConsoleDeMentira::respondendo(&["1", "s", "JMicron Generic"]);

        let mut bancada = Bancada::nova("iso-pasta-sem-dispositivo", console());
        bancada.arquivos =
            ArquivosEmMemoria::novo().com(NA_PASTA_DE_DOWNLOADS, "o zip, de mentira");

        let erro = executar(
            &bancada.contexto(false),
            None,
            Some(Path::new(A_PASTA_DE_DOWNLOADS)),
        )
        .unwrap_err();

        assert!(
            matches!(
                erro,
                Erro::PacoteRecusado(pacote::RecusaDoPacote::EUmaPasta { .. })
            ),
            "{erro}"
        );
        assert_eq!(
            bancada.console.lidas.get(),
            0,
            "o menu leu antes da recusa da pasta"
        );
        assert!(
            !bancada.particionador.particionou(),
            "a pasta no `--iso` apagou o disco escolhido no menu"
        );

        // O controle: sem `--iso`, o mesmo console é lido pelo menu. Sem ele,
        // o zero acima não distinguiria "recusou antes do menu" de "o menu não
        // leu nada".
        let controle = Bancada::nova("iso-pasta-sem-dispositivo-controle", console());
        let _ = executar(&controle.contexto(false), None, None);
        assert!(
            controle.console.lidas.get() >= 1,
            "sem `--iso` o menu nao leu nada"
        );
    }

    #[test]
    fn depois_do_ponto_sem_volta_uma_pasta_cai_no_certutil() {
        // A metade da regra que o pré-voo não alcança. `EUmaPasta` promete que
        // nada foi apagado, e só pode prometer porque nasce antes do passo 0.
        // Os passos 6 e 7 não perguntam por pasta: um `--iso` que virasse pasta
        // depois do pré-voo cai no `certutil`, como um que sumisse, e a frase
        // dele não promete nada — ali o disco já foi.
        let mut bancada = Bancada::nova("iso-pasta-no-passo-7", ConsoleDeMentira::mudo());
        bancada.arquivos =
            ArquivosEmMemoria::novo().com(NA_PASTA_DE_DOWNLOADS, "o zip, de mentira");

        let erro = obter_o_pacote(
            &bancada.contexto(false),
            Path::new(A_PASTA_DE_DOWNLOADS),
            Path::new(r"E:\"),
        )
        .unwrap_err();

        assert!(
            matches!(
                erro,
                Erro::PacoteRecusado(pacote::RecusaDoPacote::NaoDeuParaResumir(_))
            ),
            "{erro}"
        );
        assert!(!erro.to_string().contains("Nada foi apagado"), "{erro}");
        assert_eq!(
            bancada.sistema.resumidos(),
            vec![PathBuf::from(r"C:\Users\Ana Paula\Downloads")]
        );
        assert!(
            !bancada.arquivos.foi_consultado(A_PASTA_DE_DOWNLOADS),
            "o passo 7 perguntou pela pasta"
        );
    }

    // ─────────── o `--iso` que o Windows não deixa olhar ───────────

    #[test]
    fn um_iso_que_nao_se_deixa_olhar_nao_sai_como_ausente() {
        // "Não consegui olhar" nunca vira "não há nada lá". Até 29/09/2026
        // (WPC-68) o pré-voo perguntava com `Path::exists`, que responde
        // `false` quando os metadados não se deixam ler, e a recusa era
        // `NaoEstaLa`: o ARCA dizia que procurou e não achou nada. O texto é o
        // que o usuário aprovou em 29/09/2026.
        let bancada = Bancada::nova("iso-ilegivel", ConsoleDeMentira::mudo());
        let arquivos = ArquivosQueRecusam::com(
            NA_PASTA_DE_DOWNLOADS,
            std::io::ErrorKind::PermissionDenied,
            "Acesso negado. (os error 5)",
        );
        let mut contexto = bancada.contexto(false);
        contexto.arquivos = &arquivos;

        let erro =
            executar(&contexto, Some(1), Some(Path::new(NA_PASTA_DE_DOWNLOADS))).unwrap_err();

        match &erro {
            Erro::PacoteRecusado(recusa @ pacote::RecusaDoPacote::NaoDeuParaOlhar { .. }) => {
                assert_eq!(
                    recusa.to_string(),
                    r"nao deu para saber se `C:\Users\Ana Paula\Downloads\clonezilla-live-3.3.3-15-amd64.zip` esta la (Acesso negado. (os error 5)). Isto NAO e o mesmo que ele nao estar la. Nada foi apagado (PR-2)"
                );
            }
            outro => panic!("o `--iso` que nao se deixou olhar virou {outro}"),
        }
        assert!(
            bancada.sistema.resumidos().is_empty(),
            "o `--iso` que nao se deixou olhar foi mandado ao certutil"
        );
        assert_eq!(bancada.console.lidas.get(), 0, "seguiu para a pergunta");
        assert!(
            !bancada.particionador.particionou(),
            "o `--iso` que nao se deixou olhar apagou o disco"
        );
    }

    #[test]
    fn o_pacote_que_nao_se_deixa_olhar_dentro_da_pasta_nao_sai_como_ausente() {
        // A mesma regra, na segunda pergunta do pré-voo. Até 29/09/2026
        // (WPC-68), o pacote de dentro que não se deixava olhar fazia a recusa
        // da pasta dizer `Esse arquivo nao esta nesta pasta`. Aqui a pasta tem
        // o pacote, que o Windows não deixa olhar, e o ISO ao lado dele.
        let dentro = r"C:\Users\Ana Paula\Downloads\clonezilla-live-3.3.3-15-amd64.zip";
        let bancada = Bancada::nova("iso-pasta-com-pacote-ilegivel", ConsoleDeMentira::mudo());
        let arquivos = ArquivosQueRecusam::com(
            dentro,
            std::io::ErrorKind::PermissionDenied,
            "Acesso negado. (os error 5)",
        )
        .com_arquivo(
            r"C:\Users\Ana Paula\Downloads\clonezilla-live-3.3.3-15-amd64.iso",
            "o iso, de mentira",
        );
        let mut contexto = bancada.contexto(false);
        contexto.arquivos = &arquivos;

        let erro = executar(&contexto, Some(1), Some(Path::new(A_PASTA_DE_DOWNLOADS))).unwrap_err();

        match &erro {
            Erro::PacoteRecusado(pacote::RecusaDoPacote::NaoDeuParaOlhar { caminho, .. }) => {
                assert_eq!(caminho, Path::new(dentro));
            }
            outro => panic!("o pacote que nao se deixou olhar virou {outro}"),
        }
        let dito = erro.to_string();
        assert!(!dito.contains("nao esta nesta pasta"), "{dito}");
        assert!(dito.contains("Nada foi apagado"), "{dito}");
        assert!(
            !bancada.particionador.particionou(),
            "o pacote que nao se deixou olhar apagou o disco"
        );
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

    /// O firmware lido antes do passo 5, sem entrada do ARCA — o caso em que
    /// o plano promete criar.
    fn sem_entrada() -> firmware::Leitura {
        firmware::ler(&ordem_desta_mesa())
    }

    /// O pacote de um preparo sem `--iso` quando o plano sai: guardado neste
    /// PC e conferido (PR-7).
    fn guardado_da_mesa() -> PacoteDoPreVoo {
        PacoteDoPreVoo::Guardado(PathBuf::from(
            r"C:\Users\Ana Paula\AppData\Local\ARCA\clonezilla-live-3.3.3-15-amd64.zip",
        ))
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
        let saida = montar_o_plano(&preparacao_da_mesa(), &guardado_da_mesa(), &sem_entrada());

        assert!(saida.contains("Dell Beta Apps NO IA WSL"), "{saida}");
        assert!(saida.contains("NTFS"), "{saida}");
        assert!(saida.contains("E:"), "{saida}");
        assert!(saida.contains("447,1 GB"), "{saida}");
        assert!(saida.contains("APAGADO"), "{saida}");
    }

    #[test]
    fn o_plano_mostra_as_duas_particoes_que_vao_ficar() {
        let saida = montar_o_plano(&preparacao_da_mesa(), &guardado_da_mesa(), &sem_entrada());

        assert!(saida.contains("ARCAVAULT"), "{saida}");
        assert!(saida.contains("ARCABOOT"), "{saida}");
        assert!(saida.contains("GPT"), "{saida}");
        assert!(saida.contains("1,6 GB"), "o tamanho do ARCABOOT: {saida}");
    }

    #[test]
    fn o_plano_nao_explica_o_esquema_nem_cita_adr() {
        // **A tela do plano diz o que vai acontecer com o disco, e mais nada.**
        //
        // Até o ADR-0025 ela trazia um parágrafo justificando o esquema, e ele
        // existia por uma razão que o ADR-0014 escreveu: quem lesse `MBR` num
        // dispositivo novo em 2026 acharia que era engano. Com GPT a estranheza
        // some, e o que sobrava era a data de uma medição, o número de um ADR e
        // a menção a uma partição que o comando remove sozinho — nota de
        // desenvolvimento na tela de quem só quer preparar um disco.
        //
        // O porquê do esquema é registro de projeto, e mora no ADR.
        let saida = montar_o_plano(&preparacao_da_mesa(), &guardado_da_mesa(), &sem_entrada());

        for vazamento in [
            "ADR-",
            "BOOTOU",
            "Microsoft Reserved",
            "nao e uma ESP",
            "25/08/2026",
            "efibootmgr",
        ] {
            assert!(
                !saida.contains(vazamento),
                "a tela do plano vazou `{vazamento}`, que e coisa de registro de projeto:\n{saida}"
            );
        }

        // O que ela **tem** de dizer continua lá: o esquema e as duas
        // partições, com tamanho e para que serve cada uma.
        assert!(saida.contains("GPT  1  NTFS"), "{saida}");
        assert!(saida.contains(ARCAVAULT), "{saida}");
        assert!(saida.contains(ARCABOOT), "{saida}");
    }

    #[test]
    fn a_linha_do_sistema_diz_o_que_foi_lido_e_nao_so_que_passou() {
        // Um `nao e o disco do Windows` sozinho e o ARCA afirmando; com os
        // valores ao lado, quem lê pode conferir a afirmacao. E a mesma razao
        // pela qual o §6.1 imprime os dois discos em setores em vez do
        // veredito de R-7 resumido.
        let saida = montar_o_plano(&preparacao_da_mesa(), &guardado_da_mesa(), &sem_entrada());

        assert!(saida.contains("IsSystem false"), "{saida}");
        assert!(saida.contains("IsBoot false"), "{saida}");
    }

    #[test]
    fn o_plano_diz_que_vai_mexer_no_firmware() {
        // Criar entrada de boot mexe na NVRAM, que e o lugar onde um erro
        // deixa alguem sem bootar. Quem lê um plano antes de apagar um disco
        // tem o direito de saber que o plano nao para no disco.
        let saida = montar_o_plano(&preparacao_da_mesa(), &guardado_da_mesa(), &sem_entrada());

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

        let saida = montar_o_plano(&preparacao, &guardado_da_mesa(), &sem_entrada());
        assert!(saida.contains("JA E UM DISPOSITIVO ARCA"), "{saida}");
        assert!(saida.contains("AS IMAGENS"), "{saida}");
        assert!(saida.contains("B-10"), "{saida}");
    }

    #[test]
    fn preparar_por_cima_de_um_dispositivo_nomeado_avisa_das_imagens() {
        // C-16: um dispositivo com o `ARCABOOT` renomeado continua sendo um
        // dispositivo, e as imagens dele se perdem do mesmo jeito.
        let mut discos = discos_para_preparar_desta_mesa();
        discos[2].particoes[1].rotulo = Some("ARCA-CASA".to_string());
        let dispositivo = discos.iter().find(|disco| disco.indice == 2);
        let preparacao = preparacao::julgar(2, dispositivo, &[0, 1, 2], Some('C'))
            .expect("ele passa as defesas");

        let saida = montar_o_plano(&preparacao, &guardado_da_mesa(), &sem_entrada());
        assert!(
            saida.contains("ESTE DISCO JA E UM DISPOSITIVO ARCA"),
            "{saida}"
        );
        assert!(saida.contains("AS IMAGENS"), "{saida}");
    }

    #[test]
    fn um_disco_qualquer_nao_ganha_o_aviso_das_imagens() {
        // Conselho que aparece sempre vira ruido — a licao que a E10 pagou no
        // `arca resultado` e a E11 repetiu em V-1.
        let saida = montar_o_plano(&preparacao_da_mesa(), &guardado_da_mesa(), &sem_entrada());
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

        let saida = montar_o_plano(&preparacao, &guardado_da_mesa(), &sem_entrada());
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

        let saida = montar_o_plano(&preparacao, &guardado_da_mesa(), &sem_entrada());
        assert!(saida.contains("crua"), "{saida}");
        assert!(saida.contains("(sem rotulo)"), "{saida}");
        assert!(saida.contains("(sem letra)"), "{saida}");
    }

    #[test]
    fn com_iso_a_tela_nao_promete_download() {
        let saida = montar_o_plano(
            &preparacao_da_mesa(),
            &PacoteDoPreVoo::DoIso(PathBuf::from(r"D:\cz.zip")),
            &sem_entrada(),
        );

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
            !saida.contains("menu do Clonezilla, faca um backup"),
            "a tela voltou a mandar para o menu do Clonezilla:\n{saida}"
        );
        assert!(
            !saida.contains("§6.4"),
            "o §6.4 e o caminho de quando o Windows nao boota, e nao o do primeiro backup:\n{saida}"
        );
    }

    #[test]
    fn o_fim_manda_sondar_sem_explicar_o_projeto() {
        // **Este teste já foi o contrário, e a troca foi deliberada.**
        //
        // Ele se chamava `o_fim_explica_a_razao_em_vez_de_so_mandar_fazer` e
        // cobrava o oposto: que a tela dissesse `blkdev.list`, `nvme1n1`, o
        // §4.5 e o P-26. O argumento era que *"um aviso que só diz `rode arca
        // sondar` empurra o problema de volta para quem não sabe por que ele
        // existe"*.
        //
        // Quem opera o comando decidiu que não: aquilo é coisa de
        // desenvolvimento, e não tem que aparecer para quem acabou de preparar
        // um disco. A tela diz **o que fazer**, **o que vai acontecer** — ele
        // reinicia a máquina — e **o que acontece se não fizer**. Isso é o que
        // a pessoa precisa para se organizar. O resto é registro de projeto, e
        // mora no §4.5 e no ADR-0019.
        //
        // Está escrito aqui para a próxima pessoa não "consertar" de volta
        // achando que foi descuido.
        let saida = montar_o_fim(&o_que_o_particionamento_deixou(), None);

        assert!(saida.contains("arca sondar"), "o que fazer: {saida}");
        assert!(
            saida.contains("reinicia a maquina"),
            "o que vai acontecer, que e o que muda o dia de quem le: {saida}"
        );
        assert!(
            saida.contains("recusa"),
            "o que acontece se nao fizer: {saida}"
        );

        for vazamento in ["blkdev.list", "nvme1n1", "§4.5", "P-26", "lsblk"] {
            assert!(
                !saida.contains(vazamento),
                "a tela do fim vazou `{vazamento}`, que e coisa de registro de projeto:\n{saida}"
            );
        }
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
    fn o_menu_diz_o_nome_do_dispositivo_arca() {
        // A marca sozinha diz que o disco e um dispositivo; com dois SSDs
        // iguais na mesa, e o nome que diz **qual** (C-16).
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
                rotulo: Some("ARCA-CASA".to_string()),
                sistema_de_arquivos: Some("FAT32".to_string()),
                tamanho_bytes: crate::preparacao::ARCABOOT_BYTES,
            },
        ];

        let saida = montar_o_menu(&preparacao::Oferta::de(&discos, Some('C')));
        let linha_do_disco = saida
            .lines()
            .find(|linha| linha.contains("disco 1 "))
            .expect("o disco 1 esta no menu");
        assert!(
            linha_do_disco.ends_with("· JA E UM DISPOSITIVO ARCA (ARCA-CASA)"),
            "{linha_do_disco}"
        );
    }

    #[test]
    fn o_menu_do_dispositivo_sem_nome_sai_como_antes() {
        // C-16 (28/09/2026) acrescenta o nome à marca so quando o `ARCABOOT`
        // tem um. O disco 2 desta mesa se chama `ARCABOOT`, e o menu inteiro
        // continua o de antes — comparado com o literal, e nao com um texto
        // remontado pelo codigo que esta sendo testado.
        const ANTES: &str = "\nDiscos desta maquina:\n\n  \
            [1]  disco 1   JMicron Generic      447,1 GB · USB · MBR · 1 particao (E:)\n  \
            [2]  disco 2   KGSSE100 256         238,5 GB · USB · MBR · 2 particoes (D:, R:) \
            · JA E UM DISPOSITIVO ARCA\n\n  \
            Sem numero, e o `arca prepare` nao prepara:\n       \
            disco 0   KINGSTON SNV3S500G   465,8 GB · NVMe · GPT · 1 particao (C:)\n                 \
            e o disco do sistema E o disco de boot desta maquina (PR-5)\n\n  \
            O numero entre colchetes e o que se digita; o `disco N` e o indice do\n  \
            Windows, que e o que o `--dispositivo` recebe. Escolher um numero so\n  \
            mostra o plano — nada e apagado antes da confirmacao digitada.\n";

        assert_eq!(
            montar_o_menu(&preparacao::Oferta::de(
                &discos_para_preparar_desta_mesa(),
                Some('C')
            )),
            ANTES
        );
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

        // A leitura de antes do passo 5 (PR-6) é a mesma que o `executar`
        // faria: o `firmware` inteiro, ainda coerente.
        criar_a_entrada(&bancada.contexto(false), 'R', &leitura)
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

    // ─────────── C-15 e PR-6: o firmware lido antes, e a recusa que nao apaga a listagem ───────────

    /// A mensagem com que o `bcdedit` desta maquina fechou toda listagem em
    /// 27/08/2026.
    const DISPOSITIVO_INEXISTENTE: &str = "Foi especificado um dispositivo inexistente.";

    #[test]
    fn a_entrada_pendurada_e_reusada_e_o_device_e_a_primeira_escrita() {
        // **O caso de 27/08/2026, de ponta a ponta.** A entrada `ARCA` esta na
        // listagem com `device unknown` — a particao para onde ela apontava foi
        // apagada — e o `bcdedit` sai com codigo 1 em todo `/enum`. O comando
        // tem de reusar a entrada e, antes de qualquer releitura, mandar o
        // `/set device` para a particao nova: e ele que devolve o codigo 0.
        let antes = format!(
            "{}{}",
            ordem_desta_mesa(),
            entrada_com(firmware::ARCA, "unknown", CAMINHO_DO_EFI)
        );
        let mut bancada = Bancada::nova("c15-pendurada", ConsoleDeMentira::mudo());
        bancada.firmware = FirmwareDeMentira::novo()
            .respondendo(FWBOOTMGR, &ordem_desta_mesa())
            .respondendo(FIRMWARE, &antes)
            .respondendo(
                GUID_DA_ENTRADA,
                &entrada_com(firmware::ARCA, "partition=R:", CAMINHO_DO_EFI),
            )
            .listando_e_recusando_no_fim(1, DISPOSITIVO_INEXISTENTE);
        let contexto = bancada.contexto(false);

        let leitura = ler_o_firmware_antes(&contexto).expect("a entrada esta la: reusar pode");
        assert_eq!(leitura.codigo_da_recusa, Some(1));

        let criada = criar_a_entrada(&contexto, 'R', &leitura).expect("reusada e reapontada");
        assert!(criada.ja_existia);
        assert_eq!(criada.identificador, GUID_DA_ENTRADA);
        assert_eq!(criada.alvo, Alvo::ParticaoComLetra('R'));

        let executados = bancada.firmware.executados();
        assert_eq!(
            executados.first().map(Vec::as_slice),
            Some(&["/set", GUID_DA_ENTRADA, "device", "partition=R:"].map(String::from)[..]),
            "o `/set device` tem de ser a primeira escrita: {executados:?}"
        );
        assert!(
            !executados
                .iter()
                .any(|argumentos| argumentos.first().map(String::as_str) == Some("/copy")),
            "criou uma segunda entrada em vez de reusar: {executados:?}"
        );
    }

    #[test]
    fn sem_entrada_para_reusar_a_leitura_recusada_nao_apaga_o_disco() {
        // C-15 com C-4: o `bcdedit` listou e recusou, e nao ha `ARCA` nem
        // `Clonezilla` na listagem. Criar seria apostar que a listagem esta
        // completa. A recusa acontece **antes do plano** — nenhuma pergunta,
        // nenhum disco tocado.
        let mut bancada = Bancada::nova(
            "c15-sem-entrada",
            ConsoleDeMentira::respondendo(&["s", "JMicron Generic"]),
        );
        bancada.firmware = FirmwareDeMentira::novo()
            .respondendo(FIRMWARE, &ordem_desta_mesa())
            .listando_e_recusando_no_fim(1, DISPOSITIVO_INEXISTENTE);

        let erro = executar(&bancada.contexto(false), Some(1), None).unwrap_err();

        assert!(
            matches!(erro, Erro::EntradaNaoNasceDeLeituraRecusada { codigo: 1 }),
            "{erro:?}"
        );
        assert!(erro.to_string().contains("C-15"), "{erro}");
        assert!(erro.to_string().contains("Nada foi apagado"), "{erro}");
        assert!(!bancada.particionador.particionou(), "apagou o disco");
        assert_eq!(bancada.console.lidas.get(), 0, "chegou a perguntar");
    }

    #[test]
    fn um_bcdedit_sem_privilegio_recusa_antes_de_apagar_qualquer_coisa() {
        // PR-6 pelo outro lado: o firmware e lido antes do ponto sem volta
        // justamente para que "nao consigo ler o firmware" seja uma recusa
        // barata — o disco continua como estava.
        let mut bancada = Bancada::nova(
            "pr6-acesso-negado",
            ConsoleDeMentira::respondendo(&["s", "JMicron Generic"]),
        );
        bancada.firmware =
            FirmwareDeMentira::novo().recusando_o_enumerar(Erro::FerramentaRecusou {
                ferramenta: "bcdedit",
                codigo: 1,
                saida: "Acesso negado.".to_string(),
            });

        let erro = executar(&bancada.contexto(false), Some(1), None).unwrap_err();

        assert!(
            matches!(erro, Erro::FerramentaRecusou { codigo: 1, .. }),
            "{erro:?}"
        );
        assert!(!bancada.particionador.particionou(), "apagou o disco");
        assert_eq!(bancada.console.lidas.get(), 0);
    }

    #[test]
    fn o_plano_diz_reapontada_quando_a_entrada_ja_existe_e_criada_quando_nao() {
        // O plano e lido antes de apagar um disco, e um "criada" sobre uma
        // entrada que existe seria a tela afirmando o que o comando nao vai
        // fazer. Com o firmware lido antes (PR-6), o plano sabe qual dos dois.
        let com_entrada = firmware::ler(&format!(
            "{}{}",
            ordem_desta_mesa(),
            entrada_com(firmware::ARCA, "partition=X:", CAMINHO_DO_EFI)
        ));
        let saida = montar_o_plano(&preparacao_da_mesa(), &guardado_da_mesa(), &com_entrada);
        assert!(saida.contains("REAPONTADA"), "{saida}");
        assert!(saida.contains(GUID_DA_ENTRADA), "{saida}");
        assert!(saida.contains("C-4"), "{saida}");
        assert!(!saida.contains("e criada no firmware"), "{saida}");

        let saida = montar_o_plano(&preparacao_da_mesa(), &guardado_da_mesa(), &sem_entrada());
        assert!(saida.contains("e criada no firmware"), "{saida}");
        assert!(!saida.contains("REAPONTADA"), "{saida}");
    }

    #[test]
    fn a_ordem_de_antes_e_a_lida_antes_do_apagar() {
        // A conferencia de C-5 em `tirar_da_ordem` compara com a ordem lida
        // **antes do passo 5**, e nao com uma releitura do passo 11: o
        // `prepare` nao le o firmware entre o apagar e o reapontar. Uma
        // entrada que sumisse da ordem nesse intervalo apareceria aqui.
        let ordem_com_outra = ordem_desta_mesa().replace(
            "displayorder            {bootmgr}",
            "displayorder            {bootmgr}\r\n                        {outra}",
        );
        let antes = firmware::ler(&format!(
            "{ordem_com_outra}{}",
            entrada_com(firmware::ARCA, "partition=X:", CAMINHO_DO_EFI)
        ));
        assert_eq!(antes.ordem_permanente.len(), 2);

        let mut bancada = Bancada::nova("pr6-ordem", ConsoleDeMentira::mudo());
        bancada.firmware = FirmwareDeMentira::novo()
            .respondendo(FWBOOTMGR, &ordem_desta_mesa())
            .respondendo(FIRMWARE, &ordem_desta_mesa())
            .respondendo(
                GUID_DA_ENTRADA,
                &entrada_com(firmware::ARCA, "partition=R:", CAMINHO_DO_EFI),
            );

        let erro = criar_a_entrada(&bancada.contexto(false), 'R', &antes)
            .expect_err("a `{outra}` sumiu da ordem entre a leitura de antes e a de depois");
        assert!(
            matches!(erro, Erro::OrdemPermanenteAlterada { .. }),
            "{erro:?}"
        );
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
