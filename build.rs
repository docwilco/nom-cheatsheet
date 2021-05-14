use anyhow::Result;
use itertools::Itertools;
use nom::{
    bytes::complete::{tag, take_until},
    character::complete::{char, line_ending},
    combinator::recognize,
    multi::many1,
    sequence::pair,
    IResult,
};
use std::{
    env,
    fs::{read_to_string, File},
    io::Write,
    path::Path,
};

static TABLE_HEADER: &str = "| combinator | usage | input | output | description |
|---|---|---|---|---|
";

#[derive(Debug)]
struct Combinator<'a> {
    name: String,
    urls: Vec<(String, String, String)>,
    usage: &'a str,
    input: &'a str,
    description: &'a str,
}

fn parse_combinator<'a>(input: &'a str) -> IResult<&'a str, Combinator<'a>> {
    let sep = " | ";
    let (input, _) = tag("| ")(input)?;
    let (input, urls): (&str, &str) = take_until(sep)(input)?;
    let (input, _) = tag(sep)(input)?;
    let (input, _) = char('`')(input)?;
    let (input, usage) = take_until("`")(input)?;
    let (input, _) = char('`')(input)?;
    let (input, _) = tag(sep)(input)?;
    let (input, _) = char('`')(input)?;
    let (input, example_input) = take_until("`")(input)?;
    let (input, _) = char('`')(input)?;
    let (input, _) = tag(sep)(input)?;
    let (input, _) = tag(sep)(input)?;
    let (input, description) = take_until(" |")(input)?;
    let (input, _) = tag(" |")(input)?;
    let (input, _) = line_ending(input)?;

    let urls = urls.split("<br>").fold(Vec::new(), |mut acc, url| {
        if url.len() == 0 {
            return acc;
        }
        let mut parts = url.split("::").collect::<Vec<_>>();
        let name = parts.pop().unwrap().to_string();
        let path = parts.join("::");
        let mut url: String = "https://docs.rs/nom/latest/nom/".to_string();
        for part in parts {
            url.push_str(part);
            url.push('/');
        }
        url.push_str("fn.");
        url.push_str(&name);
        url.push_str(".html");
        acc.push((path, name, url));
        acc
    });
    let mut name = "".to_string();
    if urls.len() > 0 {
        name = urls[0].1.clone();
    }
    Ok((
        input,
        Combinator {
            name,
            urls,
            usage,
            input: example_input,
            description,
        },
    ))
}

fn parse_preamble_and_combinators(input: &str) -> IResult<&str, (&str, Vec<Combinator>)> {
    let (input, preamble) = recognize(pair(take_until(TABLE_HEADER), tag(TABLE_HEADER)))(input)?;
    let (input, combinators) = many1(parse_combinator)(input)?;
    Ok((input, (preamble, combinators)))
}

fn main() -> Result<()> {
    let input = read_to_string("src/combinators-template.md")?;

    let (input, result): (&str, Vec<(&str, Vec<Combinator>)>) =
        many1(parse_preamble_and_combinators)(&input).unwrap();

    let mut fnmain: Vec<u8> = Vec::new();
    let mut uses: Vec<String> = Vec::new();
    writeln!(&mut fnmain, "{{")?;
    for table in result {
        // Preamble already ends with a newline, so use print instead of println
        // escape braces first, though
        let preamble = table.0.replace("{", "{{");
        let preamble = preamble.replace("}", "}}");
        writeln!(&mut fnmain, r#####"print!(r####"{}"####);"#####, preamble)?;
        for combinator in table.1 {
            let mut input = combinator.input.to_string();
            if input.starts_with("b\"") {
                input.push_str(" as &[u8]");
            }
            for (module, name, _) in combinator.urls.iter().filter(|(module, _, _)| {
                !(module.ends_with("streaming") || module.starts_with("bits"))
            }) {
                uses.push(format!(
                    "#[allow(unused_imports)]\nuse nom::{}::{};\n",
                    module, name
                ));
            }
            let test_code = format!("{}({})", combinator.usage, input);
            let urls = combinator
                .urls
                .iter()
                .map(|(module, name, docsurl)| format!("{}::[{}]({})", module, name, docsurl))
                .collect::<Vec<_>>()
                .join("<br>");
            println!("{}", test_code);
            writeln!(
                &mut fnmain,
                r#####"let debug: IResult<_, _> = {};"#####,
                test_code
            )?;
            let usage = combinator.usage.replace("{", "{{");
            let usage = usage.replace("}", "}}");
            writeln!(
                &mut fnmain,
                r#####"println!(r####"| {} | `{}` | `{}` | `{{:?}}` | {} |"####, debug);"#####,
                urls, usage, combinator.input, combinator.description,
            )?;
        }
    }
    let remainder = input.replace("{", "{{");
    let remainder = remainder.replace("}", "}}");

    writeln!(&mut fnmain, r#####"print!(r####"{}"####);"#####, remainder)?;

    writeln!(&mut fnmain, "}}")?;

    let main_file = Path::new(&env::var("OUT_DIR").unwrap()).join("main.rs");
    let mut main_file = File::create(main_file)?;
    main_file.write_all(&fnmain)?;

    let uses_file = Path::new(&env::var("OUT_DIR").unwrap()).join("uses.rs");
    let mut uses_file = File::create(uses_file)?;
    let uses = uses.into_iter().sorted().dedup().collect::<String>();
    uses_file.write_all(uses.as_bytes())?;
    Ok(())
}
