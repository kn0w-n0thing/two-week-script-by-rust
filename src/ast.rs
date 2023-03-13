use std::collections::LinkedList;

use crate::lexer::Token;

pub trait ASTree {
    fn child(&self, i: usize) -> Result<&Box<dyn ASTree>, String>;
    fn children_number(&self) -> usize;
    // fn children(&self) -> Iter<'_, Box<dyn ASTree>>;
    fn location(&self) -> String;
}

pub struct ASLeaf {
    token: Box<dyn Token>,
}

impl ASLeaf {
    pub fn new(token: Box<dyn Token>) -> ASLeaf {
        Self { token }
    }

    pub fn name(&self) -> String {
        self.token.get_text()
    }
}

impl ASTree for ASLeaf {
    fn child(&self, _i: usize) -> Result<&Box<dyn ASTree>, String> {
        Err("ASLeaf has no child!".to_owned())
    }

    fn children_number(&self) -> usize {
        0
    }

    fn location(&self) -> String {
        format!("at line: {}", self.token.get_line_number())
    }
}

pub struct ASList {
    children: LinkedList<Box<dyn ASTree>>,
}

impl ASTree for ASList {
    fn child(&self, i: usize) -> Result<&Box<dyn ASTree>, String> {
        if i < self.children.len() {
            Ok(&self.children.iter().nth(i).unwrap())
        } else {
            Err("".to_owned())
        }
    }

    fn children_number(&self) -> usize {
        self.children.len()
    }

    fn location(&self) -> String {
        let mut ret = "".to_owned();
        for child in &self.children {
            ret.push_str(child.location().as_str())
        }
        ret
    }
}