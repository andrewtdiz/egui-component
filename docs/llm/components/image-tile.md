id: image-tile
label: Image Tile
family: composed
summary: Grid-friendly image tile with optional custom body and playback overlay.

# Image Tile

Use this for media-gallery or asset-grid tiles that need a fixed image region, optional custom body content, and an optional play/pause affordance for previewable media.

## Family Guide

Read `family-composed.md` before editing this component.

## Notes

- Keep the runtime API typed.
- Prefer shared primitives over inline paint code.
- Keep the default path mechanical for both humans and agents.
- Support both preset sizes and exact image-region sizing.
- The body region must stay optional and closure-based so callers can render title/subtext or nothing at all.
