Convert regex(es) to dfa, to help implement the lexing stage of a compiler.

Only limit regex grammar & limited regex usage will be supported. To be more specific, the ultimate purpose of the dfa is to **find the longest match of a group of regexes in a string**.

## Benchmarks
Tokenize a decaf file of about 300 lines:

```
test decaf_dfa    ... bench:      33,487 ns/iter (+/- 238)
test decaf_re     ... bench:   2,560,722 ns/iter (+/- 6,705)
test decaf_re_set ... bench:     608,057 ns/iter (+/- 1,523)
```

're' brutally uses every re to try to match the string and get longest match; 're_set' only uses possible candidates indicated by RegexSet.

See folder 'benches' for more detail.

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