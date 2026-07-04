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
      n && (e._sentryDebugIds = e._sentryDebugIds || {}, e._sentryDebugIds[n] = "f205ec75-91da-465d-8299-ec82d3079132", e._sentryDebugIdIdentifier = "sentry-dbid-f205ec75-91da-465d-8299-ec82d3079132");
    })();
  } catch (e) {
  }
}
;
import { bQ as useSync, bw as useLanguage, am as createMemo, ad as createComponent, L as List, aL as insert, O as Show, aP as memo, D as Dialog, bh as template, az as delegateEvents } from "./main-D_cwiNV1.js";
import { S as Switch } from "./switch-C568Vph6.js";
import { u as useMcpToggle } from "./mcp-DlCvjYX6.js";
import "./LROKH5N7-r-w1NcZW.js";
var _tmpl$ = /* @__PURE__ */ template(`<span class="text-11-regular text-text-weaker">`), _tmpl$2 = /* @__PURE__ */ template(`<span class="text-11-regular text-text-weaker truncate">`), _tmpl$3 = /* @__PURE__ */ template(`<div class="w-full flex items-center justify-between gap-x-3"><div class="flex flex-col gap-0.5 min-w-0"><div class="flex items-center gap-2"><span class=truncate></span></div></div><div>`);
const statusLabels = {
  connected: "mcp.status.connected",
  failed: "mcp.status.failed",
  needs_auth: "mcp.status.needs_auth",
  needs_client_registration: "mcp.status.needs_client_registration",
  disabled: "mcp.status.disabled"
};
const DialogSelectMcp = () => {
  const sync = useSync();
  const language = useLanguage();
  const items = createMemo(() => Object.entries(sync().data.mcp ?? {}).map(([name, status]) => ({
    name,
    status: status.status
  })).sort((a, b) => a.name.localeCompare(b.name)));
  const toggle = useMcpToggle();
  const enabledCount = createMemo(() => items().filter((i) => i.status === "connected").length);
  const totalCount = createMemo(() => items().length);
  return createComponent(Dialog, {
    get title() {
      return language.t("dialog.mcp.title");
    },
    get description() {
      return language.t("dialog.mcp.description", {
        enabled: enabledCount(),
        total: totalCount()
      });
    },
    get children() {
      return createComponent(List, {
        "class": "px-3",
        get search() {
          return {
            placeholder: language.t("common.search.placeholder"),
            autofocus: true
          };
        },
        get emptyMessage() {
          return language.t("dialog.mcp.empty");
        },
        key: (x) => x?.name ?? "",
        items,
        filterKeys: ["name", "status"],
        sortBy: (a, b) => a.name.localeCompare(b.name),
        onSelect: (x) => {
          if (!x || toggle.isPending) return;
          toggle.mutate(x.name);
        },
        children: (i) => {
          const mcpStatus = () => sync().data.mcp[i.name];
          const status = () => mcpStatus()?.status;
          const statusLabel = () => {
            const key = status() ? statusLabels[status()] : void 0;
            if (!key) return;
            return language.t(key);
          };
          const error = () => {
            const s = mcpStatus();
            if (s?.status === "failed" || s?.status === "needs_client_registration") return s.error;
          };
          const enabled = () => status() === "connected";
          return (() => {
            var _el$ = _tmpl$3(), _el$2 = _el$.firstChild, _el$3 = _el$2.firstChild, _el$4 = _el$3.firstChild, _el$7 = _el$2.nextSibling;
            insert(_el$4, () => i.name);
            insert(_el$3, createComponent(Show, {
              get when() {
                return statusLabel();
              },
              get children() {
                var _el$5 = _tmpl$();
                insert(_el$5, statusLabel);
                return _el$5;
              }
            }), null);
            insert(_el$2, createComponent(Show, {
              get when() {
                return error();
              },
              get children() {
                var _el$6 = _tmpl$2();
                insert(_el$6, error);
                return _el$6;
              }
            }), null);
            _el$7.$$click = (e) => e.stopPropagation();
            insert(_el$7, createComponent(Switch, {
              get checked() {
                return enabled();
              },
              get disabled() {
                return memo(() => !!toggle.isPending)() && toggle.variables === i.name;
              },
              onChange: () => {
                if (toggle.isPending) return;
                toggle.mutate(i.name);
              }
            }));
            return _el$;
          })();
        }
      });
    }
  });
};
delegateEvents(["click"]);
export {
  DialogSelectMcp
};
//# sourceMappingURL=dialog-select-mcp-9smdmNFb.js.map
