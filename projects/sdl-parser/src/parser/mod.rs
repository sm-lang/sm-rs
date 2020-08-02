mod can_parse;
mod config;
mod regroup;

pub use crate::parser::can_parse::CanParse;
pub use crate::parser::config::ParserConfig;
use crate::ParserResult;
use sdl_ast::{ASTKind, AST};
use sdl_pest::{Pair, Pairs, Parser, Rule, SDLParser};
use url::Url;

macro_rules! debug_cases {
    ($i:ident) => {{
        println!("Rule::{:?}=>continue,", $i.as_rule());
        println!("Span: {:?}", $i.as_span());
        println!("Text: {}", $i.as_str());
        unreachable!();
    }};
}

impl ParserConfig {
    pub fn parse(&mut self, input: impl CanParse) -> ParserResult<AST> {
        if let Some(s) = input.as_url() {
            self.file_url = Some(s)
        }
        let input = input
            .as_text()?
            .replace("\r\n", "\n")
            .replace("\\\n", "")
            .replace("\t", &" ".repeat(self.tab_size));
        let pairs = SDLParser::parse(Rule::program, &input)?;
        self.parse_program(pairs)
    }
    pub fn parse_program(&self, pairs: Pairs<Rule>) -> ParserResult<AST> {
        // let r = self.get_position(pairs.as_span());
        let mut codes = vec![];
        for pair in pairs {
            let code = match pair.as_rule() {
                Rule::EOI => continue,
                /*
                Rule::WHITE_SPACE => continue,
                Rule::LINE_SEPARATOR => continue,
                Rule::HorizontalRule => {
                    unimplemented!();
                    // AST::from("<hr/>")
                }
                Rule::Header => self.parse_header(pair),
                Rule::TextBlock => self.parse_paragraph(pair),
                Rule::List => {
                    codes.extend(self.parse_list(pair));
                    continue;
                }
                Rule::Table => {
                    codes.extend(self.parse_table(pair));
                    continue;
                }
                Rule::Code => self.parse_code_block(pair),
                Rule::CommandBlock => self.parse_command_block(pair),
                */
                _ => debug_cases!(pair),
            };
            // println!("{:?}", code);
            codes.push(code);
        }
        // FIXME: fix range
        Ok(AST::statements(codes, Default::default()))
    }
    /*
    fn parse_list(&self, pairs: Pair<Rule>) -> Vec<AST> {
        // let r = self.get_position(pairs.as_span());
        let mut list_terms: Vec<(usize, &str, Vec<AST>)> = vec![];
        for pair in pairs.into_inner() {
            match pair.as_rule() {
                Rule::LINE_SEPARATOR => continue,
                Rule::ListFirstLine | Rule::ListRestLine => {
                    let mut kind = "";
                    let mut indent = 0;
                    let mut inner = pair.into_inner();
                    while let Some(n) = inner.next() {
                        match n.as_rule() {
                            Rule::WHITE_SPACE => indent += 1,
                            Rule::ListMark | Rule::Vertical => {
                                kind = n.as_str();
                                break;
                            }
                            _ => debug_cases!(n),
                        }
                    }
                    let terms = inner.map(|pair| self.parse_span_term(pair)).collect();
                    list_terms.push((indent, kind, terms))
                }
                _ => debug_cases!(pair),
            };
        }
        return regroup_list_view(&list_terms);
    }
    fn parse_table(&self, pairs: Pair<Rule>) -> Vec<AST> {
        // let r = self.get_position(pairs.as_span());
        let mut table_terms = vec![];
        for pair in pairs.into_inner() {
            match pair.as_rule() {
                Rule::LINE_SEPARATOR => continue,
                Rule::TableFirstLine | Rule::TableRestLine => {
                    let mut line = vec![];
                    let mut inner = pair.into_inner();
                    let head = inner.next().unwrap();
                    let mut item = match head.as_rule() {
                        Rule::Vertical => vec![],
                        _ => vec![self.parse_span_term(head)],
                    };
                    for n in inner {
                        match n.as_rule() {
                            Rule::Vertical => {
                                line.push(item);
                                item = vec![]
                            }
                            _ => item.push(self.parse_span_term(n)),
                        }
                    }
                    if item.is_empty() {
                        table_terms.push(line)
                    }
                    else {
                        line.push(item);
                        table_terms.push(line)
                    }
                }
                _ => unreachable!(),
            };
        }
        return regroup_table_view(&table_terms);
    }
    pub fn parse_code_block(&self, pairs: Pair<Rule>) -> AST {
        let r = self.get_position(pairs.as_span());
        let mut lang = String::new();
        let mut code = String::new();
        for pair in pairs.into_inner() {
            match pair.as_rule() {
                Rule::WHITE_SPACE => continue,
                Rule::CodeLevel => continue,
                Rule::CodeMark => continue,
                Rule::SYMBOL => lang = pair.as_str().to_string(),
                Rule::CodeText => code = pair.as_str().to_string(),
                _ => debug_cases!(pair),
            };
        }
        let code = CodeBlock { lang, code, inline: false, high_line: vec![] };
        AST::code(code, r)
    }

    fn parse_header(&self, pairs: Pair<Rule>) -> AST {
        let r = self.get_position(pairs.as_span());
        let mut level = 0;
        let mut children = vec![];
        for pair in pairs.into_inner() {
            match pair.as_rule() {
                Rule::WHITE_SPACE => continue,
                Rule::Sharp => level += 1,
                _ => children.push(self.parse_span_term(pair)),
            };
        }
        return AST::header(children, level, r);
    }
    pub fn parse_command_block(&self, pairs: Pair<Rule>) -> AST {
        let r = self.get_position(pairs.as_span());
        let mut cmd = String::new();
        for pair in pairs.into_inner() {
            match pair.as_rule() {
                Rule::Escape => continue,
                Rule::SYMBOL => cmd = pair.as_str().to_string(),
                _ => debug_cases!(pair),
            };
        }
        let cmd = Command { cmd, kind: CommandKind::Normal, args: vec![], kvs: Default::default() };
        AST::command(cmd, r)
    }
    pub fn parse_paragraph(&self, pairs: Pair<Rule>) -> AST {
        let r = self.get_position(pairs.as_span());
        let codes = self.parse_span(pairs);
        match codes.len() {
            0 => {
                return AST::default();
            }
            1 => {
                if let ASTKind::MathDisplay(math) = &codes[0].kind() {
                    return AST::math(math.as_ref().to_owned(), "block", r);
                }
            }
            _ => (),
        }
        AST::paragraph(codes, r)
    }
    fn parse_span(&self, pairs: Pair<Rule>) -> Vec<AST> {
        pairs.into_inner().map(|pair| self.parse_span_term(pair)).collect()
    }
    fn parse_span_term(&self, pair: Pair<Rule>) -> AST {
        match pair.as_rule() {
            Rule::EOI => AST::default(),
            Rule::Style => self.parse_styled_text(pair),
            Rule::TextRest => self.parse_normal_text(pair),
            Rule::TildeLine => self.parse_tilde_text(pair),
            Rule::Raw => self.parse_raw_text(pair),
            Rule::Math => self.parse_math_text(pair),
            Rule::RawRest | Rule::StyleRest | Rule::TildeRest | Rule::MathRest => self.parse_normal_text(pair),
            Rule::WHITE_SPACE | Rule::LINE_SEPARATOR => self.parse_normal_text(pair),
            Rule::Escaped => self.parse_escaped(pair),

            _ => debug_cases!(pair),
        }
    }

    fn parse_normal_text(&self, pairs: Pair<Rule>) -> AST {
        let r = self.get_position(pairs.as_span());
        AST::text(pairs.as_str().to_string(), "normal", r)
    }
    fn parse_styled_text(&self, pairs: Pair<Rule>) -> AST {
        let s = pairs.as_str().to_string();
        let r = self.get_position(pairs.as_span());
        let mut level = 0;
        let mut text = vec![];
        for pair in pairs.into_inner() {
            match pair.as_rule() {
                Rule::Asterisk => continue,
                Rule::StyleLevel => level += pair.as_str().len(),
                Rule::StyleText => text.extend(self.parse_span(pair)),
                _ => debug_cases!(pair),
            };
        }
        match level {
            1 => AST::style(text, "*", r),
            2 => AST::style(text, "**", r),
            3 => AST::style(text, "***", r),
            _ => AST::text(s, "normal", r),
        }
    }
    fn parse_tilde_text(&self, pairs: Pair<Rule>) -> AST {
        let s = pairs.as_str().to_string();
        let r = self.get_position(pairs.as_span());
        let mut level = 0;
        let mut text = vec![];
        for pair in pairs.into_inner() {
            match pair.as_rule() {
                Rule::Tilde => continue,
                Rule::TildeLevel => level += pair.as_str().len(),
                Rule::TildeText => text = self.parse_span(pair),
                _ => debug_cases!(pair),
            };
        }
        match level {
            1 => AST::style(text, "~", r),
            2 => AST::style(text, "~~", r),
            3 => AST::style(text, "~~~", r),
            _ => AST::text(s, "normal", r),
        }
    }
    fn parse_raw_text(&self, pairs: Pair<Rule>) -> AST {
        let r = self.get_position(pairs.as_span());
        for pair in pairs.into_inner() {
            if let Rule::RawText = pair.as_rule() {
                return AST::text(pair.as_str().to_string(), "raw", r);
            };
        }
        return AST::default();
    }
    fn parse_math_text(&self, pairs: Pair<Rule>) -> AST {
        let s = pairs.as_str().to_string();
        let r = self.get_position(pairs.as_span());
        let mut inner = pairs.into_inner();
        let level = inner.next().unwrap().as_str().len();
        let text = inner.next().unwrap().as_str().to_string();
        match level {
            1 => AST::math(text, "inline", r),
            2 => AST::math(text, "display", r),
            _ => AST::text(s, "normal", r),
        }
    }
    fn parse_escaped(&self, pairs: Pair<Rule>) -> AST {
        let r = self.get_position(pairs.as_span());
        let c = match pairs.as_str().chars().next() {
            None => '\\',
            Some(s) => s,
        };
        AST::escaped(c, r)
    }
    */
}

