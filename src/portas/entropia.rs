//! A porta da entropia — de onde sai o selo.
//!
//! Uma porta para dezesseis bytes parece exagero, e nao e. O selo e a unica
//! coisa que liga um job ao seu desfecho (C-11, §4.3), e um selo que se
//! repetisse faria dois jobs diferentes serem indistinguiveis — que e
//! exatamente o que o mecanismo existe para impedir. Atras de uma porta, a
//! geracao tem duplo, e o teste do `estado.json` deixa de depender de sorte.
//!
//! # S-6 nao proibe usar o tempo para **gerar**
//!
//! Parece contradicao e nao e, e vale deixar escrito onde alguem va procurar.
//! S-6 proibe comparar uma data escrita pelo Windows com outra escrita pelo
//! Linux para **decidir** se um desfecho pertence a um job. Gerar um
//! identificador a partir do relogio nao decide nada — o que decide e a
//! igualdade entre duas cadeias de dezesseis digitos.
//!
//! O que tira o relogio da jogada aqui e outra coisa: **o selo nao precisa ser
//! imprevisivel, precisa ser nao repetido**, e um valor derivado do relogio
//! colide quando duas execucoes caem no mesmo milissegundo. Uma fonte de
//! entropia do sistema nao colide na pratica. Ver
//! `docs/adr/0006-o-selo-e-o-estado-sem-dependencia-nova.md`.
//!
//! # S-1 continua valendo
//!
//! Como as outras portas, esta nao entrega handle de dispositivo, caminho
//! bruto nem deslocamento em setores. Ela entrega bytes.

use crate::erro::Resultado;

pub trait Entropia {
    /// Preenche `destino` inteiro com bytes imprevisiveis.
    ///
    /// Ou preenche tudo, ou falha. Um preenchimento parcial que passasse por
    /// bom deixaria zeros no fim do selo, e zeros no fim de um selo sao
    /// exatamente o que [`crate::receita::Selo::de_ensaio`] usa para dizer
    /// "isto nao e de verdade".
    fn preencher(&self, destino: &mut [u8]) -> Resultado<()>;
}
