#[must_use]
pub fn markdown_format_code(input: &str) -> String {
    // Find longest sequence of backticks
    let mut max = 0;
    let mut count = 0;
    for c in input.chars() {
        if c == '`' {
            count += 1;
            // Do this here, because last character could be a backtick
            max = max.max(count);
        } else {
            count = 0;
        }
    }
    // Then use one more backtick than the longest sequence
    let backticks = "`".repeat(max + 1);

    // ` a ` and `a` both render to just `a`, but ` a` and `a ` render to ` a`
    // and `a ` respectively. And `  a  ` renders to ` a `. So if we start and
    // end with a space, we need to add an extra space to the start and end to
    // make sure they are preserved in the rendered output.

    // Surround the input with spaces if it starts or ends with a backtick
    let spacing = if (input.starts_with('`') || input.ends_with('`'))
        || (input.starts_with(' ') && input.ends_with(' '))
    {
        " "
    } else {
        ""
    };
    format!("{backticks}{spacing}{input}{spacing}{backticks}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_format_code() {
        assert_eq!(markdown_format_code("abc"), "`abc`");
        assert_eq!(markdown_format_code("`abc`"), "`` `abc` ``");
        assert_eq!(markdown_format_code(" `abc` "), "``  `abc`  ``");
        assert_eq!(markdown_format_code(" `abc`"), "``  `abc` ``");
        assert_eq!(markdown_format_code("`abc` "), "`` `abc`  ``");
        assert_eq!(markdown_format_code("``abc``"), "``` ``abc`` ```");
        assert_eq!(markdown_format_code("`"), "`` ` ``");
        assert_eq!(markdown_format_code("``"), "``` `` ```");
    }
}
