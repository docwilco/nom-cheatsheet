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
| character::complete::char<br>character::streaming::char | `char('a')` | `"abc"` |  | Matches one specific character |
| | `char('a')` | `"cba"` | | If that character isn't the immediate input, parsing fails |
| | `char('💞')` | `"💞🦀"` | | Multi-byte characters work as well |
| character::complete::anychar<br>character::streaming::anychar | `anychar` | `"abc"` |  | Matches any single character |
| | `anychar` | `"💞🦀"` || Multi-byte characters work as well | 
| character::complete::one_of<br>character::streaming::one_of | `one_of("abc")` | `"abc"` |  | Matches one of the provided characters |
| character::complete::none_of<br>character::streaming::none_of | `none_of("abc")` | `"xyab"` |  | Matches a single character that is anything but the provided characters |

### Sequence of bytes or characters parsers

These parsers will return a slice of bytes or characters. Those suffixed with `0` can return an empty slice if they match nothing. They usually have variants that are suffixed with `1` that will refuse to match unless there's at least 1 byte or character they can match. For instance, `digit0` will match an empty string, but `digit1` will not:

| parser | usage | input | output | description |
|---|---|---|---|---|
| character::complete::digit0<br>character::streaming::digit0 | `digit0` | `"123abc"` |  | Matches zero or more numerical ASCII characters (`0-9`) |
| character::complete::digit1<br>character::streaming::digit1 | `digit1` | `"123abc"` |  | Matches one or more numerical ASCII characters (`0-9`) |
| | `digit0` | `"abc123"` |  | Because it is allowed to return an empty string, this does not error |
| | `digit1` | `"abc123"` |  | This however does error, because there must be at least one numerical ASCII character |

This goes for all the `0` and `1` suffixed parsers below:

