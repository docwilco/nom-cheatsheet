use std::fs::File;
use std::io::{Result, Write};
use std::path::Path;
use std::str;

use nom::number::Endianness;
use nom::{
    character::{complete::digit1, is_alphanumeric},
    IResult,
};

use comrak::{markdown_to_html, ComrakOptions};

include! {concat!(env!("OUT_DIR"), "/uses.rs")}

fn number(input: &str) -> IResult<&str, usize> {
    map(digit1, |s: &str| s.parse().unwrap())(input)
}

fn my_alpha1(input: &str) -> IResult<&str, &str> {
    nom::character::complete::alpha1(input)
}

fn main() -> Result<()> {
    let mut markdown: Vec<u8> = Vec::new();

    include!(concat!(env!("OUT_DIR"), "/main.rs"));

    let markdown_file = Path::new(concat!(env!("OUT_DIR"), "/nom-cheatsheet.md"));
    println!("{:?}", markdown_file);
    let mut markdown_file = File::create(markdown_file)?;
    markdown_file.write_all(&markdown)?;

    let mut options = ComrakOptions::default();
    options.extension.table = true;
    options.render.unsafe_ = true;
    let html = markdown_to_html(str::from_utf8(&markdown).unwrap(), &options);

    let html_file = Path::new(concat!(env!("OUT_DIR"), "/nom-cheatsheet.html"));
    println!("{:?}", html_file);
    let mut html_file = File::create(html_file)?;
    html_file.write_all(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Nom Cheatsheet</title>
    <style>
"##
        .as_bytes(),
    )?;
    html_file.write_all(include_bytes!("github-markdown.css"))?;
    html_file.write_all(
        r##"

.markdown-body {
    margin: 0 auto;
    padding: 45px;
}

@media (max-width: 767px) {
    .markdown-body {
        padding: 15px;
    }
}
    </style>
</head>
<body class="markdown-body">
<article>
"##
        .as_bytes(),
    )?;
    html_file.write_all(html.as_bytes())?;
    html_file.write_all(
        r##"</article>
</body>
</html>
"##
        .as_bytes(),
    )?;
    Ok(())
}
