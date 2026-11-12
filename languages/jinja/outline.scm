((statement
  (tag_name) @context
  .
  (identifier) @name) @item
  (#eq? @context "block"))

((statement
  (tag_name) @context
  .
  (call
    (identifier) @name)) @item
  (#eq? @context "macro"))

((statement
  (tag_name) @context
  .
  (string) @name) @item
  (#eq? @context "extends"))
