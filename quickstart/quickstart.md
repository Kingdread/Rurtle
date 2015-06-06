% Rurtle quickstart

What is Rurtle?
===============

Rurtle is an implementation of the concept of turtle graphics in Rust. In turtle
graphics, you draw on a canvas using a "turtle" that has a pen attached. By
commanding the turtle to walk on the canvas, you can draw lines and more complex
pictures.

Rurtle has its own little programming language in which you can command the
turtle.

Installation
============

Make sure that you have the [Rust nightly](http://rust-lang.org) toolchain
installed. After that, just run

```text
git clone https://github.com/Kingdread/Rurtle.git
cd Rurtle
cargo run
```

to run Rurtle. Rurtle will automatically enter the interactive loop so that you
can give it commands.

To exit Rurtle, enter Crtl-D or close the window.

Command line arguments
----------------------

Each argument is interpreted as a filename that will be read loaded upon
startup, example: `cargo run -- my_cool_functions.rtl`. If you just use `cargo
run file.rtl`, the arguments will be interpreted by cargo and not passed to
Rurtle itself.

Your first rectangle
====================

Once you are in the Rurtle prompt, you can start typing in commands. For the
beginning, try this:

```text
forward 100 right 90 forward 100 right 90 forward 100 right 90 forward 100
```

You should now see the turtle moving on the screen, painting a simple square.

`forward` and `right` are functions that the turtle understands, and 100 and 90
are the parameters to those functions. In the case of `forward`, the parameter
gives the amount of steps to move forward. With `right`, the parameter gives the
angle in degrees which the turtle should turn.

The first thing to notice here is that Rurtle does not require parenthesis
to call a function. You can just write down the function's name, followed by its
arguments. The next thing is that you do not need to delimit the function calls,
Rurtle just executes it in a sequence.

You can play around a bit more in the interactive interpreter. If you want to
clear your drawing, use the `clear` function. It takes no argument. You can also
try `backward` and `left`, both with 1 argument each. Given `forward` and
`right`, I guess you can figure out what their argument means and what they do.

Onto the next section...

Our first loop
==============

In the last example, we had to do the same two commands 4 times. Now imagine you
want to create an octagon, you would have to write

```text
forward 100 right 45
forward 100 right 45
forward 100 right 45
forward 100 right 45
forward 100 right 45
forward 100 right 45
forward 100 right 45
forward 100 right 45
```

That's the same line copied 8 times! And what about a 16-gon? The same line 16
times. And what if you want to change the size? You have to change every single
line!

Rurtle has a construct that makes it easier to do the same thing `n` times. It's
called the `repeat`-loop and the syntax is

```text
repeat n do <statements> end
```

Let's try it:

```text
repeat 8 do forward 100 right 45 end
```

And it works, just as we expected! A clearly superior approach to the "copy and
paste"-loop from above.

Teaching the turtle
===================

Still, writing the repeat line every time you want an octagon is... not the best
thing to do. And what if mathematics change and an octagon will have 9 sides in
the future? You'd have to search every "octagon"-line an replace it accordingly.
Wouldn't it be better if you can teach your turtle a new command, like `forward`
is one? Then you'd just have to write `octagon` each time you want an octagon!

Surprise, you can! You can define a new function (that is a new command) via the
`learn` construct:

```text
learn octagon do repeat 8 do forward 100 right 45 end end
```

If you write this into a file, it is better to reformat the code for more
readability:

```text
learn octagon do
    repeat 8 do
        forward 100
        right 45
    end
end
```

The first form still works and you need it in the interactive interpreter, since
it doesn't support multiline input *(yet)*.

Now, if you want an octagon, all you have to do is enter `octagon`.

Fighting forgetfulness
----------------------

Rurtle does not automatically save your functions. If you exit Rurtle and start
it again, it will have forgotten how to draw an octagon. Thus you should put
your important functions in a seperate file, let's call it `functions.rtl`, and
then start Rurtle with `cargo run -- functions.rtl`. It will read and execute
the file, thus learning all the functions.

One size fits all
=================

While we managed to seperate out the octagon into a new function, we have lost
the ability to change the size of the octagon. It will always draw an octagon
with a length of 100 per side. But we already know that we can make commands
that take an argument to tell them how "big" something has to be. Can we do that
with `octagon` too?

Sure, we just need to change the defintion slighty:

```text
learn octagon :size do
    repeat 8 do
        forward :size
        right 45
    end
end
```

Boom! Like `forward`, `octagon` now takes an argument, saying how big the
octagon should be. We can call it like `octagon 100`, `octagon 10` or even
`octagon 10*10`

We can even write a more general `ngon` function now:

```text
learn ngon :n :size do
    repeat :n do
        forward :size
        right 360 / :n
    end
end
```

Now we can get a triangle with `ngon 3 100`, a square with `ngon 4 100`, ...

Variables
=========

The last section introduced function arguments, which are a form of variables.
Variables are just expressions that might change their value during the
execution. In the last section, `:size` was a variable that changed its value
depending on the arguments you called the function with. You can also manually
change the value of variables:

```text
make "i" 5
repeat 20 do
    forward :i
    right 90
    make "i" :i + 5
end
```

This will draw a spiral. You can see how we use the special `make` function to
change the value of a variable. Each iteration of the `repeat`-loop, `i` is
increased and the spiral arm gets a bit longer.

`make` always creates a variable in the current function (or the global scope if
we're not inside a function). If you want to force the variable to be global,
use `global name value`: `global "i" 5`

... our second loop
===================

Always having to specify how many times a loop should run can be hard or even
impossible, that's why there is a second loop, called the `while`-loop. But
before we can understand this kind of loop, we first need to talk about...

Conditionals
------------

Conditionals are expressions that are either true (represented by 1) or false
(represented by 0). Conditionals may be the result of functions or specific
operators, namely the comparison operators. Rurtle has the following comparison
operators:

* < less than
* = equal to
* > greater than
* <= less or equal
* >= greater or equal
* <> unequal/different

A few examples for conditionals:

* 2 < 5
* :i = 10
* 23 >= :i

Back to the loop
----------------

Now that we know conditionals, the `while` loop is fairly easy to understand: It
runs as long as the given conditional evaluates to true

```text
make "i" 0
while :i < 8 do
    forward 100
    right 45
    make "i" :i + 1
end
```

Do you recognize the output? It's the same as our `repeat` example from earlier,
but written in a different form! You can rewrite every `repeat` loop this way.

If-Statements
-------------

We just introduced conditionals, so lets introduce another way to use them: `if`
-statements:

```text
make "i" 0
if :i = 0 do
    forward 100
end
```

Will the code run? Will it run if you change `i` to 1? The difference between a
`while` loop and `if` can be summarised as

* A `while` loop runs as long as the condition is true, i.e. 0 or more times
* An `if` statement runs once if the condition is true, i.e. 0 or 1 time(s)

Lists
=====

So far, we've only worked with numbers, but what if we want more numbers? We
want to write a function that takes a variable count of arguments and for each
argument calls `octagon :size`. But how can we define this function? We can only
give `learn` a fixed count of arguments, so we can't write

```text
alloctagon 100 120 140
alloctagon 160 180
```

But Rurtle saves us! We can use a list. The syntax for a list is simple:

```text
[value1 value2 value3 ...]
```

So we can write

```text
alloctagon [100 120 140]
alloctagon [160 180]
```

Now we just need to find the right definition for `alloctagon`. A list is a
single value, so we start with `learn alloctagon :sizes do ... end`.

To process the list, we use the built-in functions `length` and `getindex`:

```text
learn alloctagon :sizes do
    make "i" 0
    repeat length :sizes do
        octagon getindex :sizes :i
        make "i" :i + 1
    end
end
```

We can oberserve now:

* Many values can be accumulated in a list
* The function `length` can be used to get the length of a list
* With `getindex` we can get a specific element of a list. The first element has
  index 0, the second has index 1, ...
* Function calls can be nested: We use the return value of `getindex` as the
  argument for `octagon`

...but wait, return values?

Reporting back
==============

A function that can do something is useful, but sometimes we don't want to do
something, but we want to calculate something and use that result for something
else. For example, calculating the mean of a list can be done with

```text
make "list" [1 2 3 4 5 6 7]
make "accum" 0
make "i" 0
repeat length :list do
    make "accum" :accum + (getindex :list :i)
    make "i" :i + 1
end
make "mean" :accum / (length :list)
```

and it works, but writing this chunk of code every time you need the mean value
of a list is tedious. Earlier, we could factor out the code into a function to
solve that problem. But this time, we need the value back. Can we still do it?

Yes, functions can return values, as seen with `length` and `getindex`. For
that, just use the `return` keyword:

```text
learn mean :list do
    make "accum" 0
    make "i" 0
    repeat length :list do
        make "accum" :accum + (getindex :list :i)
        make "i" :i + 1
    end
    return :accum / (length :list)
end
```

```text
Rurtle> print mean [1 2 3 4 5 6 7]
4
```

Works!

Dealing with errors
===================

Sometimes a function doesn't work "right" because it is used in the wrong way,
for example accessing an index of a list that isn't even there or converting a
string to a number that doesn't make sense:

```text
Rurtle> getindex [1] 1
runtime error: Index out of bounds: 1 >= 1
Rurtle> tonumber "foo"
runtime error: invalid float literal
```

Such functions aren't defined for all inputs (e.g. `tonumber` is not defined for
every possible string) and those functions need a way to signal "hey, I can't
make sense of this input". One way would be to return a default value, but that
might lead to bugs later. Plus, what is the default value for a number? 0,
because n + 0 = n? Or is it 1, because n * 1 = n?

As you can see, default values aren't the best way to deal with that kind of
error, so functions may throw errors that will stop exection and display an
error message to the user.

If you know beforehand that the function may throw an error, you can prepare for
that case with a `TRY ... ELSE ... END` block:

```text
TRY
    func1
    func2
    func3
ELSE
    func4
    func5
END
```

`func1`, `func2` and `func3` are executed as normal, except when one throws an
error, then execution switches to `func4` and `func5`. This allows for programs
like this:

```text
learn promptnumber :prompt do
    while 1=1 do
        try
            return tonumber prompt :prompt
        else
            print "That was not a number"
        end
    end
end
```

This function keeps asking the user for a number until a valid number literal is
entered:

```text
Rurtle> print promptnumber "Number: "
Number: foo
That was not a number
Number: bar
That was not a number
Number: 3
3
```

Language reference
==================

Arithmetic operations
---------------------

* Number + Number -> Number, standard addition
* String + String -> String, string concatenation
* String + Number -> String, append the number to the string
* List + List -> List, list concatenation
* List + Other -> List, append to list

Subtraction is only defined for Number - Number

* Number * Number -> Number, standard multiplication
* String * Number -> String, replicate the string n times
* List * Number -> List, replicate the list n times

Division is only defined for Number / Number

Comparison operators
--------------------

* `a = b` a equals b
* `a <> b` a is not equal to b
* `a < b` a is less than b
* `a > b` a is greater than b
* `a <= b` a is less or equal to b
* `a >= b` a is greater or equal to b

Drawing functions
-----------------

*forward [amount]*: move the turtle forward by [amount] steps

*backward [amount]*: move the turtle backward by [amount] steps

*left [angle]*: turn the turtle left by [angle] degrees

*right [angle]*: turn the turtle right by [angle] degrees

*color [r] [g] [b]*: set the turtle's color to the given RGB value, where
`0 <= r <= 1, 0 <= g <= 1` and `0 <= 1`

*bgcolor [r] [g] [b]*: set the background color to the given RGB value.

*clear*: clear the screen

*penup*: lift the pen, the turtle will stop drawing until you lower the pen
again

*pendown*: lower the pen again

*home*: go back to the origin

*realign [angle]*: set the turtle's orientation to [angle], where 0 is north, 90
is west, 180 is south and 270 is east.

*hide*: Hide the turtle so it won't show on the screen

*show*: Show the turtle again

Environment functions
---------------------

*make [name] [value]*: set the local variable [name] to [value]

*global [name] [value]*: set the global variable [name] to [value]

*screenshot [filename]*: save a screenshot of the drawing as [filename] \(PNG
format\). **Warning**: This will overwrite [filename] if it exists already! Be
careful!

*prompt [text]*: ask the user for input, displaying the given [text]

*throw [error]*: throw a runtime error with the given text as message

List functions
--------------

*head [list]*: return the first element of the list

*tail [list]*: return everything but the first element of the list

*length [list]*: return the length of the list

*isempty [list]*: return if the list is empty

*getindex [list] [index]*: return the [index]th element of [list]. Note that
indices start at 0, so the first element is `getindex [list] 0`

*find [list] [elem]*: return the index of the first occurence of [elem] in
[list]. If [elem] is not found, return -1 instead.

Boolean functions
-----------------

*not [value]*: return the negated [value]

Type conversion functions
-------------------------

*tonumber [string]*: try to make a number out of the given string

*tostring [value]*: return a string representation of the given value

*nothing*: always return the "nothing" value without doing anything else