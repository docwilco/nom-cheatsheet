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
    env,
    fs::{read_to_string, File},
    io::{Result, Write},
    path::Path,
};

static TABLE_HEADER1: &str = "| combinator | usage | input | output | description |";
static TABLE_HEADER2: &str = "|---|---|---|---|---|";

#[derive(Debug)]
struct Combinator<'a> {
    _name: String,
    urls: Vec<(String, String, String)>,
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
    let usage = if usage.is_empty() { None } else { Some(usage) };
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
    let urls = urls.split("<br>").fold(Vec::new(), |mut acc, url| {
        if url.is_empty() {
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
    let mut name = String::new();
    if !urls.is_empty() {
        name.clone_from(&urls[0].1);
    }
    let usage = usage.unwrap_or(&name).to_string();
    Ok((
        input,
        Combinator {
            _name: name,
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
        take_until(TABLE_HEADER1),
        tag(TABLE_HEADER1),
        line_ending,
        tag(TABLE_HEADER2),
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
    let mut uses: Vec<String> = Vec::new();

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
            // XXX: As said in the parser, there's transformations here that should
            // be done elsewhere. Leaving that for later.
            let mut input = combinator.input.to_string();
            if input.starts_with("b\"") {
                input.push_str(" as &[u8]");
            }
            for (module, name, _) in &combinator.urls {
                // filter out any modules that end with streaming or start with bits
                if module.ends_with("streaming") || module.starts_with("bits") {
                    continue;
                }
                // We put all of these into a Vec so we can dedup them, as `use` statements
                // can't be duplicated
                //
                // Allow unused imports for these specific ones, as not all are used in the
                // examples
                uses.push(format!(
                    "#[allow(unused_imports)]\nuse nom::{module}::{name};\n"
                ));
            }

            let urls = combinator
                .urls
                .iter()
                .map(|(module, name, docsurl)| format!("{module}::[{name}]({docsurl})"))
                .collect::<Vec<_>>()
                .join("<br>");

            // Some examples need explicit types in the let statement, they will
            // start with "let output", the rest don't for brevity.
            let usage = combinator.usage.to_string();
            writeln!(&mut fnmain, "let input = {input};")?;
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
                urls, usage, input, combinator.description,
            )?;
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
    let uses = uses.into_iter().sorted().dedup().collect::<String>();
    uses_file.write_all(uses.as_bytes())?;
    Ok(())
}
