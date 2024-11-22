use comrak::{
    markdown_to_html_with_plugins, plugins::syntect::SyntectAdapterBuilder, Options, Plugins,
};
use nom::IResult;
use nom_cheatsheet_shared::markdown_format_code;
use std::{
    fs::File,
    io::{BufWriter, Result, Write},
    path::Path,
    str,
};
use syntect::{
    highlighting::ThemeSet,
    html::{css_for_theme_with_class_style, ClassStyle},
};

include! {concat!(env!("OUT_DIR"), "/uses.rs")}

trait SubsliceOffset {
    /**
    Returns the index of the first character of the subslice in the original slice.

    # Example
    ```
    let string = "a\nb\nc";
    let lines: Vec<&str> = string.lines().collect();
    assert_eq!(string.subslice_offset(lines[0]), Some(0));
    assert_eq!(string.subslice_offset(lines[1]), Some(2));
    assert_eq!(string.subslice_offset(lines[2]), Some(4));
    assert_eq!(string.subslice_offset("other"), None);
    assert_eq!(string.subslice_offset("a"), None);
    ```
    */
    fn subslice_offset_bytes(&self, subslice: &Self) -> Option<usize>;
}

impl SubsliceOffset for str {
    fn subslice_offset_bytes(&self, subslice: &str) -> Option<usize> {
        let self_ptr = self.as_ptr() as usize;
        let self_end = self_ptr.checked_add(self.len())?;
        let subslice_ptr = subslice.as_ptr() as usize;
        let subslice_end = subslice_ptr.checked_add(subslice.len())?;
        if subslice_ptr < self_ptr || subslice_ptr == self_end || subslice_end > self_end {
            return None;
        }
        if subslice_ptr < self_ptr || subslice_ptr > self_ptr.checked_add(self.len())? {
            return None;
        }
        // This is safe because we've already checked that subslice_ptr is never
        // smaller than self_ptr.
        Some(subslice_ptr - self_ptr)
    }
}

impl SubsliceOffset for &str {
    fn subslice_offset_bytes(&self, subslice: &Self) -> Option<usize> {
        (*self).subslice_offset_bytes(*subslice)
    }
}

impl SubsliceOffset for [u8] {
    fn subslice_offset_bytes(&self, subslice: &Self) -> Option<usize> {
        let self_ptr = self.as_ptr() as usize;
        let self_end = self_ptr.checked_add(self.len())?;
        let subslice_ptr = subslice.as_ptr() as usize;
        let subslice_end = subslice_ptr.checked_add(subslice.len())?;
        if subslice_ptr < self_ptr || subslice_end > self_end {
            return None;
        }
        // This is safe because we've already checked that subslice_ptr is never
        // smaller than self_ptr.
        Some(subslice_ptr - self_ptr)
    }
}

impl SubsliceOffset for &[u8] {
    fn subslice_offset_bytes(&self, subslice: &Self) -> Option<usize> {
        (*self).subslice_offset_bytes(*subslice)
    }
}

trait Length {
    fn length(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.length() == 0
    }
}

impl Length for str {
    fn length(&self) -> usize {
        self.len()
    }
}

impl Length for &str {
    fn length(&self) -> usize {
        self.len()
    }
}

impl Length for [u8] {
    fn length(&self) -> usize {
        self.len()
    }
}

impl Length for &[u8] {
    fn length(&self) -> usize {
        self.len()
    }
}

fn number(input: &str) -> IResult<&str, usize> {
    map(digit1, |s: &str| s.parse().unwrap())(input)
}

// Just to make the example compile
fn my_alpha1(input: &str) -> IResult<&str, &str> {
    nom::character::complete::alpha1(input)
}

fn format_remainder<I>(remainder: &I) -> String
where
    I: std::fmt::Debug + SubsliceOffset,
{
    markdown_format_code(&format!("{remainder:#04x?}"))
        .replace('\n', "")
        .replace(' ', "")
        .replace(",]", "]")
        .replace(',', ", ")
        .replace("[", "&[")
}

