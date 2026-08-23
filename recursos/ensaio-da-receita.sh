#!/bin/bash
# Roda as receitas do ARCA num bash de verdade, com o Clonezilla substituido
# por comandos falsos que saem com o codigo que se pedir.
#
# Existe porque os testes de `src/receita.rs` provam o que a *string* contem,
# e nao o que o *bash* faz com ela. A receita e um `if/then/else` aninhado
# dentro de um `bash -c '...'` — e o aninhamento e codigo novo, que nenhuma
# execucao real exercitou (P-16 do PRD). A pergunta que so o bash responde e
# se cada desfecho escreve o rastro certo.
#
# Uso: bash recursos/ensaio-da-receita.sh
# Precisa de bash. No Windows, o Git Bash serve.
#
# Ao mudar a receita em `src/receita.rs`, atualize as duas strings abaixo com
# o que `cargo run --example receita_ao_lado_da_que_rodou` imprimir, e rode
# isto de novo.

set -u

BACKUP='mkdir -p /home/partimag/ARCA-LOGS/backup-ARCA-TESTE-02; echo ARCA_SELO=a3f1c9e07b2d4856 > /home/partimag/ARCA-LOGS/backup-ARCA-TESTE-02/arca-fim.txt; if ocs-sr -q2 -j2 -z9p -i 4096 -gm -sfsck -senc -batch -p true savedisk ARCA-TESTE-02 nvme0n1; then echo ARCA_BACKUP=OK >> /home/partimag/ARCA-LOGS/backup-ARCA-TESTE-02/arca-fim.txt; if ocs-chkimg -b -or /home/partimag ARCA-TESTE-02 > /home/partimag/ARCA-TESTE-02/arca-check.log 2>&1; then echo ARCA_VEREDITO=APROVADA >> /home/partimag/ARCA-TESTE-02/arca-check.log; else echo ARCA_VEREDITO=REPROVADA >> /home/partimag/ARCA-TESTE-02/arca-check.log; fi; else echo ARCA_BACKUP=FALHOU >> /home/partimag/ARCA-LOGS/backup-ARCA-TESTE-02/arca-fim.txt; fi; echo ARCA_FIM >> /home/partimag/ARCA-LOGS/backup-ARCA-TESTE-02/arca-fim.txt; sleep 20; poweroff'

RESTAURACAO='mkdir -p /home/partimag/ARCA-LOGS/restauracao-ARCA-TESTE-02; echo ARCA_SELO=a3f1c9e07b2d4856 > /home/partimag/ARCA-LOGS/restauracao-ARCA-TESTE-02/arca-fim.txt; if ocs-sr -e1 auto -e2 -batch -j2 -k0 -iefi -p true restoredisk ARCA-TESTE-02 nvme0n1 > /home/partimag/ARCA-LOGS/restauracao-ARCA-TESTE-02/arca-restore.log 2>&1; then echo ARCA_RESTORE=OK >> /home/partimag/ARCA-LOGS/restauracao-ARCA-TESTE-02/arca-fim.txt; else echo ARCA_RESTORE=FALHOU >> /home/partimag/ARCA-LOGS/restauracao-ARCA-TESTE-02/arca-fim.txt; fi; echo ARCA_FIM >> /home/partimag/ARCA-LOGS/restauracao-ARCA-TESTE-02/arca-fim.txt; sleep 20; poweroff'

# A terceira, da etapa E11. Ela e a menor, e o que ela tem de proprio e o
# `>>` no `arca-check.log`: a de backup usa `>`, porque a imagem acabou de
# nascer e o log nao existe; aqui ele existe, e e o veredito do backup que a
# criou. Ver `montar_verificacao` em `src/receita.rs`.
VERIFICACAO='mkdir -p /home/partimag/ARCA-LOGS/verificacao-ARCA-TESTE-02; echo ARCA_SELO=a3f1c9e07b2d4856 > /home/partimag/ARCA-LOGS/verificacao-ARCA-TESTE-02/arca-fim.txt; if ocs-chkimg -b -or /home/partimag ARCA-TESTE-02 >> /home/partimag/ARCA-TESTE-02/arca-check.log 2>&1; then echo ARCA_VEREDITO=APROVADA >> /home/partimag/ARCA-TESTE-02/arca-check.log; echo ARCA_VERIFY=OK >> /home/partimag/ARCA-LOGS/verificacao-ARCA-TESTE-02/arca-fim.txt; else echo ARCA_VEREDITO=REPROVADA >> /home/partimag/ARCA-TESTE-02/arca-check.log; echo ARCA_VERIFY=FALHOU >> /home/partimag/ARCA-LOGS/verificacao-ARCA-TESTE-02/arca-fim.txt; fi; echo ARCA_FIM >> /home/partimag/ARCA-LOGS/verificacao-ARCA-TESTE-02/arca-fim.txt; sleep 20; poweroff'

