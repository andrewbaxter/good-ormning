use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    process::Command,
};

#[test]
fn test_compile_fail() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cases_dir = manifest_dir.join("tests/compile_fail");
    let mut cases = BTreeMap::new();
    for entry in fs::read_dir(&cases_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        let file = syn::parse_file(&fs::read_to_string(&path).unwrap()).unwrap();
        let mut expected = None;
        for item in &file.items {
            let syn::Item::Const(c) = item else {
                continue;
            };
            if c.ident != "EXPECTED_ERROR" {
                continue;
            }
            let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &*c.expr else {
                panic!("{}: `EXPECTED_ERROR` must be a string literal", path.display());
            };
            expected = Some(s.value());
        }
        let Some(expected) = expected else {
            panic!("{} has no `const EXPECTED_ERROR: &str = \"...\";`", path.display());
        };
        cases.insert(path.file_stem().unwrap().to_str().unwrap().to_string(), (path, expected));
    }
    let crate_dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("compile_fail");
    fs::create_dir_all(&crate_dir).unwrap();
    let mut manifest =
        format!(
            "[package]\nname = \"compile_fail_cases\"\nversion = \"0.0.0\"\nedition = \"2021\"\npublish = false\n\n[workspace]\n\n[dependencies]\ngood-ormning = {{ path = {:?}, features = [\"chrono\", \"jiff\", \"pg\", \"sqlite\", \"mariadb\"] }}\n",
            manifest_dir.join("..").to_str().unwrap()
        );
    for (name, (path, _)) in &cases {
        manifest.push_str(&format!("\n[[bin]]\nname = {:?}\npath = {:?}\n", name, path.to_str().unwrap()));
    }
    fs::write(crate_dir.join("Cargo.toml"), manifest).unwrap();
    fs::copy(manifest_dir.join("../Cargo.lock"), crate_dir.join("Cargo.lock")).unwrap();
    let output =
        Command::new(env!("CARGO"))
            .arg("check")
            .arg("--keep-going")
            .arg("--bins")
            .arg("--message-format=json")
            .arg("--manifest-path")
            .arg(crate_dir.join("Cargo.toml"))
            .env("CARGO_TARGET_DIR", crate_dir.join("target"))
            .env("OUT_DIR", env!("OUT_DIR"))
            .output()
            .unwrap();
    let mut actual: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for line in String::from_utf8(output.stdout).unwrap().lines() {
        let Ok(message) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if message["reason"] != "compiler-message" || message["message"]["level"] != "error" {
            continue;
        }
        let Some(target) = message["target"]["name"].as_str() else {
            continue;
        };
        actual
            .entry(target.to_string())
            .or_default()
            .push(message["message"]["message"].as_str().unwrap_or_default().to_string());
    }
    let mut failures = vec![];
    for (name, (path, expected)) in &cases {
        let got = actual.get(name).cloned().unwrap_or_default();
        if got.is_empty() {
            failures.push(format!("{} compiled without errors", path.display()));
            continue;
        }
        if !got.iter().any(|g| g.contains(expected.as_str())) {
            failures.push(
                format!("{}: expected an error containing {:?}, got:\n{}", path.display(), expected, got.join("\n")),
            );
        }
    }
    assert!(
        failures.is_empty(),
        "{}\n\ncargo stderr:\n{}",
        failures.join("\n\n"),
        String::from_utf8_lossy(&output.stderr)
    );
}
