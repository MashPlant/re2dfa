Convert regex(es) to dfa, to help implement the lexing stage of a compiler.

Only limit regex grammar & limited regex usage will be supported. To be more specific, the ultimate purpose of the dfa is to **find the longest match of a group of regexes in a string**.

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

Well this may be somewhat too big for a naive codegen, so a future work is to classify input chars into different symbols before feeding them into dfa. The constraint is 

```
If 2 char can lead a node go to 2 different nodes, the cannot be mapped to a same symbol.
```

And the criterion can be

```
1. Minimize the number of the kinds of symbols, and we get a graph coloring problem.
2. Minimize code volume, that is, minimize the sum of the edges of every nodes. WTF this is?
```