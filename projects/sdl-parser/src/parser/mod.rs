mod can_parse;
mod config;
mod regroup;

pub use crate::parser::{can_parse::CanParse, config::ParserConfig};
use crate::ParserResult;
use sdl_ast::{Template, AST};
use sdl_pest::{Pair, Pairs, Parser, Rule, SDLParser};

use crate::parser::regroup::PREC_CLIMBER;
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
        let input = input.as_text()?.replace("\r\n", "\n").replace("\\\n", "").replace("\t", &" ".repeat(self.tab_size));
        Ok(self.parse_program(SDLParser::parse(Rule::program, &input)?))
    }
    fn parse_program(&self, pairs: Pairs<Rule>) -> AST {
        let mut codes = vec![];
        for pair in pairs {
            if let Rule::statement = pair.as_rule() {
                codes.push(self.parse_statement(pair));
            };
        }
        AST::program(codes)
    }
    fn parse_statement(&self, pairs: Pair<Rule>) -> AST {
        let r = self.get_position(pairs.as_span());
        let mut codes = vec![];
        for pair in pairs.into_inner() {
            let code = match pair.as_rule() {
                Rule::WHITESPACE => continue,
                Rule::expression => self.parse_expression(pair),
                Rule::if_statement => self.parse_if_else(pair),
                Rule::for_statement => self.parse_for_in(pair),

                _ => debug_cases!(pair),
            };
            codes.push(code);
        }
        AST::statement(codes, r)
    }
    fn parse_block(&self, pairs: Pair<Rule>) -> AST {
        let pair = pairs.into_inner().nth(0).unwrap();
        match pair.as_rule() {
            Rule::statement => self.parse_statement(pair),
            _ => unreachable!(),
        }
    }
}

impl ParserConfig {
    fn parse_if_else(&self, pairs: Pair<Rule>) -> AST {
        let r = self.get_position(pairs.as_span());
        let mut conditions = vec![];
        let mut actions = vec![];
        for pair in pairs.into_inner() {
            match pair.as_rule() {
                Rule::expr => conditions.push(self.parse_expr(pair)),
                Rule::block => actions.push(self.parse_block(pair)),
                _ => debug_cases!(pair),
            };
        }
        AST::if_else_chain(conditions, actions, r)
    }

    fn parse_for_in(&self, pairs: Pair<Rule>) -> AST {
        let r = self.get_position(pairs.as_span());
        // let mut codes = vec![];
        let (mut pattern, mut terms, mut block) = Default::default();
        for pair in pairs.into_inner() {
            match pair.as_rule() {
                Rule::WHITESPACE => continue,
                Rule::pattern => pattern = self.parse_pattern(pair),
                Rule::expr => terms = self.parse_expr(pair),
                Rule::block => block = self.parse_block(pair),
                _ => debug_cases!(pair),
            };
        }
        AST::for_in_loop(pattern, terms, block, r)
    }
}

impl ParserConfig {
    fn parse_expression(&self, pairs: Pair<Rule>) -> AST {
        let r = self.get_position(pairs.as_span());
        let mut terms = pairs.into_inner();
        let expr = self.parse_expr(terms.next().unwrap());
        let eos = terms.next().is_some();
        AST::expression(expr, eos, r)
    }

    #[rustfmt::skip]
    fn parse_expr(&self, pairs: Pair<Rule>) -> AST {
        let r = self.get_position(pairs.as_span());
        PREC_CLIMBER.climb(
            pairs.into_inner(),
            |pair: Pair<Rule>| match pair.as_rule() {
                //Rule::expr => self.parse_expr(pair),
                Rule::term => self.parse_term(pair),
                _ => debug_cases!(pair),
            },
            |left: AST, op: Pair<Rule>, right: AST| match op.as_rule() {
                _ => AST::infix_expression(self.parse_operation(op, "="), left, right, r),
            },
        )
    }

    fn parse_term(&self, pairs: Pair<Rule>) -> AST {
        // let pos = get_position(pairs.as_span());
        let mut base = AST::default();
        // let mut prefix = vec![];
        // let mut suffix = vec![];
        for pair in pairs.into_inner() {
            match pair.as_rule() {
                Rule::WHITESPACE => continue,
                Rule::chain_call => base = self.parse_chain_call(pair),
                //Rule::term => base = self.parse_node(pair),
                //Rule::Prefix => prefix.push(pair.as_str().to_string()),
                //Rule::Suffix => suffix.push(pair.as_str().to_string()),
                _ => debug_cases!(pair),
                //_ => unreachable!(),
            };
        }
        return base;
    }

    fn parse_pattern(&self, pairs: Pair<Rule>) -> AST {
        let pair = pairs.into_inner().nth(0).unwrap();
        match pair.as_rule() {
            Rule::SYMBOL => self.parse_symbol(pair),
            _ => debug_cases!(pair),
        }
    }

