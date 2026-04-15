type ClassDictionary = Record<string, boolean | null | undefined>;
export type ClassValue = string | number | boolean | null | undefined | ClassDictionary | ClassValue[];

function appendClass(out: string[], value: ClassValue): void {
  if (value == null || value === false || value === true) {
    return;
  }

  if (Array.isArray(value)) {
    for (const item of value) {
      appendClass(out, item);
    }
    return;
  }

  if (typeof value === "object") {
    for (const [name, enabled] of Object.entries(value)) {
      if (enabled) {
        out.push(name);
      }
    }
    return;
  }

  const text = String(value).trim();
  if (text !== "") {
    out.push(text);
  }
}

export function cn(...values: ClassValue[]): string {
  const out: string[] = [];
  for (const value of values) {
    appendClass(out, value);
  }
  return out.join(" ");
}
