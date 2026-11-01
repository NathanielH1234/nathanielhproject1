use std::env; //test comment 1
use std::fs;
use std::process::Command;

pub trait Compiler {
    fn compile(&mut self, source: &str);
    fn next_token(&mut self) -> String;
    fn parse(&mut self);
    fn current_token(&self) -> String;
    fn set_current_token(&mut self, tok: String);
}

pub trait LexicalAnalyzer {
    fn get_char(&mut self) -> char;
    fn add_char(&mut self, c: char);
    fn lookup(&self, s: &str) -> bool;
}

pub trait SyntaxAnalyzer {
    fn parse_lolcode(&mut self);
    fn parse_head(&mut self);
    fn parse_title(&mut self);
    fn parse_comment(&mut self);
    fn parse_body(&mut self);
    fn parse_paragraph(&mut self);
    fn parse_inner_paragraph(&mut self);
    fn parse_inner_text(&mut self);
    fn parse_variable_define(&mut self);
    fn parse_variable_use(&mut self);
    fn parse_bold(&mut self);
    fn parse_list(&mut self);
    fn parse_list_items(&mut self);
    fn parse_inner_list(&mut self);
    fn parse_link(&mut self);
    fn parse_newline(&mut self);
    fn parse_text(&mut self);
}

pub struct SimpleLexicalAnalyzer {
    input: Vec<char>,
    position: usize,
    line: usize,
    current_build: String,
    build_line: usize,
    pub tokens: Vec<String>,
    pub token_lines: Vec<usize>,
    pub annotations: Vec<String>,
    pub keywords: Vec<String>,
}

impl SimpleLexicalAnalyzer {
    pub fn new(source: &str) -> Self {
        Self {
            input: source.chars().collect(),
            position: 0,
            line: 1,
            current_build: String::new(),
            build_line: 1,
            tokens: Vec::new(),
            token_lines: Vec::new(),
            annotations: vec![
                "#HAI".into(), "#KBYE".into(), "#OBTW".into(), "#TLDR".into(),
                "#MAEK".into(), "#OIC".into(), "#GIMMEH".into(), "#MKAY".into(),
                "#NEWLINE".into(), "#IHAZ".into(), "#ITIZ".into(),
            ],
            keywords: vec![
                "HEAD".into(), "TITLE".into(), "PARAGRAF".into(), "BOLD".into(),
                "LIST".into(), "ITEM".into(), "LINX".into(), "VAR".into(),
            ],
        }
    }

    fn peek_char(&self) -> char {
        if self.position < self.input.len() { self.input[self.position] } else { '\0' }
    }

    fn lexical_error(&self, line: usize, msg: &str) {
        eprintln!("Lexical error on line {}: {}", line, msg);
        std::process::exit(1);
    }

    fn flush_text(&mut self) {
        let text = std::mem::take(&mut self.current_build);
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            self.tokens.push(trimmed.to_string());
            self.token_lines.push(self.build_line);
        }
    }

    fn read_annotation(&mut self) {
        let line = self.line;
        self.add_char('#');
        while self.peek_char().is_ascii_alphabetic() {
            let c = self.get_char();
            self.add_char(c);
        }
        let lexeme = std::mem::take(&mut self.current_build).to_uppercase();
        if !self.lookup(&lexeme) {
            self.lexical_error(line, &format!("'{}' is not a recognized annotation.", lexeme));
        }
        self.tokens.push(lexeme.clone());
        self.token_lines.push(line);
        if lexeme == "#MAEK" || lexeme == "#GIMMEH" {
            self.read_keyword(&lexeme);
        }
    }

    fn read_keyword(&mut self, after: &str) {
        while self.peek_char().is_whitespace() {
            self.get_char();
        }
        let line = self.line;
        loop {
            let c = self.peek_char();
            if c == '\0' || c == '#' || c.is_whitespace() {
                break;
            }
            let c = self.get_char();
            self.add_char(c);
        }
        let word = std::mem::take(&mut self.current_build).to_uppercase();
        if word.is_empty() {
            self.lexical_error(line, &format!("a keyword was expected after {}.", after));
        }
        if !self.lookup(&word) {
            self.lexical_error(line, &format!("'{}' is not a recognized keyword after {}.", word, after));
        }
        self.tokens.push(word);
        self.token_lines.push(line);
    }

    pub fn tokenize(&mut self) {
        loop {
            let c = self.get_char();
            if c == '\0' {
                break;
            }
            if c == '#' {
                self.flush_text();
                self.read_annotation();
            } else {
                self.add_char(c);
            }
        }
        self.flush_text();
        self.tokens.reverse();
        self.token_lines.reverse();
    }
}

