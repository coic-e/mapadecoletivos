//! Regras de moderador, sem nada de HTTP.
//!
//! Existem três portas para uma conta de moderador — o binário `create_admin`,
//! a semente do ambiente na primeira subida, e o login que a usa — e a regra
//! estava escrita em três lugares. As duas que criam conta divergiam: a
//! semente conferia só o comprimento da senha, o binário conferia comprimento
//! e lista de senhas vazadas. O comentário da semente afirmava que as duas
//! exigiam o mesmo. `ADMIN_PASSWORD=123456789012` passava por uma e era
//! recusada pela outra.
//!
//! Aqui a regra tem um dono. Porta nova que apareça herda o mesmo piso sem
//! ninguém precisar lembrar.
use validator::Validate;

use crate::auth::{create_token, hash_password, spend_verification_time, verify_password};
use crate::config::AppConfig;
use crate::db::DbConnection;
use crate::errors::ApiError;
use db_types::admin::{Admin, NewAdmin};

use super::repository::AdminRepository;

/// Senha mínima para uma conta que aprova e rejeita cadastros. Comprimento
/// pesa mais que variedade de símbolos contra ataque offline.
pub const MIN_PASSWORD_LEN: usize = 12;

/// Senhas que aparecem no topo de qualquer lista de vazamento. Passar do
/// comprimento mínimo não salva `123456789012`.
const OBVIOUS_PASSWORDS: &[&str] = &[
    "password",
    "senha123",
    "12345678",
    "123456789012",
    "administrador",
    "adminadmin",
];

/// Tetos de entrada no login.
///
/// Recusados antes de tocar no banco: 320 é o maior e-mail que um servidor de
/// e-mail aceita, e uma senha de megabytes só serve para fazer o argon2
/// trabalhar de graça.
const MAX_EMAIL_LEN: usize = 320;
const MAX_PASSWORD_LEN: usize = 512;

/// O piso de senha, num lugar só.
pub fn check_password_strength(password: &str) -> Result<(), String> {
    if password.chars().count() < MIN_PASSWORD_LEN {
        return Err(format!(
            "a senha precisa de pelo menos {MIN_PASSWORD_LEN} caracteres"
        ));
    }

    let lowered = password.to_lowercase();

    if OBVIOUS_PASSWORDS.iter().any(|obvia| lowered == *obvia) {
        return Err("essa senha está em toda lista de senhas vazadas".to_string());
    }

    Ok(())
}

/// Normaliza um e-mail para virar chave de conta.
///
/// Maiúscula e espaço em volta são erro de digitação, não conta diferente —
/// sem isto, `Mod@Exemplo.com` viraria uma segunda conta que ninguém consegue
/// usar para entrar.
pub fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

/// Cria uma conta, aplicando o piso de senha.
pub fn create(
    conn: &mut DbConnection,
    name: &str,
    email: &str,
    password: &str,
) -> Result<Admin, ApiError> {
    if let Err(motivo) = check_password_strength(password) {
        return Err(ApiError::ValidationError(
            vec![("password".to_string(), vec![motivo])]
                .into_iter()
                .collect(),
        ));
    }

    let new_admin = NewAdmin {
        name: name.trim().to_string(),
        email: normalize_email(email),
        password_hash: hash_password(password)?,
    };

    new_admin.validate().map_err(ApiError::from)?;

    AdminRepository::create(conn, &new_admin)
}

/// Semente do primeiro moderador, lida do ambiente.
pub struct AdminSeed {
    pub name: String,
    pub email: String,
    pub password: String,
}

impl AdminSeed {
    /// Ausência das variáveis não é erro: é o caso normal depois da primeira
    /// subida.
    pub fn from_env() -> Option<Self> {
        let email = std::env::var("ADMIN_EMAIL").ok()?;
        let password = std::env::var("ADMIN_PASSWORD").ok()?;

        Some(AdminSeed {
            name: std::env::var("ADMIN_NAME").unwrap_or_else(|_| "Moderação".to_string()),
            email: normalize_email(&email),
            password,
        })
    }
}

