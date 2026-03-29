# cutcut

`cutcut` is a CLI for splitting text by arbitrary strings.

Unlike the standard `cut`, the delimiter can be a multi-character string such as `"aaa"`, and you can specify multiple delimiters with repeated `-d`.

## Usage

```bash
cutcut --detail
cutcut -d DELIMITER [-d DELIMITER ...] [-a | -f FIELD [FIELD ...] | -c FIELD [FIELD ...] | --count] [-r REPLACEMENT] [-i PATTERN] [-m COUNT] [-o FILE] [-x|--regex] [TEXT...]
cat file.txt | cutcut -d DELIMITER [-d DELIMITER ...] [-a | -f FIELD [FIELD ...] | -c FIELD [FIELD ...] | --count] [-r REPLACEMENT] [-i PATTERN] [-m COUNT] [-o FILE] [-x|--regex]
```

## Options

- `-d`, `--delimiter`
  Delimiter string. Repeatable. Use `-d " "` or `-d "space"` for whitespace mode.
- `-a`, `--all`
  Explicitly print all fields. This is the default when `-f` is omitted.
- `-f`, `--field`
  Print one or more fields. Positive numbers count from the front. Negative numbers count from the back.
- `-c`, `--component`
  Print one or more positions from the full split stream, not line by line.
- `-r`, `--replace`
  Rejoin fields with this string instead of `, `. Useful for replacement.
- `-i`, `--ignore`
  Ignore lines containing the specified pattern. With `--regex`, ignore lines matching the regex.
- `-m`, `--max-split`
  Split at most `COUNT` times. Negative counts split from the end.
- `-o`, `--output`
  Write output to `FILE` instead of stdout.
- `-x`, `--regex`
  Interpret `-d` and `-i` patterns as regular expressions.
- `--count`
  Print the number of fields after splitting.
- `--detail`
  Show this README content directly from the executable.
- `-h`, `--help`
  Show help.

Long options may be abbreviated as long as the prefix is unique, for example `--del` for `--delimiter`.

Empty fields are kept by default for fixed-string and regex splitting.

## Detailed help from the executable

If you want the full README-style manual from the binary itself, run:

```bash
cutcut --detail
```

Most examples below show both the command and the resulting output.

## Basic examples

Count fields after splitting:

```bash
cutcut -d "/" --count aa/bb/cc
```

Output:

```text
3
```

```bash
printf 'aa/bb/cc\nxx/yy\n' | cutcut -d "/" --count
```

Output:

```text
3
2
```

Split by a multi-character delimiter:

```bash
cutcut -d "aaa" baaabbbaaaccc
```

Output:

```text
b, bbb, ccc
```

Extract the second field:

```bash
cutcut -d "aaa" -f 2 baaabbbaaaccc
```

Output:

```text
bbb
```

Extract multiple fields:

```bash
cutcut -d "/" -f 1 3 -1 aa/bb/cc
```

Output:

```text
aa, cc, cc
```

`-c` selects components from the full split stream:

```bash
cutcut -d "space" -c 1 2 3 "Station[L] : name=YAMAGU34"
```

Output:

```text
Station[L]
:
name=YAMAGU34
```

Use `-f` if you want per-line field selection instead.

If the requested field does not exist, the output is an empty line.

## Negative field index

`-f -1` means the last field, `-f -2` means the second field from the end.

```bash
cutcut -d "/" -f -1 aa/bb/cc
```

Output:

```text
cc
```

```bash
cutcut -d "/" -f -2 aa/bb/cc
```

Output:

```text
bb
```

## Multiple delimiters

Repeat `-d` to split on several strings.

```bash
cutcut -d "," -d "." -d "|" "aa,bb.cc|dd"
```

Output:

```text
aa, bb, cc, dd
```

If multiple delimiters match at the same position, the longest match wins.

```bash
cutcut -d "aa" -d "aaaa" baaaac
```

Output:

```text
b, c
```

## Max split

Use `-m` to limit how many times `cutcut` splits a line.

```bash
cutcut -d "=" -m 1 a=b=c=d
```

Output:

```text
a, b=c=d
```

```bash
cutcut -d "=" -m -1 a=b=c=d
```

Output:

```text
a=b=c, d
```

```bash
cutcut --regex -d "=+" -m 2 "a==b===c====d"
```

Output:

```text
a, b, c====d
```

## Regex mode

With `-x` or `--regex`, `-d` and `-i` are interpreted as regular expressions.

Split on one or more `#` characters:

```bash
cutcut --regex -d "#+" "aa######bb##cc"
```

Output:

```text
aa, bb, cc
```

Split on whitespace with a regex:

```bash
cutcut --regex -d "[[:space:]]+" "aa   bb\tcc"
```

Output:

```text
aa, bb, cc
```

Ignore lines starting with `#`:

```bash
printf '# comment\naa/bb/cc\n' | cutcut --regex -d "/" -f 2 -i "^#"
```

Output:

```text
bb
```

Ignore lines ending with `#`:

```bash
printf 'keep\nskip#\nnext\n' | cutcut --regex -d "space" -a -i "#$"
```

Output:

```text
keep
next
```

