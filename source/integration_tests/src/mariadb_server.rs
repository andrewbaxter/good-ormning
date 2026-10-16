use {
    good_ormning::runtime::mariadb::mysql_async::{
        Conn,
        Opts,
        OptsBuilder,
        prelude::Queryable,
    },
    std::{
        env,
        hash::{
            DefaultHasher,
            Hash,
            Hasher,
        },
        io::Write,
        os::unix::net::UnixStream,
        path::{
            Path,
            PathBuf,
        },
        process::{
            ChildStdin,
            Command,
            Stdio,
        },
        sync::{
            LazyLock,
            atomic::{
                AtomicUsize,
                Ordering,
            },
        },
        time::{
            Duration,
            Instant,
        },
    },
};

struct Server {
    opts: Opts,
    _supervisor_stdin: Option<ChildStdin>,
    db_counter: AtomicUsize,
}

pub async fn db_opts() -> Result<Opts, loga::Error> {
    static SERVER: LazyLock<Result<Server, String>> = LazyLock::new(|| -> Result<Server, String> {
        if let Ok(url) = env::var("GOOD_ORMNING_TEST_MARIADB_URL") {
            return Ok(Server {
                opts: Opts::from_url(&url).map_err(|e| format!("Invalid GOOD_ORMNING_TEST_MARIADB_URL: {}", e))?,
                _supervisor_stdin: None,
                db_counter: AtomicUsize::new(0),
            });
        }
        let mariadbd = match env::var_os("GOOD_ORMNING_TEST_MARIADBD") {
            Some(p) => PathBuf::from(p),
            None => env::split_paths(&env::var_os("PATH").unwrap_or_default())
                .map(|dir| dir.join("mariadbd"))
                .find(|p| p.is_file())
                .ok_or_else(
                    || "Couldn't find mariadbd on PATH; install MariaDB, set GOOD_ORMNING_TEST_MARIADBD to the path of mariadbd, or set GOOD_ORMNING_TEST_MARIADB_URL to use an existing server".to_string(),
                )?,
        };
        let mariadbd = mariadbd.canonicalize().map_err(|e| format!("Error resolving {:?}: {}", mariadbd, e))?;
        let bin_dir = mariadbd.parent().unwrap();
        let template = {
            let mut hasher = DefaultHasher::new();
            mariadbd.hash(&mut hasher);
            env::temp_dir().join(format!("good-ormning-mariadb-template-{:016x}", hasher.finish()))
        };
        if !template.exists() {
            let install_db = bin_dir.join("mariadb-install-db");
            let staging = PathBuf::from(format!("{}.{}", template.to_string_lossy(), std::process::id()));
            let out =
                Command::new(&install_db)
                    .arg("--no-defaults")
                    .arg("--skip-test-db")
                    .arg("--auth-root-authentication-method=normal")
                    .arg(format!("--datadir={}", staging.to_string_lossy()))
                    .output()
                    .map_err(|e| format!("Error running {:?}: {}", install_db, e))?;
            if !out.status.success() {
                return Err(
                    format!(
                        "{:?} failed: {}\n{}",
                        install_db,
                        String::from_utf8_lossy(&out.stdout),
                        String::from_utf8_lossy(&out.stderr)
                    ),
                );
            }
            if std::fs::rename(&staging, &template).is_err() {
                _ = std::fs::remove_dir_all(&staging);
            }
        }
        let shm = Path::new("/dev/shm");
        let dir = tempfile::Builder::new().prefix("good-ormning-mariadb-").tempdir_in(if shm.is_dir() {
            shm.to_path_buf()
        } else {
            env::temp_dir()
        }).map_err(|e| format!("Error creating server dir: {}", e))?.keep();
        let data_dir = dir.join("data");
        let out =
            Command::new("cp")
                .arg("-r")
                .arg(&template)
                .arg(&data_dir)
                .output()
                .map_err(|e| format!("Error copying template data dir: {}", e))?;
        if !out.status.success() {
            return Err(format!("Error copying template data dir: {}", String::from_utf8_lossy(&out.stderr)));
        }
        let socket = dir.join("sock");
        let log = dir.join("log");
        let mut supervisor =
            Command::new("sh")
                .arg("-c")
                .arg(r#"
dir="$1"
shift
"$@" &
server=$!
read _ || true
kill "$server" 2>/dev/null
wait "$server" 2>/dev/null
rm -rf "$dir"
"#)
                .arg("sh")
                .arg(&dir)
                .arg(&mariadbd)
                .arg("--no-defaults")
                .arg(format!("--datadir={}", data_dir.to_string_lossy()))
                .arg(format!("--socket={}", socket.to_string_lossy()))
                .arg(format!("--pid-file={}", dir.join("pid").to_string_lossy()))
                .arg(format!("--log-error={}", log.to_string_lossy()))
                .arg("--skip-networking")
                .arg("--skip-grant-tables")
                .arg("--skip-log-bin")
                .arg("--innodb-flush-log-at-trx-commit=0")
                .arg("--innodb-doublewrite=0")
                .arg("--innodb-buffer-pool-size=64M")
                .arg("--max-connections=1000")
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| format!("Error starting {:?}: {}", mariadbd, e))?;
        let supervisor_stdin = supervisor.stdin.take().unwrap();
        let start = Instant::now();
        loop {
            if UnixStream::connect(&socket).is_ok() {
                break;
            }
            if start.elapsed() > Duration::from_secs(60) {
                let log = std::fs::read_to_string(&log).unwrap_or_default();
                let mut stdin = supervisor_stdin;
                _ = stdin.write_all(b"\n");
                return Err(format!("Timed out waiting for MariaDB to start, log:\n{}", log));
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        return Ok(Server {
            opts: OptsBuilder::default().socket(Some(socket.to_string_lossy())).user(Some("root")).into(),
            _supervisor_stdin: Some(supervisor_stdin),
            db_counter: AtomicUsize::new(0),
        });
    });
    let server = match &*SERVER {
        Ok(s) => s,
        Err(e) => return Err(loga::err(e.clone())),
    };
    let name = format!("test_{}_{}", std::process::id(), server.db_counter.fetch_add(1, Ordering::Relaxed));
    let mut conn = Conn::new(server.opts.clone()).await.map_err(loga::err)?;
    conn.query_drop(format!("create database `{}`", name)).await.map_err(loga::err)?;
    conn.disconnect().await.map_err(loga::err)?;
    return Ok(OptsBuilder::from_opts(server.opts.clone()).db_name(Some(name)).into());
}

pub async fn db() -> Result<Conn, loga::Error> {
    return Ok(Conn::new(db_opts().await?).await.map_err(loga::err)?);
}
