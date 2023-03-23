use std::collections::LinkedList;
use std::fmt::{Debug, Formatter};
use std::io::{BufRead, BufReader, Read};
use std::string::ToString;

use regex::Regex;

const TOKEN_REG_STR: &str = "\\s*((//.*)|([0-9]+)|(\"(\\\\\"|\\\\\\\\|\\\\n|[^\"])*\")|[A-Z_a-z][A-Z_a-z0-9]*|==|<=|>=|&&|\\|\\||[[:punct:]])?";
const EOF_ERR_STR: &str = "Already reach EOF!";

pub struct Lexer<R: Read> {
    has_more: bool,
    token_queue: LinkedList<Token>,
    line_number: usize,
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

    pub fn read(&mut self) -> Result<Token, String> {
        let result = self.fill_queue(0)?;
        return if result {
            self.token_queue.pop_front().ok_or(String::from(""))
        } else {
            Err(String::from(EOF_ERR_STR))
        };
    }

    pub fn peek(&mut self, i: usize) -> Result<&Token, String> {
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
                self.token_queue.push_back(Token::EOF { token_base: TokenBase { line_number: self.line_number, text: "".to_string() } });
            }
            _ => {
                self.line_number += 1;
                for cap in Self::get_token_regex().captures_iter(line.as_str()) {
                    if cap.get(1) == None || cap.get(2) != None {
                        // spaces or comments
                        break;
                    }

                    let token: Token;

                    if cap.get(3) != None {
                        let number = (&cap[3]).parse::<i32>().map_err(|err| err.to_string())?;
                        token = Token::NUMBER { token_base: TokenBase { line_number: self.line_number, text: cap[3].to_string() }, number };
                    } else if cap.get(4) != None {
                        token = Token::STRING { token_base: TokenBase { line_number: self.line_number, text: cap[4][1..cap[4].len() - 1].to_string() } };
                    } else {
                        token = Token::IDENTIFIER { token_base: TokenBase { line_number: self.line_number, text: cap[1].to_string() } };
                    }
                    self.token_queue.push_back(token);
                }
                self.token_queue.push_back(Token::EOL { token_base: TokenBase { line_number: self.line_number, text: "".to_string() } });
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenBase {
    pub text: String,
    pub line_number: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    IDENTIFIER { token_base: TokenBase },
    NUMBER { token_base: TokenBase, number: i32 },
    STRING { token_base: TokenBase },
    EOL { token_base: TokenBase },
    EOF { token_base: TokenBase },
}

impl Token {
    pub fn get_text(&self) -> String {
        match self {
            Token::IDENTIFIER { token_base, .. }
            | Token::NUMBER { token_base, .. }
            | Token::STRING { token_base, .. }
            | Token::EOL { token_base, .. }
            | Token::EOF { token_base, .. } => { token_base.text.clone() }
        }
    }

    pub fn get_line_number(&self) -> usize {
        match self {
            Token::IDENTIFIER { token_base, .. }
            | Token::NUMBER { token_base, .. }
            | Token::STRING { token_base, .. }
            | Token::EOL { token_base, .. }
            | Token::EOF { token_base, .. } => { token_base.line_number.clone() }
        }
    }

    pub fn get_number(&self) -> Result<&i32, String> {
        match self {
            Token::NUMBER { number, .. } => { Ok(number) }
            _ => { Err("Unsupported function!".to_string()) }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::io::{BufReader, Read};

    use stringreader::StringReader;

    use crate::lexer::{EOF_ERR_STR, Lexer, Token, TokenBase};

    fn get_bufreader_from_str(string: &str) -> BufReader<StringReader> {
        let string_reader = StringReader::new(&string);
        BufReader::new(string_reader)
    }

    pub fn assert_number_token(token: &Token, line_number: usize, text: &str) {
        assert_eq!(*token, Token::NUMBER {
            token_base: TokenBase { line_number, text: text.to_string() },
            number: text.parse().unwrap(),
        });
    }

    pub fn assert_string_token(token: &Token, line_number: usize, text: &str) {
        assert_eq!(*token, Token::STRING { token_base: TokenBase { line_number, text: text.to_string() } });
    }

    pub fn assert_id_token(token: &Token, line_number: usize, text: &str) {
        assert_eq!(*token, Token::IDENTIFIER { token_base: TokenBase { line_number, text: text.to_string() } });
    }

    fn assert_eol_token(token: &Token, line_number: usize) {
        assert_eq!(*token, Token::EOL { token_base: TokenBase { line_number, text: "".to_string() } });
    }

    fn assert_eof_token(token: &Token, line_number: usize) {
        assert_eq!(*token, Token::EOF { token_base: TokenBase { line_number, text: "".to_string() } });
    }

    fn read_and_assert_number_token<R: Read>(lexer: &mut Lexer<R>, line_number: usize, text: &str) {
        let read_result = lexer.read();
        assert!(read_result.is_ok());
        let token = read_result.unwrap();
        assert_number_token(&token, line_number, text);
    }

    fn read_and_assert_id_token<R: Read>(lexer: &mut Lexer<R>, line_number: usize, text: &str) {
        let read_result = lexer.read();
        assert!(read_result.is_ok());
        let token = read_result.unwrap();
        assert_id_token(&token, line_number, text);
    }

    fn read_and_assert_string_token<R: Read>(lexer: &mut Lexer<R>, line_number: usize, text: &str) {
        let read_result = lexer.read();
        assert!(read_result.is_ok());
        let token = read_result.unwrap();
        assert_string_token(&token, line_number, text);
    }

    fn read_and_assert_eof_token<R: Read>(lexer: &mut Lexer<R>, line_number: usize) {
        let read_result = lexer.read();
        assert!(read_result.is_ok());
        let token = read_result.unwrap();
        assert_eof_token(&token, line_number);
    }

    fn peek_and_assert_eof_token<R: Read>(lexer: &mut Lexer<R>, i: usize, line_number: usize) {
        let peek_result = lexer.peek(i);
        assert!(peek_result.is_ok());
        let token_ref = peek_result.unwrap();
        assert_eof_token(token_ref, line_number);
    }

    fn read_and_assert_eol_token<R: Read>(lexer: &mut Lexer<R>, line_number: usize) {
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

        peek_and_assert_error(&mut lexer, 1, EOF_ERR_STR);

        peek_and_assert_error(&mut lexer, 2, EOF_ERR_STR);
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