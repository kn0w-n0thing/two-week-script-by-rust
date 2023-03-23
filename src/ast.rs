use std::collections::LinkedList;

use crate::lexer::Token;

#[derive(Debug)]
pub enum ASTree {
    LEAF { token: Token },
    LIST { token_list: LinkedList<Token> },
}

impl ASTree {
    pub fn child(&self, i: usize) -> Result<&Token, String> {
        match self {
            ASTree::LEAF { .. } => {
                Err("Out of bounds!".to_string())
            }
            ASTree::LIST { token_list } => {
                token_list.iter().nth(i).ok_or("Out of bounds!".to_string())
            }
        }
    }

    pub fn children_number(&self) -> usize {
        match self {
            ASTree::LEAF { .. } => {
                0
            }
            ASTree::LIST { token_list } => {
                token_list.len()
            }
        }
    }

    pub fn children(&self) -> impl Iterator<Item=&Token> {
        ASTreeIter{ value: self, index: 0}
    }

    pub fn location(&self) -> String {
        match self {
            ASTree::LEAF { token } => {
                Self::get_location_from_token(token)
            }
            ASTree::LIST { token_list } => {
                Self::get_location_from_token(token_list.iter().nth(0).unwrap())
            }
        }
    }

    pub fn get_location_from_token(token: &Token) -> String {
        format!("at line: {}", token.get_line_number())
    }
}

pub struct ASTreeIter<'a> {
    value: &'a ASTree,
    index: usize,
}

impl<'a> Iterator for ASTreeIter<'a> {
    type Item = &'a Token;

    fn next(&mut self) -> Option<Self::Item> {
        match *self.value {
            ASTree::LEAF { .. } => {None},
            ASTree::LIST { .. } => {
                self.index += 1;
                self.value.child(self.index - 1).ok()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::LinkedList;
    use std::os::unix::raw::time_t;
    use regex::internal::Input;
    use ASTree::LEAF;
    use Token::IDENTIFIER;
    use crate::ast::ASTree;
    use crate::ast::ASTree::LIST;
    use crate::lexer::{Token, TokenBase};
    use crate::lexer::tests::{assert_id_token, assert_number_token};

    #[test]
    #[warn(non_snake_case)]
    fn ASTLeaf() -> () {
        let leaf = LEAF { token: IDENTIFIER {
            token_base: TokenBase { text: "".to_string(), line_number: 1, }
        }};

        assert_eq!(leaf.children_number(), 0);
        assert_eq!(leaf.child(0), Err("Out of bounds!".to_string()));
        let mut children = leaf.children();
        assert_eq!(children.next(), None);
    }

    fn ASTList() -> () {
        let token_number1 = Token::NUMBER { token_base: TokenBase { text: "1".to_string(), line_number: 1 }, number: 1 };
        let token_plus = Token::IDENTIFIER { token_base: TokenBase { text: "+".to_string(), line_number: 1 } };
        let token_number2 = Token::NUMBER { token_base: TokenBase { text: "2".to_string(), line_number: 1 }, number: 2 };
        let mut token_list = LinkedList::new();
        token_list.push_back(token_number1);
        token_list.push_back(token_plus);
        token_list.push_back(token_number2);
        let list = LIST { token_list };

        assert_eq!(list.children_number(), 3);
        let mut children = list.children();

        assert_number_token(children.next().unwrap(), 1, "1");
        assert_id_token(children.next().unwrap(), 1, "+");
        assert_number_token(children.next().unwrap(), 1, "2");
        assert_eq!(list.children_number(), 3);
    }
}