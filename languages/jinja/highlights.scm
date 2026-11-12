; Zed gives a node the style of the LAST pattern that captures it, so general
; patterns come first and more specific ones follow.

; Variables
(identifier) @variable

((identifier) @variable.special
  (#any-of? @variable.special "loop" "self" "super" "caller" "varargs" "kwargs"))

; Attribute access: user.name
(attribute
  (identifier) @property)

; Calls: name(...), obj.method(...)
(call
  (identifier) @function)

; name=value inside a call
(keyword_argument
  (identifier) @variable.parameter)

; Literals
(string) @string

(number) @number

(boolean) @boolean

(none) @constant.builtin

(comment) @comment

; Punctuation
[
  ","
  ":"
  ";"
  "."
] @punctuation.delimiter

[
  "("
  ")"
  "["
  "]"
  "{"
  "}"
] @punctuation.bracket

; Operators
(operator) @operator

[
  "|"
  "="
] @operator

[
  "and"
  "or"
  "not"
  "in"
  "is"
] @keyword.operator

; Inline `a if b else c`, and the filter in `{% for x in y if z %}`
[
  "if"
  "else"
] @keyword

; Filters: the name after `|`
(filter
  (identifier) @function.filter)

(filter
  (call
    (identifier) @function.filter))

; Tests: the name after `is` or `is not`
(test
  (identifier) @function.test)

(test
  (call
    (identifier) @function.test))

; Statement keywords. The word after `{%` is always the tag name, so tags
; added by extensions are covered too.
(tag_name) @keyword

; Words that are keywords only inside a particular tag
((statement
  (tag_name) @keyword
  (identifier) @keyword.modifier)
  (#eq? @keyword "for")
  (#eq? @keyword.modifier "recursive"))

((statement
  (tag_name) @keyword
  (identifier) @keyword.modifier)
  (#eq? @keyword "block")
  (#any-of? @keyword.modifier "scoped" "required"))

((statement
  (tag_name) @keyword
  (identifier) @keyword.modifier)
  (#eq? @keyword "include")
  (#any-of? @keyword.modifier "ignore" "missing" "with" "without" "context"))

((statement
  (tag_name) @keyword
  (identifier) @keyword.modifier)
  (#eq? @keyword "import")
  (#any-of? @keyword.modifier "as" "with" "without" "context"))

((statement
  (tag_name) @keyword
  (identifier) @keyword.modifier)
  (#eq? @keyword "from")
  (#any-of? @keyword.modifier "import" "as" "with" "without" "context"))

((statement
  (tag_name) @keyword
  (identifier) @keyword.modifier)
  (#eq? @keyword "trans")
  (#any-of? @keyword.modifier "trimmed" "notrimmed"))

; {% macro name(param, other=default) %}
((statement
  (tag_name) @keyword
  .
  (call
    (identifier) @function.definition))
  (#eq? @keyword "macro"))

((statement
  (tag_name) @keyword
  .
  (call
    (arguments
      (identifier) @variable.parameter)))
  (#eq? @keyword "macro"))

; {% call(param) name() %}
((statement
  (tag_name) @keyword
  .
  (parenthesized
    (identifier) @variable.parameter))
  (#eq? @keyword "call"))

; {% block name %} and {% endblock name %}
((statement
  (tag_name) @keyword
  .
  (identifier) @label)
  (#any-of? @keyword "block" "endblock"))

; {% filter name %}
((statement
  (tag_name) @keyword
  .
  [
    (identifier) @function.filter
    (call
      (identifier) @function.filter)
  ])
  (#eq? @keyword "filter"))

; Template paths
((statement
  (tag_name) @keyword
  .
  (string) @string.special)
  (#any-of? @keyword "extends" "include" "import" "from"))

; Delimiters get their own scope so Jinja tags do not read like HTML tags.
[
  "{{"
  "{{-"
  "{{+"
  "}}"
  "-}}"
  "+}}"
  "{%"
  "{%-"
  "{%+"
  "%}"
  "-%}"
  "+%}"
] @punctuation.special.jinja
