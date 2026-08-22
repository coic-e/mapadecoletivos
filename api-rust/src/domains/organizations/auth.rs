//! Quem pode ver o quê, num arquivo só.
//!
//! A regra que sustenta o site é que cadastro pendente ou rejeitado não existe
//! para quem está de fora. Hoje ela mora na escolha de qual função do
//! repositório o handler chama: `find_approved_by_id` filtra por status,
//! `find_by_id` não. As duas têm a mesma assinatura e nomes parecidos, e nada
//! impede que uma rota pública chame a errada — o cadastro pendente de alguém
//! vazaria e nenhum teste que não pensasse nisso perceberia.
//!
//! O witness resolve isso pelo tipo. `SeeEveryStatus` só pode ser construído
//! aqui dentro, e só a partir de um `AdminIdentity`; as funções que enxergam
//! qualquer status passam a exigi-lo. Uma rota pública não tem como produzir
//! um, então não compila.
//!
//! O que ele garante e o que não garante: o acidente que ele fecha é a rota
//! pública chamar a query errada, porque rota pública não tem `AdminIdentity`
//! nenhum para oferecer. Ele não é barreira contra quem fabrica um
//! `AdminIdentity` na mão — os campos são públicos, e é assim que os testes
//! montam o seu. Fechar isso exigiria esconder a construção da identidade
//! também, e o que se ganharia é proteção contra um ato deliberado, que já
//! apareceria na revisão de qualquer jeito.
use crate::auth::AdminIdentity;

/// Prova de que quem está pedindo é moderador e pode ver qualquer estado.
///
/// O campo privado é o que impede construção de fora deste módulo: sem ele,
/// qualquer lugar do código escreveria `SeeEveryStatus {}` e a garantia
/// evaporaria.
#[derive(Debug, Clone, Copy)]
pub struct SeeEveryStatus {
    admin_id: i32,
    _private: (),
}

impl SeeEveryStatus {
    /// Quem decidiu, para gravar junto da moderação.
    pub fn admin_id(&self) -> i32 {
        self.admin_id
    }
}

/// A única porta de entrada. Recebe a identidade que o extractor já conferiu
/// contra o banco.
pub fn moderating(identity: &AdminIdentity) -> SeeEveryStatus {
    SeeEveryStatus {
        admin_id: identity.id,
        _private: (),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_witness_carries_who_decided() {
        let identity = AdminIdentity {
            id: 7,
            name: "Moderação".to_string(),
            email: "mod@exemplo.com".to_string(),
        };

        assert_eq!(moderating(&identity).admin_id(), 7);
    }

    // Não há teste de "construir sem passar pela porta": isso é erro de
    // compilação, e o que garante a regra é o campo privado, não um assert.
}
