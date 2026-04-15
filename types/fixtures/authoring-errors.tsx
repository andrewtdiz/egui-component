import { render } from "egui";

render(
  <div id="root" data-slot="column">
    {/* @ts-expect-error unknown contract prop */}
    <button id="bad-prop" data-slot="button" unknownProp="nope" />

    {/* @ts-expect-error invalid variant literal */}
    <button id="bad-variant" data-slot="button" variant="tertiary" />

    {/* @ts-expect-error unsupported event handler */}
    <button id="bad-handler" data-slot="button" onBlur={() => {}} />

    {/* @ts-expect-error unknown contract family */}
    <div id="bad-slot" data-slot="not-a-family" />
  </div>,
);