| parser | usage | input | output | description |
|---|---|---|---|---|
| bytes::complete::is_a<br>bytes::streaming::is_a | `is_a("ab")` | `"ababc"` |  | Matches a sequence of any of the characters passed as arguments |
| bytes::complete::is_not<br>bytes::streaming::is_not | `is_not("cd")` | `"ababc"` |  | Matches a sequence of none of the characters passed as arguments |
| character::complete::alpha0<br>character::streaming::alpha0 | `alpha0` | `"abc123"` |  | Matches zero or more alphabetical ASCII characters (`a-zA-Z`) |
| character::complete::alpha1<br>character::streaming::alpha1 | `alpha1` | `"abc123"` |  | Matches one or more alphabetical ASCII characters (`a-zA-Z`) |
| | `alpha1` | `"ααα"` |  | Only ASCII counts for these, not all of the unicode alphabetical characters. (These are Greek Alphas.) |
| character::complete::digit0<br>character::streaming::digit0 | `digit0` | `"123abc"` |  | Matches zero or more numerical ASCII characters (`0-9`) |
| character::complete::digit1<br>character::streaming::digit1 | `digit1` | `"123abc"` |  | Matches one or more numerical ASCII characters (`0-9`) |
| character::complete::alphanumeric0<br>character::streaming::alphanumeric0 | `alphanumeric0` | `"abc123"` |  | Matches zero or more alphanumeric ASCII characters (`a-zA-Z0-9`) |
| character::complete::alphanumeric1<br>character::streaming::alphanumeric1 | `alphanumeric1` | `"abc123"` |  | Matches one or more alphanumeric ASCII characters (`a-zA-Z0-9`) |
| character::complete::hex_digit0<br>character::streaming::hex_digit0 | `hex_digit0` | `"123abcghi"` |  | Matches zero or more hexadecimal ASCII characters (`0-9a-fA-F`) |
| character::complete::hex_digit1<br>character::streaming::hex_digit1 | `hex_digit1` | `"123abcghi"` |  | Matches one or more hexadecimal ASCII characters (`0-9a-fA-F`) |
| bytes::complete::tag<br>bytes::streaming::tag<br>bits::complete::tag<br>bits::streaming::tag | `tag("hello")` | `"hello world"` |  | Recognizes a specific suite of characters, bytes, or bits |
| bytes::complete::tag_no_case<br>bytes::streaming::tag_no_case | `tag_no_case("hello")` | `"HeLLo World"` |  | Recognizes a specific suite of characters, in a case insensitive manner |
| | `tag_no_case("γειά")` | `"Γειά Κόσμο"` | | This also works with non-ASCII characters. A `γ` is a lowercase `Γ`. (Greek Gamma) |
| character::complete::newline<br>character::streaming::newline | `newline` | `"\nhello"` |  | Matches a newline character, also known as line feed, `\n`, or `LF` |
| character::complete::crlf<br>character::streaming::crlf | `crlf` | `"\r\nhello"` |  | Matches a carriage return followed by a newline, also known as `\r\n` or `CRLF` |
| character::complete::line_ending<br>character::streaming::line_ending | `line_ending` | `"\r\nhello"` |  | Matches an end of line, either Unix style (`\n`/`LF`) or Windows style (`\r\n`/`CRLF`) |
| | `line_ending` | `"\nhello"` |  | Basically `line_ending` is the same as [`alt((crlf, newline))`](alt), but has a better performance |
| character::complete::space0<br>character::streaming::space0 | `space0` | `" \t\nhello"` |  | Matches zero or more spaces (`' '`) and tabs (`\t`) |
| character::complete::space1<br>character::streaming::space1 | `space1` | `" \t\nhello"` |  | Matches one or more spaces (`' '`) and tabs (`\t`) |
| character::complete::multispace0<br>character::streaming::multispace0 | `multispace0` | `" \t\nhello"` |  | Matches zero or more spaces (`' '`), tabs (`\t`), line feeds (`\n`), and carriage returns (`\r`) |
| character::complete::multispace1<br>character::streaming::multispace1 | `multispace1` | `" \t\nhello"` |  | Matches one or more spaces (`' '`), tabs (`\t`), line feeds (`\n`), and carriage returns (`\r`) |
| bytes::complete::take<br>bytes::streaming::take<br>bits::complete::take<br>bits::streaming::take | `take(4_u8)` | `"hello"` |  | Takes a specific number of characters, bytes, or bits |
| bytes::complete::take_while<br>bytes::streaming::take_while<br>bytes::complete::take_while1<br>bytes::streaming::take_while1 | `take_while(\|c\| c as u32 > 64)` | `"abc123"` |  | Returns the longest consecutive list of bytes or characters for which the provided function returns true. `take_while1` does the same, but must return at least one character |
| | `take_while(\|c\| c < 0x7f)` | `&[0x01, 0x02, 0x03, 0xf0, 0x9f, 0x92, 0x9e]` |  |  |
| | `take_while(\|c\| c as u32 > 64)` | `"💞🦀⌨"` |  | Be careful with casting `char` to `u8`. Casting to `u32` works as expected |
| | `take_while(\|c\| c as u8 > 64)` | `"💞🦀⌨"` |  | But casting to `u8` is lossy |
| bytes::complete::take_while_m_n<br>bytes::streaming::take_while_m_n | `take_while_m_n(4, 5, \|c: char\| c.is_ascii_alphanumeric())` | `"abcd123"` |  | Like `take_while`, but with a minimum and maximum length for the match |
|  | `take_while_m_n(4, 5, \|c: char\| c.is_ascii_alphanumeric())` | `"abcd-123"` |  | In the example above, parsing stops because the upper limit is reached. In this one, the predicate stops being true |
|  | `take_while_m_n(4, 5, \|c: char\| c.is_ascii_alphanumeric())` | `"abc-123"` |  | And here the lower limit isn't reached yet when the predicate stops being true |
| bytes::complete::take_till<br>bytes::streaming::take_till<br>bytes::complete::take_till1<br>bytes::streaming::take_till1 | `take_till(\|c\| c as u32 <= 64)` | `"abc123"` |  | Returns the longest list of consecutive bytes or characters for which the provided function returns false. `take_till1` does the same, but must return at least one character. Basically `take_till` is the same as `take_while` but with the result of the provided function negated |
| bytes::complete::take_until<br>bytes::streaming::take_until<br>bytes::complete::take_until1<br>bytes::streaming::take_until1 | `take_until("world")` | `"Hello world"` |  | Returns the longest list of bytes or characters until the provided tag is found. `take_until1` does the same, but must return at least one character |
| bytes::complete::escaped<br>bytes::streaming::escaped | `escaped(digit1, '\\', one_of(r#""n\"#))` | `r#"12\"34"#` |  | Matches a string with escaped characters. The first parser is for regular characters, the second is the control (escape) character, and the third is for the escaped characters. Note that the string is delimited with `r#"` and `"#`, so the backslash is in the string. |
| | `escaped(digit1, '\\', one_of(r#""n\"#))` | `r#"12"34"#` |  | Note how the `"` between `2` and `3` is not preceded by a `\` here, and thus parsing ends here |
| | `delimited(char('@'), escaped(is_not("@;"), ';', one_of("@;")), char('@'))` | `"@hello;@world;;@"` |  | This is a good example of why `escaped` is useful. First of all, the value we're looking for is delimited at start and end by a `@`. But it also contains a `@` which is escaped by a `;`. So the normal characters parser says "anything except `@` and `;`." The parsing doesn't stop at the escaped `@` because it's escaped with the `;`, and allowed by the third parser. Likewise the `;;` at the end is allowed as well |
| | `delimited(char('"'), escaped(is_not(r#""\"#), '\\', one_of(r#""\"#)), char('"'))` | `r#""hello\"world\\""#` |  | This is identical to the previous example, except we use `\` as the control character, and `"` as the delimiter. It is just a lot harder to read because of the escaping we have to do to get Rust to grok our strings |
| | `delimited(char('"'), escaped(is_not("\"\\"), '\\', one_of("\"\\")), char('"'))` | `"\"hello\\\"world\\\\\""` |  | And again, the same as previous but with different notation |
| | `escaped(digit1, '\\', tag("boop"))` | `r"12\boop34boo"` |  | The escaped parser can actually be any parser, so here we're looking for the string `boop` instead of just a single character |
| bytes::complete::escaped_transform<br>bytes::streaming::escaped_transform | `escaped_transform(alpha1, '\\', value("n", char('n')))` | `r"ab\ncd"` |  | Similar to `escaped`, but the third parser can return a different value into which the control character and escaped character are transformed. [`value`](#general-combinators) is very useful for this, but you can use your own parsers as well |
| | `escaped_transform(alpha1, '\\', value("BOO", char('n')))` | `r"ab\ncd"` |  | Above `\n` is transformed into just `n`, but here that combo is transformed into `BOO` |
| | `escaped_transform(alpha1, '\\', alt((value("BOO", char('n')), value("EEK", char('c')))))` | `r"ab\ncd\cef"` |  | [`alt`](#choice-combinators) is useful to transform multiple different escape sequences into different values. In addition to `\n` into `BOO`, `\c` is converted into `EEK` |

### Numbers

Nom can parse numbers either in [text](#text-to-number) or [binary](#binary-to-number) formats.

#### Text to number

| parser | usage | input | output | description |
|---|---|---|---|---|
| character::complete::i8<br>character::streaming::i8<br>character::complete::i16<br>character::streaming::i16<br>character::complete::i32<br>character::streaming::i32<br>character::complete::i64<br>character::streaming::i64<br>character::complete::i128<br>character::streaming::i128 | `i8` | `"123"` |  | Recognizes a signed integer. Various bitsize functions are available |
| | `i8` | `"123abc"` |  | As always, remaining characters are ignored |
| | `i8` | `"+123"` |  | You can use a sign if you want to |
| | `i8` | `"-123"` |  |  |
| | `i8` | `"-200"` |  | If the digits make a number that's too large, you will get an error |
| character::complete::u8<br>character::streaming::u8<br>character::complete::u16<br>character::streaming::u16<br>character::complete::u32<br>character::streaming::u32<br>character::complete::u64<br>character::streaming::u64<br>character::complete::u128<br>character::streaming::u128 | `u8` | `"123"` |  | Recognizes an unsigned integer. Various bitsize functions are available |
| | `u8` | `"123abc"` |  |  |
| | `u8` | `"+123"` |  |  |
| | `u8` | `"-123"` |  |  |
| number::complete::double<br>number::streaming::double<br>number::complete::float<br>number::streaming::float | `double` | `"123E-02"` |  | `double` recognizes floating point number in text format and returns an `f64`.  `float` does the same for `f32` |
| | `double` | `"123.456"` |  |  |
| | `double` | `"123.456E-02"` |  |  |
| | `double` | `"123.456E+02"` |  |  |
| | `double` | `"123.456hello"` |  |  |
| | `double` | `"123.456e0hi"` |  |  |
| number::complete::recognize_float<br>number::streaming::recognize_float | `recognize_float` | `"123E-02"` |  | Recognizes floating point number in text format and returns the corresponding slice (there is no `recognize_double` as there is no difference in the text form of float vs double) |
| | `recognize_float` | `"123.456"` |  |  |
| | `recognize_float` | `"123.456E-02"` |  |  |
| | `recognize_float` | `"123.456E+02"` |  |  |
| | `recognize_float` | `"123.456hello"` |  | As always, remaining characters are ignored |
| | `recognize_float` | `"123.456e0hi"` |  |  |
| | `recognize(float)` | `"123E-02"` |  | `recognize_float` is basically a slightly more optimal version of `recognize(double)` or `recognize(float)` |
| | `recognize(double)` | `"123E-02"` |  |  |
| number::complete::hex_u32<br>number::streaming::hex_u32 | `hex_u32` | `b"abcxyz"` |  | Recognizes hex-encoded `u32`. This only works with `&[u8]` inputs |
| | `hex_u32` | `&[0x61, 0x62, 0x63, 0x78, 0x79, 0x7a]` |  | But for some reason, we're doing character recognition (this is the same as the `b"abcxyz"` above) |

#### Binary to number

| parser | usage | input | output | description |
|---|---|---|---|---|
| number::complete::i8<br>number::streaming::i8 | `i8` | `&[0xf0]` |  | Recognizes a signed integer. Endianness does not matter for single byte numbers, so there's no `Endianness` parameter |
| number::complete::u8<br>number::streaming::u8 | `u8` | `&[0xf0]` |  | Recognizes a unsigned integer. Endianness does not matter for single byte numbers, so there's no `Endianness` parameter |
| number::complete::i16<br>number::streaming::i16<br>number::complete::i24<br>number::streaming::i24<br>number::complete::i32<br>number::streaming::i32<br>number::complete::i64<br>number::streaming::i64<br>number::complete::i128<br>number::streaming::i128 | `i16(Endianness::Big)` | `&[0xff, 0x00]` |  | Recognizes a signed integer. Various bitsize functions are available. Endianness handled according to parameter |
| number::complete::u16<br>number::streaming::u16<br>number::complete::u24<br>number::streaming::u24<br>number::complete::u32<br>number::streaming::u32<br>number::complete::u64<br>number::streaming::u64<br>number::complete::u128<br>number::streaming::u128 | `u16(Endianness::Big)` | `&[0xff, 0x00]` |  | Recognizes a unsigned integer. Various bitsize functions are available. Endianness handled according to parameter |
| number::Endianness | `u16(Endianness::Little)` | `&[0xff, 0x00]` |  | Endianness can be `Big`, `Little`, or `Native` |
| | `u16(Endianness::Native)` | `&[0xff, 0x00]` |  |  |
| number::complete::be_i8<br>number::streaming::be_i8<br>number::complete::be_i16<br>number::streaming::be_i16<br>number::complete::be_i24<br>number::streaming::be_i24<br>number::complete::be_i32<br>number::streaming::be_i32<br>number::complete::be_i64<br>number::streaming::be_i64<br>number::complete::be_i128<br>number::streaming::be_i128 | `be_i16` | `&[0xff, 0xaa]` |  | Recognizes a big endian signed integer |
| number::complete::be_u8<br>number::streaming::be_u8<br>number::complete::be_u16<br>number::streaming::be_u16<br>number::complete::be_u24<br>number::streaming::be_u24<br>number::complete::be_u32<br>number::streaming::be_u32<br>number::complete::be_u64<br>number::streaming::be_u64<br>number::complete::be_u128<br>number::streaming::be_u128 | `be_u16` | `&[0xff, 0xaa]` |  | Recognizes a big endian unsigned integer |
| number::complete::le_i8<br>number::streaming::le_i8<br>number::complete::le_i16<br>number::streaming::le_i16<br>number::complete::le_i24<br>number::streaming::le_i24<br>number::complete::le_i32<br>number::streaming::le_i32<br>number::complete::le_i64<br>number::streaming::le_i64<br>number::complete::le_i128<br>number::streaming::le_i128 | `le_i16` | `&[0xff, 0xaa]` |  | Recognizes a big endian signed integer |
| number::complete::le_u8<br>number::streaming::le_u8<br>number::complete::le_u16<br>number::streaming::le_u16<br>number::complete::le_u24<br>number::streaming::le_u24<br>number::complete::le_u32<br>number::streaming::le_u32<br>number::complete::le_u64<br>number::streaming::le_u64<br>number::complete::le_u128<br>number::streaming::le_u128 | `le_u16` | `&[0xff, 0xaa]` |  | Recognizes a big endian unsigned integer |
| number::complete::be_f32<br>number::streaming::be_f32<br>number::complete::be_f64<br>number::streaming::be_f64 | `be_f32` | `&[0x41, 0x48, 0x00, 0x00]` |  | Recognizes a big endian floating point number |
| number::complete::le_f32<br>number::streaming::le_f32<br>number::complete::le_f64<br>number::streaming::le_f64 | `le_f32` | `&[0x00, 0x00, 0x48, 0x41]` |  | Recognizes a big endian floating point number |
| | `le_f32` | `&[0x00, 0x00, 0x48, 0x41, 0x06, 0x09]` |  | All of these parsers only ever consume the exact number of bytes of their corresponding type |

## General combinators

A combinator is a function that takes one or more parsers as arguments and returns a new parser. This allows you to combine parsers in various ways to create more complex parsers.

| combinator | usage | input | output | description |
|---|---|---|---|---|
| combinator::value | `value(1234, alpha1)` | `"abc789def"` |  | Returns the provided value if the parser succeeds |
| combinator::map | `map(digit1, \|s: &str\| s.parse::<u8>().unwrap())` | `"123abc"` |  | Maps a function on the result of a parser |
| combinator::map_opt | `map_opt(digit1, \|s: &str\| s.parse::<u8>().ok())` | `"123abc"` |  | Same as `map()` but requires the function to return an `Option` |
| combinator::map_res | `map_res(digit1, \|s: &str\| s.parse::<u8>())` | `"123abc"` |  | Same as `map()` but requires the function to return an `Result` |
| combinator::flat_map | `flat_map(u8, take)` | `&[2, 90, 91, 92, 93]` |  | Apply the first parser, then use its output as the argument for the second parser and apply that to the rest of the input. In this example `u8` reads a single byte as an unsigned integer, then makes that the argument to `take` causing it to read the next 2 bytes |
| combinator::map_parser | `map_parser(take(5u8), digit1)` | `"123abc"` |  | Apply the second parser on the result of the first parser |
| combinator::not | `not(alpha1)` | `"123"` |  | Succeeds if the child parser returns an error |
| combinator::opt | `opt(alpha1)` | `"abc123"` |  | Returns an `Option` of the child parser. `Some()` if the child parser is succesful, and `None` if not |
| combinator::peek | `peek(alpha1)` | `"abc123"` |  | Applies the child parser but does not consume the input |
|  | `alpha1` | `"abc123"` |  |  |
| combinator::recognize | `recognize(separated_pair(alpha1, char(','), alpha1))` | `"abc,def"` |  | Returns a slice of the input consumed by the child parser/combinator. No matter how complex/nested, or whether combinators throw parts away, this will return a single slice with everything that was consumed |
| | `separated_pair(alpha1, char(','), alpha1)` | `"abc,def"` |  | Here the return value is a tuple of two strings and the comma is discarded, but above only a single string is returned |
| combinator::rest | `rest` | `"abc"` |  | Returns the remaining input. Mainly useful for combining with other combinators |
| combinator::rest_len | `rest_len` | `"abc"` |  | Returns the length of the remaining input, does not consume anything |
| combinator::into | `let output: IResult<&str, Vec<u8>> = into(my_alpha1)` | `"abcd"` |  | Use Rust's `Into` trait to convert the result of a parser if possible |

## Choice combinators

| combinator | usage | input | output | description |
|---|---|---|---|---|
| branch::alt | `alt((tag("ab"), tag("cd")))` | `"cdef"` |  | Try a list of parsers and return the result of the first successful one |
| combinator::success | `success(1)` | `"abc"` |  | Always succeeds and returns the given value without consuming any input. Useful for giving `alt` a default |
|  | `alt((value(-1, char('-')), value(1, char('+')), success(1)))` | `"10"` |  |  |
| branch::permutation | `permutation((tag("ab"), tag("cd"), tag("12")))` | `"cd12abc"` |  | Succeeds when all its child parser have succeeded, whatever the order |
| | `permutation((tag("ab"), tag("cd"), tag("12")))` | `"abcd12"` |  |  |
| | `permutation((tag("ab"), tag("cd"), tag("12")))` | `"12cd"` |  | But _all_ parsers need to succeed |
| combinator::cond | `cond(true, alpha1)` | `"abc123"` |  | Return result from the parser if the first argument is true, otherwise return `None` |

## Sequence combinators

| combinator | usage | input | output | description |
|---|---|---|---|---|
| sequence::delimited | `delimited(char('('), take(2u8), char(')'))` | `"(ab)cd"` |  | Returns only the second parser out of three |
| sequence::preceded | `preceded(tag("ab"), tag("XY"))` | `"abXYZ"` |  | Returns only the second parser out of two |
| sequence::terminated | `terminated(tag("ab"), tag("XY"))` | `"abXYZ"` |  | Returns only the result from the first parser out of two, discarding the other |
| sequence::pair | `pair(tag("ab"), tag("XY"))` | `"abXYZ"` |  | Applies two parsers, returns their results as a tuple |
| sequence::separated_pair | `separated_pair(tag("hello"), char(','), tag("world"))` | `"hello,world!"` |  | Returns the results from the first and third parsers as a tuple, discarding the second |
| sequence::tuple | `tuple((tag("ab"), tag("XY"), take(1u8)))` | `"abXYZ!"` |  | Chains parsers and assembles the sub results in a tuple. You can use as many child parsers as you can put elements in a tuple |

## Applying a parser multiple times

| combinator | usage | input | output | description |
|---|---|---|---|---|
| multi::count | `count(take(2u8), 3)` | `"abcdefgh"` |  | Applies the child parser a specified number of times and returns the list of results in a `Vec` |
| multi::many0<br>multi::many1 | `many0(tag("ab"))` | `"abababc"` |  | Applies the parser 0 or more times and returns the list of results in a `Vec`. `many1` does the same operation but must return at least one element |
| multi::many_m_n | `many_m_n(2, 2, tag("ab"))` | `"ababc"` |  | Applies the parser at least `m` and at most `n` times and returns the list of results in a `Vec` |
| multi::many_till | `many_till(tag("ab"), tag("ef"))` | `"ababefg"` |  | Applies the first parser until the second applies. Returns a tuple containing the list of results from the first in a `Vec` and the result of the second |
| multi::separated_list0<br>multi::separated_list1 | `separated_list0(tag(","), tag("ab"))` | `"ab,ab,ab."` |  | Using the first parser to match separators, returns a `Vec` of zero or more results from the second parser. `separated_list1` does the same operation but must return at least one element |
| multi::fold_many0<br>multi::fold_many1<br>multi::fold_many_m_n | `fold_many0(take(1_u8), Vec::new, \|mut acc, item\| { acc.push(item); acc })` | `"abc"` |  | Applies the parser 0 or more times and folds the list of return values. The `fold_many1` version must apply the parser at least one time, and `fold_many_m_n` must apply the parser at least `m` and at most `n` times |
| multi::length_count | `length_count(nom::character::complete::u8, tag("ab"))` | `"2ababab"` |  | Gets a number from the first parser, then applies the second parser that many times. `number` is a custom defined parser along the lines of text to integer parsers below |

## Combinators to do with completeness

| combinator | usage | input | output | description |
|---|---|---|---|---|
| combinator::all_consuming | `all_consuming(pair(alpha1, number))` | `"abc123"` |  | Returns what the child parser returned if, and only if, the input is exhausted. Otherwise returns an error |
|  | `all_consuming(pair(alpha1, number))` | `"abc123abc"` |  |  |
| combinator::complete | `complete(nom::bytes::streaming::take(5u8))` | `"abcd"` |  | Turns an `Incomplete` result from a streaming parser into an error. The example is the equivalent of `nom::bytes::complete::take(5u8)` |
|  | `nom::bytes::streaming::take(5u8)` | `"abcd"` |  |  |
| combinator::eof | `eof` | `""` |  | Returns an error if the input is not exhausted, otherwise returns the input |
|  | `eof` | `"abc"` |  |  |

### Binary to number


# Fin
