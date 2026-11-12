; Everything outside Jinja delimiters is HTML. All fragments are parsed as one
; combined document, so an expression in the middle of a tag or attribute
; (`<a href="{{ url }}">`) does not break the surrounding HTML.
;
; `raw_content` is the text between {% raw %} and {% endraw %}.
([
  (content)
  (raw_content)
] @injection.content
  (#set! injection.language "html")
  (#set! injection.combined))
