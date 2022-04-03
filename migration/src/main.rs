use migration::Migrator;
use sea_schema::migration::*;
use std::fs;

#[async_std::main]
async fn main() {
    let postgres_host = std::env::var("POSTGRES_HOST").expect("env POSTGRES_HOST not found");
    let postgres_port = std::env::var("POSTGRES_PORT").expect("env POSTGRES_PORT not found");
    let postgres_db = std::env::var("POSTGRES_DB").expect("env POSTGRES_DB not found");

    let postgres_user_file =
        std::env::var("POSTGRES_USER_FILE").expect("env POSTGRES_USER_FILE not found");
    let postgres_password_file =
        std::env::var("POSTGRES_PASSWORD_FILE").expect("env POSTGRES_PASSWORD_FILE not found");

    let postgres_user =
        fs::read_to_string(postgres_user_file).expect("Cannot read POSTGRES_USER_FILE");
    let postgres_password =
        fs::read_to_string(postgres_password_file).expect("Cannot read POSTGRES_PASSWORD_FILE");

    let url = format!("postgresql://{postgres_user}:{postgres_password}@{postgres_host}:{postgres_port}/{postgres_db}");

    std::env::set_var("DATABASE_URL", &url);

    cli::run_cli(Migrator).await;
}
