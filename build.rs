use itertools::Itertools;
use nom::{
    bytes::complete::{is_a, tag, take_until},
    character::complete::{line_ending, space0},
    combinator::recognize,
    multi::many1,
    sequence::tuple,
    IResult,
};
use nom_cheatsheet_shared::markdown_format_code;
use std::{
    collections::HashMap,
    env,
    fs::{read_to_string, File},
    io::{Result, Write},
    path::Path,
};

static TABLE_HEADER_SEP: &str = "|---|---|---|---|---|";

#[derive(Clone, Debug)]
struct Url {
    module: String,
    name: String,
    docsurl: String,
}

#[derive(Debug)]
struct Combinator<'a> {
    urls: Vec<Url>,
    usage: String,
    input: &'a str,
    description: &'a str,
}

fn parse_code(input: &str) -> IResult<&str, &str> {
    let (input, backticks) = is_a("`")(input)?;
    let (input, code) = take_until(backticks)(input)?;
    let (input, _) = tag(backticks)(input)?;
    // Strip a single space from the beginning and the end of the code,
    // but only if they're both there. If only one is there, leave it.
    let code = if code.len() >= 2 && code.starts_with(' ') && code.ends_with(' ') {
        &code[1..code.len() - 1]
    } else {
        code
    };
    Ok((input, code))
}

fn sep(input: &str) -> IResult<&str, &str> {
    let (input, _) = space0(input)?;
    let (input, _) = tag("|")(input)?;
    let (input, _) = space0(input)?;
    Ok((input, ""))
}

// This parses a single table row
fn parse_combinator(input: &str) -> IResult<&str, Combinator> {
    let (input, _) = sep(input)?;
    let (input, urls): (&str, &str) = take_until("|")(input)?;
    let urls = urls.trim_end();
    //println!("cargo:warning=URLS={:?}", urls);
    let (input, _) = space0(input)?;
    let (input, _) = sep(input)?;
    let (input, usage) = parse_code(input)?;
    //println!("cargo:warning=Usage={:?}", usage);
    let (input, _) = sep(input)?;
    let (input, example_input) = parse_code(input)?;
    //println!("cargo:warning=Example Input={:?}", example_input);
    let (input, _) = sep(input)?;
    let (input, _) = sep(input)?;
    let (input, description) = take_until("|")(input)?;
    let description = description.trim_end();
    //println!("cargo:warning=Description={:?}", description);
    let (input, _) = sep(input)?;
    let (input, _) = line_ending(input)?;

    /*
     * Unfortunately some of the processing happens here in the parser, and
     * some of it happens in the generator. Ideally, we'd follow compilers'
     * style. First just parse, then do any transformations separately
     * and do generation as a third separate step.
     *
     * But for now, just putting this comment here. O:)
     */
    let urls = urls
        .split("<br>")
        .filter_map(|url| {
            if url.is_empty() {
                return None;
            }
            let mut parts = url.split("::").collect::<Vec<_>>();
            let name = parts.pop().unwrap().to_string();
            let path = parts.join("::");
            let mut url: String = "https://docs.rs/nom/latest/nom/".to_string();
            for part in parts {
                url.push_str(part);
                url.push('/');
            }
            if name.chars().next().unwrap().is_lowercase() {
                url.push_str("fn.");
            } else {
                url.push_str("enum.");
            }
            url.push_str(&name);
            url.push_str(".html");
            Some(Url {
                module: path,
                name,
                docsurl: url,
            })
        })
        .collect::<Vec<_>>();
    let mut name = String::new();
    if !urls.is_empty() {
        name.clone_from(&urls[0].name);
    }
    let usage = usage.to_string();
    Ok((
        input,
        Combinator {
            urls,
            usage,
            input: example_input,
            description,
        },
    ))
}

// This parses a single table and returns a vector of combinators, and also returns the
// text before the table.
fn parse_preamble_and_combinators(input: &str) -> IResult<&str, (&str, Vec<Combinator>)> {
    let (input, preamble) = recognize(tuple((
        take_until(TABLE_HEADER_SEP),
        tag(TABLE_HEADER_SEP),
        line_ending,
    )))(input)?;

    let (input, combinators) = many1(parse_combinator)(input)?;
    Ok((input, (preamble, combinators)))
}

