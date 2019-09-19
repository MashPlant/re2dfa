# Features

There were a lot of other components in this repository, including some benchmarks comparing speed with regex, and a `derive` proc macro that works on `enum`. They are now removed because they are actually never used, and maintaining them takes too much time.

Now only the core components of re2dfa are left. The process of how re2dfa works is shown in `src/lib.rs`, and each component can also be used separately.

The goal of re2dfa is to convert a set of regexes into a dfa that can be used in the implementation of a compiler's lexer. The effect of this dfa is equivalent to: use all the regexes to match the input string successively, select the one with the longest match result as the result; if there are multiple results with the same length, select the first regex in these results.

In addition to this core function, the only remaining feature that has no practical use is to print the graphics of nfa or dfa to a `dot` file.

# Regex

re2dfa supports a subset of regex, here are a few points that fail to meet the regex standards:

1. `{n}`，`{m,n}`，`^`，`$` are not supported. But `{`，`}`，`^`，`$` still need using `\` to escape
2. `()` has no effect on grouping
3. only greedy matching is supported
4. although `\s`，`\d`，`\w` are supported，`\S`，`\D`，`\W` are not
5. `.` match all characters，instead of all characters except `\n`

There is no guarantee that all other standards in regex are properly implemented, either.

# Character set

re2dfa works on bytes (`u8`) only. However, you can still match a character with multiple bytes. It is just a simple concatenation.

However, items like `\s`，`\d`，`\w` are restricted to ascii. And `.` simply match any byte in `0, 1, ..., 255`.