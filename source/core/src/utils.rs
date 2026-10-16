use {
    proc_macro2::TokenStream,
    serde::{
        Deserialize,
        Serialize,
    },
    std::{
        cell::RefCell,
        collections::HashMap,
        fmt,
        rc::Rc,
    },
};

pub const DEFAULT_DB_NAME: &str = "";

#[derive(Clone)]
pub struct Errs(Rc<RefCell<Errs_>>);

impl Errs {
    pub fn err(&self, path: &rpds::Vector<String>, t: String) {
        let mut s = self.0.as_ref().borrow_mut();
        let mut out = String::new();
        for (i, k) in path.iter().enumerate() {
            if i > 0 {
                out.push('/');
            }
            out.push_str(k.as_ref());
        }
        out.push_str(" -- ");
        out.push_str(&t);
        s.errs.push(out);
    }

    pub fn new() -> Self {
        return Self(Rc::new(RefCell::new(Errs_ { errs: vec![] })));
    }

    pub fn raise(self) -> Result<(), Vec<String>> {
        let errs = self.0.borrow_mut().errs.split_off(0);
        if !errs.is_empty() {
            return Err(errs);
        }
        return Ok(());
    }
}

impl Default for Errs {
    fn default() -> Self {
        return Self::new();
    }
}

pub struct Errs_ {
    errs: Vec<String>,
}

pub fn json_file_name(engine: &str, db_name: &str) -> String {
    format!("good_ormning_{}_{}.json", engine, db_name)
}

pub fn rs_file_name(db_name: &str) -> String {
    format!("good_ormning_{}.rs", db_name)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum TimeLibrary {
    #[cfg(feature = "chrono")]
    Chrono,
    #[cfg(feature = "jiff")]
    Jiff,
}

pub fn time_library(setting: Option<TimeLibrary>) -> Result<Option<TimeLibrary>, String> {
    if setting.is_some() {
        return Ok(setting);
    }
    let enabled: Vec<TimeLibrary> = vec![
        #[cfg(feature = "chrono")]
        TimeLibrary::Chrono,
        #[cfg(feature = "jiff")]
        TimeLibrary::Jiff,
    ];
    if enabled.len() > 1 {
        return Err(
            "Both the `chrono` and `jiff` features are enabled, so set `default_time_library` in `GenerateArgs` to choose the Rust type of untyped dates and times (like `now()`)".to_string(),
        );
    }
    return Ok(enabled.into_iter().next());
}

#[derive(Serialize, Deserialize)]
pub struct SchemaFile<V> {
    pub default_time_library: Option<TimeLibrary>,
    pub versions: HashMap<usize, V>,
}

pub struct RustTypes {
    pub arg_type: TokenStream,
    pub custom_trait: TokenStream,
    pub ret_type: TokenStream,
}

pub fn sanitize_ident(v: &str) -> (bool, String) {
    return match v {
        "as" |
        "break" |
        "const" |
        "continue" |
        "crate" |
        "else" |
        "enum" |
        "extern" |
        "false" |
        "fn" |
        "for" |
        "if" |
        "impl" |
        "in" |
        "let" |
        "loop" |
        "match" |
        "mod" |
        "move" |
        "mut" |
        "pub" |
        "ref" |
        "return" |
        "self" |
        "Self" |
        "static" |
        "struct" |
        "super" |
        "trait" |
        "true" |
        "type" |
        "unsafe" |
        "use" |
        "where" |
        "while" |
        "async" |
        "await" |
        "dyn" |
        "abstract" |
        "become" |
        "box" |
        "do" |
        "final" |
        "macro" |
        "override" |
        "priv" |
        "typeof" |
        "unsized" |
        "virtual" |
        "yield" |
        "try" => (
            true,
            format!("{}_", v),
        ),
        s => (false, s.into()),
    };
}

pub struct Tokens {
    out: String,
    quote: char,
}

impl Tokens {
    pub fn f(&mut self, f: impl FnOnce(&mut Self)) -> &mut Self {
        f(self);
        return self;
    }

    pub fn id(&mut self, i: &str) -> &mut Self {
        if !self.out.is_empty() {
            self.out.push(' ');
        }
        self.out.push(self.quote);
        if self.quote == '`' {
            self.out.push_str(&i.replace('`', "``"));
        } else {
            self.out.push_str(i);
        }
        self.out.push(self.quote);
        return self;
    }

    pub fn new() -> Tokens {
        return Tokens {
            out: String::new(),
            quote: '"',
        };
    }

    pub fn new_mariadb() -> Tokens {
        return Tokens {
            out: String::new(),
            quote: '`',
        };
    }

    pub fn s(&mut self, s: &str) -> &mut Self {
        if !self.out.is_empty() {
            self.out.push(' ');
        }
        self.out.push_str(s);
        return self;
    }
}

impl Default for Tokens {
    fn default() -> Self {
        return Self::new();
    }
}

impl fmt::Display for Tokens {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return f.write_str(&self.out);
    }
}
