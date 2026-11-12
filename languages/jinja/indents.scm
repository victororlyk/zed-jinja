; Multi-line literals and argument lists inside a tag. Indentation of block
; bodies ({% if %} ... {% endif %}) comes from the indent patterns in
; config.toml, because the grammar does not nest statements.
(arguments
  ")" @end) @indent

(parenthesized
  ")" @end) @indent

(list
  "]" @end) @indent

(dict
  "}" @end) @indent
