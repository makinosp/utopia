#!/usr/bin/env bash
set -e

HEADER=$(mktemp)
cat > "$HEADER" << 'EOF'
---
description: "AI-DLC adaptive workflow for software development"
alwaysApply: true
---
EOF

cat "$HEADER" .vendor/aidlc-workflows/aws-aidlc-rules/core-workflow.md \
  > .cursor/rules/ai-dlc-workflow.mdc

echo "✓ Cursor rule updated"
