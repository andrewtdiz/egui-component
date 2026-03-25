id: emoji-selector
label: Emoji Selector
family: composed
summary: Emoji Selector authoring notes.

# Emoji Selector

Describe the trigger behavior, picker state ownership, and how search plus category filtering compose with Twemoji rendering.

## Family Guide

Read `family-composed.md` before editing this component.

## Notes

- Keep popup state local to the component and keyed by `Id`.
- Prefer static curated emoji data over runtime-generated catalogs.
- Preserve the default button-triggered search-first picker path.
