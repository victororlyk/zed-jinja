# Jinja for Zed

Jinja2 template support for the [Zed](https://zed.dev) editor: highlighting
that keeps working around quotes, comments, attributes and half-typed tags,
plus [jinja-lsp](https://github.com/uros-5/jinja-lsp).

Written by Viktor Orlyk.

## How it works

The extension ships its own tree-sitter grammar, in
[tree-sitter-jinja](tree-sitter-jinja). It only looks at what is inside
`{{ }}`, `{% %}` and `{# #}`. Everything else is handed to Zed's HTML grammar
as a single document, the way Zed's ERB support does it. Because the HTML
parser never sees the Jinja parts, a quote in a comment or an expression in
the middle of an attribute cannot throw it off.

The grammar is deliberately lenient. It tokenises tags rather than validating
them, so there is no template it fails on:

- `{% if %}` and `{% endif %}` are independent tags. A block that is not
  closed yet is not an error.
- Any word after `{%` is a tag name, which covers tags added by Jinja
  extensions.
- Any sequence of tokens inside a tag is accepted.
- A tag that is still open when the next `{{`, `{%` or `{#` begins ends
  there, so a tag you are in the middle of typing does not take the rest of
  the file with it.

## Install

The extension is not in the Zed registry yet, so install it as a dev extension.

1. Install Rust with [rustup](https://rustup.rs). Zed uses it to compile the
   extension.
2. Clone this repository.
3. Uninstall any other Jinja extension. Two extensions that claim the same
   file suffixes conflict.
4. Make sure Zed's *HTML* extension is installed. It provides the grammar for
   everything outside the Jinja tags.
5. In Zed, open the command palette and run `zed: install dev extension`, then
   pick the cloned folder.

After pulling changes or editing the extension, run `zed: extensions`, find
*Jinja* under the installed extensions and click **Rebuild**. `zed: open log`
shows build and grammar errors.

## File types

These suffixes open as Jinja: `.jinja`, `.jinja2`, `.j2`, `.html.j2`,
`.html.jinja`.

Plain `.html` is left alone on purpose. If your project keeps its templates in
`.html` files, map them in your Zed settings, either globally or in the
project's `.zed/settings.json`:

```json
{
  "file_types": {
    "Jinja": ["**/templates/**/*.html"]
  }
}
```

The leading `**/` lets the pattern match a `templates` folder at any depth.

## Features

- **Highlighting**
  - every statement keyword: `if` / `elif` / `else` / `endif`, `for` /
    `endfor`, `block`, `extends`, `include`, `import`, `from`, `macro`, `call`,
    `set`, `with`, `filter`, `do`, `raw`, `autoescape`, `trans`, and any tag an
    extension adds
  - filters after `|` and tests after `is` / `is not`
  - strings, numbers, booleans, `none`, operators, attribute access, calls,
    keyword arguments, macro parameters, block names
  - `{# comments #}`, whatever they contain
- **HTML, CSS and JavaScript** highlighting in the rest of the template,
  including inside `{% raw %}` blocks
- **Bracket matching** for `{{ }}`, `{% %}` and the brackets inside them
- **Outline** entries for blocks, macros and `extends`
- **Auto-indent** after a tag that opens a block, and back out on `{% end… %}`,
  `{% else %}` and `{% elif %}`
- **Block comments** with `{# #}`
- **jinja-lsp**: completion, hover, go to definition, diagnostics

### Making Jinja tags stand out

Delimiters use their own capture, `punctuation.special.jinja`. Themes colour it
like `punctuation.special`, which is already different from HTML's angle
brackets and tag names. To pick your own colour for the Jinja delimiters only,
override that capture for your theme:

```json
{
  "theme_overrides": {
    "One Dark": {
      "syntax": {
        "punctuation.special.jinja": { "color": "#e5c07b", "font_weight": 700 }
      }
    }
  }
}
```

`function.filter`, `function.test` and `keyword.modifier` (words such as
`recursive`, `as` and `with context`) can be overridden the same way. Without
an override they take the theme's `function` and `keyword` colours.

## Language server

The extension starts `jinja-lsp` for Jinja files. It looks for the binary in
this order:

1. `lsp.jinja-lsp.binary.path` in your settings
2. `jinja-lsp` on your `PATH` (for example after `cargo install jinja-lsp`)
3. the latest release binary for your platform, downloaded from GitHub

jinja-lsp wants to know where your templates and backend code live:

```json
{
  "lsp": {
    "jinja-lsp": {
      "initialization_options": {
        "templates": "./templates",
        "backend": ["./src"],
        "lang": "python"
      }
    }
  }
}
```

`lang` is `"python"` or `"rust"`. See the
[jinja-lsp README](https://github.com/uros-5/jinja-lsp#configuration) for the
other options.

## Known limits

- **Opening and closing tags are not paired.** Since statements are flat, the
  editor cannot jump between `{% if %}` and `{% endif %}`.
- **It does not report mistakes.** A misspelled tag or a broken expression is
  highlighted like any other. Diagnostics are jinja-lsp's job.
- **An unfinished tag reads on to the next tag.** Until you close it, the HTML
  between it and the following `{{`, `{%` or `{#` is tokenised as Jinja.
  Likewise an unterminated string runs to the next matching quote.
- **Contextual keywords** such as `recursive`, `as`, `import`, `ignore
  missing` and `with context` are recognised only directly inside their own
  tag, not inside brackets.
- **Auto-indent works line by line.** A block opened and closed on the same
  line, or a tag that spans several lines, is not taken into account.
- **Line statements and line comments** (`# for item in items`, `## note`)
  are treated as template text. They are disabled in Jinja unless configured.
- **Templates that are not HTML.** `.j2` is also used for config files and
  other text. Their content is still parsed as HTML, which is mostly harmless
  but not meaningful.
- **Editing inside HTML.** Zed takes comment and auto-close rules from the
  language under the cursor, so in the HTML parts of a template you get HTML's.
- **jinja-lsp** targets Jinja used from Rust (minijinja) and Python. Without
  the settings above it has little to offer.

## Development

`tests/fixtures` holds one template per highlighting bug reported against
earlier Jinja extensions, plus files that exercise the rest:

| Fixture | What to look for |
| --- | --- |
| `01-comment-quote.html.j2` | Quotes inside `{# #}` do not affect the lines after |
| `02-attribute-expression.html.j2` | `{{ }}` inside attributes; the HTML around it stays intact |
| `03-do-keyword.html.j2` | `do` is a keyword |
| `04-if-statement.html.j2` | `if` / `elif` / `else` / `endif`, inline `if`, tests |
| `05-jinja-vs-html-tags.html.j2` | Jinja tags are coloured differently from HTML tags |
| `06-all-keywords.html.j2` | Every keyword, filter, test and literal |
| `07-hash-text.html.j2` | `##` and `# if` in text stay HTML |
| `08-less-common-syntax.html.j2` | Slices, `*args`, `endblock name`, extension tags |
| `09-unfinished-tags.html.j2` | Every `ok_N` variable stays highlighted below a broken tag |

To run the grammar's tests and parse every fixture:

```bash
tests/check-fixtures.sh
```

It needs a C compiler and the tree-sitter CLI
(`npm install -g tree-sitter-cli`).

### Changing the grammar

1. Edit `tree-sitter-jinja/grammar.js` or `tree-sitter-jinja/src/scanner.c`.
2. In `tree-sitter-jinja`, run `tree-sitter generate --abi 14`, then
   `tree-sitter test`. Add cases to `test/corpus` for what you changed.
3. Commit, including the regenerated files in `src`.
4. Put that commit's hash in `[grammars.jinja]` in `extension.toml` and commit
   again. Zed builds the grammar from the pinned commit, not from the working
   tree.
5. Rebuild the extension in Zed.

## Credits

- Language server: [uros-5/jinja-lsp](https://github.com/uros-5/jinja-lsp)

## License

[MIT](LICENSE) © Viktor Orlyk
