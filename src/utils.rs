use std::{
    cell::RefCell,
    rc::Rc,
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
    err_ctx: Vec<Vec<(&'static str, String)>>,
}

#[derive(Clone)]
pub struct Errs(Rc<RefCell<Errs_>>);

impl Errs {
    pub(crate) fn new() -> Self {
        Self(Rc::new(RefCell::new(Errs_ {
            errs: vec![],
            err_ctx: vec![],
        })))
    }

    pub fn err(&self, t: String) {
        let mut s = self.0.as_ref().borrow_mut();
        let mut out = String::new();
        for (i, (k, v)) in s.err_ctx.iter().flatten().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            out.push_str(&format!("{}: {}", k, v));
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

    pub fn push_ctx(&self, v: Vec<(&'static str, String)>) {
        self.0.borrow_mut().err_ctx.push(v);
    }

    pub fn pop_ctx(&self) {
        self.0.borrow_mut().err_ctx.pop();
    }
}
