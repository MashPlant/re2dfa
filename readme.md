Convert regex(es) to dfa, to help implement the lexing stage of a compiler.

Only limit regex grammar & limited regex usage will be supported. To be more specific, the ultimate purpose of the dfa is to **find the longest match of a group of regexes in a string**.

Here is an example of a simple grammar

```
class -> 0
int -> 1
\d+ -> 2
\s+ -> 3
# actually [] is not supported yet, this is simulated by lots of |
[a-zA-Z][_a-zA-Z0-9]* -> 4
```

The merged dfa gives:

![](./dfa.png)