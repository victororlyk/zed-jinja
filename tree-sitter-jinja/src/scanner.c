// External scanner for the lenient Jinja grammar.
//
// It produces the tokens that depend on where a Jinja tag starts:
//
//   content          template text up to the next `{{`, `{%` or `{#`
//   comment          `{# ... #}`, or to the end of the file if unterminated
//   raw_content      text after `{% raw %}` up to the matching `{% endraw %}`
//   _implicit_close  zero-width; ends an unfinished tag where the next tag
//                    starts, or at the end of the file
//
// The scanner keeps no state and uses nothing from libc, which keeps the
// WebAssembly build self-contained.

#include "tree_sitter/parser.h"

enum TokenType {
    CONTENT,
    COMMENT,
    RAW_CONTENT,
    IMPLICIT_CLOSE,
    ERROR_SENTINEL,
};

static inline bool is_space(int32_t c) {
    return c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == '\f' || c == '\v';
}

static inline bool starts_tag(int32_t c) {
    return c == '{' || c == '%' || c == '#';
}

static inline void advance(TSLexer *lexer) {
    lexer->advance(lexer, false);
}

// Called with the lexer just past `{#`.
static bool scan_comment(TSLexer *lexer) {
    while (!lexer->eof(lexer)) {
        if (lexer->lookahead == '#') {
            advance(lexer);
            if (lexer->lookahead == '}') {
                advance(lexer);
                break;
            }
        } else {
            advance(lexer);
        }
    }
    lexer->mark_end(lexer);
    lexer->result_symbol = COMMENT;
    return true;
}

// `consumed` says whether the token already holds text.
static bool scan_content(TSLexer *lexer, bool consumed) {
    for (;;) {
        if (lexer->eof(lexer)) {
            lexer->mark_end(lexer);
            break;
        }
        if (lexer->lookahead == '{') {
            // The token ends here if this brace opens a tag.
            lexer->mark_end(lexer);
            advance(lexer);
            if (starts_tag(lexer->lookahead)) {
                break;
            }
            consumed = true;
            continue;
        }
        advance(lexer);
        consumed = true;
    }
    lexer->result_symbol = CONTENT;
    return consumed;
}

// Called with the lexer just past `{`. Consumes the rest of
// `{%[-+]? endraw [-+]?%}` and returns whether all of it was there. On a
// mismatch the offending character is left unconsumed.
static bool at_endraw(TSLexer *lexer) {
    static const char word[] = "endraw";

    if (lexer->lookahead != '%') {
        return false;
    }
    advance(lexer);
    if (lexer->lookahead == '-' || lexer->lookahead == '+') {
        advance(lexer);
    }
    while (is_space(lexer->lookahead)) {
        advance(lexer);
    }
    for (unsigned i = 0; word[i] != '\0'; i++) {
        if (lexer->lookahead != word[i]) {
            return false;
        }
        advance(lexer);
    }
    while (is_space(lexer->lookahead)) {
        advance(lexer);
    }
    if (lexer->lookahead == '-' || lexer->lookahead == '+') {
        advance(lexer);
    }
    if (lexer->lookahead != '%') {
        return false;
    }
    advance(lexer);
    return lexer->lookahead == '}';
}

static bool scan_raw_content(TSLexer *lexer) {
    bool consumed = false;
    for (;;) {
        if (lexer->eof(lexer)) {
            lexer->mark_end(lexer);
            break;
        }
        if (lexer->lookahead == '{') {
            lexer->mark_end(lexer);
            advance(lexer);
            if (at_endraw(lexer)) {
                break;
            }
            consumed = true;
            continue;
        }
        advance(lexer);
        consumed = true;
    }
    lexer->result_symbol = RAW_CONTENT;
    return consumed;
}

static bool scan_implicit_close(TSLexer *lexer) {
    while (is_space(lexer->lookahead)) {
        lexer->advance(lexer, true);
    }
    // Zero width: nothing after this point belongs to the token.
    lexer->mark_end(lexer);
    lexer->result_symbol = IMPLICIT_CLOSE;
    if (lexer->eof(lexer)) {
        return true;
    }
    if (lexer->lookahead != '{') {
        return false;
    }
    advance(lexer);
    return starts_tag(lexer->lookahead);
}

bool tree_sitter_jinja_external_scanner_scan(void *payload, TSLexer *lexer, const bool *valid_symbols) {
    (void)payload;

    // During error recovery every symbol is marked valid. Leave recovery to
    // the parser rather than guess.
    if (valid_symbols[ERROR_SENTINEL]) {
        return false;
    }

    if (valid_symbols[RAW_CONTENT]) {
        return scan_raw_content(lexer);
    }

    if (valid_symbols[CONTENT] || valid_symbols[COMMENT]) {
        bool consumed = false;
        if (lexer->lookahead == '{') {
            advance(lexer);
            if (lexer->lookahead == '#') {
                if (!valid_symbols[COMMENT]) {
                    return false;
                }
                advance(lexer);
                return scan_comment(lexer);
            }
            if (starts_tag(lexer->lookahead)) {
                return false;
            }
            consumed = true;
        }
        if (!valid_symbols[CONTENT]) {
            return false;
        }
        return scan_content(lexer, consumed);
    }

    if (valid_symbols[IMPLICIT_CLOSE]) {
        return scan_implicit_close(lexer);
    }

    return false;
}

void *tree_sitter_jinja_external_scanner_create(void) {
    return NULL;
}

void tree_sitter_jinja_external_scanner_destroy(void *payload) {
    (void)payload;
}

unsigned tree_sitter_jinja_external_scanner_serialize(void *payload, char *buffer) {
    (void)payload;
    (void)buffer;
    return 0;
}

void tree_sitter_jinja_external_scanner_deserialize(void *payload, const char *buffer, unsigned length) {
    (void)payload;
    (void)buffer;
    (void)length;
}
