use std::{
    cell::RefCell,
    rc::Rc,
};
use proc_macro2::{
    TokenStream,
};

pub struct Tokens(String);

impl ToString for Tokens {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl Tokens {
    pub fn new() -> Tokens {
        Tokens(String::new())
    }

    pub fn s(&mut self, s: &str) -> &mut Self {
        if !self.0.is_empty() {
            self.0.push(' ');
        }
        self.0.push_str(s);
        self
    }

    pub fn id(&mut self, i: &str) -> &mut Self {
        if !self.0.is_empty() {
            self.0.push(' ');
        }
        self.0.push_str(&format!("\"{}\"", i));
        self
    }

    pub fn f(&mut self, f: impl FnOnce(&mut Self)) -> &mut Self {
        f(self);
        self
    }
}

pub struct Errs_ {
    errs: Vec<String>,
}

#[derive(Clone)]
pub struct Errs(Rc<RefCell<Errs_>>);

impl Errs {
    pub(crate) fn new() -> Self {
        Self(Rc::new(RefCell::new(Errs_ { errs: vec![] })))
    }

    pub fn err(&self, path: &rpds::Vector<String>, t: String) {
        let mut s = self.0.as_ref().borrow_mut();
        let mut out = String::new();
        for (i, k) in path.iter().enumerate() {
            if i > 0 {
                out.push_str("/");
            }
            out.push_str(k.as_ref());
        }
        out.push_str(" -- ");
        out.push_str(&t);
        s.errs.push(out);
    }

    pub fn raise(self) -> Result<(), Vec<String>> {
        let errs = self.0.borrow_mut().errs.split_off(0);
        if !errs.is_empty() {
            return Err(errs);
        }
        Ok(())
    }
}

pub fn sanitize_ident(v: &str) -> (bool, String) {
    match v {
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
    }
}

pub struct RustTypes {
    pub custom_trait: TokenStream,
    pub ret_type: TokenStream,
    pub arg_type: TokenStream,
}