Notes for regex mode:

- `^#` matches lines starting with `#`.
- `#$` matches lines ending with `#`.
- `#*` matches the empty string, so it is rejected as a delimiter regex.
- Use `#+` when you mean one or more `#` characters.
- In `--regex` mode, `-d " "` and `-d "space"` are not special aliases. Use a regex such as `[[:space:]]+` instead.

## Replace mode

Without `-f`, `cutcut` prints all fields joined by `, ` by default.

```bash
cutcut -d "/" aa/bb/cc
```

Output:

```text
aa, bb, cc
```

With `-r`, the output separator changes.

```bash
cutcut -d "/" -r ":" aa/bb/cc
```

Output:

```text
aa:bb:cc
```

This makes `cutcut` usable as a simple replacement command:

```bash
cutcut -d "::" -r "/" "aa::bb::cc"
```

Output:

```text
aa/bb/cc
```

If `-f` is used, `-r` has no effect because only one field is printed.

## Output file

Use `-o` to write the result directly to a file.

```bash
cutcut -d "," -f 2 -o result.txt "aa,bb,cc"
```

This writes the following to `result.txt`:

```text
bb
```

```bash
printf 'a=b=c\nx=y=z\n' | cutcut -d "=" -m 1 -o out.txt
```

This writes the following to `out.txt`:

```text
a, b=c
x, y=z
```

## Whitespace mode

`-d " "` and `-d "space"` are special. They mean split on runs of whitespace.

```bash
cutcut -d " " "aa   bb   cc"
cutcut -d "space" "aa   bb   cc"
```

Both commands produce:

```text
aa, bb, cc
```

Tabs are also treated as whitespace in this mode.

```bash
printf 'aa\tbb   cc\n' | cutcut -d "space"
```

Output:

```text
aa, bb, cc
```

Whitespace mode can also be combined with `-f` and `-r`.

```bash
cutcut -d "space" -f 2 "foo   bar   baz"
```

Output:

```text
bar
```

```bash
cutcut -d "space" -r "/" "foo   bar   baz"
```

Output:

```text
foo/bar/baz
```

## Trim behavior

Fields are trimmed by default.

```bash
cutcut -d "," "aa,  bb , cc"
```

Output:

```text
aa, bb, cc
```

This also affects field extraction:

```bash
cutcut -d "," -f 2 "aa,  bb , cc"
```

Output:

```text
bb
```

## Ignore lines

Without `--regex`, `-i PATTERN` ignores lines containing `PATTERN`.

```bash
printf '# comment\naa/bb/cc\nxx # note\nxx/yy/zz\n' | cutcut -d "/" -f 2 -i "#"
```

Output:

```text
bb
yy
```

Typical use:

```bash
cutcut -d "," -i "#"
cutcut -d ":" -i "//"
cutcut -d "space" -i ";"
```

With `--regex`, `-i` behaves more like `grep -v` on a regex:

```bash
printf '# comment\naa/bb/cc\n# skipped\nxx/yy/zz\n' | cutcut --regex -d "/" -f 2 -i "^#"
```

Output:

```text
bb
yy
```

## Standard input

If `TEXT` is omitted, `cutcut` reads from standard input and processes input line by line.

```bash
printf 'aa/bb/cc\nxx/yy/zz\n' | cutcut -d "/" -f 2
```

Output:

```text
bb
yy
```

This means it works naturally with `cat`, `printf`, pipes, and redirected files.

```bash
cat file.txt | cutcut -d "," -f 3
cutcut -d ":" -r "/" < input.txt
```

## Command-line text input

If you pass the text as command-line arguments, put it after all options.

```bash
cutcut -d "aaa" -f 2 baaabbbaaaccc
cutcut -d "/" -r ":" aa/bb/cc
```

Input containing spaces should usually be quoted.

```bash
cutcut -d "space" -f 2 "foo   bar   baz"
cutcut -d "|" "aa|bb|cc"
cutcut -d "aaaa bbbb" "aaaa bbbb zzz"
```

Quotes around `-d` are not always required. They are needed when the delimiter contains spaces or shell-special characters.

Usually safe without quotes:

```bash
cutcut -d aaa baaabbbaaaccc
cutcut -d / -r : aa/bb/cc
cutcut -d , -d . value
```

Usually quote it:

```bash
cutcut -d " " "aa   bb"
cutcut -d "|" "aa|bb|cc"
cutcut -d "*"
cutcut --regex -d "#+" "aa###bb"
```

## `-a` versus default behavior

Without `-f`, all fields are printed by default, so these are equivalent:

```bash
cutcut -d "/" aa/bb/cc
cutcut -d "/" -a aa/bb/cc
```

Both produce:

```text
aa, bb, cc
```

`-a` exists only to make the intent explicit.

`-a` and `-f` cannot be used together.

## Notes

- Empty delimiters are rejected.
- Delimiter regexes that can match the empty string are rejected.
- Empty fields are kept by default for fixed-string and regex splitting.
- Field index `0` is invalid.
- If `TEXT` is passed as multiple bare command-line arguments, `cutcut` joins them with a single space before processing.
- `stdin` is usually the better choice for true multi-line input.
