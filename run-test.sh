#!/bin/bash
# File to modify
FILE="lttb.js"

# Line to add
EXPORT_LINE="module.exports = lttb;"

# Backup the original file
cp "$FILE" "${FILE}.bak"

# Add the export line to the end of the file
echo "$EXPORT_LINE" >> "$FILE"

# Run the test
node run-lttb-object.js

# Restore the original file
mv "${FILE}.bak" "$FILE"

echo "Test completed and lttb.js restored to original state."
