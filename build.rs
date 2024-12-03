use nom::{
    branch::alt,
    bytes::complete::{is_a, tag, take_until},
    character::complete::{line_ending, not_line_ending, space0},
    combinator::{opt, recognize, rest},
    multi::{many0, many1},
    sequence::{terminated, tuple},
    IResult,
};
use nom_cheatsheet_shared::markdown_format_code;
use quote::{format_ident, ToTokens};
use std::{
    collections::{HashMap, HashSet},
    env,
    fs::{self, read_to_string},
    path::Path,
};
use syn::{parse_quote, Expr, ExprLit, Item, Lit, Stmt};

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>;

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
    imports: &'a str,
    usage: Option<String>,
    input: Option<&'a str>,
    description: &'a str,
}

#[derive(Debug)]
enum Component<'a> {
    Text(&'a str),
    CodeBlock(CodeBlock<'a>),
}

#[derive(Debug)]
struct CodeBlock<'a> {
    language: &'a str,
    code: &'a str,
}

fn parse_outside_code_blocks(input: &str) -> IResult<&str, Component> {
    let (input, text) = alt((take_until("```"), rest))(input)?;
    if text.is_empty() {
        return Err(nom::Err::Error(nom::error::Error {
            input,
            code: nom::error::ErrorKind::Eof,
        }));
    }
    Ok((input, Component::Text(text)))
}

fn parse_code_block(input: &str) -> IResult<&str, Component> {
    let (input, _) = tag("```")(input)?;
    let (input, language) = terminated(not_line_ending, line_ending)(input)?;
    let (input, code) = take_until("```")(input)?;
    let (input, _) = tag("```")(input)?;
    Ok((input, Component::CodeBlock(CodeBlock { language, code })))
}

fn do_code_blocks(input: &str) -> Result<String> {
    let (input, mut components) =
        many1(alt((parse_code_block, parse_outside_code_blocks)))(input).unwrap();
    assert_eq!(input, "");
    for (index, component) in components.iter_mut().enumerate() {
        let Component::CodeBlock(code_block) = component else {
            continue;
        };
        if code_block.language == "ignore" {
            code_block.language = "rust";
            continue;
        }
        if code_block.language != "rust" && code_block.language != "rs" {
            continue;
        }
        let path = format!("examples/example{index}.rs");
        let path = Path::new(&path);
        let mut code = code_block.code.to_string();
        code.push_str(
            r"
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main() {
        main();
    }
}",
        );
        fs::write(path, code)?;
    }
    let output = components
        .into_iter()
        .map(|component| match component {
            Component::Text(text) => text.to_string(),
            Component::CodeBlock(CodeBlock { language, code }) => {
                format!("```{language}\n{code}\n```")
            }
        })
        .collect();
    Ok(output)
}

fn parse_code_span(input: &str) -> IResult<&str, &str> {
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
    let (input, _) = space0(input)?;
    let (input, _) = sep(input)?;
    let (input, usage) = opt(parse_code_span)(input)?;
    let (input, _) = sep(input)?;
    let (input, example_input) = opt(parse_code_span)(input)?;
    let (input, _) = sep(input)?;
    let (input, _) = sep(input)?;
    let (input, description) = take_until("|")(input)?;
    let description = description.trim_end();
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
    let (usage, imports) = match usage {
        Some(usage) => {
            let (usage, imports) = parse_imports_short(usage)?;
            (Some(usage.to_string()), imports)
        }
        None => (None, ""),
    };
    Ok((
        input,
        Combinator {
            urls,
            imports,
            usage,
            input: example_input,
            description,
        },
    ))
}