fn format_iresult<I, O>(input: I, result: &IResult<I, O>) -> String
where
    I: std::fmt::Debug + SubsliceOffset + Length,
    O: std::fmt::Debug,
{
    match result {
        Ok((remainder, value)) => {
            let value = markdown_format_code(&format!("{value:?}"));
            if remainder.is_empty() {
                format!("Result: {value}<br>No remainder")
            } else {
                let remainder = format_remainder(remainder);
                format!("Result: {value}<br>Remainder: {remainder}")
            }
        }
        Err(e) => match e {
            nom::Err::Incomplete(needed) => match needed {
                nom::Needed::Size(size) => format!("Incomplete<br>Needed: {size} items"),
                nom::Needed::Unknown => "Incomplete<br>Needed: unknown".to_string(),
            },
            nom::Err::Error(nom::error::Error {
                input: location,
                code,
            })
            | nom::Err::Failure(nom::error::Error {
                input: location,
                code,
            }) => {
                let kind = match e {
                    nom::Err::Error(_) => "Error",
                    nom::Err::Failure(_) => "Failure",
                    nom::Err::Incomplete(_) => unreachable!(),
                };
                let offset = input.subslice_offset_bytes(location).unwrap();
                format!("{kind}<br>Byte offset: {offset}<br>Code: {code:?}")
            }
        },
    }
}

fn main() -> Result<()> {
    let mut markdown: Vec<u8> = Vec::new();

    include!(concat!(env!("OUT_DIR"), "/main.rs"));

    let markdown_path = Path::new("dist/nom-cheatsheet.md");
    println!("Markdown file: {markdown_path:?}");
    let mut markdown_file = BufWriter::new(File::create(markdown_path)?);
    markdown_file.write_all(&markdown)?;

    let mut options = Options::default();
    options.extension.table = true;
    options.extension.header_ids = Some(String::new());
    options.render.unsafe_ = true;
    let mut plugins = Plugins::default();
    let syntect = SyntectAdapterBuilder::new().css().build();
    plugins.render.codefence_syntax_highlighter = Some(&syntect);
    let html =
        markdown_to_html_with_plugins(str::from_utf8(&markdown).unwrap(), &options, &plugins);

    let html_path = Path::new("dist/nom-cheatsheet.html");
    println!("HTML file: {html_path:?}");
    // Replace \ with / in the path
    let html_path = html_path.to_str().unwrap().replace('\\', "/");
    println!("URL: file:///{html_path}");

    let themeset = ThemeSet::load_defaults();
    let dark_theme = &themeset.themes["Solarized (dark)"];
    let css_dark = css_for_theme_with_class_style(dark_theme, ClassStyle::Spaced).unwrap();
    let light_theme = &themeset.themes["Solarized (light)"];
    let css_light = css_for_theme_with_class_style(light_theme, ClassStyle::Spaced).unwrap();

    let mut html_file = BufWriter::new(File::create(html_path)?);
    html_file.write_all(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Nom Cheatsheet</title>
    <style>
"#
        .as_bytes(),
    )?;
    html_file.write_all(include_bytes!("github-markdown.css"))?;
    html_file.write_all(r"@media (prefers-color-scheme: dark) {".as_bytes())?;
    html_file.write_all(css_dark.as_bytes())?;
    html_file.write_all(
        r"}
@media (prefers-color-scheme: light) {"
            .as_bytes(),
    )?;
    html_file.write_all(css_light.as_bytes())?;
    html_file.write_all(r"}".as_bytes())?;
    html_file.write_all(
        r#"

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
"#
        .as_bytes(),
    )?;
    html_file.write_all(html.as_bytes())?;
    html_file.write_all(
        "</article>
</body>
</html>
"
        .as_bytes(),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subslice_offset() {
        let string = "a\nb\nc";
        let lines: Vec<&str> = string.lines().collect();
        assert_eq!(string.subslice_offset_bytes(lines[0]), Some(0));
        assert_eq!(string.subslice_offset_bytes(lines[1]), Some(2));
        assert_eq!(string.subslice_offset_bytes(lines[2]), Some(4));
        assert_eq!(string.subslice_offset_bytes("other"), None);
        assert_eq!(string.subslice_offset_bytes("a"), None);

        let string = "foobar";
        let str1 = &string[0..3];
        let str2 = &string[3..];
        let str3 = &string[3..3];
        let str4 = &string[2..3];
        assert_eq!(str1.subslice_offset_bytes(str2), None);
        assert_eq!(str1.subslice_offset_bytes(str3), None);
        assert_eq!(str1.subslice_offset_bytes(str4), Some(2));
    }

    #[test]
    fn test_format_remainder() {
        let input = "abc";
        assert_eq!(format_remainder(&input), "`\"abc\"`");
        let input = &[0_u8, 1, 2, 3][..];
        assert_eq!(format_remainder(&input), "`&[0x00, 0x01, 0x02, 0x03]`");
    }
}
