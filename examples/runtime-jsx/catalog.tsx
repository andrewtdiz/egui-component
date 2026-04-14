import { log, render } from "egui";
import { ComponentCatalog } from "./ui/registry.tsx";

render(<ComponentCatalog />);
log("info", "examples/runtime-jsx/catalog.tsx rendered JSX migration catalog");
