import { useState } from "egui";
import {
  migrationManifest,
  pendingMigrationCount,
  runtimeBehaviorCount,
  runtimePrimitiveCount,
  tsxOwnedCount,
} from "./migration-manifest.ts";
import { Button, Card, Label, Separator, componentPreviews } from "./components/index.tsx";
import type { Children } from "./component-support.ts";
import { styles } from "./lib/styles.ts";

type Preview = {
  id: string;
  label: string;
  section: string;
  summary: string;
  render: () => Children;
};

const SIDEPANEL_WIDTH = 264;
const SIDEPANEL_ITEM_WIDTH = 232;
const SIDEPANEL_SCROLL_HEIGHT = 640;
const PREVIEW_SHELL_WIDTH = 820;
const PREVIEW_CONTENT_WIDTH = 720;

const setupPreviews: Preview[] = [
  {
    id: "component-authoring",
    label: "Component authoring",
    section: "setup",
    summary: "Per-component JSX modules, shared class helpers, and the preview harness.",
    render: ComponentAuthoringPreview,
  },
  {
    id: "manifest-summary",
    label: "Manifest summary",
    section: "setup",
    summary: "Current migration surface and Rust-owned runtime support that still backs the JSX layer.",
    render: ManifestSummaryPreview,
  },
];

export const previewRegistry: Preview[] = [...setupPreviews, ...componentPreviews];

export function ComponentCatalog() {
  const [selectedPreviewId, setSelectedPreviewId] = useState(previewRegistry[0]?.id ?? "");
  const sections = groupPreviews(previewRegistry);
  const activePreview = resolvePreview(selectedPreviewId);

  if (activePreview == null) {
    return (
      <div id="jsx-migration-catalog-root" data-slot="column" className="gap-[18px] items-stretch">
        <Label text="JSX component library" className="text-xl font-semibold" />
        <Label
          text="No component previews are registered."
          className="text-sm text-muted-foreground"
        />
      </div>
    );
  }

  const ActivePreview = activePreview.render;

  return (
    <div id="jsx-migration-catalog-root" data-slot="column" className="gap-[18px] items-stretch">
      <div id="jsx-migration-catalog-header" data-slot="column" className="gap-[4px] items-stretch">
        <Label text="JSX component library" className="text-xl font-semibold" />
        <Label
          text="Minimal browser for verifying each TSX component and the hot-reload authoring surface."
          className="text-sm text-muted-foreground"
          width={PREVIEW_CONTENT_WIDTH}
        />
        <div data-slot="row" className="gap-[12px] items-center">
          <Label text={`${componentPreviews.length} components`} className="text-xs text-muted-foreground" />
          <Label text={`${pendingMigrationCount} pending`} className="text-xs text-muted-foreground" />
          <Label text={`${tsxOwnedCount} TSX-owned`} className="text-xs text-muted-foreground" />
        </div>
      </div>

      <Separator id="jsx-migration-catalog-separator" />

      <div id="jsx-catalog-body" data-slot="row" className="gap-[24px] items-start">
        <div id="jsx-catalog-sidepanel-shell" data-slot="sized-box" width={SIDEPANEL_WIDTH}>
          <Card id="jsx-catalog-sidepanel" paddingX={12} paddingY={12}>
            <div data-slot="column" className="gap-[14px] items-stretch">
              <div data-slot="column" className="gap-[4px] items-stretch">
                <Label
                  id="jsx-catalog-sidepanel-title"
                  text="Browse components"
                  className="text-sm font-semibold"
                  width={SIDEPANEL_ITEM_WIDTH}
                />
                <Label
                  text="Pick a component on the left and inspect its JSX preview in the center."
                  className="text-xs text-muted-foreground"
                  width={SIDEPANEL_ITEM_WIDTH}
                />
              </div>

              <div
                id="jsx-catalog-sidepanel-scroller"
                data-slot="column"
                className="gap-[14px] items-stretch"
                height={SIDEPANEL_SCROLL_HEIGHT}
                overflowY="scroll"
              >
                {sections.map(([section, previews]) => (
                  <div
                    key={section}
                    id={`jsx-catalog-section-${section}`}
                    data-slot="column"
                    className="gap-[6px] items-stretch"
                  >
                    <Label
                      text={sectionLabel(section)}
                      className="text-xs font-semibold text-muted-foreground"
                      width={SIDEPANEL_ITEM_WIDTH}
                    />
                    {previews.map((preview) => {
                      const selected = preview.id === activePreview.id;
                      return (
                        <Button
                          key={preview.id}
                          id={`jsx-catalog-nav-${preview.id}`}
                          variant={selected ? "secondary" : "ghost"}
                          size="sm"
                          selected={selected}
                          width={SIDEPANEL_ITEM_WIDTH}
                          onClick={() => setSelectedPreviewId(preview.id)}
                        >
                          {preview.label}
                        </Button>
                      );
                    })}
                  </div>
                ))}
              </div>
            </div>
          </Card>
        </div>

        <div id="jsx-catalog-preview-shell" data-slot="sized-box" width={PREVIEW_SHELL_WIDTH}>
          <div id="jsx-catalog-preview-column" data-slot="column" className="gap-[16px] items-stretch">
            <Card id="jsx-catalog-preview-header" paddingX={16} paddingY={16}>
              <div data-slot="column" className="gap-[6px] items-stretch">
                <Label
                  id="jsx-catalog-active-section"
                  text={sectionLabel(activePreview.section)}
                  className="text-xs font-semibold text-muted-foreground"
                  width={PREVIEW_CONTENT_WIDTH}
                />
                <Label
                  id="jsx-catalog-active-label"
                  text={activePreview.label}
                  className="text-xl font-semibold"
                  width={PREVIEW_CONTENT_WIDTH}
                />
                <Label
                  id="jsx-catalog-active-summary"
                  text={activePreview.summary}
                  className="text-sm text-muted-foreground"
                  width={PREVIEW_CONTENT_WIDTH}
                />
              </div>
            </Card>

            <Card
              key={activePreview.id}
              id={`jsx-catalog-preview-panel-${activePreview.id}`}
              paddingX={18}
              paddingY={18}
            >
              <div id="jsx-catalog-active-preview" data-slot="column" className="gap-[12px] items-stretch">
                <Label
                  text="Preview region"
                  className="text-xs font-semibold text-muted-foreground"
                  width={PREVIEW_CONTENT_WIDTH}
                />
                <ActivePreview />
              </div>
            </Card>
          </div>
        </div>
      </div>
    </div>
  );
}

