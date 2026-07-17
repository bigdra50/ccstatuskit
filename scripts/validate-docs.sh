#!/usr/bin/env bash
# Validates that translated documentation stays structurally in sync with
# the English source (AGENTS.md: English files are the source of truth).
#
# Checks, for every English doc that has translations:
#   1. The .ja.md and .zh-CN.md counterparts exist
#   2. Heading counts match (code blocks excluded)
#   3. Code-fence counts match
#   4. The language-switcher line is present near the top of every file
set -uo pipefail

fail=0

count_headings() {
    awk '/^```/ { in_block = !in_block; next } !in_block && /^#/ { n++ } END { print n + 0 }' "$1"
}

count_fences() {
    grep -c '^```' "$1" || true
}

english_docs=(README.md docs/*.md)
for en in "${english_docs[@]}"; do
    case "$en" in
        *.ja.md | *.zh-CN.md) continue ;;
    esac
    base="${en%.md}"
    for lang in ja zh-CN; do
        tr="$base.$lang.md"
        if [ ! -f "$tr" ]; then
            echo "MISSING: $tr (translation of $en)"
            fail=1
            continue
        fi
        en_headings=$(count_headings "$en")
        tr_headings=$(count_headings "$tr")
        if [ "$en_headings" != "$tr_headings" ]; then
            echo "HEADING MISMATCH: $en has $en_headings headings, $tr has $tr_headings"
            fail=1
        fi
        en_fences=$(count_fences "$en")
        tr_fences=$(count_fences "$tr")
        if [ "$en_fences" != "$tr_fences" ]; then
            echo "CODE-BLOCK MISMATCH: $en has $en_fences fences, $tr has $tr_fences"
            fail=1
        fi
    done

    # Every file in the trio must carry the switcher naming all languages.
    for f in "$en" "$base.ja.md" "$base.zh-CN.md"; do
        [ -f "$f" ] || continue
        if ! head -10 "$f" | grep -q "English" ||
            ! head -10 "$f" | grep -q "日本語" ||
            ! head -10 "$f" | grep -q "简体中文"; then
            echo "SWITCHER MISSING: $f lacks the language-switcher line in its first 10 lines"
            fail=1
        fi
    done
done

if [ "$fail" -ne 0 ]; then
    echo ""
    echo "Documentation validation FAILED. English is the source of truth;"
    echo "update the .ja.md / .zh-CN.md counterparts in the same change."
    exit 1
fi
echo "Documentation validation passed."
