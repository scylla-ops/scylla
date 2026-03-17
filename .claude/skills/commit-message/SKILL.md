---
name: commit-message
description: Write a clear and concise Git commit message summarizing the changes. Use when committing anything
---

Write a commit message for the staged changes. Only output the message, no commentary.

Rules:
- Conventional commit style, imperative mood, subject and body all lowercase, no trailing punctuation
- Subject line ≤50 chars; omit body unless it adds info not in the subject
- No bullet points, no repeating the subject in the body, no raw diffs
