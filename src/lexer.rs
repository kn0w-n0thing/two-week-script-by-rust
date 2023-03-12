use std::collections::LinkedList;
use std::fmt::{Debug, Formatter};
use std::io::{BufRead, BufReader, Read};
use std::string::ToString;

use regex::Regex;

const TOKEN_REG_STR: &str = "\\s*((//.*)|([0-9]+)|(\"(\\\\\"|\\\\\\\\|\\\\n|[^\"])*\")|[A-Z_a-z][A-Z_a-z0-9]*|==|<=|>=|&&|\\|\\||[[:punct:]])?";
const EOF_ERR_STR: &str = "Already reach EOF!";

pub struct Lexer<R: Read> {
    has_more: bool,
    token_queue: LinkedList<Box<dyn Token>>,
    line_number: u32,
    reader: BufReader<R>,
}

impl<R: Read> Lexer<R> {
    #[inline]
    fn get_token_regex() -> Regex {
        Regex::new(TOKEN_REG_STR).unwrap()
    }

    pub fn new(reader: BufReader<R>) -> Self {
        Self {
            has_more: true,
            token_queue: LinkedList::new(),
            line_number: 0,
            reader,
        }
    }

    pub fn read(&mut self) -> Result<Box<dyn Token>, String> {
        let result = self.fill_queue(0)?;
        return if result {
            self.token_queue.pop_front().ok_or(String::from(""))
        } else {
            Err(String::from(EOF_ERR_STR))
        };
    }

    pub fn peek(&mut self, i: usize) -> Result<&Box<dyn Token>, String> {
        let result = self.fill_queue(i)?;
        return if result && i < self.token_queue.len() {
            Ok(self.token_queue.iter().nth(i).unwrap())
        } else {
            Err(String::from(EOF_ERR_STR))
        };
    }

