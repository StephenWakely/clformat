At attempt to recreate the Common Lisp [Format function][hyperspec] in a Rust macro.

# Supported directives

| Directive | Description                                                                                          | Supported     |
|-----------|------------------------------------------------------------------------------------------------------|---------------|
| `~A`      | Prints an argument in a human-readable form. Prints `Display`.                                       | [x]           |
| `~S`      | Prints an argument in a machine-readable form. Quotes strings.                                       | [x]           |
| `~%`      | Inserts a newline character.                                                                         | [x]           |
| `~&`      | Performs a "fresh-line" operation, moving to a new line if not already at the beginning of one.      | [ ]           |
| `~~`      | Prints a tilde (`~`).                                                                                | [ ]           |
| `~D`      | Prints an integer in decimal format.                                                                 | [x]           |
| `~X`      | Prints an integer in hexadecimal format.                                                             | [ ]           |
| `~O`      | Prints an integer in octal format.                                                                   | [ ]           |
| `~B`      | Prints an integer in binary format.                                                                  | [ ]           |
| `~F`      | Prints a floating-point number in fixed-format.                                                      | [x] (partial) |
| `~E`      | Prints a floating-point number in exponential format.                                                | [ ]           |
| `~G`      | Prints a floating-point number in either fixed-format or exponential format, depending on its value. | [ ]           |
| `~C`      | Prints a character.                                                                                  | [ ]           |
| `~P`      | Prints "s" if its argument is plural (i.e., not equal to 1); otherwise, prints nothing.              | [ ]           |
| `~R`      | Prints an integer in English words or as per other specified radix.                                  | [ ]           |
| `~T`      | Inserts horizontal tabulation (space padding) to align output.                                       | [ ]           |
| `~<...~>` | Justifies the enclosed text according to specified parameters.                                       | [x]           |
| `~[...~]` | Conditional expression with multiple clauses for case selection.                                     | [ ]           |
| `~{...~}` | Iterates over a list, applying formatting directives to each element.                                | [x]           |
| `~^`      | Exits the closest enclosing iteration or conditional expression if no more arguments are available.  | [x]           |
| `~|`      | Produces vertical tabulation (page break in some contexts).                                          | [ ]           |
| `~*`      | Consumes an argument without printing it. Useful for skipping arguments.                             | [x]           |
| `~I`      | Indents to a specified column, potentially creating new lines if required.                           | [ ]           |
| `~_`      | Conditional newline: inserts a newline character if not at the beginning of a line.                  | [ ]           |
| `~W`      | Prints an argument using "write" semantics, similar to `~S` but with more control over the output.   | [ ]           |
| `~M`      | Prints an integer in Roman numerals.                                                                 | [ ]           |
| `~N`      | Alias for `~%`, inserting a newline. Similar in use to `~%` but rare.                                | [ ]           |
| `~;`      | Separates clauses in conditional expressions (`~[...~]`).                                            | [ ]           |
| `~?`      | Embeds a recursive format operation, allowing a nested format string and arguments.                  | [ ]           |


[hyperspec]: https://www.lispworks.com/documentation/HyperSpec/Body/22_c.htm
