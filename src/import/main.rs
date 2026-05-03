use {
    aargvark::{
        help::{
            HelpPattern,
            HelpPatternElement,
            HelpState,
        },
        traits_impls::AargvarkFromStr,
        Aargvark,
        vark,
    },
    loga::ResultContext,
};

mod codegen;
mod pg;
mod sqlite;

#[derive(Clone)]
enum Engine {
    Pg,
    Sqlite,
}

impl AargvarkFromStr for Engine {
    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "pg" => return Ok(Engine::Pg),
            "sqlite" => return Ok(Engine::Sqlite),
            other => return Err(format!("Unknown engine {:?}, expected pg or sqlite", other)),
        }
    }

    fn build_help_pattern(_state: &mut HelpState) -> HelpPattern {
        return HelpPattern(vec![HelpPatternElement::Type("pg|sqlite".to_string())]);
    }
}

#[derive(Aargvark)]
struct Args {
    /// Database engine type: pg or sqlite
    engine: Engine,
    /// Connection string (for pg: postgres://..., for sqlite: path to file)
    connection: String,
    /// Database name used in the generated build.rs (default: "db")
    db_name: Option<String>,
}

#[tokio::main]
async fn main() {
    match main_inner().await {
        Ok(_) => {},
        Err(e) => {
            loga::fatal(e);
        },
    }
}

async fn main_inner() -> Result<(), loga::Error> {
    let args = vark::<Args>();
    let db_name = args.db_name.unwrap_or_else(|| "db".to_string());
    match args.engine {
        Engine::Pg => {
            let (client, conn) =
                tokio_postgres::connect(&args.connection, tokio_postgres::NoTls)
                    .await
                    .context("Connecting to PostgreSQL")?;
            tokio::spawn(async move {
                if let Err(e) = conn.await {
                    eprintln!("PostgreSQL connection error: {}", e);
                }
            });
            let version = pg::read_schema(&client).await?;
            let code = codegen::generate_pg(&version, &db_name);
            println!("{}", code);
        },
        Engine::Sqlite => {
            let conn =
                rusqlite::Connection::open(&args.connection).context("Opening SQLite database")?;
            let version = sqlite::read_schema(&conn)?;
            let code = codegen::generate_sqlite(&version, &db_name);
            println!("{}", code);
        },
    }
    return Ok(());
}
