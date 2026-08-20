//! Limite de requisições por cliente, guardado em memória.
//!
//! Cobre os dois pontos que doem sem ele: o login, que sem limite aceita força
//! bruta de senha à velocidade da rede, e o cadastro público, que sem limite
//! aceita spam automatizado até encher o disco e a fila de moderação.
//!
//! É por processo — uma frota de réplicas precisaria de um contador
//! compartilhado (Redis) para valer como limite global.
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use actix_web::HttpRequest;

use crate::config::AppConfig;
use crate::errors::ApiError;

/// Acima disso, a limpeza roda antes de aceitar chave nova. Segura o mapa de
/// crescer sem teto quando o tráfego vem de muitos IPs distintos.
const CLEANUP_THRESHOLD: usize = 10_000;

struct Bucket {
    /// Início da janela corrente.
    started_at: Instant,
    hits: u32,
}

pub struct RateLimiter {
    max_hits: u32,
    window: Duration,
    buckets: Mutex<HashMap<String, Bucket>>,
}

impl RateLimiter {
    pub fn new(max_hits: u32, window: Duration) -> Self {
        RateLimiter {
            max_hits,
            window,
            buckets: Mutex::new(HashMap::new()),
        }
    }

    /// Conta mais uma tentativa. Devolve erro quando a janela já estourou.
    pub fn check(&self, key: &str) -> Result<(), ApiError> {
        let now = Instant::now();

        // Lock envenenado não pode virar "pode passar": nesse caso o limite
        // sumiria justamente sob a carga que o quebrou.
        let mut buckets = self
            .buckets
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if buckets.len() > CLEANUP_THRESHOLD {
            let window = self.window;
            buckets.retain(|_, bucket| now.duration_since(bucket.started_at) < window);
        }

        let bucket = buckets.entry(key.to_string()).or_insert(Bucket {
            started_at: now,
            hits: 0,
        });

        if now.duration_since(bucket.started_at) >= self.window {
            bucket.started_at = now;
            bucket.hits = 0;
        }

        bucket.hits += 1;

        if bucket.hits > self.max_hits {
            let retry_in = self.window - now.duration_since(bucket.started_at);

            return Err(ApiError::TooManyRequests(retry_in.as_secs().max(1)));
        }

        Ok(())
    }

    /// Zera o contador — usado depois de um login que deu certo, para que a
    /// pessoa que só errou a senha uma vez não fique presa na janela.
    pub fn reset(&self, key: &str) {
        self.buckets
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(key);
    }
}

/// Os dois limitadores existem como tipos distintos porque o actix guarda o
/// app_data por tipo: registrados como o mesmo `RateLimiter`, o segundo
/// substituiria o primeiro e login e cadastro dividiriam um contador só.
macro_rules! named_limiter {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        pub struct $name(RateLimiter);

        impl $name {
            pub fn new(max_hits: u32, window: Duration) -> Self {
                $name(RateLimiter::new(max_hits, window))
            }
        }

        impl std::ops::Deref for $name {
            type Target = RateLimiter;

            fn deref(&self) -> &RateLimiter {
                &self.0
            }
        }
    };
}

named_limiter!(LoginRateLimiter, "Tentativas de login por IP.");
named_limiter!(SubmissionRateLimiter, "Cadastros de coletivo por IP.");

/// IP do cliente para fins de limite.
///
/// X-Forwarded-For só é considerado com TRUST_PROXY ligado: sem um proxy que
/// reescreva o cabeçalho, ele é texto livre do cliente e cada requisição
/// poderia inventar um IP novo para escapar do contador.
pub fn client_key(req: &HttpRequest, config: &AppConfig) -> String {
    if config.trust_proxy {
        if let Some(forwarded) = req
            .connection_info()
            .realip_remote_addr()
            .and_then(|value| value.parse::<IpAddr>().ok())
        {
            return forwarded.to_string();
        }
    }

    req.peer_addr()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "desconhecido".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_up_to_the_limit_and_then_blocks() {
        let limiter = RateLimiter::new(2, Duration::from_secs(60));

        assert!(limiter.check("1.2.3.4").is_ok());
        assert!(limiter.check("1.2.3.4").is_ok());
        assert!(limiter.check("1.2.3.4").is_err());
    }

    #[test]
    fn counts_each_key_separately() {
        let limiter = RateLimiter::new(1, Duration::from_secs(60));

        assert!(limiter.check("1.2.3.4").is_ok());
        assert!(limiter.check("5.6.7.8").is_ok());
    }

    #[test]
    fn window_expiry_frees_the_key() {
        let limiter = RateLimiter::new(1, Duration::from_millis(20));

        assert!(limiter.check("1.2.3.4").is_ok());
        assert!(limiter.check("1.2.3.4").is_err());

        std::thread::sleep(Duration::from_millis(30));

        assert!(limiter.check("1.2.3.4").is_ok());
    }

    #[test]
    fn reset_clears_the_counter() {
        let limiter = RateLimiter::new(1, Duration::from_secs(60));

        assert!(limiter.check("1.2.3.4").is_ok());
        limiter.reset("1.2.3.4");

        assert!(limiter.check("1.2.3.4").is_ok());
    }
}
