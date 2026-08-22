//! A porta do firmware — a unica superficie pela qual o ARCA fala com o
//! gerenciador de boot do Windows.
//!
//! O contrato entrega **texto bruto**, de proposito. O `bcdedit` nao traduz os
//! nomes de campo (so `identificador` sai em portugues), entao o parser
//! correto e por valor, e ele e codigo puro que a etapa E2 constroi e testa
//! contra saidas capturadas nos dois idiomas. Se a porta ja devolvesse
//! estruturas, o parser ficaria do lado de ca da fronteira e sem teste.

use crate::erro::Resultado;

pub trait Firmware {
    /// Saida bruta de uma consulta ao `bcdedit`, para parse por valor (C-3).
    fn enumerar(&self, alvo: &str) -> Resultado<String>;

    /// Executa uma escrita no `bcdedit` e devolve o que ele imprimiu.
    ///
    /// O retorno **nunca** basta como prova (C-3): o `bcdedit` responde
    /// "êxito" e mantem o valor antigo quando o alvo e `Removable Media`.
    /// Quem chama confere depois com [`Firmware::enumerar`].
    fn executar(&self, argumentos: &[&str]) -> Resultado<String>;
}