RAIZ=$(mktemp -d)
trap 'rm -rf "$RAIZ"' EXIT
falhas=0

# Um Clonezilla de mentira que sai com o codigo pedido. O `savedisk` que da
# certo cria a pasta da imagem — que e o que faz o redirecionamento do
# `ocs-chkimg` ter para onde apontar.
preparar() {                      # $1=codigo do ocs-sr  $2=codigo do ocs-chkimg
  rm -rf "$RAIZ/palco" && mkdir -p "$RAIZ/palco/bin"
  local palco="$RAIZ/palco"
  cat > "$palco/bin/ocs-sr" <<FIM
#!/bin/bash
echo "ocs-sr \$*" >> "$palco/chamadas.txt"
[ "$1" = "0" ] && mkdir -p "$palco/home/partimag/ARCA-TESTE-02"
exit $1
FIM
  cat > "$palco/bin/ocs-chkimg" <<FIM
#!/bin/bash
echo "ocs-chkimg \$*" >> "$palco/chamadas.txt"
exit $2
FIM
  printf '#!/bin/bash\nexit 0\n' > "$palco/bin/sleep"
  printf '#!/bin/bash\necho poweroff >> "%s/chamadas.txt"\nexit 0\n' "$palco" > "$palco/bin/poweroff"
  chmod +x "$palco"/bin/*
}

# Roda a receita com `/home/partimag` trocado pelo palco, e nada mais trocado.
executar() {                      # $1=a receita
  local palco="$RAIZ/palco"
  ( export PATH="$palco/bin:$PATH"; bash -c "${1//\/home\/partimag/$palco\/home\/partimag}" ) \
    > /dev/null 2>&1
}

conferir() {                      # $1=rotulo  $2=arquivo  $3...=linhas esperadas
  local rotulo=$1 relativo=$2 arquivo="$RAIZ/palco/$2"; shift 2
  local esperado; esperado=$(printf '%s\n' "$@")
  local obtido; obtido=$(cat "$arquivo" 2>/dev/null)

  if [ "$obtido" = "$esperado" ]; then
    echo "  ok   $rotulo · $relativo"
  else
    echo "  FALHOU $rotulo · $relativo"
    echo "    esperado: $(echo "$esperado" | tr '\n' '|')"
    echo "    obtido:   $(echo "$obtido" | tr '\n' '|')"
    falhas=$((falhas + 1))
  fi
}

ausente() {                       # $1=rotulo  $2=arquivo
  if [ -e "$RAIZ/palco/$2" ]; then
    echo "  FALHOU $1 · $2 existe, e nao devia"
    falhas=$((falhas + 1))
  else
    echo "  ok   $1 · $2 nao existe, como se quer"
  fi
}

echo "A receita parseia?"
bash -n -c "$BACKUP"      && echo "  ok   backup"
bash -n -c "$RESTAURACAO" && echo "  ok   restauracao"
bash -n -c "$VERIFICACAO" && echo "  ok   verificacao"
echo

# Os dois desfechos moram em pastas diferentes de proposito: as duas receitas
# comecam truncando o proprio `arca-fim.txt` com um `>`, e um caminho que so
# dependesse do nome da imagem faria a restauracao apagar o desfecho de um
# backup ainda nao colhido.
FIM_BACKUP=home/partimag/ARCA-LOGS/backup-ARCA-TESTE-02/arca-fim.txt
FIM_RESTAURACAO=home/partimag/ARCA-LOGS/restauracao-ARCA-TESTE-02/arca-fim.txt
FIM_VERIFICACAO=home/partimag/ARCA-LOGS/verificacao-ARCA-TESTE-02/arca-fim.txt
CHECK=home/partimag/ARCA-TESTE-02/arca-check.log

echo "Backup: o savedisk deu certo e a imagem passou"
preparar 0 0; executar "$BACKUP"
conferir "desfecho" "$FIM_BACKUP" ARCA_SELO=a3f1c9e07b2d4856 ARCA_BACKUP=OK ARCA_FIM
conferir "veredito" "$CHECK" ARCA_VEREDITO=APROVADA
echo

echo "Backup: o savedisk deu certo e a imagem foi reprovada"
preparar 0 1; executar "$BACKUP"
conferir "desfecho" "$FIM_BACKUP" ARCA_SELO=a3f1c9e07b2d4856 ARCA_BACKUP=OK ARCA_FIM
conferir "veredito" "$CHECK" ARCA_VEREDITO=REPROVADA
echo

echo "Backup: o savedisk falhou — nao ha imagem para verificar"
preparar 1 0; executar "$BACKUP"
conferir "desfecho" "$FIM_BACKUP" ARCA_SELO=a3f1c9e07b2d4856 ARCA_BACKUP=FALHOU ARCA_FIM
ausente  "verificacao" "$CHECK"
echo

echo "Restauracao: deu certo"
preparar 0 0; executar "$RESTAURACAO"
conferir "desfecho" "$FIM_RESTAURACAO" ARCA_SELO=a3f1c9e07b2d4856 ARCA_RESTORE=OK ARCA_FIM
echo

echo "Restauracao: falhou"
preparar 1 0; executar "$RESTAURACAO"
conferir "desfecho" "$FIM_RESTAURACAO" ARCA_SELO=a3f1c9e07b2d4856 ARCA_RESTORE=FALHOU ARCA_FIM
echo

echo "Verificacao: o ocs-chkimg aprovou"
preparar 0 0; mkdir -p "$RAIZ/palco/home/partimag/ARCA-TESTE-02"; executar "$VERIFICACAO"
conferir "desfecho" "$FIM_VERIFICACAO" ARCA_SELO=a3f1c9e07b2d4856 ARCA_VERIFY=OK ARCA_FIM
conferir "veredito" "$CHECK" ARCA_VEREDITO=APROVADA
echo

echo "Verificacao: o ocs-chkimg reprovou"
preparar 0 1; mkdir -p "$RAIZ/palco/home/partimag/ARCA-TESTE-02"; executar "$VERIFICACAO"
conferir "desfecho" "$FIM_VERIFICACAO" ARCA_SELO=a3f1c9e07b2d4856 ARCA_VERIFY=FALHOU ARCA_FIM
conferir "veredito" "$CHECK" ARCA_VEREDITO=REPROVADA
echo

# O que o `>>` compra, e a razao de ele nao ser `>` como no backup: o
# `arca-check.log` ja existe quando a verificacao roda — e o veredito do backup
# que criou a imagem —, e um `>` o destruiria. Com `>>` as duas marcas ficam no
# arquivo, e o leitor do ADR-0003 lê **toda forma de reprovar antes de toda
# forma de aprovar**: uma imagem que ja reprovou continua reprovada.
echo "Verificacao: o veredito antigo sobrevive, e a reprovacao vence"
preparar 0 1; mkdir -p "$RAIZ/palco/home/partimag/ARCA-TESTE-02"
executar "$VERIFICACAO"                     # reprova, e escreve REPROVADA
# So o `ocs-chkimg` muda: `preparar` apagaria o palco inteiro, e com ele o log
# que este caso existe para ver sobreviver.
printf '#!/bin/bash\nexit 0\n' > "$RAIZ/palco/bin/ocs-chkimg"
executar "$VERIFICACAO"                     # aprova, e ACRESCENTA APROVADA
conferir "as duas marcas no mesmo log" "$CHECK" ARCA_VEREDITO=REPROVADA ARCA_VEREDITO=APROVADA
echo

echo "As tres receitas nao dividem o mesmo arca-fim.txt"
preparar 0 0; executar "$BACKUP"; executar "$RESTAURACAO"; executar "$VERIFICACAO"
conferir "backup sobreviveu as outras duas" "$FIM_BACKUP" ARCA_SELO=a3f1c9e07b2d4856 ARCA_BACKUP=OK ARCA_FIM
conferir "restauracao tem o seu"            "$FIM_RESTAURACAO" ARCA_SELO=a3f1c9e07b2d4856 ARCA_RESTORE=OK ARCA_FIM
conferir "verificacao tem o seu"            "$FIM_VERIFICACAO" ARCA_SELO=a3f1c9e07b2d4856 ARCA_VERIFY=OK ARCA_FIM
echo

if [ "$falhas" -eq 0 ]; then
  echo "Todos os desfechos deixam o rastro certo."
else
  echo "$falhas conferencia(s) falharam."
fi
exit "$falhas"
