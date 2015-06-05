Rurtle
======

![Screenshot](/Screenshot.png?raw=true)

Rurtle is an implementation of the [turtle graphics][tgraphics] done in
Rust. Rurtle has its own little language, inspired by [Berkeley Logo][ucblogo]
and [Win Logo][winlogo]

Requirements
------------

Rust nightly (Rurtle uses unstable features).

The Rurtle shaders require GLSL version 120 and thus at least OpenGL 2.

Running
-------

Use `cargo run -- [file1 file2 ...]` to run Rurtle. Each file specified (if any)
will be read and interpreted. After reading and executing all files, Rurtle will
enter the REPL. You can enter any command there and play interactively with
Rurtle.

Syntax
------

* Comments: `print 1 ; comment starts here`
* String literals: `"Hello World"`
* Number literals: `13`, `3.1415`, `-42`
* Lists: `[3 4 5 6]`
* Variables: `make "i" 0`, `print :i`
* Function calls: `print 1 + 3`, `color 0.2 0.4 0.6`
* Function definitions: `learn add :a :b do return :a + :b end`
* Conditionals: `if :i = 2 do print "Two" else print "Not two" end`
* `repeat`-loop: `repeat 4 do forward 100 right 90 end`
* `while`-loop: `while :i <> 0 do make "i" :i - 1 end`
* Error handling: `try tonumber "foo" else print "not a number" end`

Loops can be nested and may have arbirary many statements in their bodies:

    repeat 4 do
        repeat 4 do
            forward 100
            right 90
        end
        right 90
    end

Functions:

    forward <i>, backward <i>, right <i>, left <i>, color <r> <g> <b>, print <e>
    penup, pendown, home, clear, realign <a>

For a complete list, see `src/environ/functions/mod.rs`.

Documentation/Language reference
--------------------------------

For documentation of the modules, see [here][docs].

For an overview of the language and the available functions, look in
`quickstart/`. The [quickstart guide](/quickstart/quickstart.md) is written in
Markdown and can be viewed on GitHub or locally converted to HTML by running
`make` in the quickstart directory.

License
-------

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.

**Disclaimer**: This program *may* eat your hamster, watch out.

[tgraphics]: https://en.wikipedia.org/wiki/Turtle_graphics
[ucblogo]: https://en.wikipedia.org/wiki/UCBLogo
[winlogo]: http://www.win-logo.de/
[docs]: http://kingdread.de/rust/rurtle/