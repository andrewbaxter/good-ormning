use {
    good_ormning::import::{
        codegen,
        mariadb,
        pg,
        sqlite,
    },
    aargvark::{
        Aargvark,
        help::{
            HelpPattern,
            HelpPatternElement,
            HelpState,
        },
        traits_impls::AargvarkFromStr,
        vark,
    },
    loga::ResultContext,
};

#[derive(Aargvark)]
struct Args {
    /// Connection string (for pg: postgres://..., for sqlite: path to file)
    connection: String,
    /// Database name used in the generated build.rs (default: "db")
    db_name: Option<String>,
    /// Database engine type: pg or sqlite
    engine: Engine,
    prefer_chrono: Option<()>,
}

#[derive(Clone)]
enum Engine {
    Mariadb,
    Pg,
    Sqlite,
}

impl AargvarkFromStr for Engine {
    fn build_help_pattern(_state: &mut HelpState) -> HelpPattern {
        return HelpPattern(vec![HelpPatternElement::Type("pg|mariadb|sqlite".to_string())]);
    }

    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "mariadb" => return Ok(Engine::Mariadb),
            "pg" => return Ok(Engine::Pg),
            "sqlite" => return Ok(Engine::Sqlite),
            other => return Err(format!("Unknown engine {:?}, expected pg, mariadb, or sqlite", other)),
        }
    }
}

#[tokio::main]
async fn main() {
    match async {
        let args = vark::<Args>();
        let db_name = args.db_name.unwrap_or_else(|| "db".to_string());
        let prefer_chrono = args.prefer_chrono.is_some();
        let failures = match args.engine {
            Engine::Mariadb => {
                let opts =
                    mysql_async::Opts::from_url(&args.connection).context("Parsing MariaDB connection string")?;
                let mut conn = mysql_async::Conn::new(opts).await.context("Connecting to MariaDB")?;
                let (version, failures) = mariadb::read_schema(&mut conn, prefer_chrono).await?;
                let code = codegen::generate_mariadb(&version, &db_name).context("Generating mariadb build.rs")?;
                println!("{}", code);
                failures
            },
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
                let (version, failures) = pg::read_schema(&client, prefer_chrono).await?;
                let code = codegen::generate_pg(&version, &db_name).context("Generating pg build.rs")?;
                println!("{}", code);
                failures
            },
            Engine::Sqlite => {
                // Read-only so a mistyped path errors instead of silently creating an empty
                // database.
                let conn =
                    rusqlite::Connection::open_with_flags(
                        &args.connection,
                        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
                    ).context("Opening SQLite database")?;
                let (version, failures) = sqlite::read_schema(&conn)?;
                let code = codegen::generate_sqlite(&version, &db_name).context("Generating sqlite build.rs")?;
                println!("{}", code);
                failures
            },
        };
        if !failures.is_empty() {
            return Err(
                loga::agg_err(
                    "Some parts of the schema couldn't be imported",
                    failures.into_iter().map(loga::err).collect(),
                ),
            );
        }
        return Ok::<(), loga::Error>(());
    }.await {
        Ok(_) => { },
        Err(e) => {
            loga::fatal(e);
        },
    }
}
