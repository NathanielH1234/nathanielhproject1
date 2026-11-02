use std::env; 
use std::fs;
use std::process::Command;

pub trait Compiler {
    //these are the main functions that the compiler needs to have

    fn compile(&mut self, source: &str);
    fn next_token(&mut self) -> String;
    fn parse(&mut self);
    fn current_token(&self) -> String;
    fn set_current_token(&mut self, tok: String);
}

pub trait LexicalAnalyzer {
    //this trait is where I deal with first section of compiling the LA. This is where the input is read
    // and separated into tokens so the compiler csan work with it
    fn get_char(&mut self) -> char;
    fn add_char(&mut self, c: char);
    fn lookup(&self, s: &str) -> bool;
}

pub trait SyntaxAnalyzer {
    //these functions determine whether the tokens are being used in the right order. 

    //I have one function fior each rule that is in the grammar
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






//This stores the input, and keeps track of where I am reading what and saves the tokens and the keywords
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

    //this converts the source into characters so we can read them one at a time.

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

                //This is where I store all the possible #keywords
                "#HAI".into(), "#KBYE".into(), "#OBTW".into(), "#TLDR".into(),
                "#MAEK".into(), "#OIC".into(), "#GIMMEH".into(), "#MKAY".into(),
                "#NEWLINE".into(), "#IHAZ".into(), "#ITIZ".into(),
            ],
            keywords: vec![

                //These are my keywards I can use after annotations
                "HEAD".into(), "TITLE".into(), "PARAGRAF".into(), "BOLD".into(),
                "LIST".into(), "ITEM".into(), "LINX".into(), "VAR".into(),
            ],
        }
    }


    //This checks the character at the current position but doesnt move forward
    fn peek_char(&self) -> char {
        if self.position < self.input.len() { self.input[self.position] } else { '\0' }
    }



    //This just displays a lexical error that shows the line number and message to help user know what they did wrong
    fn lexical_error(&self, line: usize, msg: &str) {
        eprintln!("Lexical error on line {}: {}", line, msg);
        std::process::exit(1);
    }


    //this takes the text I have built so far and removes any extra weird spaces and saves it as a token but only if the token isnt empty
    fn flush_text(&mut self) {
        let text = std::mem::take(&mut self.current_build);
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            self.tokens.push(trimmed.to_string());
            self.token_lines.push(self.build_line);
        }
    }

    //This reads the actual annotation starting with the #, checks if it is recognised and if it is it will save it as a token
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


    //This reads the keyward that is after the annotation and checks if it is recognised thjen it will save it as a token if so
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

        //This pushes the recognised keyword into the tokens and saves its line number in token_lines
        self.tokens.push(word);
        self.token_lines.push(line);
    }


    //This goes through the input character by character at a time, separates the annotations from the normal text and saves everything as tokens
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


//This gets the character at the current position, moves to the next character and keeps track of the line number
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

    //this adds a character to the lexeme I am building and if its the first real charactrer it will save the line number so errors an be found.
    fn add_char(&mut self, c: char) {
        if self.current_build.trim().is_empty() && !c.is_whitespace() {
            self.build_line = self.line;
        }
        self.current_build.push(c);
    }

    //This checks if a word is a real annotation or keyword
    fn lookup(&self, s: &str) -> bool {

        //with the uppercase function if someone wants to put in a uppercase first it doesn't matter
        let up = s.to_uppercase();
        self.annotations.iter().any(|a| *a == up) || self.keywords.iter().any(|k| *k == up)
    }
}


//helper function that adds a line of html to the output and puts a new line after so I don't have to keep typing push_str and new line every time.
fn emit(out: &mut String, s: &str) {
    out.push_str(s);
    out.push('\n');
}

//This looks up the value of a variable by its name, this is my static semantic check before using a variable that was not defined
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


//This is my main compiler, it holds the lexer, the token I am currently looking at and holds the line number

pub struct LolCompiler {
    lexer: SimpleLexicalAnalyzer,
    current_tok: String,
    current_line: usize,
    //The record and record lines save each token the parser accepted in the order it was accepted in so the translate function can use them later on the line
    record: Vec<String>,
    record_lines: Vec<usize>,
}