/// Cria o primeiro moderador, se ainda não houver nenhum.
///
/// É semente de primeira subida, não fonte permanente de senha: só age quando
/// a tabela está vazia. Se aplicasse a senha a cada boot, quem tivesse acesso
/// ao painel assumiria a conta editando uma variável e reiniciando o serviço —
/// e a senha ficaria exposta em `docker inspect` e em `/proc/<pid>/environ`.
///
/// Devolve erro só quando a semente existe e está inválida: aí é melhor falhar
/// alto do que subir sem ninguém conseguir moderar.
pub fn seed_first(conn: &mut DbConnection, seed: Option<AdminSeed>) -> Result<(), String> {
    let existentes = AdminRepository::count(conn)
        .map_err(|e| format!("não consegui contar os moderadores: {e}"))?;

    match (existentes, seed) {
        (0, None) => {
            log::warn!(
                "Nenhum moderador cadastrado e ADMIN_EMAIL/ADMIN_PASSWORD não definidos: \
                 o painel de moderação fica inacessível até criar uma conta"
            );

            Ok(())
        }

        (0, Some(seed)) => {
            // Mesmo piso do binário, porque é a mesma função — não uma cópia
            // que envelhece sozinha.
            check_password_strength(&seed.password)
                .map_err(|motivo| format!("ADMIN_PASSWORD: {motivo}"))?;

            let admin = create(conn, &seed.name, &seed.email, &seed.password)
                .map_err(|e| format!("falha ao criar o primeiro moderador: {e}"))?;

            log::info!(
                "Primeiro moderador criado: {} <{}>",
                admin.name,
                admin.email
            );
            log::warn!(
                "Remova ADMIN_EMAIL e ADMIN_PASSWORD do ambiente: a conta já existe e a \
                 senha não precisa mais ficar guardada lá"
            );

            Ok(())
        }

        (_, Some(_)) => {
            // A senha no ambiente não vale mais nada aqui, e continuar
            // guardada só aumenta a superfície de vazamento.
            log::warn!(
                "ADMIN_EMAIL/ADMIN_PASSWORD definidos mas já existem moderadores: \
                 ignorados. Remova-os do ambiente"
            );

            Ok(())
        }

        (_, None) => Ok(()),
    }
}

/// Confere e-mail e senha e emite o token da sessão.
///
/// "E-mail não existe" e "senha errada" devolvem o mesmo erro, e custam o
/// mesmo tempo: a diferença entre as duas respostas seria a lista de quem
/// modera o site, legível por cronômetro.
pub fn authenticate(
    conn: &mut DbConnection,
    email: &str,
    password: &str,
    config: &AppConfig,
) -> Result<(Admin, String), ApiError> {
    let invalido = || ApiError::Unauthorized("E-mail ou senha inválidos".to_string());

    if email.len() > MAX_EMAIL_LEN || password.len() > MAX_PASSWORD_LEN {
        return Err(invalido());
    }

    let email = normalize_email(email);

    let Some(admin) = AdminRepository::find_by_email(conn, &email)? else {
        // Gasta o tempo de um argon2 real, senão o relógio do cliente denuncia
        // quais contas existem.
        spend_verification_time(password);

        return Err(invalido());
    };

    if !verify_password(password, &admin.password_hash) {
        return Err(invalido());
    }

    let token = create_token(admin.id, &admin.email, config)?;

    Ok((admin, token))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_floor_is_the_same_for_every_door() {
        // A divergência que motivou este módulo: a semente aceitava uma senha
        // que o binário recusava, e o comentário dela dizia o contrário.
        assert!(check_password_strength("123456789012").is_err());
        assert!(check_password_strength("adminadmin").is_err());
        assert!(
            check_password_strength("ADMINADMIN").is_err(),
            "caixa não salva"
        );
        assert!(check_password_strength("curta").is_err());

        assert!(check_password_strength("uma-senha-que-presta").is_ok());
    }

    #[test]
    fn twelve_characters_is_the_line() {
        assert!(check_password_strength(&"a".repeat(11)).is_err());
        assert!(check_password_strength(&"a".repeat(12)).is_ok());
    }

    #[test]
    fn the_length_is_counted_in_characters_not_bytes() {
        // "senhaçãoçãoç" tem 12 caracteres e mais de 12 bytes. Contar bytes
        // deixaria passar uma senha mais curta do que o piso.
        assert!(
            check_password_strength("çãoçãoçãoçã").is_err(),
            "11 caracteres"
        );
        assert!(
            check_password_strength("çãoçãoçãoção").is_ok(),
            "12 caracteres"
        );
    }

    #[test]
    fn an_email_is_the_same_account_however_it_is_typed() {
        assert_eq!(normalize_email("  Mod@Exemplo.COM  "), "mod@exemplo.com");
        assert_eq!(normalize_email("mod@exemplo.com"), "mod@exemplo.com");
    }
}
