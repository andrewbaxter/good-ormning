use std::{
    cell::RefCell,
    rc::Rc,
    path::Path,
    fs,
};
use quote::quote;
use proc_macro2::TokenStream;

pub struct Output {
    pub(crate) data: Vec<TokenStream>,
}

impl Output {
    pub fn new() -> Output {
        Output { data: vec![] }
    }

    pub fn write(self, path: &Path) -> Result<(), String> {
        if let Some(p) = path.parent() {
            if let Err(e) = fs::create_dir_all(&p) {
                return Err(format!("Error creating output parent directories {}: {:?}", p.to_string_lossy(), e));
            }
        }
        let data = self.data;
        let tokens = quote!{
            #(#data) *
        };
        match genemichaels::format_str(&tokens.to_string(), &genemichaels::FormatConfig::default()) {
            Ok(src) => {
                match fs::write(path, src.rendered.as_bytes()) {
                    Ok(_) => { },
                    Err(e) => return Err(
                        format!("Failed to write generated code to {}: {:?}", path.to_string_lossy(), e),
                    ),
                };
            },
            Err(e) => {
                return Err(format!("Error formatting generated code: {:?}\n{}", e, tokens));
            },
        };
        Ok(())
    }
}

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