impl LolCompiler {
    //This actually makes a new compiler with every single empty lexer, the real lexer gets made later on in the compile process when I have the sourcwe 
    pub fn new() -> Self {
        Self {
            lexer: SimpleLexicalAnalyzer::new(""),
            current_tok: String::new(),
            current_line: 1,
            record: Vec::new(),
            record_lines: Vec::new(),
        }
    }
    
    //This looks at the token after the current one without using it up
    //main reason I need this is because some annotations like #MAEK can start different things so I gotta see if the next word is paragraf or LIST because I know what rule to use
    fn peek_token(&self) -> String {
        let n = self.lexer.tokens.len();
        if n == 0 {
            String::new()
        } else {
            self.lexer.tokens[n - 1].clone()
        }
    }


    //This checks if the current token is plain text, which is just anything that is not empty and doesnt have the # symbol
    fn is_text(&self) -> bool {
        !self.current_tok.is_empty() && !self.current_tok.starts_with('#')
    }

    //This is just a niceity, that makes the current token look really nice for the errror messages and if there is no token left it will let you know its end of the file
    fn found(&self) -> String {
        if self.current_tok.is_empty() {
            "end of file".to_string()
        } else {
            format!("'{}'", self.current_tok)
        }
    }
//This displays a syntax error with the line number and then immediately stops the program since there is no point in using bad structure
    fn syntax_error(&self, msg: &str) {
        eprintln!("Syntax error on line {}: {}", self.current_line, msg);
        std::process::exit(1);
    }
//This is a shortcut for the error when a token is in a spot it shouldn't be, so it lets you know where
    fn unexpected(&self, context: &str) {
        self.syntax_error(&format!("unexpected {} inside {}.", self.found(), context));
    }
//this actually accepts the current token by saving it inside of record and with its line number and moves on to the next token
    fn consume(&mut self) {
        self.record.push(self.current_tok.clone());
        self.record_lines.push(self.current_line);
        self.next_token();
    }

    //this ensures the current token the one I need , if it is I consume if not I give a syntax error
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