impl LexicalAnalyzer for SimpleLexicalAnalyzer {
    fn get_char(&mut self) -> char {
        if self.position < self.input.len() {
            let c = self.input[self.position];
            self.position += 1;
            if c == '\n' {
                self.line += 1;
            }
            c
        } else {
            '\0'
        }
    }

    fn add_char(&mut self, c: char) {
        if self.current_build.trim().is_empty() && !c.is_whitespace() {
            self.build_line = self.line;
        }
        self.current_build.push(c);
    }

    fn lookup(&self, s: &str) -> bool {
        let up = s.to_uppercase();
        self.annotations.iter().any(|a| *a == up) || self.keywords.iter().any(|k| *k == up)
    }
}

fn emit(out: &mut String, s: &str) {
    out.push_str(s);
    out.push('\n');
}

fn resolve(vars: &Vec<(String, String)>, name: &str, line: usize) -> String {
    let mut k = vars.len();
    while k > 0 {
        k -= 1;
        if vars[k].0 == name {
            return vars[k].1.clone();
        }
    }
    eprintln!("Static semantic error on line {}: variable '{}' is used but was never defined in an enclosing scope.", line, name);
    std::process::exit(1);
}

pub struct LolCompiler {
    lexer: SimpleLexicalAnalyzer,
    current_tok: String,
    current_line: usize,
    record: Vec<String>,
    record_lines: Vec<usize>,
}

impl LolCompiler {
    pub fn new() -> Self {
        Self {
            lexer: SimpleLexicalAnalyzer::new(""),
            current_tok: String::new(),
            current_line: 1,
            record: Vec::new(),
            record_lines: Vec::new(),
        }
    }

    fn peek_token(&self) -> String {
        let n = self.lexer.tokens.len();
        if n == 0 {
            String::new()
        } else {
            self.lexer.tokens[n - 1].clone()
        }
    }

    fn is_text(&self) -> bool {
        !self.current_tok.is_empty() && !self.current_tok.starts_with('#')
    }

    fn found(&self) -> String {
        if self.current_tok.is_empty() {
            "end of file".to_string()
        } else {
            format!("'{}'", self.current_tok)
        }
    }

    fn syntax_error(&self, msg: &str) {
        eprintln!("Syntax error on line {}: {}", self.current_line, msg);
        std::process::exit(1);
    }

    fn unexpected(&self, context: &str) {
        self.syntax_error(&format!("unexpected {} inside {}.", self.found(), context));
    }

    fn consume(&mut self) {
        self.record.push(self.current_tok.clone());
        self.record_lines.push(self.current_line);
        self.next_token();
    }

    fn expect(&mut self, expected: &str) {
        if self.current_tok == expected {
            self.consume();
        } else {
            self.syntax_error(&format!("'{}' was expected but {} was found.", expected, self.found()));
        }
    }

    fn expect_word(&mut self, what: &str) {
        if !self.is_text() {
            self.syntax_error(&format!("{} was expected but {} was found.", what, self.found()));
        }
        if self.current_tok.chars().any(|c| c.is_whitespace()) {
            self.syntax_error(&format!("{} must be a single word with no spaces, found '{}'.", what, self.current_tok));
        }
        self.consume();
    }

    fn parse_simple_item(&mut self) -> bool {
        if self.current_tok == "#OBTW" {
            self.parse_comment();
        } else if self.current_tok == "#NEWLINE" {
            self.parse_newline();
        } else if self.current_tok == "#GIMMEH" {
            let next = self.peek_token();
            if next == "BOLD" {
                self.parse_bold();
            } else if next == "LINX" {
                self.parse_link();
            } else if next == "VAR" {
                self.parse_variable_use();
            } else {
                return false;
            }
        } else if self.is_text() {
            self.parse_text();
        } else {
            return false;
        }
        true
    }

