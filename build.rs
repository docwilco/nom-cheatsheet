use itertools::Itertools;
use nom::{
    bytes::complete::{tag, take_until},
    character::complete::{char, line_ending, space0},
    combinator::recognize,
    multi::many1,
    sequence::{pair, tuple},
    IResult,
};
use std::io::Result;
use std::{
    env,
    fs::{read_to_string, File},
    io::Write,
    path::Path,
};

static TABLE_HEADER1: &str = "| combinator | usage | input | output | description |";
static TABLE_HEADER2: &str = "|---|---|---|---|---|";

#[derive(Debug)]
struct Combinator<'a> {
    name: String,
    urls: Vec<(String, String, String)>,
    usage: Option<&'a str>,
    input: &'a str,
    description: &'a str,
}

fn parse_code(input: &str) -> IResult<&str, &str> {
    let (input, _) = char('`')(input)?;
    let (input, code) = take_until("`")(input)?;
    let (input, _) = char('`')(input)?;
    Ok((input, code))
}

fn parse_separator(input: &str) -> IResult<&str, &str> {
    tag(" | ")(input)?;
    Ok((input, ""))
}

// This parses a single table row
fn parse_combinator(input: &str) -> IResult<&str, Combinator> {
    let (input, _) = tag("|")(input)?;
    let (input, _) = space0(input)?;
    let (input, urls): (&str, &str) = take_until()(input)?;
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
    let mut name = "".to_string();
    if !urls.is_empty() {
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

// This parses a single table and returns a vector of combinators, and also returns the
// text before the table.
fn parse_preamble_and_combinators(input: &str) -> IResult<&str, (&str, Vec<Combinator>)> {
    let (input, preamble) = recognize(
        tuple((
            take_until(TABLE_HEADER1),
            tag(TABLE_HEADER1),
            line_ending,
            tag(TABLE_HEADER2),
            line_ending))
        )(input)?;

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
            r#####"write!(markdown, r####"{}"####)?;"#####,
            preamble
        )?;
        for combinator in table.1 {
            // XXX: As said in the parser, there's transformations here that should
            // be done elsewhere. Leaving that for later.
            let mut input = combinator.input.to_string();
            if input.starts_with("b\"") {
                input.push_str(" as &[u8]");
            }
            for (module, name, _) in combinator.urls.iter().filter(|(module, _, _)| {
                // filter out any modules that end with streaming or start with bits
                !(module.ends_with("streaming") || module.starts_with("bits"))
            }) {
                // We put all of these into a Vec so we can dedup them, as `use` statements
                // can't be duplicated
                //
                // Allow unused imports for these specific ones, as not all are used in the
                // examples
                uses.push(format!(
                    "#[allow(unused_imports)]\nuse nom::{}::{};\n",
                    module, name
                ));
            }

            let urls = combinator
                .urls
                .iter()
                .map(|(module, name, docsurl)| format!("{}::[{}]({})", module, name, docsurl))
                .collect::<Vec<_>>()
                .join("<br>");

            // Some examples need explicit types in the let statement, they will
            // start with "let output", the rest don't for brevity.
            let assignment: String = if combinator.usage.starts_with("let output") {
                format!("{}({});\n", combinator.usage, input)
            } else {
                format!(
                    "let output: IResult<_, _> = {}({});\n",
                    combinator.usage, input
                )
            };
            let assignment = assignment.replace("\\|", "|");

            fnmain.write_all(assignment.as_bytes())?;

            println!("cargo:warning=usage: {}", combinator.usage);
            let usage = if combinator.usage.is_empty() {
                println!("cargo:warning=empty usage for {}", combinator.name);
                combinator.name.to_string()
            } else {
                println!("cargo:warning=non-empty usage for {}", combinator.name);
                combinator.usage.to_string()
            };
            let usage = usage.replace('{', "{{");
            let usage = usage.replace('}', "}}");
            writeln!(
                &mut fnmain,
                r#####"writeln!(markdown, r####"| {} | `{}` | `{}` | `{{:?}}` | {} |"####, output)?;"#####,
                urls, usage, combinator.input, combinator.description,
            )?;
        }
    }

    let remainder = remainder.replace('{', "{{").replace('}', "}}");

    writeln!(
        &mut fnmain,
        r#####"write!(markdown, r####"{}"####)?;"#####,
        remainder
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
