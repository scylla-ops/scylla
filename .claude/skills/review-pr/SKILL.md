---
name: review-pr
description: Review a pull request or set of changes for bugs, style issues, and improvements. Use when the user asks to review code or a PR.
---

Review the given PR or diff. Focus on:
- Correctness: bugs, logic errors, edge cases, security issues
- Style: naming, consistency with surrounding code, readability
- Design: unnecessary complexity, missing abstractions, or over-engineering

Be direct and specific. Reference file paths and line numbers. Skip praise for things that are simply correct. Only flag real issues or meaningful suggestions. Group feedback by severity (must-fix vs nice-to-have).

If given a PR number, fetch it with `gh pr diff <number>`. If no number given, review the current branch's diff against the base branch.
