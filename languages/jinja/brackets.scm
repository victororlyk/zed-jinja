; {{ expression }}
(output
  [
    "{{"
    "{{-"
    "{{+"
  ] @open
  [
    "}}"
    "-}}"
    "+}}"
  ] @close)

; {% statement %}
(statement
  [
    "{%"
    "{%-"
    "{%+"
  ] @open
  [
    "%}"
    "-%}"
    "+%}"
  ] @close)

; Inside expressions
("(" @open
  ")" @close)

("[" @open
  "]" @close)

("{" @open
  "}" @close)