/*
fn parse_table_align(input: &str) -> Vec<u8> {
    let pairs = SDLParser::parse(Rule::TableMode, input).unwrap_or_else(|e| panic!("{}", e));
    let mut codes = vec![];
    let mut text = String::new();
    for pair in pairs {
        match pair.as_rule() {
            Rule::EOI => continue,
            Rule::WHITE_SPACE => text.push(' '),
            Rule::TableRest => text.push_str(pair.as_str()),
            Rule::TableMark => {
                let mut code = 0;
                if text.contains(":-") {
                    code += 1 << 0
                }
                if text.contains("-:") {
                    code += 1 << 1
                }
                codes.push(code);
                text = String::new();
            }
            _ => debug_cases!(pair),
        };
    }
    return codes;
}

#[derive(Debug)]
pub enum List {
    Quote,
    Ordered,
    Orderless,
}

impl List {
    pub fn get_type(input: &str) -> (usize, List) {
        let pairs = List::parse_pairs(input);
        let mut i = 0;
        let mut m = List::Quote;
        for pair in pairs {
            match pair.as_rule() {
                Rule::WHITE_SPACE => i += 1,
                Rule::ListMark => match pair.as_str() {
                    ">" => m = List::Quote,
                    "-" => m = List::Orderless,
                    _ => m = List::Ordered,
                },
                _ => return (i, m),
            };
        }
        return (i, m);
    }
    pub fn trim_indent(line: &str, _indent: usize, ty: &List) -> (bool, String) {
        let mut new = false;
        let mut vec: VecDeque<_> = List::parse_pairs(line).into_iter().collect();
        match ty {
            List::Quote => match vec[0].as_rule() {
                Rule::ListMark => match vec[0].as_str() {
                    ">" => {
                        vec.pop_front();
                    }
                    _ => (),
                },
                _ => (),
            },
            List::Ordered => match vec[0].as_rule() {
                Rule::ListMark => match vec[0].as_str() {
                    "-" | ">" => (),
                    _ => {
                        vec.pop_front();
                        new = true
                    }
                },
                _ => (),
            },
            List::Orderless => match vec[0].as_rule() {
                Rule::ListMark => match vec[0].as_str() {
                    "-" => {
                        vec.pop_front();
                        new = true
                    }
                    _ => (),
                },
                _ => (),
            },
        }
        let v: Vec<&str> = vec.iter().map(|x| x.as_str()).collect();
        return (new, v.join(""));
    }
    fn parse_pairs(input: &str) -> Pairs<Rule> {
        let p = SDLParser::parse(Rule::ListMode, input).unwrap_or_else(|e| panic!("{}", e));
        p.into_iter().next().unwrap().into_inner()
    }
}
*/