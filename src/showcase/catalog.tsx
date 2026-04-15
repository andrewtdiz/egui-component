import { log, render } from "egui";
import { ComponentCatalog } from "./ui/registry.tsx";

render(<ComponentCatalog />);
log("info", "src/showcase/catalog.tsx rendered JSX migration catalog");