fn main() -> Result<()> {
    let input = read_to_string("src/nom-cheatsheet-template.md")?;

    // This snags a Vec of Tuples
    // .0 is all the text since the start of the file or the end of the previous table
    // upto and including the header of the current table, aka preamble
    // .1 is the vector of combinators in the current table
    let (remainder, result): (&str, Vec<(&str, Vec<Combinator>)>) =
        many1(parse_preamble_and_combinators)(&input).unwrap();

    let mut fnmain: Vec<u8> = Vec::new();
    let mut uses: HashMap<String, String> = HashMap::new();
    let mut last_urls: Vec<Url> = Vec::new();

    // the include macro only works with a single expression in the file,
    // so turn the whole file into a single block with { and }
    writeln!(&mut fnmain, "{{")?;

    for table in result {
        // Preamble already ends with a newline, so use print instead of println
        // escape braces first, though
        let preamble = table.0.replace('{', "{{").replace('}', "}}");
        // Use ##### (5#) for the raw string literal, so that the included file can use
        // #### (4#) for its raw strings. That way, those can contain up to ### (3#) without
        // any trouble. That should be enough for markdown headers.
        //
        // Preamble is put into the end result as-is.
        writeln!(
            &mut fnmain,
            r#####"write!(markdown, r####"{preamble}"####)?;"#####
        )?;
        for combinator in table.1 {
            // Put each row in its own block, so that we can `use` without
            // conflicts
            writeln!(&mut fnmain, "{{")?;

            // XXX: As said in the parser, there's transformations here that should
            // be done elsewhere. Leaving that for later.
            let mut input = combinator.input.to_string();
            // Some traits are implemented for slices, but not for references to
            // arrays. So we add `[..]` to those, to make them slices.
            if input.starts_with("&[") {
                input.push_str("[..]");
            }
            // And byte strings are &str, but we want to treat them as &[u8]
            if input.starts_with("b\"") {
                input.push_str(" as &[u8]");
            }

            let urls = if combinator.urls.is_empty() {
                last_urls
            } else {
                combinator.urls.clone()
            };
            for Url {
                module,
                name,
                docsurl: _,
            } in &urls
            {
                // filter out any modules that end with streaming or start with bits
                if module.ends_with("streaming") || module.starts_with("bits") {
                    continue;
                }
                let use_statement =
                    format!("#[allow(unused_imports)]\nuse nom::{module}::{name};\n");
                // Write it within our current block
                fnmain.write_all(use_statement.as_bytes())?;
                // We also store them all so we can have use statements at the
                // top of the file for using things in other examples.
                //
                // We also put all of these into a HashMap so we can dedup them
                // by name, keeping the last one. This is because we have both
                // character::complete::i8 and number::complete::i8, and we only
                // want one. We just keep the last one we see.
                //
                // Allow unused imports for these specific ones, as not all are
                // used in the examples
                uses.insert(name.clone(), use_statement);
            }

            writeln!(&mut fnmain, "let input = {input};")?;

            let urlstrings = combinator
                .urls
                .iter()
                .map(
                    |Url {
                         module,
                         name,
                         docsurl,
                     }| format!("{module}::[{name}]({docsurl})"),
                )
                .collect::<Vec<_>>()
                .join("<br>");

            // Some examples need explicit types in the let statement, they will
            // start with "let output", the rest don't for brevity.
            let usage = combinator.usage.to_string();
            let assignment: String = if usage.starts_with("let output") {
                format!("{usage}({input});\n")
            } else {
                format!("let output: IResult<_, _> = {usage}(input);\n")
            };
            let assignment = assignment.replace("\\|", "|");

            fnmain.write_all(assignment.as_bytes())?;

            // Make sure that output is properly escaped
            writeln!(
                &mut fnmain,
                r#"let output = format_iresult(input, &output);"#
            )?;
            // Escape braces in the usage and input strings
            let usage = markdown_format_code(&usage);
            let usage = usage.replace('{', "{{");
            let usage = usage.replace('}', "}}");
            let input = markdown_format_code(combinator.input);
            let input = input.replace('{', "{{");
            let input = input.replace('}', "}}");
            writeln!(
                &mut fnmain,
                r#####"writeln!(markdown, r####"| {} | {} | {} | {{output}} | {} |"####)?;"#####,
                urlstrings, usage, input, combinator.description,
            )?;
            writeln!(&mut fnmain, "}}")?;
            last_urls = urls;
        }
    }

    let remainder = remainder.replace('{', "{{").replace('}', "}}");

    writeln!(
        &mut fnmain,
        r#####"write!(markdown, r####"{remainder}"####)?;"#####
    )?;

    writeln!(&mut fnmain, "}}")?;

    let main_file = Path::new(&env::var("OUT_DIR").unwrap()).join("main.rs");
    let mut main_file = File::create(main_file)?;
    main_file.write_all(&fnmain)?;

    let uses_file = Path::new(&env::var("OUT_DIR").unwrap()).join("uses.rs");
    let mut uses_file = File::create(uses_file)?;
    let uses = uses
        .into_iter()
        .sorted()
        .dedup()
        .map(|(_, v)| v)
        .collect::<String>();
    uses_file.write_all(uses.as_bytes())?;
    Ok(())
}
