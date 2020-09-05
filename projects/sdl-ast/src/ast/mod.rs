mod loops;
mod template;

pub use crate::ast::template::{Template, TemplateKind};
use crate::{ast::loops::ForInLoop, TextRange};
use std::{
    collections::HashMap,
    fmt::{self, Debug, Display, Formatter},
};

#[derive(Clone, Eq, PartialEq)]
pub enum AST {
    Node {
        kind: ASTKind,
        //      children: Box<[AST]>,
        children: Vec<AST>,
        r: Option<Box<TextRange>>,
    },
    Leaf {
        kind: ASTKind,
        r: Option<Box<TextRange>>,
    },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ASTKind {
    None,
    Program,
    Block,
    Statement,
    ForInLoop(Box<ForInLoop>),

    Expression(bool),
    InfixExpression,
    PrefixExpression,
    SuffixExpression,

    Template(Box<Template>),
    List,
    Dict,
    Pair,

    Null,
    Boolean(bool),
    String(String),
}

impl Default for AST {
    fn default() -> Self {
        Self::Leaf { kind: ASTKind::None, r: Default::default() }
    }
}

impl Debug for AST {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let children = self.children();
        match self.kind() {
            ASTKind::Null => write!(f, "null"),
            ASTKind::Boolean(v) => write!(f, "{}", v),
            ASTKind::String(v) => write!(f, "{}", v),
            _ => {
                let mut out = f.debug_struct("AST");
                out.field("kind", &self.kind());
                out.field("children", &children);
                out.finish()
            }
        }
    }
}

impl AST {
    pub fn kind(&self) -> ASTKind {
        match self {
            Self::Node { kind, .. } | Self::Leaf { kind, .. } => kind.to_owned(),
        }
    }

    pub fn children(&self) -> Vec<AST> {
        match self {
            Self::Node { children, .. } => children.to_vec(),
            Self::Leaf { .. } => vec![],
        }
    }
    pub fn range(&self) -> TextRange {
        match self {
            Self::Node { r, .. } | Self::Leaf { r, .. } => r.clone().unwrap_or_default().as_ref().clone(),
        }
    }
}

impl AST {
    pub fn program(children: Vec<AST>) -> Self {
        Self::Node { kind: ASTKind::Program, children, r: Default::default() }
    }
    pub fn block(children: Vec<AST>, r: TextRange) -> Self {
        Self::Node { kind: ASTKind::Block, children, r: box_range(r) }
    }
    pub fn statement(children: Vec<AST>, r: TextRange) -> Self {
        Self::Node { kind: ASTKind::Statement, children, r: box_range(r) }
    }
    pub fn for_in_loop(pattern: AST, terms: AST, block: AST, r: TextRange) -> Self {
        let kind = ForInLoop { pattern, terms, block };
        Self::Leaf { kind: ASTKind::ForInLoop(Box::new(kind)), r: box_range(r) }
    }

    pub fn expression(children: Vec<AST>, eos: bool, r: TextRange) -> Self {
        Self::Node { kind: ASTKind::Expression(eos), children, r: box_range(r) }
    }

    pub fn infix(op: &str, lhs: AST, rhs: AST, r: TextRange) -> Self {
        Self::Leaf { kind: ASTKind::InfixExpression, r: box_range(r) }
    }

    pub fn prefix(op: &str, rhs: AST, r: TextRange) -> Self {
        Self::Leaf { kind: ASTKind::PrefixExpression, r: box_range(r) }
    }

    pub fn suffix(op: &str, lhs: AST, r: TextRange) -> Self {
        Self::Leaf { kind: ASTKind::SuffixExpression, r: box_range(r) }
    }

    pub fn template(value: Template, r: TextRange) -> Self {
        Self::Leaf { kind: ASTKind::Template(Box::new(value)), r: box_range(r) }
    }

    pub fn list(value: Vec<AST>, r: TextRange) -> Self {
        Self::Node { kind: ASTKind::List, children: value, r: box_range(r) }
    }

    pub fn null(r: TextRange) -> Self {
        Self::Leaf { kind: ASTKind::Null, r: box_range(r) }
    }
    pub fn boolean(value: bool, r: TextRange) -> Self {
        Self::Leaf { kind: ASTKind::Boolean(value), r: box_range(r) }
    }
    pub fn string(value: String, r: TextRange) -> Self {
        Self::Leaf { kind: ASTKind::String(value), r: box_range(r) }
    }
}

fn box_range(r: TextRange) -> Option<Box<TextRange>> {
    match r.sum() {
        0 => None,
        _ => Some(Box::new(r)),
    }
}