    //There are the things that can show up in the body, a paragraph or a list item
    //so I put them all in one function so I dont have to write the same if else chain 3 times. It returns true if something is found and false if not
    fn parse_simple_item(&mut self) -> bool {
        if self.current_tok == "#OBTW" {
            self.parse_comment();
        } else if self.current_tok == "#NEWLINE" {
            self.parse_newline();
        } else if self.current_tok == "#GIMMEH" {

            //#GIMMEH is used for a bunch of different things so I peak at the keyword to know which rule I should actually follow
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
//This is the section that actually turns the tokens into html. It runs after the parser is finished so I already know the tokens are in a clean valid order.
//which is why I can use fixed jumps like i+=5 without being concerned about accidentally going out of bounds
    pub fn translate(&mut self) -> String {
        let rec = self.record.clone();
        let lines = self.record_lines.clone();
        let mut out = String::new();
        let mut vars: Vec<(String, String)> = Vec::new();
        //var stores the name, value of every variable, and marks rememebers how many variables existed
        let mut marks: Vec<usize> = Vec::new();
        let mut closers: Vec<String> = Vec::new();
        //the closers store the closing html tags I still have to write 
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

                //a comment is 3 tokens (#OBTW,text,#TLDR) so I use the middle one and skip ahead 3
                emit(&mut out, &format!("<!-- {} -->", rec[i + 1]));
                i += 3;
            } else if tok == "#MAEK" {
                //MAEK opens a head, paragraph or list so I pick the right html tags for whenever one it is
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

                //I save the closing tag for later and remember how many variabels there are right now so I can delete the ones made inside this block when it ends
                emit(&mut out, open);
                closers.push(close.to_string());
                marks.push(vars.len());
                i += 2;
            } else if tok == "#GIMMEH" {
                let kind = rec[i + 1].clone();
                if kind == "TITLE" {
                    //GIMMEH title text #OIC is 4 tokens
                    emit(&mut out, &format!("<title> {} </title>", rec[i + 2]));
                    i += 4;
                } else if kind == "BOLD" {
                    //bold can have lots of text pieces and variabkles in it so I loop until I hit the #OIC tag and put everything together wit the spaces
                    i += 2;
                    let mut text = String::new();
                    while rec[i] != "#OIC" {
                        if !text.is_empty() {
                            text.push(' ');

                            //if its a #GIMMEH then its a variable use so I swap it for the value of the variable
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

                    //a list item stays open until its #OIC so it works the same as #MAEK with the closers and marks 
                    emit(&mut out, "<li>");
                    closers.push("</li>".to_string());
                    marks.push(vars.len());
                    i += 2;
                } else if kind == "LINX" {
                    //I use the address for both the href and the text you can click on if the thing doesnt open automatically
                    let addr = rec[i + 2].clone();
                    emit(&mut out, &format!("<a href=\"{}\">{}</a>", addr, addr));
                    i += 4;
                } else {

                    //the only one left is VAR so this is just using the variable 
                    let value = resolve(&vars, &rec[i + 2], lines[i + 2]);
                    emit(&mut out, &value);
                    i += 4;
                }
            } else if tok == "#IHAZ" {

                //#IHAZ name #ITIZ value #MKAY is 5 tokens, this only saves the variable and doesn't write anything to the html
                vars.push((rec[i + 1].clone(), rec[i + 3].clone()));
                i += 5;
            } else if tok == "#NEWLINE" {
                emit(&mut out, "<br>");
                i += 1;
            } else if tok == "#MKAY" || tok == "#OIC" {

                //this is the end of a block so I write the closing tag that was saved last and delete any variables that were made inside of it
                let close = closers.pop().unwrap_or_default();
                let mark = marks.pop().unwrap_or_default();
                while vars.len() > mark {
                    vars.pop();
                }
                emit(&mut out, &close);
                i += 1;
            } else {
                //anything else left is just normal text so I then copy it straight into the output 
                emit(&mut out, &tok);
                i += 1;
            }
        }
        out
    }
}

impl Compiler for LolCompiler {

    //This makes a new lexer with the source, and runs to tokenize on it to gewt all the tokens and then loads the first token so it is ready for the parserr 
    fn compile(&mut self, source: &str) {
        self.lexer = SimpleLexicalAnalyzer::new(source);
        self.lexer.tokenize();
        self.record.clear();
        self.record_lines.clear();
        self.next_token();
    }

    //This takes the next token from the back of the list and makes it the current token
    //if there are no tokens left it returns an empty string to show that it is the end of file 

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


//this is the top rule so it checks the whole program is in the right order and nothing can come after the #KBYE symbol thing
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


//I take in the filepath as a string input and use it to open  the generated HTML file in the web browser. 
//But if it fails I display a message for the user to open it manually
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

    //I need 2 arguments the program and the lol file so if the len is anymore then 2 or any less than 2 I have to pop an error
    if args.len() != 2 {
        eprintln!("Usage error: expected exactly one argument.\nUsage: {} <input_file.lol>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    if !filename.to_lowercase().ends_with(".lol") {
        eprintln!("Usage error: input file '{}' must have a .lol extension.", filename);
        std::process::exit(1);
    }


    //make sure that the source is 100 percent a .lol file and I made it lowerercased so if someone puts in .LOL it will still work thru the llowercasse function
    let source = fs::read_to_string(filename).unwrap_or_else(|err| {
        eprintln!("Error reading file '{}': {}", filename, err);
        std::process::exit(1);
    });


    //here I made the compiler that loads the first token then it parses thru the syntax check then it starts the translation into the html
    let mut compiler = LolCompiler::new();
    compiler.compile(&source);
    compiler.parse();
    let html = compiler.translate();


    //here I swap ".lol" for ".html" by chopping off the last 4 chars
    let output_name = format!("{}.html", &filename[..filename.len() - 4]);
    fs::write(&output_name, html).unwrap_or_else(|err| {
        eprintln!("Error writing file '{}': {}", output_name, err);
        std::process::exit(1);
    });
    println!("Compiled '{}' -> '{}'", filename, output_name);
    open_in_browser(&output_name);
}