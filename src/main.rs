use std::{
    env::args, fs::read_to_string, io::{self, stdin, Write}, path::PathBuf
};
struct Tape {
    tape: [u8;30000],
    pointer: usize,
}

impl Tape {
    pub fn new() -> Self {
        Self{
            tape: [0; 30000],
            pointer: 0,
        }
    }
    pub fn get(&self) -> u8{
        return self.tape[self.pointer];
    }
    pub fn ls(&mut self, amount: usize) {
        let (val, wrap) = self.pointer.overflowing_sub(amount);
        if wrap {
            self.pointer = 0;
        } else {
            self.pointer = val;
        }
    }
    pub fn rs(&mut self, amount: usize) {
        self.pointer += amount;
    }
    pub fn add(&mut self, amount: u8) {
        self.tape[self.pointer] = self.tape[self.pointer].wrapping_add(amount);
    }
    pub fn sub(&mut self, amount: u8) {
        self.tape[self.pointer] = self.tape[self.pointer].wrapping_sub(amount);
    }
    pub fn write(&mut self, num: u8) {
        self.tape[self.pointer] = num;
    }
}

struct Tokenizer {
    str: Vec<char>,
    i: usize,
}

const CHARS: &str = "[]+-<>.,";

impl Tokenizer {
    pub fn new(str:String) -> Self{
        Self{
            str: str.chars().collect(),
            i: 0,
        }
    }
    pub fn get(&self) -> Option<char> {
        if self.str.len() <= self.i {
            return None;
        }
        return Some(self.str[self.i]);
    }
    pub fn next(&mut self) -> Option<char> {
        self.i+=1;
        if let Some(v) = self.get() {
            let mut c = v;
            if !CHARS.contains(c) {
                while !CHARS.contains(c) {
                    self.i+=1;
                    if let Some(v) = self.get() {
                        c = v;
                    } else {
                        break;
                    }
                } 
            }
        }
        return self.get(); 
    }
}

macro_rules! arguments {
    ($name:ident, $usage:expr, $($value:ident: $type:ty => $index:expr),+$(,)?) => {
        #[derive(Debug)]
        struct $name {
            $(
                $value: $type
            ),+
        } 
        impl $name {
            pub fn from(vec: Vec<String>) -> Self {
                Self {
                    $(
                        $value: vec.get($index).expect(format!("Usage: {}", $usage).as_str()).into()
                    ),+
                }
            }
        }
    };
}

arguments!{
    Data,
    "brainf [file.bf]",
    file: PathBuf => 1,
}

#[derive(Debug)]
enum TokenType {
    Math,
    Shift,
    JZ,
    NZ,
    In,
    Out,
}

#[derive(Debug)]
struct Token {
    token_type: TokenType,
    value: isize,
}

fn helper(tokenizer: &mut Tokenizer, tokens: &mut Vec<Token>, token: TokenType, value: isize, a: char, s: char) {
    let mut token = Token {
        token_type: token,
        value,
    };     
    if let Some(v) = tokenizer.next() {
        let mut c = v; 
        while c == a || c == s {
            if c == a {
                token.value += 1;
            } else {
                token.value -= 1;
            }
            if let Some(v) = tokenizer.next() {
                c = v;
            } else {
                break;
            }
        }
    }
    tokens.push(token);
}

fn main() {
    let data = Data::from(args().collect()); 
    let file = read_to_string(data.file).expect("file doesn't exist");
    let mut tokenizer = Tokenizer::new(file);
    let mut tokens: Vec<Token> = Vec::new();
    let mut stack: Vec<usize> = Vec::new();
    while let Some(c) = tokenizer.get() {
        match c {
            '+' => {
                helper(&mut tokenizer, &mut tokens, TokenType::Math, 1, '+', '-'); 
            }
            '-' => {
                helper(&mut tokenizer, &mut tokens, TokenType::Math, -1, '+', '-'); 
            }
            '>' => {
                helper(&mut tokenizer, &mut tokens, TokenType::Shift, 1, '>', '<'); 
            }
            '<' => {
                helper(&mut tokenizer, &mut tokens, TokenType::Shift, -1, '>', '<'); 
            }
            '.' => {
                tokens.push(Token {
                    token_type: TokenType::Out,
                    value: 0,
                });
                tokenizer.next();
            }
            ',' => {
                tokens.push(Token {
                    token_type: TokenType::In,
                    value: 0,
                });
                tokenizer.next();
            }
            '[' => {
                let len = tokens.len();
                tokens.push(Token {
                    token_type: TokenType::JZ,
                    value: 0,
                });
                stack.push(len);
                tokenizer.next();
            }
            ']' => {
                if stack.len() == 0 {
                    panic!("no loop was opened");
                }
                let i = stack.pop().unwrap();
                let after = tokens.len();
                tokens.push(Token {
                    token_type: TokenType::NZ,
                    value: i as isize,
                });
                tokens[i].value = after as isize;
                tokenizer.next();
            }
            _ => {},
        }
    }

    let mut tape = Tape::new();
    let mut i = 0;
    while i < tokens.len() {
        let token = &tokens[i];
        match token.token_type {
            TokenType::JZ => {
                if tape.get() == 0 {
                    i = token.value as usize; 
                }
            },
            TokenType::NZ => {
                if tape.get() != 0 {
                    i = token.value as usize;
                }
            },
            TokenType::Out => {
                print!("{}",  tape.get() as char);
            },
            TokenType::In => {
                let mut s = String::new();
                println!("");
                stdin().read_line(&mut s).expect("Did not enter a correct string");
                let c = s.chars().collect::<Vec<char>>()[0];
                tape.write(c as u8); 
            },
            TokenType::Shift => {
                if token.value < 0 {
                    tape.ls(token.value.abs() as usize);
                } else {
                    tape.rs(token.value as usize);
                }
            },
            TokenType::Math => {
                let sub = token.value < 0;
                let value = token.value.abs();
                let amount = (value as f32 / 255.0).floor() as i32;
                for _ in 0..amount {
                    if sub {
                        tape.sub(255);
                    } else {
                        tape.add(255);
                    }
                }
                if sub {
                    tape.sub((value % 255) as u8);
                } else {
                    tape.add((value % 255) as u8);
                }
            },
        }
        io::stdout().flush().unwrap();
        i+=1;
    }
}
