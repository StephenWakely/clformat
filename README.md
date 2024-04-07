At attempt to recreate the Common Lisp [Format function][hyperspec] in a Rust macro.

# Supported directives

| Directive | Description                                                                                          | Supported     |
|-----------|------------------------------------------------------------------------------------------------------|---------------|
| `~A`      | Prints an argument in a human-readable form. Prints `Display`.                                       | Yes           |
| `~S`      | Prints an argument in a machine-readable form. Quotes strings.                                       | Yes           |
| `~%`      | Inserts a newline character.                                                                         | Yes           |
| `~&`      | Performs a "fresh-line" operation, moving to a new line if not already at the beginning of one.      | No            |
| `~~`      | Prints a tilde (`~`).                                                                                | No            |
| `~D`      | Prints an integer in decimal format.                                                                 | Yes           |
| `~X`      | Prints an integer in hexadecimal format.                                                             | No            |
| `~O`      | Prints an integer in octal format.                                                                   | No            |
| `~B`      | Prints an integer in binary format.                                                                  | No            |
| `~F`      | Prints a floating-point number in fixed-format.                                                      | Partial       |
| `~E`      | Prints a floating-point number in exponential format.                                                | No            |
| `~G`      | Prints a floating-point number in either fixed-format or exponential format, depending on its value. | No            |
| `~C`      | Prints a character.                                                                                  | No            |
| `~P`      | Prints "s" if its argument is plural (i.e., not equal to 1); otherwise, prints nothing.              | No            |
| `~R`      | Prints an integer in English words or as per other specified radix.                                  | No            |
| `~T`      | Inserts horizontal tabulation (space padding) to align output.                                       | No            |
| `~<...~>` | Justifies the enclosed text according to specified parameters.                                       | Yes           |
| `~[...~]` | Conditional expression with multiple clauses for case selection.                                     | No            |
| `~{...~}` | Iterates over a list, applying formatting directives to each element.                                | Yes           |
| `~^`      | Exits the closest enclosing iteration or conditional expression if no more arguments are available.  | Yes           |
| `~*`      | Consumes an argument without printing it. Useful for skipping arguments.                             | Yes           |
| `~I`      | Indents to a specified column, potentially creating new lines if required.                           | No            |
| `~_`      | Conditional newline: inserts a newline character if not at the beginning of a line.                  | No            |
| `~W`      | Prints an argument using "write" semantics, similar to `~S` but with more control over the output.   | No            |
| `~M`      | Prints an integer in Roman numerals.                                                                 | No            |
| `~N`      | Alias for `~%`, inserting a newline. Similar in use to `~%` but rare.                                | No            |
| `~;`      | Separates clauses in conditional expressions (`~[...~]`).                                            | No            |
| `~?`      | Embeds a recursive format operation, allowing a nested format string and arguments.                  | No            |


[hyperspec]: https://www.lispworks.com/documentation/HyperSpec/Body/22_c.htm