    pub fn translate(&mut self) -> String {
        let rec = self.record.clone();
        let lines = self.record_lines.clone();
        let mut out = String::new();
        let mut vars: Vec<(String, String)> = Vec::new();
        let mut marks: Vec<usize> = Vec::new();
        let mut closers: Vec<String> = Vec::new();
        let mut i = 0;

        while i < rec.len() {
            let tok = rec[i].clone();
            if tok == "#HAI" {
                emit(&mut out, "<html>");
                i += 1;
            } else if tok == "#KBYE" {
                emit(&mut out, "</html>");
                i += 1;
            } else if tok == "#OBTW" {
                emit(&mut out, &format!("<!-- {} -->", rec[i + 1]));
                i += 3;
            } else if tok == "#MAEK" {
                let kind = rec[i + 1].clone();
                let open;
                let close;
                if kind == "HEAD" {
                    open = "<head>";
                    close = "</head>";
                } else if kind == "PARAGRAF" {
                    open = "<p>";
                    close = "</p>";
                } else {
                    open = "<ul>";
                    close = "</ul>";
                }
                emit(&mut out, open);
                closers.push(close.to_string());
                marks.push(vars.len());
                i += 2;
            } else if tok == "#GIMMEH" {
                let kind = rec[i + 1].clone();
                if kind == "TITLE" {
                    emit(&mut out, &format!("<title> {} </title>", rec[i + 2]));
                    i += 4;
                } else if kind == "BOLD" {
                    i += 2;
                    let mut text = String::new();
                    while rec[i] != "#OIC" {
                        if !text.is_empty() {
                            text.push(' ');
                        }
                        if rec[i] == "#GIMMEH" {
                            text.push_str(&resolve(&vars, &rec[i + 2], lines[i + 2]));
                            i += 4;
                        } else {
                            text.push_str(&rec[i]);
                            i += 1;
                        }
                    }
                    i += 1;
                    emit(&mut out, &format!("<b> {} </b>", text));
                } else if kind == "ITEM" {
                    emit(&mut out, "<li>");
                    closers.push("</li>".to_string());
                    marks.push(vars.len());
                    i += 2;
                } else if kind == "LINX" {
                    let addr = rec[i + 2].clone();
                    emit(&mut out, &format!("<a href=\"{}\">{}</a>", addr, addr));
                    i += 4;
                } else {
                    let value = resolve(&vars, &rec[i + 2], lines[i + 2]);
                    emit(&mut out, &value);
                    i += 4;
                }
            } else if tok == "#IHAZ" {
                vars.push((rec[i + 1].clone(), rec[i + 3].clone()));
                i += 5;
            } else if tok == "#NEWLINE" {
                emit(&mut out, "<br>");
                i += 1;
            } else if tok == "#MKAY" || tok == "#OIC" {
                let close = closers.pop().unwrap_or_default();
                let mark = marks.pop().unwrap_or_default();
                while vars.len() > mark {
                    vars.pop();
                }
                emit(&mut out, &close);
                i += 1;
            } else {
                emit(&mut out, &tok);
                i += 1;
            }
        }
        out
    }
}

impl Compiler for LolCompiler {
    fn compile(&mut self, source: &str) {
        self.lexer = SimpleLexicalAnalyzer::new(source);
        self.lexer.tokenize();
        self.record.clear();
        self.record_lines.clear();
        self.next_token();
    }

    fn next_token(&mut self) -> String {
        let tok = self.lexer.tokens.pop().unwrap_or_default();
        self.current_line = self.lexer.token_lines.pop().unwrap_or(self.current_line);
        self.current_tok = tok.clone();
        tok
    }

    fn parse(&mut self) {
        self.parse_lolcode();
    }

    fn current_token(&self) -> String {
        self.current_tok.clone()
    }

    fn set_current_token(&mut self, tok: String) {
        self.current_tok = tok;
    }
}

impl SyntaxAnalyzer for LolCompiler {
    fn parse_lolcode(&mut self) {
        self.expect("#HAI");
        while self.current_tok == "#OBTW" {
            self.parse_comment();
        }
        self.parse_head();
        if self.current_tok == "#IHAZ" {
            self.parse_variable_define();
        }
        self.parse_body();
        self.expect("#KBYE");
        if !self.current_tok.is_empty() {
            self.syntax_error(&format!("{} found after #KBYE; nothing may follow it.", self.found()));
        }
    }

    fn parse_head(&mut self) {
        if self.current_tok == "#MAEK" && self.peek_token() == "HEAD" {
            self.expect("#MAEK");
            self.expect("HEAD");
            self.parse_title();
            self.expect("#MKAY");
        }
    }

    fn parse_title(&mut self) {
        self.expect("#GIMMEH");
        self.expect("TITLE");
        self.parse_text();
        self.expect("#OIC");
    }

    fn parse_comment(&mut self) {
        self.expect("#OBTW");
        self.parse_text();
        self.expect("#TLDR");
    }

