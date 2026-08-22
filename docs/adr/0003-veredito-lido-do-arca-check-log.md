# O veredito é lido do `arca-check.log`, com marcador explícito quando houver

O `arca list` precisa dizer se uma imagem foi aprovada, e a única evidência disso no dispositivo é o `arca-check.log` que B-9 manda gravar. Ao ler o dispositivo real, o arquivo apareceu em **duas formas**: a imagem `2026-08-21_WindowsCompleto` termina com uma linha `ARCA_VEREDITO=APROVADA` seguida de `ARCA_FIM`, e a `ARCA-TESTE-03` traz só a saída crua do `ocs-chkimg` — escapes de terminal e o resumo em inglês. A receita publicada em §10.1 do PRD produz a segunda forma; a primeira veio de um script do trabalho de validação manual.

Decidimos que o leitor aceita as duas, nesta ordem: **o marcador `ARCA_VEREDITO=` decide quando está presente**; sem ele, vale o resumo do `ocs-chkimg`. Um marcador é algo que alguém escreveu para ser lido; um texto de terminal é algo que se interpreta, e interpretar é onde se erra.

Dentro de cada um dos dois caminhos, **a reprovação é procurada antes da aprovação**, e pelo mesmo motivo: as duas marcas cabem no mesmo arquivo. No resumo, porque um log de falha lista as partições que prestam junto da que não presta. No marcador, porque a receita *acrescenta* a linha ao log — uma imagem verificada duas vezes fica com as duas marcas, e a antiga vem primeiro. Ler na outra ordem transformaria uma imagem quebrada em imagem aprovada, que é exatamente o contrário de S-5.

Não havendo nem marcador nem resumo reconhecível, o veredito é ausente e a listagem diz `sem veredito`. **Ausência de prova nunca vira aprovação**: imagem não verificada é suposição, e uma suposição exibida como `aprovada` é pior do que nenhuma informação.

## Consequências

A E3, que transcreve a receita de backup, pode acrescentar a linha `ARCA_VEREDITO=` ao `arca-check.log` sem quebrar nada, e passa a ser a forma preferida daí em diante. As imagens antigas do dispositivo continuam legíveis pelo caminho do resumo — nenhuma precisa ser reverificada para aparecer na listagem.

O preço é que o parser depende de frases em inglês do `ocs-chkimg`, que podem mudar de versão. É risco contido: se as frases mudarem, o veredito vira `sem veredito` — o modo de falha é a listagem admitir que não sabe, nunca aprovar o que não conferiu.

Vale para o veredito, e não para o desfecho. O que liga um `arca-fim.txt` ao job que o produziu continua sendo o selo, nunca o texto (ver [selo liga job ao desfecho](0001-selo-liga-job-ao-desfecho.md)).
