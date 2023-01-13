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

pub struct Errs {
    errs: Vec<String>,
    pub(crate) err_ctx: Vec<Vec<(&'static str, String)>>,
}

impl Errs {
    pub(crate) fn new() -> Self {
        Self {
            errs: vec![],
            err_ctx: vec![],
        }
    }

    pub fn err(&mut self, t: String) {
        let mut out = String::new();
        for (i, (k, v)) in self.err_ctx.iter().flatten().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            out.push_str(&format!("{}: {}", k, v));
        }
        out.push_str(" -- ");
        out.push_str(&t);
        self.errs.push(out);
    }

    pub fn raise(self) -> Result<(), Vec<String>> {
        if !self.errs.is_empty() {
            return Err(self.errs);
        }
        Ok(())
    }
}