    fn parse_chain_call(&self, pairs: Pair<Rule>) -> AST {
        // let r = self.get_position(pairs.as_span());
        let mut items = pairs.into_inner();
        let base = self.parse_data(items.next().unwrap());

        // let mut terms = vec![];
        for pair in items {
            match pair.as_rule() {
                _ => debug_cases!(pair),
            };
        }
        return base;
    }

    fn parse_operation(&self, pairs: Pair<Rule>, kind: &str) -> AST {
        let r = self.get_position(pairs.as_span());
        let op = pairs.as_str();
        AST::operation(op, kind, r)
    }
}

impl ParserConfig {
    fn parse_data(&self, pairs: Pair<Rule>) -> AST {
        let pair = pairs.into_inner().nth(0).unwrap();
        match pair.as_rule() {
            Rule::template => self.parse_template(pair),
            Rule::list => self.parse_list(pair),
            Rule::String => self.parse_string(pair),
            Rule::Number => self.parse_number(pair),
            Rule::Symbol => self.parse_symbol(pair),
            Rule::SpecialValue => self.parse_special(pair),

            _ => debug_cases!(pair),
        }
    }
    fn parse_template(&self, pairs: Pair<Rule>) -> AST {
        let r = self.get_position(pairs.as_span());
        let mut tag = AST::default();
        let mut attributes = vec![];
        let mut arguments = vec![];
        let mut children = vec![];
        let pair = pairs.into_inner().nth(0).unwrap();
        let mut template = match pair.as_rule() {
            Rule::SelfClose => Template::self_close(),
            Rule::HTMLBad => Template::html_bad(),
            Rule::OpenClose => Template::open_close(),
            _ => debug_cases!(pair),
        };
        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::Symbol => tag = self.parse_symbol(inner),
                Rule::HTMLBadSymbol => tag = self.parse_symbol(inner),
                Rule::text_mode => children.push(self.parse_text_mode(inner)),

                Rule::BadSymbol => attributes.push(self.parse_symbol(inner)),
                Rule::html_pair => arguments.push(self.parse_pair(inner)),
                _ => debug_cases!(inner),
            };
        }
        template.set_tag(tag);
        template.set_attributes(attributes);
        template.set_arguments(arguments);
        return AST::template(template, r);
    }
    fn parse_text_mode(&self, pairs: Pair<Rule>) -> AST {
        let r = self.get_position(pairs.as_span());
        let mut terms = vec![];
        let mut text = vec![];
        for pair in pairs.into_inner() {
            match pair.as_rule() {
                Rule::WHITESPACE => continue,
                Rule::statement => terms.push(self.parse_statement(pair)),
                Rule::text_char => text.push(pair.as_str()),
                _ => debug_cases!(pair),
            };
        }
        AST::block(terms, r)
    }

    fn parse_list(&self, pairs: Pair<Rule>) -> AST {
        let r = self.get_position(pairs.as_span());
        let mut terms = vec![];
        for pair in pairs.into_inner() {
            match pair.as_rule() {
                Rule::WHITESPACE => continue,
                Rule::Comma => continue,
                Rule::expr => terms.push(self.parse_expr(pair)),
                _ => debug_cases!(pair),
            };
        }
        AST::list(terms, r)
    }
    fn parse_pair(&self, pairs: Pair<Rule>) -> (AST, AST) {
        let (mut key, mut value) = Default::default();
        for pair in pairs.into_inner() {
            match pair.as_rule() {
                Rule::Set => continue,
                Rule::BadSymbol => key = self.parse_string(pair),
                Rule::term => value = self.parse_term(pair),
                _ => debug_cases!(pair),
            };
        }
        (key, value)
    }
    fn parse_symbol(&self, pairs: Pair<Rule>) -> AST {
        let r = self.get_position(pairs.as_span());
        let mut value = vec![];
        for pair in pairs.into_inner() {
            match pair.as_rule() {
                Rule::SYMBOL => value.push(self.parse_string(pair)),
                _ => debug_cases!(pair),
            };
        }
        AST::symbol(value, r)
    }

    fn parse_string(&self, pairs: Pair<Rule>) -> AST {
        let r = self.get_position(pairs.as_span());
        AST::string(pairs.as_str().to_string(), r)
    }

    fn parse_number(&self, pairs: Pair<Rule>) -> AST {
        let r = self.get_position(pairs.as_span());
        AST::integer(pairs.as_str().to_string(), r)
    }
    fn parse_special(&self, pairs: Pair<Rule>) -> AST {
        let r = self.get_position(pairs.as_span());
        match pairs.as_str() {
            "true" => AST::boolean(true, r),
            "false" => AST::boolean(false, r),
            _ => AST::null(r),
        }
    }
}
