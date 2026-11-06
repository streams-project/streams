#!/bin/sh
# Trunk writes its loader inline, and the page's script policy allows none, so it moves to a file.
set -eu
page="$TRUNK_STAGING_DIR/index.html"
sed -n '/^<script type="module">$/,/^<\/script>$/p' "$page" | sed '1d;$d' > "$TRUNK_STAGING_DIR/boot.js"
sed -e '/^<script type="module">$/,/^<\/script>$/d' \
  -e 's|</head>|<script src="/boot.js" type="module"></script></head>|' "$page" > "$page.new"
mv "$page.new" "$page"
