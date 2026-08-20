//! Cria uma conta de moderador.
//!
//! Não existe rota de cadastro de admin de propósito: quem cria conta é quem
//! tem acesso ao servidor e ao banco.
//!
//! ```text
//! cargo run --bin create_admin -- "Nome" email@exemplo.com
//! ```
//!
//! A senha é lida da entrada padrão, nunca da linha de comando: argumento de
//! processo aparece no `ps` de qualquer usuário da máquina e fica gravado no
//! histórico do shell.
use std::env;
use std::io::{self, BufRead, IsTerminal, Write};
use std::process::ExitCode;

use api_rust::auth::hash_password;
use api_rust::domains::admins::repository::AdminRepository;
use db_types::admin::NewAdmin;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use validator::Validate;

/// Senha mínima aceitável para uma conta que aprova e rejeita cadastros.
/// Comprimento pesa mais que variedade de símbolos contra ataque offline.
const MIN_PASSWORD_LEN: usize = 12;

/// Senhas que aparecem no topo de qualquer lista de vazamento.
const OBVIOUS_PASSWORDS: &[&str] = &[
    "password",
    "senha123",
    "12345678",
    "123456789012",
    "administrador",
    "adminadmin",
];

fn check_password_strength(password: &str) -> Result<(), String> {
    if password.chars().count() < MIN_PASSWORD_LEN {
        return Err(format!(
            "a senha precisa de pelo menos {MIN_PASSWORD_LEN} caracteres"
        ));
    }

    let lowered = password.to_lowercase();

    if OBVIOUS_PASSWORDS.iter().any(|obvious| lowered == *obvious) {
        return Err("essa senha está em toda lista de senhas vazadas".to_string());
    }

    Ok(())
}

/// Lê a senha da entrada padrão.
///
/// Sem eco no terminal seria melhor ainda, mas isso exigiria uma dependência a
/// mais só para este binário; o que importa aqui é a senha não passar por
/// argumento de processo. Num pipe (`echo ... | create_admin`) funciona igual.
fn read_password() -> io::Result<String> {
    if io::stdin().is_terminal() {
        eprint!("senha do moderador: ");
        io::stderr().flush()?;
    }

    let mut password = String::new();
    io::stdin().lock().read_line(&mut password)?;

    Ok(password.trim_end_matches(['\n', '\r']).to_string())
}

fn main() -> ExitCode {
    dotenvy::dotenv().ok();

    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("uso: cargo run --bin create_admin -- <nome> <email>");
        eprintln!("a senha é pedida na entrada padrão");
        return ExitCode::FAILURE;
    }

    let name = args[1].trim().to_string();
    let email = args[2].trim().to_lowercase();

    let password = match read_password() {
        Ok(password) => password,
        Err(e) => {
            eprintln!("erro ao ler a senha: {e}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(reason) = check_password_strength(&password) {
        eprintln!("erro: {reason}");
        return ExitCode::FAILURE;
    }

    if env::var("DATABASE_URL").is_err() {
        eprintln!("erro: DATABASE_URL não definida");
        return ExitCode::FAILURE;
    }

    let password_hash = match hash_password(&password) {
        Ok(hash) => hash,
        Err(e) => {
            eprintln!("erro ao gerar hash da senha: {e}");
            return ExitCode::FAILURE;
        }
    };

    let new_admin = NewAdmin {
        name,
        email,
        password_hash,
    };

    if let Err(errors) = new_admin.validate() {
        eprintln!("erro de validação: {errors}");
        return ExitCode::FAILURE;
    }

    // O repositório trabalha com conexão do pool (DbConnection), então o
    // binário monta um pool mínimo em vez de abrir conexão direta.
    let manager =
        ConnectionManager::<PgConnection>::new(env::var("DATABASE_URL").expect("checado acima"));

    let pool = match Pool::builder().max_size(1).build(manager) {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("erro ao conectar no banco: {e}");
            return ExitCode::FAILURE;
        }
    };

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("erro ao obter conexão: {e}");
            return ExitCode::FAILURE;
        }
    };

    match AdminRepository::create(&mut conn, &new_admin) {
        Ok(admin) => {
            println!(
                "admin criado: {} <{}> (id {})",
                admin.name, admin.email, admin.id
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("erro ao criar admin: {e}");
            ExitCode::FAILURE
        }
    }
}
