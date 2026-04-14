import {
  Fragment,
  jsx as reactJsx,
  jsxs as reactJsxs,
} from "react/jsx-runtime";
import { jsxDEV as reactJsxDEV } from "react/jsx-dev-runtime";

function withAuthorKey(props, key) {
  if (key === undefined || key === null) {
    return props == null ? {} : props;
  }
  return { ...(props ?? {}), __eguiKey: String(key) };
}

export { Fragment };

export function jsx(type, props, key) {
  return reactJsx(type, withAuthorKey(props, key), key);
}

export function jsxs(type, props, key) {
  return reactJsxs(type, withAuthorKey(props, key), key);
}

export function jsxDEV(type, props, key, isStaticChildren, source, self) {
  return reactJsxDEV(
    type,
    withAuthorKey(props, key),
    key,
    isStaticChildren,
    source,
    self,
  );
}
