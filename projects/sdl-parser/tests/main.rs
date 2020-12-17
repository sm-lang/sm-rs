mod declare;
mod for_loop;
mod literal;
mod call_chain;

use sdl_ast::{SDLContext};
use sdl_parser::{ParserConfig, Result};

#[test]
fn ready() {
    println!("it works!")
}

pub fn render(input: &str) -> Result<String> {
    let mut parser = ParserConfig::default();
    let out = parser.parse(input)?;
    // println!("{:#?}", out);
    let mut ctx = SDLContext::default();
    let out = ctx.evaluate(&out)?;
    // println!("{:?}", out);
    Ok(ctx.render(&out)?)
}

const CODE: &'static str = r#"
<img rel src="https://avatars.githubusercontent.com/u/17541209?s=60&amp;v=4" alt="@GalAster" size="20" height="20" width="20" class="avatar-user avatar avatar--small ">
"#;

#[test]
fn template() {
    println!("{}", render(CODE).unwrap());
}
