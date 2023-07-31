# RRegex

To build the regex library, run `wasm-pack build`.

Run the `www/` Next.js project by running `yarn dev` from inside the directory,
after building the library.

## Operators

Three operators are currently supported:

1.  **Concatenation:** Just writing two regexes one after the other means they are
    concatenated. Eg. "ab" is the concatenation of "a" and "b".
1.  **Union:** The pipe operator '|' means either one of the regexes should match
    the string. Eg. "a|b" is the union of "a" and "b".
1.  **Kleene star:** The star operator '\*' means zero or more repetitions of
    the regex. Eg. "a\*" is the repetition of "a".

## Examples

Sample images for some regular expressions are stored in the `examples/`
directory.
