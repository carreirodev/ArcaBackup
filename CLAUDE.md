# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# ArcaBackup

Automatizador de Clonezilla para backup, restauração e verificação de imagem de disco, em Rust, só útil no Windows. **O ARCA nunca lê nem escreve disco**: ele prepara o dispositivo, monta a receita, arma o boot único no firmware e colhe o que o Clonezilla deixou escrito do outro lado do reinício.

O `README.md` documenta os nove comandos, as quatro receitas byte a byte, o diagnóstico de cada erro e o mapa arquivo a arquivo de `src/`. Este arquivo não o repete — ele diz o que fazer antes de tocar em código.

## Comandos

```powershell
cargo test                                  # a suíte inteira
cargo test --test e12_sondar_a_maquina      # um arquivo de integração
cargo test montar_backup                    # todo teste cujo nome contenha isso
cargo test -- --nocapture                   # mostrando o que os testes imprimem
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo run --example <nome>                  # os diagnósticos de `examples/`
cargo doc --open                            # onde as razões estão
```

**Uma vez por cópia do repositório**, senão o hook versionado não roda:

```powershell
git config core.hooksPath .githooks
```

O hook de `.githooks/pre-commit` roda formatação, clippy e a suíte antes de deixar commitar; o workflow `semanal.yml` roda os mesmos três num Windows que não é esta mesa.

Duas consequências do `Cargo.toml` + `build.rs` que mudam onde o código vai:

- **`[[bin]] test = false`.** O manifesto `requireAdministrator` vale para todo artefato do alvo, inclusive o executável de teste — que o `cargo test` não conseguiria rodar sem disparar UAC. Por isso `src/main.rs` é fino e **tudo que tem teste mora na lib**.
- **Os testes que falam com o hardware desta mesa se pulam sozinhos**, imprimindo o motivo (`pulado: nenhum dispositivo ARCA conectado`). Um teste pulado não é um teste quebrado — não "conserte".

## Idioma do código — este projeto sobrepõe a regra global

Código, comentários, identificadores, nomes de arquivo, mensagens de commit e nomes de teste são **em português**. Isso contraria a regra global do `CLAUDE.core.md`, que pede inglês; essa regra vive na seção *Role* e não nas Hard Rules, então o arquivo de projeto tem precedência sobre ela. Não "corrija" o idioma do código.

O vocabulário é obrigatoriamente o do **`CONTEXT.md`** — dispositivo, receita, job, armar, desarmar, selo, desfecho, veredito, resíduo, conferência —, com os sinônimos que ele manda evitar. Onde o código diverge do glossário, é o código que está errado.

Os comentários deste projeto não dizem o que o código faz: dizem **por que ele faz assim, o que custou descobrir e em que data**. Um comentário novo que só parafraseie a linha abaixo dele está fora do padrão.

## Antes de mudar comportamento

1. **`CONTEXT.md`** — o glossário. Cada termo traz a distinção que o justifica.
2. **`docs/adr/`** — as decisões de arquitetura. Leia as que tocam a área. Se a sua mudança contradiz um ADR, diga isso em voz alta em vez de sobrescrever calado.
3. **O identificador do requisito.** Toda regra que o ARCA nunca quebra tem ID — `C-*` (comum), `B-*` (backup), `R-*` (restauração), `S-*` (segurança), `L-*`, `V-*` (verificação), `PR-*` (preparação), `SD-*` (sondagem) — definidos no `PRD/PRD-ARCA-v5_1.md` e tabelados no README §13. Testes e comentários citam o ID; achar o ID é como se descobre o que a linha defende.

## Arquitetura

**Toda conversa do ARCA com o mundo passa por uma porta.** `src/portas/` são os nove traits (firmware, discos, arquivos, sistema, particionador, entropia, console, privilégios, relógio); `src/adaptadores/windows/` são as implementações de verdade (`bcdedit`, WMI via PowerShell, registro, `BCryptGenRandom`); `src/duplos.rs` são as de mentira. É isso que dá teste sem hardware ao parser do `bcdedit`, ao validador da receita e à regra de espaço.

`main.rs` é fino de propósito e a ordem importa: registra a invocação → analisa com o `clap` → eleva → despacha. `app.rs` carrega o `Contexto` (as portas + `--dry-run`) e faz o `match`. `comandos/` tem um arquivo por comando. O resto da raiz de `src/` é lógica pura, sem I/O. O mapa arquivo a arquivo está no README §14.

**S-1 é uma propriedade das assinaturas de `portas/`**: nenhuma entrega handle de dispositivo, caminho bruto ou deslocamento em setores. Uma porta nova que precisasse disso denunciaria a si mesma.

## O que quebra de um jeito que não parece com a causa

- **Três testes varrem `src/` atrás de padrões proibidos**, e falham nomeando um arquivo que você não tocou: `s1_nenhum_acesso_raw.rs` (acesso raw a disco), `s6_o_tempo_nao_decide.rs` (código que julga desfecho alcançando o relógio) e `b10_nada_e_apagado.rs` (qualquer forma de exclusão).
- **`recursos/capturas/` é evidência, não fixture ajustável.** São saídas que outra ferramenta escreveu, e os testes de integração dizem explicitamente que o oráculo deles está fora deste repositório. Nunca edite uma captura para um teste passar.
- **`.gitattributes` protege as duas coisas que a normalização quebraria**: `recursos/capturas/** -text`, porque o `bcdedit` escreve CRLF e o parser precisa aguentar o que chega dele em produção; e `*.sh text eol=lf`, porque um shebang com `\r` falha dizendo "bad interpreter", sem falar em fim de linha.
- **C-2 proíbe não-ASCII na receita**, que é a string gravada no `grub.cfg` — é regra do conteúdo da receita, não dos identificadores do Rust.
- **Contagens envelhecem sozinhas.** Quantos testes a suíte tem, quantos ADRs há: o README já traz dois números diferentes entre si e o PRD um terceiro. Ao escrever documentação, aponte para a coisa em vez de contá-la.

## Agent skills

### Issue tracker

Issues vivem como GitHub issues em `carreirodev/ArcaBackup`, operados pelo CLI `gh`. Veja `docs/agents/issue-tracker.md`.

### Triage labels

Os cinco papéis canônicos de triagem, cada label idêntica ao seu nome. Veja `docs/agents/triage-labels.md`.

### Domain docs

Single-context: `CONTEXT.md` e `docs/adr/` na raiz do repositório. Veja `docs/agents/domain.md`.