    fn fill_queue(&mut self, i: usize) -> Result<bool, String> {
        while i >= self.token_queue.len() {
            if self.has_more {
                self.read_line()?;
            } else {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn read_line(&mut self) -> Result<(), String> {
        let mut line = String::new();
        let size = self.reader.read_line(&mut line)
            .map_err(|err| err.to_string())?;

        match size {
            0 => {
                self.has_more = false;
                self.token_queue.push_back(
                    Box::new(EofToken::new(self.line_number))
                );
            }
            _ => {
                self.line_number += 1;
                for cap in Self::get_token_regex().captures_iter(line.as_str()) {
                    if cap.get(1) == None || cap.get(2) != None {
                        // spaces or comments
                        break;
                    }

                    let token: Box<dyn Token>;

                    if cap.get(3) != None {
                        let number = (&cap[3]).parse::<i32>().map_err(|err| err.to_string())?;
                        token = Box::new(NumberToken::new(self.line_number, number));
                    } else if cap.get(4) != None {
                        token = Box::new(
                            StringToken::new(
                                self.line_number,
                                String::from(&cap[4]),
                            )
                        );
                    } else {
                        token = Box::new(
                            IdToken::new(
                                self.line_number,
                                String::from(&cap[1]),
                            )
                        );
                    }
                    self.token_queue.push_back(token);
                }
                self.token_queue.push_back(
                    Box::new(EolToken::new(self.line_number))
                );
            }
        }

        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Category {
    IDENTIFIER = 0,
    NUMBER,
    STRING,
    EOL,
    EOF,
}

#[derive(Debug)]
pub struct TokenBase {
    pub category: Category,
    pub line_number: u32,
}

pub trait Token {
    fn get_category(&self) -> Category;
    fn get_text(&self) -> String;
    fn get_number(&self) -> Result<i32, String>;
    fn get_line_number(&self) -> u32;
}

pub struct IdToken {
    pub token_base: TokenBase,
    pub id: String,
}

impl IdToken {
    pub fn new(line_number: u32, id: String) -> Self {
        Self {
            token_base: TokenBase {
                category: Category::IDENTIFIER,
                line_number,
            },
            id,
        }
    }
}

impl Token for IdToken {
    fn get_category(&self) -> Category {
        self.token_base.category
    }

    fn get_text(&self) -> String {
        self.id.clone()
    }

    fn get_number(&self) -> Result<i32, String> {
        Err("It's not a number token!".to_owned())
    }

    fn get_line_number(&self) -> u32 {
        self.token_base.line_number
    }
}

impl Debug for dyn Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "category: {:?}, text: {}, line_number: {}",
               self.get_category(), self.get_text(), self.get_line_number())
    }
}

#[derive(Debug)]
pub struct NumberToken {
    pub token_base: TokenBase,
    pub number: i32,
}

impl NumberToken {
    pub fn new(line_number: u32, number: i32) -> Self {
        Self {
            token_base: TokenBase {
                category: Category::NUMBER,
                line_number,
            },
            number,
        }
    }
}

impl Token for NumberToken {
    fn get_category(&self) -> Category {
        self.token_base.category
    }

    fn get_text(&self) -> String {
        self.number.to_string()
    }

    fn get_number(&self) -> Result<i32, String> {
        Ok(self.number)
    }

    fn get_line_number(&self) -> u32 {
        self.token_base.line_number
    }
}

#[derive(Debug)]
pub struct StringToken {
    pub token_base: TokenBase,
    pub literal: String,
}

impl StringToken {
    pub fn new(line_number: u32, text: String) -> Self {
        // remove "" from the both end
        Self {
            token_base: TokenBase {
                category: Category::STRING,
                line_number,
            },
            literal: String::from(&text[1..text.len() - 1]),
        }
    }
}

impl Token for StringToken {
    fn get_category(&self) -> Category {
        self.token_base.category
    }

    fn get_text(&self) -> String {
        self.literal.clone()
    }

    fn get_number(&self) -> Result<i32, String> {
        Err("It's not a number token!".to_owned())
    }

    fn get_line_number(&self) -> u32 {
        self.token_base.line_number
    }
}

#[derive(Debug)]
pub struct EolToken {
    pub token_base: TokenBase,
}

impl EolToken {
    pub fn new(line_number: u32) -> Self {
        Self {
            token_base: TokenBase {
                category: Category::EOL,
                line_number,
            },
        }
    }
}

impl Token for EolToken {
    fn get_category(&self) -> Category {
        self.token_base.category
    }

    fn get_text(&self) -> String {
        String::from("")
    }

    fn get_number(&self) -> Result<i32, String> {
        Err("It's not a number token!".to_owned())
    }

    fn get_line_number(&self) -> u32 {
        self.token_base.line_number
    }
}

#[derive(Debug)]
pub struct EofToken {
    pub token_base: TokenBase,
}

impl EofToken {
    pub fn new(line_number: u32) -> Self {
        Self {
            token_base: TokenBase {
                category: Category::EOF,
                line_number,
            },
        }
    }
}

impl Token for EofToken {
    fn get_category(&self) -> Category {
        self.token_base.category
    }

    fn get_text(&self) -> String {
        String::from("")
    }

    fn get_number(&self) -> Result<i32, String> {
        Err("It's not a number token!".to_owned())
    }

    fn get_line_number(&self) -> u32 {
        self.token_base.line_number
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Read};

    use stringreader::StringReader;

    use crate::lexer::{Category, EOF_ERR_STR, Lexer, Token};

    fn get_bufreader_from_str(string: &str) -> BufReader<StringReader> {
        let string_reader = StringReader::new(&string);
        BufReader::new(string_reader)
    }

    fn assert_number_token(token: &Box<dyn Token>, line_number: u32, text: &str) {
        assert_eq!(token.get_line_number(), line_number);
        assert_eq!(token.get_text(), text);
        assert_eq!(token.get_category(), Category::NUMBER);
    }

    fn assert_string_token(token: &Box<dyn Token>, line_number: u32, text: &str) {
        assert_eq!(token.get_line_number(), line_number);
        assert_eq!(token.get_text(), text);
        assert_eq!(token.get_category(), Category::STRING);
    }

    fn assert_id_token(token: &Box<dyn Token>, line_number: u32, text: &str) {
        assert_eq!(token.get_line_number(), line_number);
        assert_eq!(token.get_text(), text);
        assert_eq!(token.get_category(), Category::IDENTIFIER);
    }

    fn assert_eol_token(token: &Box<dyn Token>, line_number: u32) {
        assert_eq!(token.get_line_number(), line_number);
        assert_eq!(token.get_text(), "");
        assert_eq!(token.get_category(), Category::EOL);
    }

    fn assert_eof_token(token: &Box<dyn Token>, line_number: u32) {
        assert_eq!(token.get_line_number(), line_number);
        assert_eq!(token.get_text(), "");
        assert_eq!(token.get_category(), Category::EOF);
    }

    fn read_and_assert_number_token<R: Read>(lexer: &mut Lexer<R>, line_number: u32, text: &str) {
        let read_result = lexer.read();
        assert!(read_result.is_ok());
        let token = read_result.unwrap();
        assert_number_token(&token, line_number, text);
    }

    fn read_and_assert_id_token<R: Read>(lexer: &mut Lexer<R>, line_number: u32, text: &str) {
        let read_result = lexer.read();
        assert!(read_result.is_ok());
        let token = read_result.unwrap();
        assert_id_token(&token, line_number, text);
    }

    fn read_and_assert_string_token<R: Read>(lexer: &mut Lexer<R>, line_number: u32, text: &str) {
        let read_result = lexer.read();
        assert!(read_result.is_ok());
        let token = read_result.unwrap();
        assert_string_token(&token, line_number, text);
    }

    fn read_and_assert_eof_token<R: Read>(lexer: &mut Lexer<R>, line_number: u32) {
        let read_result = lexer.read();
        assert!(read_result.is_ok());
        let token = read_result.unwrap();
        assert_eof_token(&token, line_number);
    }

    fn peek_and_assert_eof_token<R: Read>(lexer: &mut Lexer<R>, i: usize, line_number: u32) {
        let peek_result = lexer.peek(i);
        assert!(peek_result.is_ok());
        let token_ref = peek_result.unwrap();
        assert_eof_token(token_ref, line_number);
    }

    fn read_and_assert_eol_token<R: Read>(lexer: &mut Lexer<R>, line_number: u32) {
        let read_result = lexer.read();
        assert!(read_result.is_ok());
        let token = read_result.unwrap();
        assert_eol_token(&token, line_number);
    }

    fn read_and_assert_error<R: Read>(lexer: &mut Lexer<R>, error_str: &str) {
        let read_result = lexer.read();
        assert!(read_result.is_err());
        assert_eq!(read_result.err().unwrap(), error_str);
    }

    fn peek_and_assert_error<R: Read>(lexer: &mut Lexer<R>, i: usize, error_str: &str) {
        let peek_result = lexer.peek(i);
        assert!(peek_result.is_err());
        assert_eq!(peek_result.err().unwrap(), error_str);
    }

    #[test]
    fn read_empty() {
        let bufreader = get_bufreader_from_str("");
        let mut lexer = Lexer::new(bufreader);

        read_and_assert_eof_token(&mut lexer, 0);

        read_and_assert_error(&mut lexer, EOF_ERR_STR);
    }

    #[test]
    fn peek_empty() {
        let bufreader = get_bufreader_from_str("");
        let mut lexer = Lexer::new(bufreader);

        peek_and_assert_eof_token(&mut lexer, 0, 0);

        peek_and_assert_eof_token(&mut lexer, 0, 0);

        peek_and_assert_error(&mut lexer, 1,EOF_ERR_STR);

        peek_and_assert_error(&mut lexer, 2,EOF_ERR_STR);
    }

    #[test]
    fn read_one_number() {
        let bufreader = get_bufreader_from_str("100");
        let mut lexer = Lexer::new(bufreader);

        read_and_assert_number_token(&mut lexer, 1, "100");

        read_and_assert_eol_token(&mut lexer, 1);

        read_and_assert_eof_token(&mut lexer, 1);
    }

    #[test]
    fn read_one_id() {
        let string_reader = StringReader::new("i");
        let bufreader = BufReader::new(string_reader);
        let mut lexer = Lexer::new(bufreader);

        read_and_assert_id_token(&mut lexer, 1, "i");

        read_and_assert_eol_token(&mut lexer, 1);

        read_and_assert_eof_token(&mut lexer, 1);
    }

    #[test]
    fn read_one_string() {
        let string_reader = StringReader::new("\"hello\"");
        let bufreader = BufReader::new(string_reader);
        let mut lexer = Lexer::new(bufreader);

        read_and_assert_string_token(&mut lexer, 1, "hello");

        read_and_assert_eol_token(&mut lexer, 1);

        read_and_assert_eof_token(&mut lexer, 1);
    }

    #[test]
    fn read_assign_number() {
        let string_reader = StringReader::new("i = 1");
        let bufreader = BufReader::new(string_reader);
        let mut lexer = Lexer::new(bufreader);

        read_and_assert_id_token(&mut lexer, 1, "i");

        read_and_assert_id_token(&mut lexer, 1, "=");

        read_and_assert_number_token(&mut lexer, 1, "1");

        read_and_assert_eol_token(&mut lexer, 1);

        read_and_assert_eof_token(&mut lexer, 1);
    }
}