    fn parse_body(&mut self) {
        while self.current_tok != "#KBYE" {
            if self.current_tok == "#MAEK" {
                let next = self.peek_token();
                if next == "PARAGRAF" {
                    self.parse_paragraph();
                } else if next == "LIST" {
                    self.parse_list();
                } else {
                    self.unexpected("the document body");
                }
            } else if !self.parse_simple_item() {
                self.unexpected("the document body");
            }
        }
    }

    fn parse_paragraph(&mut self) {
        self.expect("#MAEK");
        self.expect("PARAGRAF");
        if self.current_tok == "#IHAZ" {
            self.parse_variable_define();
        }
        self.parse_inner_paragraph();
        self.expect("#MKAY");
    }

    fn parse_inner_paragraph(&mut self) {
        while self.current_tok != "#MKAY" {
            self.parse_inner_text();
        }
    }

    fn parse_inner_text(&mut self) {
        if self.current_tok == "#MAEK" {
            if self.peek_token() == "LIST" {
                self.parse_list();
            } else {
                self.unexpected("a paragraph");
            }
        } else if !self.parse_simple_item() {
            self.unexpected("a paragraph");
        }
    }

    fn parse_variable_define(&mut self) {
        self.expect("#IHAZ");
        self.expect_word("a variable name");
        self.expect("#ITIZ");
        self.parse_text();
        self.expect("#MKAY");
    }

    fn parse_variable_use(&mut self) {
        self.expect("#GIMMEH");
        self.expect("VAR");
        self.expect_word("a variable name");
        self.expect("#OIC");
    }

    fn parse_bold(&mut self) {
        self.expect("#GIMMEH");
        self.expect("BOLD");
        let mut count = 0;
        while self.current_tok != "#OIC" {
            if self.current_tok == "#GIMMEH" && self.peek_token() == "VAR" {
                self.parse_variable_use();
            } else {
                self.parse_text();
            }
            count += 1;
        }
        if count == 0 {
            self.syntax_error("a bold annotation must contain text.");
        }
        self.expect("#OIC");
    }

    fn parse_list(&mut self) {
        self.expect("#MAEK");
        self.expect("LIST");
        self.parse_list_items();
        self.expect("#MKAY");
    }

    fn parse_list_items(&mut self) {
        let mut count = 0;
        while self.current_tok == "#GIMMEH" && self.peek_token() == "ITEM" {
            self.expect("#GIMMEH");
            self.expect("ITEM");
            if self.current_tok == "#IHAZ" {
                self.parse_variable_define();
            }
            self.parse_inner_list();
            self.expect("#OIC");
            count += 1;
        }
        if count == 0 {
            self.syntax_error(&format!("a list must start with #GIMMEH ITEM but {} was found.", self.found()));
        }
    }

    fn parse_inner_list(&mut self) {
        while self.current_tok != "#OIC" {
            if !self.parse_simple_item() {
                self.unexpected("a list item");
            }
        }
    }

    fn parse_link(&mut self) {
        self.expect("#GIMMEH");
        self.expect("LINX");
        self.expect_word("a link address");
        self.expect("#OIC");
    }

    fn parse_newline(&mut self) {
        self.expect("#NEWLINE");
    }

    fn parse_text(&mut self) {
        if self.is_text() {
            self.consume();
        } else {
            self.syntax_error(&format!("plain text was expected but {} was found.", self.found()));
        }
    }
}

fn open_in_browser(path: &str) {
    let result = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", "start", "", path]).status()
    } else {
        Command::new("open").arg(path).status()
    };
    if result.is_err() {
        eprintln!("Note: could not open the browser automatically; open '{}' yourself.", path);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage error: expected exactly one argument.\nUsage: {} <input_file.lol>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    if !filename.to_lowercase().ends_with(".lol") {
        eprintln!("Usage error: input file '{}' must have a .lol extension.", filename);
        std::process::exit(1);
    }

    let source = fs::read_to_string(filename).unwrap_or_else(|err| {
        eprintln!("Error reading file '{}': {}", filename, err);
        std::process::exit(1);
    });

    let mut compiler = LolCompiler::new();
    compiler.compile(&source);
    compiler.parse();
    let html = compiler.translate();

    let output_name = format!("{}.html", &filename[..filename.len() - 4]);
    fs::write(&output_name, html).unwrap_or_else(|err| {
        eprintln!("Error writing file '{}': {}", output_name, err);
        std::process::exit(1);
    });
    println!("Compiled '{}' -> '{}'", filename, output_name);
    open_in_browser(&output_name);
}