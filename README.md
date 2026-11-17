At attempt to recreate the Common Lisp [Format function][hyperspec] in a Rust macro.

# Supported directives
[hyperspec]: https://www.lispworks.com/documentation/HyperSpec/Body/22_c.htm

| Directive | Description                                                                                          | Supported     |
|-----------|------------------------------------------------------------------------------------------------------|---------------|
| `~A`      | Prints an argument in a human-readable form. Prints `Display`.                                       | Yes           |
| `~S`      | Prints an argument in a machine-readable form. Quotes strings.                                       | Yes           |
| `~%`      | Inserts a newline character.                                                                         | Yes           |
| `~&`      | Performs a "fresh-line" operation, moving to a new line if not already at the beginning of one.      | Yes           |
| `~~`      | Prints a tilde (`~`).                                                                                | Yes           |
| `~D`      | Prints an integer in decimal format.                                                                 | Yes           |
| `~X`      | Prints an integer in hexadecimal format.                                                             | Yes           |
| `~O`      | Prints an integer in octal format.                                                                   | Yes           |
| `~B`      | Prints an integer in binary format.                                                                  | Yes           |
| `~F`      | Prints a floating-point number in fixed-format.                                                      | Partial       |
| `~E`      | Prints a floating-point number in exponential format.                                                | Partial       |
| `~G`      | Prints a floating-point number in either fixed-format or exponential format, depending on its value. | Partial       |
| `~C`      | Prints a character.                                                                                  | Yes           |
| `~P`      | Prints "s" if its argument is plural (i.e., not equal to 1); otherwise, prints nothing.              | Yes           |
| `~R`      | Prints an integer in English words or as per other specified radix.                                  | Yes           |
| `~T`      | Inserts horizontal tabulation (space padding) to align output.                                       | Partial       |
| `~<...~>` | Justifies the enclosed text according to specified parameters.                                       | Yes           |
| `~[...~]` | Conditional expression with multiple clauses for case selection.                                     | Yes           |
| `~{...~}` | Iterates over a list, applying formatting directives to each element.                                | Yes           |
| `~^`      | Exits the closest enclosing iteration or conditional expression if no more arguments are available.  | Yes           |
| `~*`      | Consumes an argument without printing it. Useful for skipping arguments.                             | Yes           |
| `~I`      | Indents to a specified column, potentially creating new lines if required.                           | No            |
| `~_`      | Conditional newline: inserts a newline character if not at the beginning of a line.                  | Partial       |
| `~W`      | Prints an argument using "write" semantics, similar to `~S` but with more control over the output.   | Partial       |
| `~M`      | Prints an integer in Roman numerals.                                                                 | Yes           |
| `~N`      | Alias for `~%`, inserting a newline. Similar in use to `~%` but rare.                                | Yes           |
| `~;`      | Separates clauses in conditional expressions (`~[...~]`).                                            | Yes           |
| `~?`      | Embeds a recursive format operation, allowing a nested format string and arguments.                  | No            |

## Notes on partial/unsupported directives

A few directives are implemented pragmatically rather than to the letter of
the HyperSpec, and a couple are not implemented at all because they don't fit
this crate's architecture. Specifics:

- **`~F`** - the `d` (num digits), `k` (scale factor) and overflow-character
  parameters are not supported.
- **`~E`** - implemented via Rust's native exponential (`{:e}`) formatting.
  This gets you a correctly rounded mantissa and exponent, but does not
  implement the `k` (scale factor) or overflow-character parameters.
- **`~G`** - chooses between fixed and exponential format using a heuristic
  similar to C's `%g`, rather than the exact HyperSpec algorithm. The `k`
  (scale factor) and overflow-character parameters are not supported.
- **`~T`** - column tracking works for the `nil` (`String`) and `t`
  (stdout) output forms, and for a custom writer, but does not account for
  multi-byte/wide characters (columns are counted in bytes, matching the rest
  of the crate). The colon ("section relative") modifier is accepted but
  treated the same as the default case.
- **`~_`** - approximated as a fresh-line (`~&`): a newline is inserted if
  not already at the start of a line. This crate does not implement the
  Lisp pretty-printer's logical blocks, so the full "conditional newline"
  semantics (which depend on whether the enclosing block fits on the current
  line) don't apply.
- **`~W`** - implemented as an alias for `~S` (prints with `Debug`). The
  HyperSpec's `~W` additionally respects dynamic printer-control variables
  (`*print-pretty*`, `*print-length*`, etc.) that Rust's `Debug` has no
  equivalent for.
- **`~I`** - not implemented. Like `~_`, this is a pretty-printer directive
  that only makes sense inside a logical block (`~<...~:>`), which this crate
  does not implement.
- **`~?`** - not implemented. This directive takes a *runtime* control
  string and recursively formats it, but `clformat!` parses its control
  string entirely at compile time (that's how it validates the directives
  and argument count ahead of time, and how it avoids pulling in a runtime
  interpreter). Supporting `~?` properly would require a second, fully
  general runtime formatter, which is a different project.

[hyperspec]: https://www.lispworks.com/documentation/HyperSpec/Body/22_c.htm
