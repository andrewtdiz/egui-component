id: pagination
label: Pagination
family: composed
summary: Pagination control with previous/next navigation and truncated page ranges.

# Pagination

Describe the dominant use-case, the semantic fields that matter, and the primitives this component should compose.

## Family Guide

Read `family-composed.md` before editing this component.

## Notes

- Keep the runtime API typed.
- Treat `current_page` as 1-based.
- Hide `Previous` on page 1 and `Next` on the last page.
- Insert ellipsis when the full page range should be truncated.
- Prefer shared primitives over inline paint code.
- Keep the default path mechanical for both humans and agents.
