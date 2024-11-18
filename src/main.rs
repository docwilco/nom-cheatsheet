use std::fs::File;
use std::io::{Result, Write};
use std::path::Path;
use std::str;

use comrak::plugins::syntect::SyntectAdapterBuilder;
use nom::number::Endianness;
use nom::{character::is_alphanumeric, IResult};

use comrak::{markdown_to_html_with_plugins, Options, Plugins};
use syntect::highlighting::ThemeSet;
use syntect::html::{css_for_theme_with_class_style, ClassStyle};

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

    let mut options = Options::default();
    options.extension.table = true;
    options.render.unsafe_ = true;
    let mut plugins = Plugins::default();
    let syntect = SyntectAdapterBuilder::new().css().build();
    plugins.render.codefence_syntax_highlighter = Some(&syntect);
    let html =
        markdown_to_html_with_plugins(str::from_utf8(&markdown).unwrap(), &options, &plugins);

    let html_file = Path::new(concat!(env!("OUT_DIR"), "/nom-cheatsheet.html"));
    println!("{:?}", html_file);
    // Replace \ with / in the path
    let html_file = html_file.to_str().unwrap().replace("\\", "/");
    println!("file:///{}", html_file);

    let themeset = ThemeSet::load_defaults();
    let dark_theme = &themeset.themes["Solarized (dark)"];
    let css_dark = css_for_theme_with_class_style(dark_theme, ClassStyle::Spaced).unwrap();
    let light_theme = &themeset.themes["Solarized (light)"];
    let css_light = css_for_theme_with_class_style(light_theme, ClassStyle::Spaced).unwrap();

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
        r##"@media (prefers-color-scheme: dark) {"##
            .as_bytes())?;
    html_file.write_all(css_dark.as_bytes())?;
    html_file.write_all(
        r##"}
@media (prefers-color-scheme: light) {"##
            .as_bytes())?;
    html_file.write_all(css_light.as_bytes())?;
    html_file.write_all(
        r##"}"##
            .as_bytes())?;
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

//    let css_path = Path::new(concat!(env!("OUT_DIR"), "/syntax.css"));
//    let mut css_file = File::create(css_path)?;
//    css_file.write_all(css_dark.as_bytes())?;
//    println!("{:?}", css_path);
    Ok(())
}