fn parse_imports_short(input: &str) -> IResult<&str, &str> {
    recognize(many0(tuple((
        tag("use "),
        take_until(";"),
        tag(";"),
        space0,
    ))))(input)
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

#[allow(clippy::too_many_lines)]
fn main() -> Result<()> {
    let input = read_to_string("src/nom-cheatsheet-template.md")?;

    let input = do_code_blocks(&input)?;

    // This snags a Vec of Tuples
    // .0 is all the text since the start of the file or the end of the previous table
    // upto and including the header of the current table, aka preamble
    // .1 is the vector of combinators in the current table
    let (remainder, result): (&str, Vec<(&str, Vec<Combinator>)>) =
        many1(parse_preamble_and_combinators)(&input).unwrap();

    let mut uses = HashMap::<String, Item>::new();
    let mut uses_conflicts = HashSet::<String>::new();
    let mut last_urls: Vec<Url> = Vec::new();

    // These will be all the statements that go into `generate()`
    let mut statements: Vec<Stmt> = Vec::new();

    for table in result {
        // Preamble already ends with a newline, so use write instead of writeln
        //
        // Escape braces because we're putting this string straight into a
        // format
        //
        // Otherwise preamble goes into the resulting markdown as-is
        let preamble = table.0;
        let preamble = parse_quote! {
            write!(markdown, "{}", #preamble)?;
        };
        statements.push(preamble);

        for combinator in table.1 {
            // Put each row in the table in its own block, so that we can `use`
            // without conflicts

            let urls = if combinator.urls.is_empty() {
                last_urls
            } else {
                combinator.urls.clone()
            };
            let mut imports: syn::File = syn::parse_str(combinator.imports)?;
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
                let module = format!("nom::{module}");
                let module: syn::Path = syn::parse_str(&module)?;
                let name_ident = format_ident!("{name}");
                let use_statement = Item::Use(
                    parse_quote! {
                        #[allow(unused_imports)]
                        use #module::#name_ident;
                    }
                );
                imports.items.push(use_statement.clone());
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
                let use_statement_clone = use_statement.clone();
                if let Some(conflict) = uses.insert(name.clone(), use_statement) {
                    if conflict != use_statement_clone {
                        uses_conflicts.insert(name.clone());
                    }
                }
            }

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

            match (combinator.input, combinator.usage) {
                (None, None) => {
                    let row = format!(
                        "| {urlstrings} |  |  |  | {desc} |",
                        desc = combinator.description
                    );
                    let block = parse_quote! {
                        {
                            writeln!(markdown, "{}", #row)?;
                        }
                    };
                    statements.push(block);
                }
                (Some(_), None) | (None, Some(_)) => {
                    panic!("Both usage and input must be present, or neither.");
                }
                (Some(input), Some(usage)) => {
                    // XXX: As said in the parser, there's transformations here
                    // that should be done elsewhere. Leaving that for later.
                    let mut input_code: Expr = syn::parse_str(input)?;
                    // Some traits are implemented for slices, but not for
                    // references to arrays. So we add `[..]` to those, to make
                    // them slices.
                    if let Expr::Reference(reference) = &input_code {
                        if let Expr::Array(_) = reference.expr.as_ref() {
                            input_code = parse_quote! { #input_code[..] };
                        }
                    }
                    // And byte strings are &str, but we want to treat them as
                    // &[u8]
                    if let Expr::Lit(ExprLit {
                        lit: Lit::ByteStr(_),
                        ..
                    }) = &input_code
                    {
                        input_code = parse_quote! { #input_code as &[u8] };
                    }

                    // Some examples need explicit types in the let statement, they will
                    // start with "let output", the rest don't for brevity.
                    let usage_code = usage.replace("\\|", "|");
                    let usage_with_input = usage_code.clone() + "(input);";
                    let assignment =
                        if let Ok(Stmt::Local(local)) = syn::parse_str::<Stmt>(&usage_with_input) {
                            assert!(local
                                .pat
                                .to_token_stream()
                                .to_string()
                                .starts_with("output"));
                            Stmt::Local(local)
                        } else {
                            let expr: Expr = syn::parse_str(&usage_code).unwrap();
                            parse_quote! {
                                let output: IResult<_, _> = #expr(input);
                            }
                        };

                    let usage = markdown_format_code(&usage);
                    let input = markdown_format_code(input);
                    let description = combinator.description;
                    let block = parse_quote! {
                        {
                            #imports
                            let input = #input_code;
                            #assignment;
                            let output = format_iresult(&input, &output);
                            writeln!(
                                markdown,
                                "| {urlstrings} | {usage} | {input} | {output} | {desc} |",
                                urlstrings = #urlstrings,
                                usage = #usage,
                                input = #input,
                                desc = #description
                            )?;
                        }
                    };
                    statements.push(block);
                }
            };
            last_urls = urls;
        }
    }

    let remainder = parse_quote! {
        write!(markdown, "{}", #remainder)?;
    };
    statements.push(remainder);

    for conflict in uses_conflicts {
        uses.remove(&conflict);
    }
    let mut uses = uses.values().cloned().collect::<Vec<_>>();
    uses.sort_by_key(|item| item.to_token_stream().to_string());

    let generated_file: syn::File = parse_quote! {
        #(#uses)*
        use std::io::Write;
        use super::{IResult, Result, format_iresult, my_alpha1, number, str};

        #[allow(clippy::too_many_lines)]
        pub fn generate() -> Result<Vec<u8>> {
            let mut markdown = Vec::new();
            #(#statements)*
            Ok(markdown)
        }
    };

    let generated_file_path = Path::new(&env::var("OUT_DIR").unwrap()).join("generated.rs");
    let formatted = prettyplease::unparse(&generated_file);
    fs::write(generated_file_path, formatted)?;

    Ok(())
}
