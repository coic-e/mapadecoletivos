//! Quem pode decidir sobre um pedido de correção.
//!
//! Sugerir é aberto; aplicar e recusar são da moderação. O witness é o que
//! separa as duas coisas na assinatura das ações, e não só na rota.
use crate::auth::AdminIdentity;
use crate::domains::organizations::auth::{moderating, SeeEveryStatus};

/// Prova de que quem está decidindo é moderador.
///
/// Carrega junto o witness de organizações porque aplicar um pedido termina
/// relendo o cadastro em qualquer estado — inclusive um que ainda está na fila
/// de moderação. Sem isso, a ação teria que fabricar aquela prova por fora, e
/// a garantia dela deixaria de valer.
#[derive(Debug, Clone, Copy)]
pub struct ReviewEditRequests {
    admin_id: i32,
    organizations: SeeEveryStatus,
    _private: (),
}

impl ReviewEditRequests {
    /// Quem decidiu, para gravar no pedido.
    pub fn admin_id(&self) -> i32 {
        self.admin_id
    }

    /// A prova para reler o cadastro depois de aplicar.
    pub fn organizations(&self) -> SeeEveryStatus {
        self.organizations
    }
}

/// A única porta de entrada.
pub fn reviewing(identity: &AdminIdentity) -> ReviewEditRequests {
    ReviewEditRequests {
        admin_id: identity.id,
        organizations: moderating(identity),
        _private: (),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_witness_carries_who_decided_and_the_right_to_reread() {
        let identity = AdminIdentity {
            id: 9,
            name: "Moderação".to_string(),
            email: "mod@exemplo.com".to_string(),
        };

        let w = reviewing(&identity);

        assert_eq!(w.admin_id(), 9);
        assert_eq!(w.organizations().admin_id(), 9);
    }
}
