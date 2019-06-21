Convert regex(es) to dfa, to help implement the lexing stage of a compiler.

Only limit regex grammar & limited regex usage will be supported. To be more specific, the ultimate purpose of the dfa is to **find the longest match of a group of regexes in a string**.

## Benchmarks
Tokenize a decaf file of about 350 lines:

```
test decaf_dfa    ... bench:      42,064 ns/iter (+/- 310)
test decaf_re     ... bench:   3,203,889 ns/iter (+/- 24,237)
test decaf_re_set ... bench:     791,404 ns/iter (+/- 4,057)
```

're' brutally uses every re to try to match the string and get longest match; 're_set' only uses possible candidates indicated by RegexSet.

See folder 'benches' for more detail.

## #[derive(Dfa)] 
Add this to an enum, and proc macro will automatically generate a type named XXXLexer, where XXX is the enum name.

This is mostly inspired by Logos, but I am not good at using proc macro now, so the code may seem silly.

## What does dfa looks like

Here is an example of a simple grammar

```
class -> 0
int -> 1
\d+ -> 2
\s+ -> 3
[a-zA-Z][_a-zA-Z0-9]* -> 4
```

The merged dfa gives:

![](./dfa.png)

And another example: the lexer for the [decaf](https://github.com/MashPlant/decaf) language:

![](./decaf.png)