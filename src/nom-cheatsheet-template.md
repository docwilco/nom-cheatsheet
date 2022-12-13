# Nom cheatsheet

This is inspired by [`choosing_a_combinator.md`](https://github.com/Geal/nom/blob/master/doc/choosing_a_combinator.md) in that it collects a bunch of the available things in one page and shows short examples of how each works.

## Quick introduction to Nom

For those new to Nom, all parsers and combinators actually return a function, and said function is what the input is fed to. This is what allows you to combine a bunch of parsers using combinators. This causes syntax that looks slightly odd when you're not used to it. For example, the `char` parser used directly would look like this:

```rust
let (input, my_char) = char('a')(input)?;
```

`input` is a separate column in the examples below, since it's not an argument to the function, but an argument to the result of the function.

The `output` column likewise is the result of calling the resulting function with the input. Feel free to re-read that last sentence a couple of time.

If the parser or combinator succeeded, the result will be an `Ok()` containing a tuple of the remaining input and then the actual result of the parser or combinator. The remaining input is passed back like that so that it can then be used with other parsers or combinators.

If you are writing a function that takes in input and returns a struct, you should write it so that it returns an `IResult` with the remaining input as well. This then allows you to use things like the `many0` combinator with your function to easily get a `Vec` of your custom structs.

```rust

```

## Basic elements

Those are used to recognize the lowest level elements of your grammar, like, "here is a dot", "here is a number", "here is a line ending". These are split up into matching a single byte or character, and matching multiple bytes or characters. 

### Single byte or character parsers

All of these parsers will return a single byte or character.

| combinator | usage | input | output | description |
|---|---|---|---|---|
| character::complete::char<br>character::streaming::char | `char('a')` | `"abc"` |  | Matches one character (works with non ASCII chars too) |
| character::complete::one_of<br>character::streaming::one_of | `one_of("abc")` | `"abc"` |  | Matches one of the provided characters (works with non ASCII chars too) |
| character::complete::none_of<br>character::streaming::none_of | `none_of("abc")` | `"xyab"` |  | Matches anything but the provided characters |
| 

### Sequence of bytes or characters parsers

These parsers will return a slice of bytes or characters. Those suffixed with `0` can return an empty slice if they match nothing. They usually have variants that are suffixed with `1` that will refuse to match unless there's at least 1 byte or character they can match. 

| combinator | usage | input | output | description |
|---|---|---|---|---|
| bytes::complete::is_a<br>bytes::streaming::is_a | `is_a("ab")` | `"ababc"` |  | Matches a sequence of any of the characters passed as arguments |
| bytes::complete::is_not<br>bytes::streaming::is_not | `is_not("cd")` | `"ababc"` |  | Matches a sequence of none of the characters passed as arguments |
| character::complete::alpha0<br>character::streaming::alpha0 | `alpha0` | `"abc123"` |  | Matches zero or more alphabetical ASCII characters (`a-zA-Z`) |
| character::complete::alpha1<br>character::streaming::alpha1 | `alpha1` | `"abc123"` |  | Matches one or more alphabetical ASCII characters (`a-zA-Z`) |
|  | `alpha0` | `"123abc"` |  |  |
|  | `alpha1` | `"123abc"` |  |  |
| character::complete::digit0<br>character::streaming::digit0 |  | `"123abc"` |  | Matches zero or more numerical ASCII characters (`0-9`) |
| bytes::complete::tag<br>bytes::streaming::tag<br>bits::complete::tag<br>bits::streaming::tag | `tag("hello")` | `"hello world"` |  | Recognizes a specific suite of characters, bytes, or bits |
| bytes::complete::tag_no_case<br>bytes::streaming::tag_no_case | `tag_no_case("hello")` | `"HeLLo World"` |  | Recognizes a specific suite of characters, in a case insensitive manner |
| bytes::complete::take<br>bytes::streaming::take<br>bits::complete::take<br>bits::streaming::take | `take(4u8)` | `"hello"` |  | Takes a specific number of characters, bytes, or bits |
| bytes::complete::take_while<br>bytes::streaming::take_while<br>bytes::complete::take_while1<br>bytes::streaming::take_while1 | `take_while(\|c\| c as u8 > 64)` | `"abc123"` |  | Returns the longest consecutive list of bytes for which the provided function returns true. `take_while1` does the same, but must return at least one character |
| bytes::complete::take_while_m_n<br>bytes::streaming::take_while_m_n | `take_while_m_n(4, 5, \|c\| is_alphanumeric(c as u8))` | `"abcd123"` |  | Like `take_while`, but with a minimum and maximum length for the match. |
|  | `take_while_m_n(4, 5, \|c\| is_alphanumeric(c as u8))` | `"abcd-123"` |  |  |
| bytes::complete::take_till<br>bytes::streaming::take_till<br>bytes::complete::take_till1<br>bytes::streaming::take_till1 | `take_till(\|c\| c as u8 <= 64)` | `"abc123"` |  | Returns the longest list of consecutive bytes for which the provided function returns false. `take_till1` does the same, but must return at least one character. Basically `take_till` is the same as `take_while` but with the result of the provided function negated. |
| bytes::complete::take_until<br>bytes::streaming::take_until<br>bytes::complete::take_until1<br>bytes::streaming::take_until1 | `take_until("world")` | `"Hello world"` |  | Returns the longest list of bytes or characters until the provided tag is found. `take_until1` does the same, but must return at least one character |
| bytes::complete::escaped<br>bytes::streaming::escaped | `escaped(digit1, '\\', one_of(r#""n\"#))` | `r#"12\"34"#` |  | XXX: no idea why this is useful |
| bytes::complete::escaped_transform<br>bytes::streaming::escaped_transform | `escaped_transform(alpha1, '\\', value("n", tag("n")))` | `r#"ab\ncd"#` |  | XXX: no idea why this is useful |

## General combinators

| combinator | usage | input | output | description |
|---|---|---|---|---|
| combinator::value | `value(1234, alpha1)` | `"abc789def"` |  | Returns the provided value if the parser succeeds |
| combinator::map | `map(digit1, \|s: &str\| s.parse::<u8>().unwrap())` | `"123abc"` |  | Maps a function on the result of a parser |
| combinator::map_opt | `map_opt(digit1, \|s: &str\| s.parse::<u8>().ok())` | `"123abc"` |  | Same as `map()` but requires the function to return an `Option`. |
| combinator::map_res | `map_res(digit1, \|s: &str\| s.parse::<u8>())` | `"123abc"` |  | Same as `map()` but requires the function to return an `Result`. |
| combinator::flat_map | `flat_map(u8, take)` | `&[2, 90, 91, 92, 93][..]` |  | Apply the first parser, then use its output as the argument for the second parser and apply that to the rest of the input. In this example `u8` reads a single byte as an unsigned integer, then makes that the argument to `take` causing it to read the next 2 bytes |
| combinator::map_parser | `map_parser(take(5u8), digit1)` | `"123abc"` |  | Apply the second parser on the result of the first parser |
| combinator::not | `not(alpha1)` | `"123"` |  | Succeeds if the child parser returns an error |
| combinator::opt | `opt(alpha1)` | `"abc123"` |  | Returns an `Option` of the child parser. `Some()` if the child parser is succesful, and `None` if not |
| combinator::peek | `peek(alpha1)` | `"abc123"` |  | Applies the child parser but does not consume the input |
|  | `alpha1` | `"abc123"` |  |  |
| combinator::recognize | `recognize(separated_pair(alpha1, char(','), alpha1))` | `"abc,def"` |  | Returns a slice of the input consumed by the child parser/combinator. No matter how complex/nested, or whether combinators throw parts away, this will return a single slice with everything that was consumed |
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
| multi::count | `count(take(2u8), 3)` | `"abcdefgh"` |  | Applies the child parser a specified number of timesand returns the list of results in a `Vec` |
| multi::many0<br>multi::many1 | `many0(tag("ab"))` | `"abababc"` |  | Applies the parser 0 or more times and returns the list of results in a `Vec`. `many1` does the same operation but must return at least one element |
| multi::many_m_n | `many_m_n(2, 2, tag("ab"))` | `"ababc"` |  | Applies the parser at least `m` and at most `n` times and returns the list of results in a `Vec` |
| multi::many_till | `many_till(tag("ab"), tag("ef"))` | `"ababefg"` |  | Applies the first parser until the second applies. Returns a tuple containing the list of results from the first in a `Vec` and the result of the second |
| multi::separated_list0<br>multi::separated_list1 | `separated_list0(tag(","), tag("ab"))` | `"ab,ab,ab."` |  | Using the first parser to match separators, returns a `Vec` of zero or more results from the second parser. `separated_list1` does the same operation but must return at least one element |
| multi::fold_many0<br>multi::fold_many1<br>multi::fold_many_m_n | `fold_many0(take(1u8), Vec::new, \|mut acc, item\| { acc.push(item); acc })` | `"abc"` |  | Applies the parser 0 or more times and folds the list of return values. The `fold_many1` version must apply the parser at least one time, and `fold_many_m_n` must apply the parser at least `m` and at most `n` times |
| multi::length_count | `length_count(number, tag("ab"))` | `"2ababab"` |  | Gets a number from the first parser, then applies the second parser that many times. `number` is a custom defined parser along the lines of text to integer parsers below |

## Combinators to do with completeness

| combinator | usage | input | output | description |
|---|---|---|---|---|
| combinator::all_consuming | `all_consuming(pair(alpha1, number))` | `"abc123"` |  | Returns what the child parser returned if, and only if, the input is exhausted. Otherwise returns an error |
|  | `all_consuming(pair(alpha1, number))` | `"abc123abc"` |  |  |
| combinator::complete | `complete(nom::bytes::streaming::take(5u8))` | `"abcd"` |  | Turns an `Incomplete` result from a streaming parser into an error. The example is the equivalent of `nom::bytes::complete::take(5u8)` |
|  | `nom::bytes::streaming::take(5u8)` | `"abcd"` |  |  |
| combinator::eof | `eof` | `""` |  | Returns an error if the input is not exhausted, otherwise returns the input |
|  | `eof` | `"abc"` |  |  |

## Numbers

### Text to number

Nom does not provide helper functions for converting text to integers, so here are some
various ways to get started:

#### Generic integer with optional +/- sign
```rust
fn number<T>(input: &str) -> IResult<&str, T>
where
    T: FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    map(
        recognize(pair(opt(one_of("+-")), digit1)),
        |s: &str| s.parse::<T>().unwrap()
    )(input)
}
```
#### Generic integer with optional - sign
```rust
fn number<T>(input: &str) -> IResult<&str, T>
where
    T: FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    map(
        recognize(pair(opt(tag("-")), digit1)),
        |s: &str| s.parse::<T>().unwrap()
    )(input)
}
```
#### `usize`
```rust
fn number(input: &str) -> IResult<&str, usize> {
    map(digit1, |s: &str| s.parse::<usize>().unwrap())(input)
}
```
#### Provided integer parsers

| combinator | usage | input | output | description |
|---|---|---|---|---|
| number::complete::double<br>number::streaming::double<br>number::complete::float<br>number::streaming::float | `double` | `"123E-02"` |  | Recognizes floating point number in a byte string and returns an `f64`.  `float` does the same for `f32` |
| number::complete::recognize_float<br>number::streaming::recognize_float | `recognize_float` | `"123E-02"` |  | Recognizes floating point number in a byte string and returns the corresponding slice. |
| number::complete::hex_u32<br>number::streaming::hex_u32 | `hex_u32` | `b"abcxyz"` |  | Recognizes hex-encoded `u32` |

### Binary to number

Parsing integers from binary formats can be done in two ways: With parser functions, or combinators with configurable endianness:

| combinator | usage | input | output | description |
|---|---|---|---|---|
| number::complete::i8<br>number::streaming::i8 | `i8` | `&[0xf0][..]` |  | Recognizes a signed integer. Endianness does not matter for single byte numbers, so there's no `nom::Endianness` parameter |
| number::complete::i16<br>number::streaming::i16<br>number::complete::i24<br>number::streaming::i24<br>number::complete::i32<br>number::streaming::i32<br>number::complete::i64<br>number::streaming::i64<br>number::complete::i128<br>number::streaming::i128 | `i16(Endianness::Big)` | `&[0xff, 0x00][..]` |  | Recognizes a signed integer. Various bitsize functions are available. Endianness handled according to parameter |
| number::complete::u8<br>number::streaming::u8 | `u8` | `&[0xf0][..]` |  | Recognizes a unsigned integer. Endianness does not matter for single byte numbers, so there's no `nom::Endianness` parameter |
| number::complete::u16<br>number::streaming::u16<br>number::complete::u24<br>number::streaming::u24<br>number::complete::u32<br>number::streaming::u32<br>number::complete::u64<br>number::streaming::u64<br>number::complete::u128<br>number::streaming::u128 | `u16(Endianness::Big)` | `&[0xff, 0x00][..]` |  | Recognizes a unsigned integer. Various bitsize functions are available. Endianness handled according to parameter |
| number::complete::be_i8<br>number::streaming::be_i8<br>number::complete::be_i16<br>number::streaming::be_i16<br>number::complete::be_i24<br>number::streaming::be_i24<br>number::complete::be_i32<br>number::streaming::be_i32<br>number::complete::be_i64<br>number::streaming::be_i64<br>number::complete::be_i128<br>number::streaming::be_i128 | `be_i16` | `&[0xff, 0xaa][..]` |  | Recognizes a big endian signed integer |
| number::complete::be_u8<br>number::streaming::be_u8<br>number::complete::be_u16<br>number::streaming::be_u16<br>number::complete::be_u24<br>number::streaming::be_u24<br>number::complete::be_u32<br>number::streaming::be_u32<br>number::complete::be_u64<br>number::streaming::be_u64<br>number::complete::be_u128<br>number::streaming::be_u128 | `be_u16` | `&[0xff, 0xaa][..]` |  | Recognizes a big endian unsigned integer |
| number::complete::le_i8<br>number::streaming::le_i8<br>number::complete::le_i16<br>number::streaming::le_i16<br>number::complete::le_i24<br>number::streaming::le_i24<br>number::complete::le_i32<br>number::streaming::le_i32<br>number::complete::le_i64<br>number::streaming::le_i64<br>number::complete::le_i128<br>number::streaming::le_i128 | `le_i16` | `&[0xff, 0xaa][..]` |  | Recognizes a big endian signed integer |
| number::complete::le_u8<br>number::streaming::le_u8<br>number::complete::le_u16<br>number::streaming::le_u16<br>number::complete::le_u24<br>number::streaming::le_u24<br>number::complete::le_u32<br>number::streaming::le_u32<br>number::complete::le_u64<br>number::streaming::le_u64<br>number::complete::le_u128<br>number::streaming::le_u128 | `le_u16` | `&[0xff, 0xaa][..]` |  | Recognizes a big endian unsigned integer |
| number::complete::be_f32<br>number::streaming::be_f32<br>number::complete::be_f64<br>number::streaming::be_f64 | `be_f32` | `&[0x41, 0x48, 0x00, 0x00][..]` |  | Recognizes a big endian floating point number |
| number::complete::le_f32<br>number::streaming::le_f32<br>number::complete::le_f64<br>number::streaming::le_f64 | `le_f32` | `&[0x00, 0x00, 0x48, 0x41][..]` |  | Recognizes a big endian floating point number |

# Fin
