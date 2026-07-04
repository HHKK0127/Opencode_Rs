!(function() {
  try {
    var e = "undefined" != typeof window ? window : "undefined" != typeof global ? global : "undefined" != typeof globalThis ? globalThis : "undefined" != typeof self ? self : {};
    e.SENTRY_RELEASE = { id: "desktop@1.17.11" };
  } catch (e2) {
  }
})();
;
{
  try {
    (function() {
      var e = "undefined" != typeof window ? window : "undefined" != typeof global ? global : "undefined" != typeof globalThis ? globalThis : "undefined" != typeof self ? self : {}, n = new e.Error().stack;
      n && (e._sentryDebugIds = e._sentryDebugIds || {}, e._sentryDebugIds[n] = "dcccca72-382b-4fcd-aa1b-b4b465ee7d86", e._sentryDebugIdIdentifier = "sentry-dbid-dcccca72-382b-4fcd-aa1b-b4b465ee7d86");
    })();
  } catch (e) {
  }
}
;
import { aL as insert, bh as template } from "./main-D_cwiNV1.js";
var _tmpl$ = /* @__PURE__ */ template(`<div data-component=settings-v2-list>`);
const SettingsListV2 = (props) => {
  return (() => {
    var _el$ = _tmpl$();
    insert(_el$, () => props.children);
    return _el$;
  })();
};
export {
  SettingsListV2
};
//# sourceMappingURL=list-Ux16SPGo.js.map
