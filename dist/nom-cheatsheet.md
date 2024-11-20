# Nom cheatsheet

This is inspired by [`choosing_a_combinator.md`](https://github.com/Geal/nom/blob/master/doc/choosing_a_combinator.md "test") in that it collects a bunch of the available things in one page and shows short examples of how each works.

## Quick introduction to Nom

For those new to Nom, most parsers and combinators actually return a function, and said function is what the input is fed to. This is what allows you to combine a bunch of parsers using combinators. This causes syntax that looks slightly odd when you're not used to it. For example, the `char` parser used directly would look like this:

```rust
let (input, my_char) = char('a')(input)?;
```

As you can see, there's two sets of parentheses after `char`. The first set is the arguments to the `char` function, and makes a new function that is a parser that only accepts a single `a`. Then the second set is the actual call to that parser with the input. The `?` at the end is Rust's typical way of handling errors, and is used to return early if the parser fails. Nom parsers use `IResult` as their return type, which is a rather specific type alias of `Result`.

In all the examples in the tables below, `input` is a separate column since it's not an argument to the function, but an argument to the result of the function.

The `output` column likewise is the result of calling the parser, but for `Ok()` results, the result and the remaining input are shown in a nice way, instead of `Ok(("remaining input", "result"))`, which can be a bit hard to read.

If the parser or combinator succeeded, the result will be an `Ok()` containing a tuple of the remaining input and then the actual result of the parser or combinator. The remaining input is passed back like that so that it can then be used with other parsers or combinators. That is why the `input` variable is rebound in the examples above.

If you are writing a function that takes in input and returns a struct, you should write it so that it returns an `IResult` with the remaining input as well. This then allows you to use things like the `many0` combinator with your function to easily get a `Vec` of your custom structs.

```rust
use nom::{
    character::complete::{char, i32, line_ending, newline},
    multi::separated_list0,
    sequence::separated_pair,
    IResult,
};

#[derive(Debug, Eq, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

fn parse_point(input: &str) -> IResult<&str, Point> {
    // When you call a parser like `i32`, it will return a tuple of the
    // remaining input and the parsed value. If you unpack the `IResult` above,
    // you'll see `parse_point` also returns a tuple of the remaining input and
    // the parsed value
    let (input, x) = i32(input)?;
    // Because input is rebound to the remaining input in the line above, the
    // following line will parse and consume the comma. Since we don't care
    // about the comma, we use the `_` to ignore it
    let (input, _) = char(',')(input)?;
    // And now input is only the y value
    let (input, y) = i32(input)?;
    // Finally, we construct our return value, and return it alongside the
    // remaining input
    Ok((input, Point { x, y }))
}

fn main() {
    let input = "123,456\n789,1011";
    // Here we construct a parser that will parse a list of `Point`s separated
    // by `line_ending`.
    //
    // Note that the `separated_list0` takes parsers as arguments, so we don't
    // give `line_ending` or `parse_point` any arguments.
    let mut parse_points = separated_list0(line_ending, parse_point);
    let (input, points) = parse_points(input).unwrap();
    // `points` is now a `Vec<Point>` containing the two points we parsed
    assert_eq!(
        points,
        vec![Point { x: 123, y: 456 }, Point { x: 789, y: 1011 }]
    );
    // And the remaining input should now be empty
    assert_eq!(input, "");

    // Or setting up and using a parser in a single line:
    let input = "34,56\n21,98";
    let (input, points) = separated_list0(newline, parse_point_concise)(input).unwrap();
    assert_eq!(points, vec![Point { x: 34, y: 56 }, Point { x: 21, y: 98 }]);
    assert_eq!(input, "");
}

fn parse_point_concise(input: &str) -> IResult<&str, Point> {
    // `separated_pair` is a combinator that takes three parsers, and returns a
    // parser that returns a tuple of the results of the first and third
    // parsers, using the second parser as a separator. This allows us to
    // rewrite `parse_point` as follows:
    let (input, (x, y)) = separated_pair(i32, char(','), i32)(input)?;
    // Then we construct our return value, and return it alongside the remaining
    // input.
    Ok((input, Point { x, y }))
}
```

## Basic elements

Those are used to recognize the lowest level elements of your grammar, like, "here is a dot", "here is a number", "here is a line ending". These are split up into matching a single byte or character, and matching multiple bytes or characters. 

### Single byte or character parsers

All of these parsers will return a single byte or character.

| parser | usage | input | output | description |
|---|---|---|---|---|
| character::complete::[char](https://docs.rs/nom/latest/nom/character/complete/fn.char.html)<br>character::streaming::[char](https://docs.rs/nom/latest/nom/character/streaming/fn.char.html) | `char('a')` | `"abc"` | Result: `'a'`<br>Remainder: `"bc"` | Matches one specific character |
|  | `char('a')` | `"cba"` | Error<br>Byte offset: 0<br>Code: Char | If that character isn't the immediate input, parsing fails |
|  | `char('💞')` | `"💞🦀"` | Result: `'💞'`<br>Remainder: `"🦀"` | Multi-byte characters work as well |
| character::complete::[anychar](https://docs.rs/nom/latest/nom/character/complete/fn.anychar.html)<br>character::streaming::[anychar](https://docs.rs/nom/latest/nom/character/streaming/fn.anychar.html) | `anychar` | `"abc"` | Result: `'a'`<br>Remainder: `"bc"` | Matches any single character |
|  | `anychar` | `"💞🦀"` | Result: `'💞'`<br>Remainder: `"🦀"` | Multi-byte characters work as well |
| character::complete::[one_of](https://docs.rs/nom/latest/nom/character/complete/fn.one_of.html)<br>character::streaming::[one_of](https://docs.rs/nom/latest/nom/character/streaming/fn.one_of.html) | `one_of("abc")` | `"abc"` | Result: `'a'`<br>Remainder: `"bc"` | Matches one of the provided characters |
| character::complete::[none_of](https://docs.rs/nom/latest/nom/character/complete/fn.none_of.html)<br>character::streaming::[none_of](https://docs.rs/nom/latest/nom/character/streaming/fn.none_of.html) | `none_of("abc")` | `"xyab"` | Result: `'x'`<br>Remainder: `"yab"` | Matches a single character that is anything but the provided characters |

### Sequence of bytes or characters parsers

These parsers will return a slice of bytes or characters. Those suffixed with `0` can return an empty slice if they match nothing. They usually have variants that are suffixed with `1` that will refuse to match unless there's at least 1 byte or character they can match. For instance, `digit0` will match an empty string, but `digit1` will not:

| parser | usage | input | output | description |
|---|---|---|---|---|
| character::complete::[digit0](https://docs.rs/nom/latest/nom/character/complete/fn.digit0.html)<br>character::streaming::[digit0](https://docs.rs/nom/latest/nom/character/streaming/fn.digit0.html) | `digit0` | `"123abc"` | Result: `"123"`<br>Remainder: `"abc"` | Matches zero or more numerical ASCII characters (`0-9`) |
| character::complete::[digit1](https://docs.rs/nom/latest/nom/character/complete/fn.digit1.html)<br>character::streaming::[digit1](https://docs.rs/nom/latest/nom/character/streaming/fn.digit1.html) | `digit1` | `"123abc"` | Result: `"123"`<br>Remainder: `"abc"` | Matches one or more numerical ASCII characters (`0-9`) |
|  | `digit0` | `"abc123"` | Result: `""`<br>Remainder: `"abc123"` | Because it is allowed to return an empty string, this does not error |
|  | `digit1` | `"abc123"` | Error<br>Byte offset: 0<br>Code: Digit | This however does error, because there must be at least one numerical ASCII character |

This goes for all the `0` and `1` suffixed parsers below:

| parser | usage | input | output | description |
|---|---|---|---|---|
| bytes::complete::[is_a](https://docs.rs/nom/latest/nom/bytes/complete/fn.is_a.html)<br>bytes::streaming::[is_a](https://docs.rs/nom/latest/nom/bytes/streaming/fn.is_a.html) | `is_a("ab")` | `"ababc"` | Result: `"abab"`<br>Remainder: `"c"` | Matches a sequence of any of the characters passed as arguments |
| bytes::complete::[is_not](https://docs.rs/nom/latest/nom/bytes/complete/fn.is_not.html)<br>bytes::streaming::[is_not](https://docs.rs/nom/latest/nom/bytes/streaming/fn.is_not.html) | `is_not("cd")` | `"ababc"` | Result: `"abab"`<br>Remainder: `"c"` | Matches a sequence of none of the characters passed as arguments |
| character::complete::[alpha0](https://docs.rs/nom/latest/nom/character/complete/fn.alpha0.html)<br>character::streaming::[alpha0](https://docs.rs/nom/latest/nom/character/streaming/fn.alpha0.html) | `alpha0` | `"abc123"` | Result: `"abc"`<br>Remainder: `"123"` | Matches zero or more alphabetical ASCII characters (`a-zA-Z`) |
| character::complete::[alpha1](https://docs.rs/nom/latest/nom/character/complete/fn.alpha1.html)<br>character::streaming::[alpha1](https://docs.rs/nom/latest/nom/character/streaming/fn.alpha1.html) | `alpha1` | `"abc123"` | Result: `"abc"`<br>Remainder: `"123"` | Matches one or more alphabetical ASCII characters (`a-zA-Z`) |
|  | `alpha1` | `"ααα"` | Error<br>Byte offset: 0<br>Code: Alpha | Only ASCII counts for these, not all of the unicode alphabetical characters. (These are Greek Alphas.) |
| character::complete::[digit0](https://docs.rs/nom/latest/nom/character/complete/fn.digit0.html)<br>character::streaming::[digit0](https://docs.rs/nom/latest/nom/character/streaming/fn.digit0.html) | `digit0` | `"123abc"` | Result: `"123"`<br>Remainder: `"abc"` | Matches zero or more numerical ASCII characters (`0-9`) |
| character::complete::[digit1](https://docs.rs/nom/latest/nom/character/complete/fn.digit1.html)<br>character::streaming::[digit1](https://docs.rs/nom/latest/nom/character/streaming/fn.digit1.html) | `digit1` | `"123abc"` | Result: `"123"`<br>Remainder: `"abc"` | Matches one or more numerical ASCII characters (`0-9`) |
| character::complete::[alphanumeric0](https://docs.rs/nom/latest/nom/character/complete/fn.alphanumeric0.html)<br>character::streaming::[alphanumeric0](https://docs.rs/nom/latest/nom/character/streaming/fn.alphanumeric0.html) | `alphanumeric0` | `"abc123"` | Result: `"abc123"`<br>No remainder | Matches zero or more alphanumeric ASCII characters (`a-zA-Z0-9`) |
| character::complete::[alphanumeric1](https://docs.rs/nom/latest/nom/character/complete/fn.alphanumeric1.html)<br>character::streaming::[alphanumeric1](https://docs.rs/nom/latest/nom/character/streaming/fn.alphanumeric1.html) | `alphanumeric1` | `"abc123"` | Result: `"abc123"`<br>No remainder | Matches one or more alphanumeric ASCII characters (`a-zA-Z0-9`) |
| character::complete::[hex_digit0](https://docs.rs/nom/latest/nom/character/complete/fn.hex_digit0.html)<br>character::streaming::[hex_digit0](https://docs.rs/nom/latest/nom/character/streaming/fn.hex_digit0.html) | `hex_digit0` | `"123abcghi"` | Result: `"123abc"`<br>Remainder: `"ghi"` | Matches zero or more hexadecimal ASCII characters (`0-9a-fA-F`) |
| character::complete::[hex_digit1](https://docs.rs/nom/latest/nom/character/complete/fn.hex_digit1.html)<br>character::streaming::[hex_digit1](https://docs.rs/nom/latest/nom/character/streaming/fn.hex_digit1.html) | `hex_digit1` | `"123abcghi"` | Result: `"123abc"`<br>Remainder: `"ghi"` | Matches one or more hexadecimal ASCII characters (`0-9a-fA-F`) |
| bytes::complete::[tag](https://docs.rs/nom/latest/nom/bytes/complete/fn.tag.html)<br>bytes::streaming::[tag](https://docs.rs/nom/latest/nom/bytes/streaming/fn.tag.html)<br>bits::complete::[tag](https://docs.rs/nom/latest/nom/bits/complete/fn.tag.html)<br>bits::streaming::[tag](https://docs.rs/nom/latest/nom/bits/streaming/fn.tag.html) | `tag("hello")` | `"hello world"` | Result: `"hello"`<br>Remainder: `"world"` | Recognizes a specific suite of characters, bytes, or bits |
| bytes::complete::[tag_no_case](https://docs.rs/nom/latest/nom/bytes/complete/fn.tag_no_case.html)<br>bytes::streaming::[tag_no_case](https://docs.rs/nom/latest/nom/bytes/streaming/fn.tag_no_case.html) | `tag_no_case("hello")` | `"HeLLo World"` | Result: `"HeLLo"`<br>Remainder: `"World"` | Recognizes a specific suite of characters, in a case insensitive manner |
|  | `tag_no_case("γειά")` | `"Γειά Κόσμο"` | Result: `"Γειά"`<br>Remainder: `"Κόσμο"` | This also works with non-ASCII characters. A `γ` is a lowercase `Γ`. (Greek Gamma) |
| character::complete::[newline](https://docs.rs/nom/latest/nom/character/complete/fn.newline.html)<br>character::streaming::[newline](https://docs.rs/nom/latest/nom/character/streaming/fn.newline.html) | `newline` | `"\nhello"` | Result: `'\n'`<br>Remainder: `"hello"` | Matches a newline character, also known as line feed, `\n`, or `LF` |
| character::complete::[crlf](https://docs.rs/nom/latest/nom/character/complete/fn.crlf.html)<br>character::streaming::[crlf](https://docs.rs/nom/latest/nom/character/streaming/fn.crlf.html) | `crlf` | `"\r\nhello"` | Result: `"\r\n"`<br>Remainder: `"hello"` | Matches a carriage return followed by a newline, also known as `\r\n` or `CRLF` |
| character::complete::[line_ending](https://docs.rs/nom/latest/nom/character/complete/fn.line_ending.html)<br>character::streaming::[line_ending](https://docs.rs/nom/latest/nom/character/streaming/fn.line_ending.html) | `line_ending` | `"\r\nhello"` | Result: `"\r\n"`<br>Remainder: `"hello"` | Matches an end of line, either Unix style (`\n`/`LF`) or Windows style (`\r\n`/`CRLF`) |
|  | `line_ending` | `"\nhello"` | Result: `"\n"`<br>Remainder: `"hello"` | Basically `line_ending` is the same as [`alt((crlf, newline))`](alt), but has a better performance |
| character::complete::[space0](https://docs.rs/nom/latest/nom/character/complete/fn.space0.html)<br>character::streaming::[space0](https://docs.rs/nom/latest/nom/character/streaming/fn.space0.html) | `space0` | `" \t\nhello"` | Result: `" \t"`<br>Remainder: `"\nhello"` | Matches zero or more spaces (`' '`) and tabs (`\t`) |
| character::complete::[space1](https://docs.rs/nom/latest/nom/character/complete/fn.space1.html)<br>character::streaming::[space1](https://docs.rs/nom/latest/nom/character/streaming/fn.space1.html) | `space1` | `" \t\nhello"` | Result: `" \t"`<br>Remainder: `"\nhello"` | Matches one or more spaces (`' '`) and tabs (`\t`) |
| character::complete::[multispace0](https://docs.rs/nom/latest/nom/character/complete/fn.multispace0.html)<br>character::streaming::[multispace0](https://docs.rs/nom/latest/nom/character/streaming/fn.multispace0.html) | `multispace0` | `" \t\nhello"` | Result: `" \t\n"`<br>Remainder: `"hello"` | Matches zero or more spaces (`' '`), tabs (`\t`), line feeds (`\n`), and carriage returns (`\r`) |
| character::complete::[multispace1](https://docs.rs/nom/latest/nom/character/complete/fn.multispace1.html)<br>character::streaming::[multispace1](https://docs.rs/nom/latest/nom/character/streaming/fn.multispace1.html) | `multispace1` | `" \t\nhello"` | Result: `" \t\n"`<br>Remainder: `"hello"` | Matches one or more spaces (`' '`), tabs (`\t`), line feeds (`\n`), and carriage returns (`\r`) |
| bytes::complete::[take](https://docs.rs/nom/latest/nom/bytes/complete/fn.take.html)<br>bytes::streaming::[take](https://docs.rs/nom/latest/nom/bytes/streaming/fn.take.html)<br>bits::complete::[take](https://docs.rs/nom/latest/nom/bits/complete/fn.take.html)<br>bits::streaming::[take](https://docs.rs/nom/latest/nom/bits/streaming/fn.take.html) | `take(4_u8)` | `"hello"` | Result: `"hell"`<br>Remainder: `"o"` | Takes a specific number of characters, bytes, or bits |
| bytes::complete::[take_while](https://docs.rs/nom/latest/nom/bytes/complete/fn.take_while.html)<br>bytes::streaming::[take_while](https://docs.rs/nom/latest/nom/bytes/streaming/fn.take_while.html)<br>bytes::complete::[take_while1](https://docs.rs/nom/latest/nom/bytes/complete/fn.take_while1.html)<br>bytes::streaming::[take_while1](https://docs.rs/nom/latest/nom/bytes/streaming/fn.take_while1.html) | `take_while(\|c\| c as u32 > 64)` | `"abc123"` | Result: `"abc"`<br>Remainder: `"123"` | Returns the longest consecutive list of bytes or characters for which the provided function returns true. `take_while1` does the same, but must return at least one character |
|  | `take_while(\|c\| c < 0x7f)` | `&[0x01, 0x02, 0x03, 0xf0, 0x9f, 0x92, 0x9e]` | Result: `[1, 2, 3]`<br>Remainder: `&[0xf0, 0x9f, 0x92, 0x9e]` |  |
|  | `take_while(\|c\| c as u32 > 64)` | `"💞🦀⌨"` | Result: `"💞🦀⌨"`<br>No remainder | Be careful with casting `char` to `u8`. Casting to `u32` works as expected |
|  | `take_while(\|c\| c as u8 > 64)` | `"💞🦀⌨"` | Result: `"💞🦀"`<br>Remainder: `"⌨"` | But casting to `u8` is lossy |
| bytes::complete::[take_while_m_n](https://docs.rs/nom/latest/nom/bytes/complete/fn.take_while_m_n.html)<br>bytes::streaming::[take_while_m_n](https://docs.rs/nom/latest/nom/bytes/streaming/fn.take_while_m_n.html) | `take_while_m_n(4, 5, \|c: char\| c.is_ascii_alphanumeric())` | `"abcd123"` | Result: `"abcd1"`<br>Remainder: `"23"` | Like `take_while`, but with a minimum and maximum length for the match |
|  | `take_while_m_n(4, 5, \|c: char\| c.is_ascii_alphanumeric())` | `"abcd-123"` | Result: `"abcd"`<br>Remainder: `"-123"` | In the example above, parsing stops because the upper limit is reached. In this one, the predicate stops being true |
|  | `take_while_m_n(4, 5, \|c: char\| c.is_ascii_alphanumeric())` | `"abc-123"` | Error<br>Byte offset: 0<br>Code: TakeWhileMN | And here the lower limit isn't reached yet when the predicate stops being true |
| bytes::complete::[take_till](https://docs.rs/nom/latest/nom/bytes/complete/fn.take_till.html)<br>bytes::streaming::[take_till](https://docs.rs/nom/latest/nom/bytes/streaming/fn.take_till.html)<br>bytes::complete::[take_till1](https://docs.rs/nom/latest/nom/bytes/complete/fn.take_till1.html)<br>bytes::streaming::[take_till1](https://docs.rs/nom/latest/nom/bytes/streaming/fn.take_till1.html) | `take_till(\|c\| c as u32 <= 64)` | `"abc123"` | Result: `"abc"`<br>Remainder: `"123"` | Returns the longest list of consecutive bytes or characters for which the provided function returns false. `take_till1` does the same, but must return at least one character. Basically `take_till` is the same as `take_while` but with the result of the provided function negated |
| bytes::complete::[take_until](https://docs.rs/nom/latest/nom/bytes/complete/fn.take_until.html)<br>bytes::streaming::[take_until](https://docs.rs/nom/latest/nom/bytes/streaming/fn.take_until.html)<br>bytes::complete::[take_until1](https://docs.rs/nom/latest/nom/bytes/complete/fn.take_until1.html)<br>bytes::streaming::[take_until1](https://docs.rs/nom/latest/nom/bytes/streaming/fn.take_until1.html) | `take_until("world")` | `"Hello world"` | Result: `"Hello "`<br>Remainder: `"world"` | Returns the longest list of bytes or characters until the provided tag is found. `take_until1` does the same, but must return at least one character |
| bytes::complete::[escaped](https://docs.rs/nom/latest/nom/bytes/complete/fn.escaped.html)<br>bytes::streaming::[escaped](https://docs.rs/nom/latest/nom/bytes/streaming/fn.escaped.html) | `escaped(digit1, '\\', one_of(r#""n\"#))` | `r#"12\"34"#` | Result: `"12\\\"34"`<br>No remainder | Matches a string with escaped characters. The first parser is for regular characters, the second is the control (escape) character, and the third is for the escaped characters. Note that the string is delimited with `r#"` and `"#`, so the backslash is in the string. |
|  | `escaped(digit1, '\\', one_of(r#""n\"#))` | `r#"12"34"#` | Result: `"12"`<br>Remainder: `"\"34"` | Note how the `"` between `2` and `3` is not preceded by a `\` here, and thus parsing ends here |
|  | `delimited(char('@'), escaped(is_not("@;"), ';', one_of("@;")), char('@'))` | `"@hello;@world;;@"` | Result: `"hello;@world;;"`<br>No remainder | This is a good example of why `escaped` is useful. First of all, the value we're looking for is delimited at start and end by a `@`. But it also contains a `@` which is escaped by a `;`. So the normal characters parser says "anything except `@` and `;`." The parsing doesn't stop at the escaped `@` because it's escaped with the `;`, and allowed by the third parser. Likewise the `;;` at the end is allowed as well |
|  | `delimited(char('"'), escaped(is_not(r#""\"#), '\\', one_of(r#""\"#)), char('"'))` | `r#""hello\"world\\""#` | Result: `"hello\\\"world\\\\"`<br>No remainder | This is identical to the previous example, except we use `\` as the control character, and `"` as the delimiter. It is just a lot harder to read because of the escaping we have to do to get Rust to grok our strings |
|  | `delimited(char('"'), escaped(is_not("\"\\"), '\\', one_of("\"\\")), char('"'))` | `"\"hello\\\"world\\\\\""` | Result: `"hello\\\"world\\\\"`<br>No remainder | And again, the same as previous but with different notation |
|  | `escaped(digit1, '\\', tag("boop"))` | `r"12\boop34boo"` | Result: `"12\\boop34"`<br>Remainder: `"boo"` | The escaped parser can actually be any parser, so here we're looking for the string `boop` instead of just a single character |
| bytes::complete::[escaped_transform](https://docs.rs/nom/latest/nom/bytes/complete/fn.escaped_transform.html)<br>bytes::streaming::[escaped_transform](https://docs.rs/nom/latest/nom/bytes/streaming/fn.escaped_transform.html) | `escaped_transform(alpha1, '\\', value("n", char('n')))` | `r"ab\ncd"` | Result: `"abncd"`<br>No remainder | Similar to `escaped`, but the third parser can return a different value into which the control character and escaped character are transformed. [`value`](#general-combinators) is very useful for this, but you can use your own parsers as well |
|  | `escaped_transform(alpha1, '\\', value("BOO", char('n')))` | `r"ab\ncd"` | Result: `"abBOOcd"`<br>No remainder | Above `\n` is transformed into just `n`, but here that combo is transformed into `BOO` |
|  | `escaped_transform(alpha1, '\\', alt((value("BOO", char('n')), value("EEK", char('c')))))` | `r"ab\ncd\cef"` | Result: `"abBOOcdEEKef"`<br>No remainder | [`alt`](#choice-combinators) is useful to transform multiple different escape sequences into different values. In addition to `\n` into `BOO`, `\c` is converted into `EEK` |

### Numbers

Nom can parse numbers either in [text](#text-to-number) or [binary](#binary-to-number) formats.

#### Text to number

| parser | usage | input | output | description |
|---|---|---|---|---|
| character::complete::[i8](https://docs.rs/nom/latest/nom/character/complete/fn.i8.html)<br>character::streaming::[i8](https://docs.rs/nom/latest/nom/character/streaming/fn.i8.html)<br>character::complete::[i16](https://docs.rs/nom/latest/nom/character/complete/fn.i16.html)<br>character::streaming::[i16](https://docs.rs/nom/latest/nom/character/streaming/fn.i16.html)<br>character::complete::[i32](https://docs.rs/nom/latest/nom/character/complete/fn.i32.html)<br>character::streaming::[i32](https://docs.rs/nom/latest/nom/character/streaming/fn.i32.html)<br>character::complete::[i64](https://docs.rs/nom/latest/nom/character/complete/fn.i64.html)<br>character::streaming::[i64](https://docs.rs/nom/latest/nom/character/streaming/fn.i64.html)<br>character::complete::[i128](https://docs.rs/nom/latest/nom/character/complete/fn.i128.html)<br>character::streaming::[i128](https://docs.rs/nom/latest/nom/character/streaming/fn.i128.html) | `i8` | `"123"` | Result: `123`<br>No remainder | Recognizes a signed integer. Various bitsize functions are available |
|  | `i8` | `"123abc"` | Result: `123`<br>Remainder: `"abc"` | As always, remaining characters are ignored |
|  | `i8` | `"+123"` | Result: `123`<br>No remainder | You can use a sign if you want to |
|  | `i8` | `"-123"` | Result: `-123`<br>No remainder |  |
|  | `i8` | `"-200"` | Error<br>Byte offset: 0<br>Code: Digit | If the digits make a number that's too large, you will get an error |
| character::complete::[u8](https://docs.rs/nom/latest/nom/character/complete/fn.u8.html)<br>character::streaming::[u8](https://docs.rs/nom/latest/nom/character/streaming/fn.u8.html)<br>character::complete::[u16](https://docs.rs/nom/latest/nom/character/complete/fn.u16.html)<br>character::streaming::[u16](https://docs.rs/nom/latest/nom/character/streaming/fn.u16.html)<br>character::complete::[u32](https://docs.rs/nom/latest/nom/character/complete/fn.u32.html)<br>character::streaming::[u32](https://docs.rs/nom/latest/nom/character/streaming/fn.u32.html)<br>character::complete::[u64](https://docs.rs/nom/latest/nom/character/complete/fn.u64.html)<br>character::streaming::[u64](https://docs.rs/nom/latest/nom/character/streaming/fn.u64.html)<br>character::complete::[u128](https://docs.rs/nom/latest/nom/character/complete/fn.u128.html)<br>character::streaming::[u128](https://docs.rs/nom/latest/nom/character/streaming/fn.u128.html) | `u8` | `"123"` | Result: `123`<br>No remainder | Recognizes an unsigned integer. Various bitsize functions are available |
|  | `u8` | `"123abc"` | Result: `123`<br>Remainder: `"abc"` |  |
|  | `u8` | `"+123"` | Error<br>Byte offset: 0<br>Code: Digit |  |
|  | `u8` | `"-123"` | Error<br>Byte offset: 0<br>Code: Digit |  |
| number::complete::[double](https://docs.rs/nom/latest/nom/number/complete/fn.double.html)<br>number::streaming::[double](https://docs.rs/nom/latest/nom/number/streaming/fn.double.html)<br>number::complete::[float](https://docs.rs/nom/latest/nom/number/complete/fn.float.html)<br>number::streaming::[float](https://docs.rs/nom/latest/nom/number/streaming/fn.float.html) | `double` | `"123E-02"` | Result: `1.23`<br>No remainder | `double` recognizes floating point number in text format and returns an `f64`.  `float` does the same for `f32` |
|  | `double` | `"123.456"` | Result: `123.456`<br>No remainder |  |
|  | `double` | `"123.456E-02"` | Result: `1.23456`<br>No remainder |  |
|  | `double` | `"123.456E+02"` | Result: `12345.6`<br>No remainder |  |
|  | `double` | `"123.456hello"` | Result: `123.456`<br>Remainder: `"hello"` |  |
|  | `double` | `"123.456e0hi"` | Result: `123.456`<br>Remainder: `"hi"` |  |
| number::complete::[recognize_float](https://docs.rs/nom/latest/nom/number/complete/fn.recognize_float.html)<br>number::streaming::[recognize_float](https://docs.rs/nom/latest/nom/number/streaming/fn.recognize_float.html) | `recognize_float` | `"123E-02"` | Result: `"123E-02"`<br>No remainder | Recognizes floating point number in text format and returns the corresponding slice (there is no `recognize_double` as there is no difference in the text form of float vs double) |
|  | `recognize_float` | `"123.456"` | Result: `"123.456"`<br>No remainder |  |
|  | `recognize_float` | `"123.456E-02"` | Result: `"123.456E-02"`<br>No remainder |  |
|  | `recognize_float` | `"123.456E+02"` | Result: `"123.456E+02"`<br>No remainder |  |
|  | `recognize_float` | `"123.456hello"` | Result: `"123.456"`<br>Remainder: `"hello"` | As always, remaining characters are ignored |
|  | `recognize_float` | `"123.456e0hi"` | Result: `"123.456e0"`<br>Remainder: `"hi"` |  |
|  | `recognize(float)` | `"123E-02"` | Result: `"123E-02"`<br>No remainder | `recognize_float` is basically a slightly more optimal version of `recognize(double)` or `recognize(float)` |
|  | `recognize(double)` | `"123E-02"` | Result: `"123E-02"`<br>No remainder |  |
| number::complete::[hex_u32](https://docs.rs/nom/latest/nom/number/complete/fn.hex_u32.html)<br>number::streaming::[hex_u32](https://docs.rs/nom/latest/nom/number/streaming/fn.hex_u32.html) | `hex_u32` | `b"abcxyz"` | Result: `2748`<br>Remainder: `&[0x78, 0x79, 0x7a]` | Recognizes hex-encoded `u32`. This only works with `&[u8]` inputs |
|  | `hex_u32` | `&[0x61, 0x62, 0x63, 0x78, 0x79, 0x7a]` | Result: `2748`<br>Remainder: `&[0x78, 0x79, 0x7a]` | But for some reason, we're doing character recognition (this is the same as the `b"abcxyz"` above) |

#### Binary to number

| parser | usage | input | output | description |
|---|---|---|---|---|
| number::complete::[i8](https://docs.rs/nom/latest/nom/number/complete/fn.i8.html)<br>number::streaming::[i8](https://docs.rs/nom/latest/nom/number/streaming/fn.i8.html) | `i8` | `&[0xf0]` | Result: `-16`<br>No remainder | Recognizes a signed integer. Endianness does not matter for single byte numbers, so there's no `Endianness` parameter |
| number::complete::[u8](https://docs.rs/nom/latest/nom/number/complete/fn.u8.html)<br>number::streaming::[u8](https://docs.rs/nom/latest/nom/number/streaming/fn.u8.html) | `u8` | `&[0xf0]` | Result: `240`<br>No remainder | Recognizes a unsigned integer. Endianness does not matter for single byte numbers, so there's no `Endianness` parameter |
| number::complete::[i16](https://docs.rs/nom/latest/nom/number/complete/fn.i16.html)<br>number::streaming::[i16](https://docs.rs/nom/latest/nom/number/streaming/fn.i16.html)<br>number::complete::[i24](https://docs.rs/nom/latest/nom/number/complete/fn.i24.html)<br>number::streaming::[i24](https://docs.rs/nom/latest/nom/number/streaming/fn.i24.html)<br>number::complete::[i32](https://docs.rs/nom/latest/nom/number/complete/fn.i32.html)<br>number::streaming::[i32](https://docs.rs/nom/latest/nom/number/streaming/fn.i32.html)<br>number::complete::[i64](https://docs.rs/nom/latest/nom/number/complete/fn.i64.html)<br>number::streaming::[i64](https://docs.rs/nom/latest/nom/number/streaming/fn.i64.html)<br>number::complete::[i128](https://docs.rs/nom/latest/nom/number/complete/fn.i128.html)<br>number::streaming::[i128](https://docs.rs/nom/latest/nom/number/streaming/fn.i128.html) | `i16(Endianness::Big)` | `&[0xff, 0x00]` | Result: `-256`<br>No remainder | Recognizes a signed integer. Various bitsize functions are available. Endianness handled according to parameter |
| number::complete::[u16](https://docs.rs/nom/latest/nom/number/complete/fn.u16.html)<br>number::streaming::[u16](https://docs.rs/nom/latest/nom/number/streaming/fn.u16.html)<br>number::complete::[u24](https://docs.rs/nom/latest/nom/number/complete/fn.u24.html)<br>number::streaming::[u24](https://docs.rs/nom/latest/nom/number/streaming/fn.u24.html)<br>number::complete::[u32](https://docs.rs/nom/latest/nom/number/complete/fn.u32.html)<br>number::streaming::[u32](https://docs.rs/nom/latest/nom/number/streaming/fn.u32.html)<br>number::complete::[u64](https://docs.rs/nom/latest/nom/number/complete/fn.u64.html)<br>number::streaming::[u64](https://docs.rs/nom/latest/nom/number/streaming/fn.u64.html)<br>number::complete::[u128](https://docs.rs/nom/latest/nom/number/complete/fn.u128.html)<br>number::streaming::[u128](https://docs.rs/nom/latest/nom/number/streaming/fn.u128.html) | `u16(Endianness::Big)` | `&[0xff, 0x00]` | Result: `65280`<br>No remainder | Recognizes a unsigned integer. Various bitsize functions are available. Endianness handled according to parameter |
| number::[Endianness](https://docs.rs/nom/latest/nom/number/enum.Endianness.html) | `u16(Endianness::Little)` | `&[0xff, 0x00]` | Result: `255`<br>No remainder | Endianness can be `Big`, `Little`, or `Native` |
|  | `u16(Endianness::Native)` | `&[0xff, 0x00]` | Result: `255`<br>No remainder |  |
| number::complete::[be_i8](https://docs.rs/nom/latest/nom/number/complete/fn.be_i8.html)<br>number::streaming::[be_i8](https://docs.rs/nom/latest/nom/number/streaming/fn.be_i8.html)<br>number::complete::[be_i16](https://docs.rs/nom/latest/nom/number/complete/fn.be_i16.html)<br>number::streaming::[be_i16](https://docs.rs/nom/latest/nom/number/streaming/fn.be_i16.html)<br>number::complete::[be_i24](https://docs.rs/nom/latest/nom/number/complete/fn.be_i24.html)<br>number::streaming::[be_i24](https://docs.rs/nom/latest/nom/number/streaming/fn.be_i24.html)<br>number::complete::[be_i32](https://docs.rs/nom/latest/nom/number/complete/fn.be_i32.html)<br>number::streaming::[be_i32](https://docs.rs/nom/latest/nom/number/streaming/fn.be_i32.html)<br>number::complete::[be_i64](https://docs.rs/nom/latest/nom/number/complete/fn.be_i64.html)<br>number::streaming::[be_i64](https://docs.rs/nom/latest/nom/number/streaming/fn.be_i64.html)<br>number::complete::[be_i128](https://docs.rs/nom/latest/nom/number/complete/fn.be_i128.html)<br>number::streaming::[be_i128](https://docs.rs/nom/latest/nom/number/streaming/fn.be_i128.html) | `be_i16` | `&[0xff, 0xaa]` | Result: `-86`<br>No remainder | Recognizes a big endian signed integer |
| number::complete::[be_u8](https://docs.rs/nom/latest/nom/number/complete/fn.be_u8.html)<br>number::streaming::[be_u8](https://docs.rs/nom/latest/nom/number/streaming/fn.be_u8.html)<br>number::complete::[be_u16](https://docs.rs/nom/latest/nom/number/complete/fn.be_u16.html)<br>number::streaming::[be_u16](https://docs.rs/nom/latest/nom/number/streaming/fn.be_u16.html)<br>number::complete::[be_u24](https://docs.rs/nom/latest/nom/number/complete/fn.be_u24.html)<br>number::streaming::[be_u24](https://docs.rs/nom/latest/nom/number/streaming/fn.be_u24.html)<br>number::complete::[be_u32](https://docs.rs/nom/latest/nom/number/complete/fn.be_u32.html)<br>number::streaming::[be_u32](https://docs.rs/nom/latest/nom/number/streaming/fn.be_u32.html)<br>number::complete::[be_u64](https://docs.rs/nom/latest/nom/number/complete/fn.be_u64.html)<br>number::streaming::[be_u64](https://docs.rs/nom/latest/nom/number/streaming/fn.be_u64.html)<br>number::complete::[be_u128](https://docs.rs/nom/latest/nom/number/complete/fn.be_u128.html)<br>number::streaming::[be_u128](https://docs.rs/nom/latest/nom/number/streaming/fn.be_u128.html) | `be_u16` | `&[0xff, 0xaa]` | Result: `65450`<br>No remainder | Recognizes a big endian unsigned integer |
| number::complete::[le_i8](https://docs.rs/nom/latest/nom/number/complete/fn.le_i8.html)<br>number::streaming::[le_i8](https://docs.rs/nom/latest/nom/number/streaming/fn.le_i8.html)<br>number::complete::[le_i16](https://docs.rs/nom/latest/nom/number/complete/fn.le_i16.html)<br>number::streaming::[le_i16](https://docs.rs/nom/latest/nom/number/streaming/fn.le_i16.html)<br>number::complete::[le_i24](https://docs.rs/nom/latest/nom/number/complete/fn.le_i24.html)<br>number::streaming::[le_i24](https://docs.rs/nom/latest/nom/number/streaming/fn.le_i24.html)<br>number::complete::[le_i32](https://docs.rs/nom/latest/nom/number/complete/fn.le_i32.html)<br>number::streaming::[le_i32](https://docs.rs/nom/latest/nom/number/streaming/fn.le_i32.html)<br>number::complete::[le_i64](https://docs.rs/nom/latest/nom/number/complete/fn.le_i64.html)<br>number::streaming::[le_i64](https://docs.rs/nom/latest/nom/number/streaming/fn.le_i64.html)<br>number::complete::[le_i128](https://docs.rs/nom/latest/nom/number/complete/fn.le_i128.html)<br>number::streaming::[le_i128](https://docs.rs/nom/latest/nom/number/streaming/fn.le_i128.html) | `le_i16` | `&[0xff, 0xaa]` | Result: `-21761`<br>No remainder | Recognizes a big endian signed integer |
| number::complete::[le_u8](https://docs.rs/nom/latest/nom/number/complete/fn.le_u8.html)<br>number::streaming::[le_u8](https://docs.rs/nom/latest/nom/number/streaming/fn.le_u8.html)<br>number::complete::[le_u16](https://docs.rs/nom/latest/nom/number/complete/fn.le_u16.html)<br>number::streaming::[le_u16](https://docs.rs/nom/latest/nom/number/streaming/fn.le_u16.html)<br>number::complete::[le_u24](https://docs.rs/nom/latest/nom/number/complete/fn.le_u24.html)<br>number::streaming::[le_u24](https://docs.rs/nom/latest/nom/number/streaming/fn.le_u24.html)<br>number::complete::[le_u32](https://docs.rs/nom/latest/nom/number/complete/fn.le_u32.html)<br>number::streaming::[le_u32](https://docs.rs/nom/latest/nom/number/streaming/fn.le_u32.html)<br>number::complete::[le_u64](https://docs.rs/nom/latest/nom/number/complete/fn.le_u64.html)<br>number::streaming::[le_u64](https://docs.rs/nom/latest/nom/number/streaming/fn.le_u64.html)<br>number::complete::[le_u128](https://docs.rs/nom/latest/nom/number/complete/fn.le_u128.html)<br>number::streaming::[le_u128](https://docs.rs/nom/latest/nom/number/streaming/fn.le_u128.html) | `le_u16` | `&[0xff, 0xaa]` | Result: `43775`<br>No remainder | Recognizes a big endian unsigned integer |
| number::complete::[be_f32](https://docs.rs/nom/latest/nom/number/complete/fn.be_f32.html)<br>number::streaming::[be_f32](https://docs.rs/nom/latest/nom/number/streaming/fn.be_f32.html)<br>number::complete::[be_f64](https://docs.rs/nom/latest/nom/number/complete/fn.be_f64.html)<br>number::streaming::[be_f64](https://docs.rs/nom/latest/nom/number/streaming/fn.be_f64.html) | `be_f32` | `&[0x41, 0x48, 0x00, 0x00]` | Result: `12.5`<br>No remainder | Recognizes a big endian floating point number |
| number::complete::[le_f32](https://docs.rs/nom/latest/nom/number/complete/fn.le_f32.html)<br>number::streaming::[le_f32](https://docs.rs/nom/latest/nom/number/streaming/fn.le_f32.html)<br>number::complete::[le_f64](https://docs.rs/nom/latest/nom/number/complete/fn.le_f64.html)<br>number::streaming::[le_f64](https://docs.rs/nom/latest/nom/number/streaming/fn.le_f64.html) | `le_f32` | `&[0x00, 0x00, 0x48, 0x41]` | Result: `12.5`<br>No remainder | Recognizes a big endian floating point number |
|  | `le_f32` | `&[0x00, 0x00, 0x48, 0x41, 0x06, 0x09]` | Result: `12.5`<br>Remainder: `&[0x06, 0x09]` | All of these parsers only ever consume the exact number of bytes of their corresponding type |

## General combinators

A combinator is a function that takes one or more parsers as arguments and returns a new parser. This allows you to combine parsers in various ways to create more complex parsers.

| combinator | usage | input | output | description |
|---|---|---|---|---|
| combinator::[value](https://docs.rs/nom/latest/nom/combinator/fn.value.html) | `value(1234, alpha1)` | `"abc789def"` | Result: `1234`<br>Remainder: `"789def"` | Returns the provided value if the parser succeeds |
| combinator::[map](https://docs.rs/nom/latest/nom/combinator/fn.map.html) | `map(digit1, \|s: &str\| s.parse::<u8>().unwrap())` | `"123abc"` | Result: `123`<br>Remainder: `"abc"` | Maps a function on the result of a parser |
| combinator::[map_opt](https://docs.rs/nom/latest/nom/combinator/fn.map_opt.html) | `map_opt(digit1, \|s: &str\| s.parse::<u8>().ok())` | `"123abc"` | Result: `123`<br>Remainder: `"abc"` | Same as `map()` but requires the function to return an `Option` |
| combinator::[map_res](https://docs.rs/nom/latest/nom/combinator/fn.map_res.html) | `map_res(digit1, \|s: &str\| s.parse::<u8>())` | `"123abc"` | Result: `123`<br>Remainder: `"abc"` | Same as `map()` but requires the function to return an `Result` |
| combinator::[flat_map](https://docs.rs/nom/latest/nom/combinator/fn.flat_map.html) | `flat_map(u8, take)` | `&[2, 90, 91, 92, 93]` | Result: `[90, 91]`<br>Remainder: `&[0x5c, 0x5d]` | Apply the first parser, then use its output as the argument for the second parser and apply that to the rest of the input. In this example `u8` reads a single byte as an unsigned integer, then makes that the argument to `take` causing it to read the next 2 bytes |
| combinator::[map_parser](https://docs.rs/nom/latest/nom/combinator/fn.map_parser.html) | `map_parser(take(5u8), digit1)` | `"123abc"` | Result: `"123"`<br>Remainder: `"c"` | Apply the second parser on the result of the first parser |
| combinator::[not](https://docs.rs/nom/latest/nom/combinator/fn.not.html) | `not(alpha1)` | `"123"` | Result: `()`<br>Remainder: `"123"` | Succeeds if the child parser returns an error |
| combinator::[opt](https://docs.rs/nom/latest/nom/combinator/fn.opt.html) | `opt(alpha1)` | `"abc123"` | Result: `Some("abc")`<br>Remainder: `"123"` | Returns an `Option` of the child parser. `Some()` if the child parser is succesful, and `None` if not |
| combinator::[peek](https://docs.rs/nom/latest/nom/combinator/fn.peek.html) | `peek(alpha1)` | `"abc123"` | Result: `"abc"`<br>Remainder: `"abc123"` | Applies the child parser but does not consume the input |
|  | `alpha1` | `"abc123"` | Result: `"abc"`<br>Remainder: `"123"` |  |
| combinator::[recognize](https://docs.rs/nom/latest/nom/combinator/fn.recognize.html) | `recognize(separated_pair(alpha1, char(','), alpha1))` | `"abc,def"` | Result: `"abc,def"`<br>No remainder | Returns a slice of the input consumed by the child parser/combinator. No matter how complex/nested, or whether combinators throw parts away, this will return a single slice with everything that was consumed |
| combinator::[rest](https://docs.rs/nom/latest/nom/combinator/fn.rest.html) | `rest` | `"abc"` | Result: `"abc"`<br>No remainder | Returns the remaining input. Mainly useful for combining with other combinators |
| combinator::[rest_len](https://docs.rs/nom/latest/nom/combinator/fn.rest_len.html) | `rest_len` | `"abc"` | Result: `3`<br>Remainder: `"abc"` | Returns the length of the remaining input, does not consume anything |
| combinator::[into](https://docs.rs/nom/latest/nom/combinator/fn.into.html) | `let output: IResult<&str, Vec<u8>> = into(my_alpha1)` | `"abcd"` | Result: `[97, 98, 99, 100]`<br>No remainder | Use Rust's `Into` trait to convert the result of a parser if possible |

## Choice combinators

| combinator | usage | input | output | description |
|---|---|---|---|---|
| branch::[alt](https://docs.rs/nom/latest/nom/branch/fn.alt.html) | `alt((tag("ab"), tag("cd")))` | `"cdef"` | Result: `"cd"`<br>Remainder: `"ef"` | Try a list of parsers and return the result of the first successful one |
| combinator::[success](https://docs.rs/nom/latest/nom/combinator/fn.success.html) | `success(1)` | `"abc"` | Result: `1`<br>Remainder: `"abc"` | Always succeeds and returns the given value without consuming any input. Useful for giving `alt` a default |
|  | `alt((value(-1, char('-')), value(1, char('+')), success(1)))` | `"10"` | Result: `1`<br>Remainder: `"10"` |  |
| branch::[permutation](https://docs.rs/nom/latest/nom/branch/fn.permutation.html) | `permutation((tag("ab"), tag("cd"), tag("12")))` | `"cd12abc"` | Result: `("ab", "cd", "12")`<br>Remainder: `"c"` | Succeeds when all its child parser have succeeded, whatever the order |
|  | `permutation((tag("ab"), tag("cd"), tag("12")))` | `"abcd12"` | Result: `("ab", "cd", "12")`<br>No remainder |  |
|  | `permutation((tag("ab"), tag("cd"), tag("12")))` | `"12cd"` | Error<br>Byte offset: 4<br>Code: Tag | But _all_ parsers need to succeed |
| combinator::[cond](https://docs.rs/nom/latest/nom/combinator/fn.cond.html) | `cond(true, alpha1)` | `"abc123"` | Result: `Some("abc")`<br>Remainder: `"123"` | Return result from the parser if the first argument is true, otherwise return `None` |

## Sequence combinators

| combinator | usage | input | output | description |
|---|---|---|---|---|
| sequence::[delimited](https://docs.rs/nom/latest/nom/sequence/fn.delimited.html) | `delimited(char('('), take(2u8), char(')'))` | `"(ab)cd"` | Result: `"ab"`<br>Remainder: `"cd"` | Returns only the second parser out of three |
| sequence::[preceded](https://docs.rs/nom/latest/nom/sequence/fn.preceded.html) | `preceded(tag("ab"), tag("XY"))` | `"abXYZ"` | Result: `"XY"`<br>Remainder: `"Z"` | Returns only the second parser out of two |
| sequence::[terminated](https://docs.rs/nom/latest/nom/sequence/fn.terminated.html) | `terminated(tag("ab"), tag("XY"))` | `"abXYZ"` | Result: `"ab"`<br>Remainder: `"Z"` | Returns only the result from the first parser out of two, discarding the other |
| sequence::[pair](https://docs.rs/nom/latest/nom/sequence/fn.pair.html) | `pair(tag("ab"), tag("XY"))` | `"abXYZ"` | Result: `("ab", "XY")`<br>Remainder: `"Z"` | Applies two parsers, returns their results as a tuple |
| sequence::[separated_pair](https://docs.rs/nom/latest/nom/sequence/fn.separated_pair.html) | `separated_pair(tag("hello"), char(','), tag("world"))` | `"hello,world!"` | Result: `("hello", "world")`<br>Remainder: `"!"` | Returns the results from the first and third parsers as a tuple, discarding the second |
| sequence::[tuple](https://docs.rs/nom/latest/nom/sequence/fn.tuple.html) | `tuple((tag("ab"), tag("XY"), take(1u8)))` | `"abXYZ!"` | Result: `("ab", "XY", "Z")`<br>Remainder: `"!"` | Chains parsers and assembles the sub results in a tuple. You can use as many child parsers as you can put elements in a tuple |

## Applying a parser multiple times

| combinator | usage | input | output | description |
|---|---|---|---|---|
| multi::[count](https://docs.rs/nom/latest/nom/multi/fn.count.html) | `count(take(2u8), 3)` | `"abcdefgh"` | Result: `["ab", "cd", "ef"]`<br>Remainder: `"gh"` | Applies the child parser a specified number of times and returns the list of results in a `Vec` |
| multi::[many0](https://docs.rs/nom/latest/nom/multi/fn.many0.html)<br>multi::[many1](https://docs.rs/nom/latest/nom/multi/fn.many1.html) | `many0(tag("ab"))` | `"abababc"` | Result: `["ab", "ab", "ab"]`<br>Remainder: `"c"` | Applies the parser 0 or more times and returns the list of results in a `Vec`. `many1` does the same operation but must return at least one element |
| multi::[many_m_n](https://docs.rs/nom/latest/nom/multi/fn.many_m_n.html) | `many_m_n(2, 2, tag("ab"))` | `"ababc"` | Result: `["ab", "ab"]`<br>Remainder: `"c"` | Applies the parser at least `m` and at most `n` times and returns the list of results in a `Vec` |
| multi::[many_till](https://docs.rs/nom/latest/nom/multi/fn.many_till.html) | `many_till(tag("ab"), tag("ef"))` | `"ababefg"` | Result: `(["ab", "ab"], "ef")`<br>Remainder: `"g"` | Applies the first parser until the second applies. Returns a tuple containing the list of results from the first in a `Vec` and the result of the second |
| multi::[separated_list0](https://docs.rs/nom/latest/nom/multi/fn.separated_list0.html)<br>multi::[separated_list1](https://docs.rs/nom/latest/nom/multi/fn.separated_list1.html) | `separated_list0(tag(","), tag("ab"))` | `"ab,ab,ab."` | Result: `["ab", "ab", "ab"]`<br>Remainder: `"."` | Using the first parser to match separators, returns a `Vec` of zero or more results from the second parser. `separated_list1` does the same operation but must return at least one element |
| multi::[fold_many0](https://docs.rs/nom/latest/nom/multi/fn.fold_many0.html)<br>multi::[fold_many1](https://docs.rs/nom/latest/nom/multi/fn.fold_many1.html)<br>multi::[fold_many_m_n](https://docs.rs/nom/latest/nom/multi/fn.fold_many_m_n.html) | `fold_many0(take(1u8), Vec::new, \|mut acc, item\| { acc.push(item); acc })` | `"abc"` | Result: `["a", "b", "c"]`<br>No remainder | Applies the parser 0 or more times and folds the list of return values. The `fold_many1` version must apply the parser at least one time, and `fold_many_m_n` must apply the parser at least `m` and at most `n` times |
| multi::[length_count](https://docs.rs/nom/latest/nom/multi/fn.length_count.html) | `length_count(number, tag("ab"))` | `"2ababab"` | Result: `["ab", "ab"]`<br>Remainder: `"ab"` | Gets a number from the first parser, then applies the second parser that many times. `number` is a custom defined parser along the lines of text to integer parsers below |

## Combinators to do with completeness

| combinator | usage | input | output | description |
|---|---|---|---|---|
| combinator::[all_consuming](https://docs.rs/nom/latest/nom/combinator/fn.all_consuming.html) | `all_consuming(pair(alpha1, number))` | `"abc123"` | Result: `("abc", 123)`<br>No remainder | Returns what the child parser returned if, and only if, the input is exhausted. Otherwise returns an error |
|  | `all_consuming(pair(alpha1, number))` | `"abc123abc"` | Error<br>Byte offset: 6<br>Code: Eof |  |
| combinator::[complete](https://docs.rs/nom/latest/nom/combinator/fn.complete.html) | `complete(nom::bytes::streaming::take(5u8))` | `"abcd"` | Error<br>Byte offset: 0<br>Code: Complete | Turns an `Incomplete` result from a streaming parser into an error. The example is the equivalent of `nom::bytes::complete::take(5u8)` |
|  | `nom::bytes::streaming::take(5u8)` | `"abcd"` | Incomplete<br>Needed: unknown |  |
| combinator::[eof](https://docs.rs/nom/latest/nom/combinator/fn.eof.html) | `eof` | `""` | Result: `""`<br>No remainder | Returns an error if the input is not exhausted, otherwise returns the input |
|  | `eof` | `"abc"` | Error<br>Byte offset: 0<br>Code: Eof |  |

### Binary to number


# Fin