function ComponentAuthoringPreview() {
  return (
    <div id="jsx-component-authoring-preview" data-slot="column" className="gap-[10px] items-stretch">
      <div data-slot="row" className="gap-[8px] items-center">
        <Button id="jsx-migration-primary-action" variant="primary">
          Primary
        </Button>
        <Button id="jsx-migration-secondary-action" variant="secondary">
          Secondary
        </Button>
        <Button id="jsx-migration-ghost-action" variant="ghost">
          Ghost
        </Button>
      </div>
      <div data-slot="column" className="gap-[4px] items-stretch">
        <Label text="Static class subset" className="text-base font-semibold" />
        <Label
          text="Keep previews clean: className handles typography and spacing while explicit props keep renderer state obvious."
          className="text-sm text-muted-foreground"
        />
        <Label
          text="Destructive and stateful variants still stay explicit in the egui contract."
          className={styles.destructive}
        />
      </div>
    </div>
  );
}

function ManifestSummaryPreview() {
  const composed = migrationManifest.filter((item) => item.strategy === "compose").length;
  const aliases = migrationManifest.filter((item) => item.strategy === "family-alias").length;
  const implemented = migrationManifest.filter((item) => item.status === "implemented").length;

  return (
    <div id="jsx-migration-manifest-preview" data-slot="column" className="gap-[8px] items-stretch">
      <div data-slot="row" className="gap-[12px] items-center">
        <Label text={`${componentPreviews.length} previews`} className="text-xs text-muted-foreground" />
        <Label text={`${composed} composed`} className="text-xs text-muted-foreground" />
        <Label text={`${aliases} aliases`} className="text-xs text-muted-foreground" />
        <Label text={`${implemented} implemented`} className="text-xs text-muted-foreground" />
      </div>
      <div data-slot="row" className="gap-[12px] items-center">
        <Label text={`${runtimePrimitiveCount} Rust primitives`} className="text-xs text-muted-foreground" />
        <Label text={`${runtimeBehaviorCount} behavior families`} className="text-xs text-muted-foreground" />
        <Label text={`${tsxOwnedCount} TSX-owned`} className="text-xs text-muted-foreground" />
      </div>
      {migrationManifest.slice(0, 6).map((item) => (
        <div key={item.id} data-slot="row" className="gap-[8px] items-center">
          <Label text={item.label} />
          <Label text={item.strategy} className="text-xs text-muted-foreground" />
        </div>
      ))}
    </div>
  );
}

function groupPreviews(previews: Preview[]) {
  const groups = new Map<string, Preview[]>();
  for (const preview of previews) {
    const section = groups.get(preview.section);
    if (section == null) {
      groups.set(preview.section, [preview]);
      continue;
    }
    section.push(preview);
  }
  return [...groups.entries()];
}

function resolvePreview(previewId: string) {
  return previewRegistry.find((preview) => preview.id === previewId) ?? previewRegistry[0];
}

function sectionLabel(section: string) {
  return section
    .split("-")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}
