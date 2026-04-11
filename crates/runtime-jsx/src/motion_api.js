import { jsx } from "egui/jsx-runtime";

const cachedMotionComponents = new Map();

export const motion = new Proxy(
  {},
  {
    get(_target, family) {
      if (typeof family !== "string") {
        return undefined;
      }
      if (!cachedMotionComponents.has(family)) {
        const component = function MotionHostComponent(props) {
          return jsx(family, { ...props, __motion_host: true });
        };
        component.displayName = `motion.${family}`;
        cachedMotionComponents.set(family, component);
      }
      return cachedMotionComponents.get(family);
    },
  },
);

export function MotionConfig(props) {
  return props.children;
}

export function AnimatePresence(props) {
  return props.children;
}
