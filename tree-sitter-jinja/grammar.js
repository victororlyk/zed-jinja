/**
 * @file A lenient Jinja grammar for editors.
 * @author Viktor Orlyk
 * @license MIT
 *
 * The grammar finds the Jinja tags in a template and tokenises what is inside
 * them. It does not validate templates:
 *
 * - Statements are flat. `{% if %}` and `{% endif %}` are two independent
 *   nodes, so an unfinished block is not an error.
 * - Any sequence of tokens inside a tag is accepted, and any word after `{%`
 *   is a tag name, so tags added by extensions parse too.
 * - A tag that is still open when the next `{{`, `{%` or `{#` starts (or the
 *   file ends) is closed there. Those sequences cannot occur inside a valid
 *   expression, and this keeps a tag that is being typed from swallowing the
 *   rest of the file.
 *
 * Everything outside the tags is a `content` node, meant to be handed to
 * another grammar through an injection.
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

// Words with a meaning inside expressions. They are also valid tag names
// (`{% if %}`, `{% else %}`), where they are aliased to `tag_name`.
const EXPRESSION_KEYWORDS = ['and', 'or', 'not', 'in', 'is', 'if', 'else'];
const BOOLEANS = ['true', 'false', 'True', 'False'];
const NONES = ['none', 'None'];

module.exports = grammar({
  name: 'jinja',

  externals: $ => [
    $.content,
    $.comment,
    $.raw_content,
    $._implicit_close,
    $._error_sentinel,
  ],

  extras: _ => [/\s/],

  word: $ => $.identifier,

  rules: {
    template: $ => repeat($._node),

    _node: $ => choice($.content, $.comment, $.output, $.statement, $._raw),

    // {{ expression }}
    output: $ =>
      seq(
        choice('{{', '{{-', '{{+'),
        repeat($._top_item),
        choice('}}', '-}}', '+}}', $._implicit_close),
      ),

    // {% tag ... %}
    statement: $ =>
      choice(
        seq(
          $._statement_open,
          optional(
            choice(
              seq($._tag_word, repeat($._top_item)),
              seq($._nonword_top_item, repeat($._top_item)),
              // `raw` with anything after it does not start a raw block
              seq(alias('raw', $.tag_name), repeat1($._top_item)),
            ),
          ),
          choice($._statement_close, $._implicit_close),
        ),
        // Neither does an unfinished `{% raw`
        seq($._statement_open, alias('raw', $.tag_name), $._implicit_close),
      ),

    // {% raw %} followed by text that is not parsed, up to {% endraw %}.
    // The closing tag is an ordinary statement.
    _raw: $ =>
      prec(1, seq(alias($._raw_open, $.statement), optional($.raw_content))),

    _raw_open: $ =>
      seq($._statement_open, alias('raw', $.tag_name), $._statement_close),

    _statement_open: _ => choice('{%', '{%-', '{%+'),
    _statement_close: _ => choice('%}', '-%}', '+%}'),

    _tag_word: $ =>
      choice(
        alias($.identifier, $.tag_name),
        ...[...EXPRESSION_KEYWORDS, ...BOOLEANS, ...NONES].map(word =>
          alias(word, $.tag_name),
        ),
      ),

    // Directly inside a tag, closing brackets with no opener are tolerated.
    _top_item: $ => choice($._item, ')', ']', '}'),
    _nonword_top_item: $ => choice($._nonword_item, ')', ']', '}'),

    _item: $ => choice($._word_item, $._nonword_item),

    _word_item: $ =>
      choice(
        $.identifier,
        $.call,
        $.test,
        $.boolean,
        $.none,
        'and',
        'or',
        'not',
        'in',
        'if',
        'else',
        // `is` that is not followed by a test name
        'is',
        seq('is', 'not'),
      ),

    _nonword_item: $ =>
      choice(
        $.string,
        $.number,
        $.operator,
        $.attribute,
        $.filter,
        $.parenthesized,
        $.list,
        $.dict,
        ',',
        ':',
        ';',
        '.',
        '|',
        $.unknown,
      ),

    // name(...)
    call: $ => prec(3, seq($.identifier, $.arguments)),

    // .name and .name(...)
    attribute: $ => prec(2, seq('.', choice($.identifier, $.call))),

    // | name and | name(...)
    filter: $ => prec(2, seq('|', choice($.identifier, $.call))),

    // is name, is not name, is name(...)
    test: $ =>
      prec(
        2,
        seq(
          'is',
          optional('not'),
          choice($.identifier, $.call, $.boolean, $.none),
        ),
      ),

    arguments: $ =>
      prec.right(
        seq('(', repeat(choice($._item, $.keyword_argument)), optional(')')),
      ),

    // The `name =` of `name=value`
    keyword_argument: $ => prec(2, seq($.identifier, '=')),

    // Closing brackets are optional so that an expression that is still being
    // typed does not turn into an error. Right associativity keeps the items
    // that follow an opening bracket inside it.
    parenthesized: $ => prec.right(seq('(', repeat($._item), optional(')'))),

    list: $ => prec.right(seq('[', repeat($._item), optional(']'))),

    // `}` must stay required: if a dict could end anywhere, `}}` would be
    // read as the end of the tag while still inside nested dicts.
    dict: $ =>
      seq(
        '{',
        repeat(choice($._item, ')', ']')),
        choice('}', $._implicit_close),
      ),

    operator: _ =>
      choice(
        '+',
        '-',
        '*',
        '/',
        '//',
        '%',
        '**',
        '==',
        '!=',
        '<',
        '>',
        '<=',
        '>=',
        '=',
        '~',
        '!',
      ),

    boolean: _ => choice(...BOOLEANS),

    none: _ => choice(...NONES),

    string: _ =>
      token(
        choice(
          seq('"', repeat(choice(/[^"\\]/, /\\[\s\S]/)), '"'),
          seq("'", repeat(choice(/[^'\\]/, /\\[\s\S]/)), "'"),
        ),
      ),

    number: _ =>
      token(
        choice(
          /0[xX][0-9a-fA-F_]+/,
          /0[oO][0-7_]+/,
          /0[bB][01_]+/,
          /\d[\d_]*(\.\d[\d_]*)?([eE][+-]?\d[\d_]*)?/,
        ),
      ),

    identifier: _ => /[_\p{XID_Start}][_\p{XID_Continue}]*/,

    // Any other character. Keeps stray input from becoming a parse error.
    unknown: _ => token(prec(-1, /[^\s]/)),
  },
});
