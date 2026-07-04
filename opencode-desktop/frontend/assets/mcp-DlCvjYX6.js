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
      n && (e._sentryDebugIds = e._sentryDebugIds || {}, e._sentryDebugIds[n] = "f5b03dc0-afb8-4150-8c66-f7241bf745e0", e._sentryDebugIdIdentifier = "sentry-dbid-f5b03dc0-afb8-4150-8c66-f7241bf745e0");
    })();
  } catch (e) {
  }
}
;
import { bQ as useSync, bw as useLanguage, bA as useMutation, bd as showToast } from "./main-D_cwiNV1.js";
function useMcpToggle() {
  const sync = useSync();
  const language = useLanguage();
  return useMutation(() => ({
    mutationFn: sync().mcp.toggle,
    onError: (error) => showToast({
      variant: "error",
      title: language.t("common.requestFailed"),
      description: error instanceof Error ? error.message : String(error)
    })
  }));
}
export {
  useMcpToggle as u
};
//# sourceMappingURL=mcp-DlCvjYX6.js.map
