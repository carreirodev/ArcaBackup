#!/usr/bin/env bash
# Mede um arca-restore.log contra as cinco previsoes de P-23.
#
# A hipotese: o log nao "comeca no meio" — ele foi truncado por baixo. O `>` da
# receita abre o arquivo e o `ocs-sr` escreve por ele; na ultima passagem o
# Clonezilla reabre o MESMO arquivo com truncamento e o partclone escreve a tela
# dele a partir do byte 0; o descritor da receita, com o offset intacto, retoma
# la em cima — e o intervalo vira zeros.
#
# A previsao 5 mudou depois da medicao de 24/08, e a mudanca esta explicada no
# ADR-0022: ela julgava o buraco inteiro, e o inicio dele NAO carrega informacao
# nenhuma. O inicio e o tamanho da tela do partclone, que e constante porque a
# tela e constante; foi 4.085 nas duas restauracoes, byte a byte. Quem responde
# "o corte cai sempre no mesmo lugar?" e o FIM, que e onde o `ocs-sr` chegou.
#
# Uso:  ./medir-arca-restore-log.sh /d/ARCA-LOGS/restauracao-<nome>/arca-restore.log

set -u

f="${1:?informe o caminho do arca-restore.log}"
[ -r "$f" ] || { echo "nao da para ler: $f" >&2; exit 2; }

tam=$(wc -c < "$f")
nuls=$(tr -dc '\0' < "$f" | wc -c)
telas=$(grep -obUaP '\x1b\)0\x1b\[1;24r' "$f" | wc -l)
primeiro_nul=$(grep -obUaP '\x00' "$f" | head -1 | cut -d: -f1)
ultimo_nul=$(grep -obUaP '\x00' "$f" | tail -1 | cut -d: -f1)
starting_ocs=$(grep -cobUaF 'Starting /usr/sbin/ocs-sr' "$f" || true)
ending_ocs=$(grep -cobUaF 'Ending /usr/sbin/ocs-sr' "$f" || true)
particao=$(grep -obUaF 'Starting to restore image' "$f" | head -1)
alvo=$(grep -oaU 'to device (/dev/[a-z0-9]*)' "$f" | head -1)

echo "arquivo ............ $f"
echo "tamanho ............ $tam bytes"
echo "NULs ............... $nuls  (offsets ${primeiro_nul:-—} a ${ultimo_nul:-—})"
echo "telas de partclone . $telas"
echo "Starting ocs-sr .... $starting_ocs"
echo "Ending ocs-sr ...... $ending_ocs"
echo "primeira tela ...... ${particao:-nenhuma}  $alvo"
echo

ok=0; falhou=0
julgar() { # $1 = veredito (0 ok), $2 = texto
  if [ "$1" -eq 0 ]; then echo "  [bate]     $2"; ok=$((ok+1))
  else echo "  [NAO BATE] $2"; falhou=$((falhou+1)); fi
}

echo "As cinco previsoes de P-23:"
julgar "$([ "$telas" -eq 1 ] && echo 0 || echo 1)" \
  "1. uma unica inicializacao de terminal — so a ultima passagem do partclone sobreviveu"
julgar "$([ "${particao:-}" != "" ] && [ "${particao%%:*}" -lt 4096 ] && echo 0 || echo 1)" \
  "2. a tela do partclone abre o arquivo (offset < 4 KiB): $alvo"
julgar "$([ "$nuls" -gt 0 ] && echo 0 || echo 1)" \
  "3. ha um bloco de NULs — prova de truncamento com descritor aberto atras"
julgar "$([ "$ending_ocs" -ge 1 ] && [ "$starting_ocs" -eq 0 ] && echo 0 || echo 1)" \
  "4. ha 'Ending /usr/sbin/ocs-sr' e NAO ha o 'Starting' correspondente"
if [ "$nuls" -eq 0 ]; then
  echo "  [n/a]      5. sem buraco, a previsao 5 nao se aplica"
else
  julgar "$([ "${ultimo_nul}" != "12890" ] && echo 0 || echo 1)" \
    "5. o buraco NAO TERMINA onde terminou em 22/08 (12.890) — ele termina onde o ocs-sr chegou"
fi

echo
echo "  $ok de 5 batem, $falhou nao."
if [ "$falhou" -eq 0 ]; then
  echo "  → P-23 fecha: o corte nao e do ARCA nem do redirecionamento."
else
  echo "  → NAO feche P-23. O que nao bate e o achado, e ele vale mais do que a previsao."
fi

# O inicio do buraco nao e previsao: e consequencia. Ele mede a tela do
# partclone, que a hipotese diz ser o que reabre o arquivo. Sai como reforco,
# fora da contagem, porque um valor constante aqui CONFIRMA a hipotese em vez de
# a contrariar — e foi por ler isso ao contrario que a previsao 5 nasceu errada.
echo
if [ "$nuls" -gt 0 ]; then
  if [ "${primeiro_nul}" = "4085" ]; then
    echo "  reforco: o buraco comeca em 4.085, o mesmo de 22/08 — a tela do partclone"
    echo "           tem tamanho constante, e e ela que reabre o arquivo."
  else
    echo "  atencao: o buraco comeca em ${primeiro_nul}, e nao em 4.085. A tela do partclone"
    echo "           mudou de tamanho — outra particao, outro layout de terminal, outra versao."
  fi
fi

echo
echo "Contexto para o registro:"
echo "  22/08 ... 16.600 bytes · 8.806 NULs (4.085–12.890, 53% do arquivo) · 1 tela · p4 de 4"
echo "  24/08 ... 16.641 bytes · 8.840 NULs (4.085–12.924, 53% do arquivo) · 1 tela · p4 de 4"
echo -n "  agora ... $tam bytes · $nuls NULs (${primeiro_nul:-—}–${ultimo_nul:-—}) · $telas tela(s) · "
echo "$alvo"
