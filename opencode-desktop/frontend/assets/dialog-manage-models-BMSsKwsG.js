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
      n && (e._sentryDebugIds = e._sentryDebugIds || {}, e._sentryDebugIds[n] = "71cc988e-905d-4072-9e10-bc727180955d", e._sentryDebugIdIdentifier = "sentry-dbid-71cc988e-905d-4072-9e10-bc727180955d");
    })();
  } catch (e) {
  }
}
;
import { by as useLocal, bw as useLanguage, br as useDialog, ad as createComponent, L as List, aL as insert, Z as Tooltip, B as Button, D as Dialog, bh as template, b5 as popularProviders, ay as decode64, az as delegateEvents } from "./main-D_cwiNV1.js";
import { S as Switch } from "./switch-C568Vph6.js";
import { a as DialogSelectProvider } from "./dialog-select-provider-bKjtPeTG.js";
import "./LROKH5N7-r-w1NcZW.js";
import "./dialog-connect-provider-BF5IVAo-.js";
var _tmpl$ = /* @__PURE__ */ template(`<span>`), _tmpl$2 = /* @__PURE__ */ template(`<div class="w-full flex items-center justify-between gap-x-3"><span></span><div>`);
const DialogManageModels = () => {
  const local = useLocal();
  const language = useLanguage();
  const dialog = useDialog();
  const directory = () => decode64(local.slug());
  const handleConnectProvider = () => {
    dialog.show(() => createComponent(DialogSelectProvider, {
      directory
    }));
  };
  const providerRank = (id) => popularProviders.indexOf(id);
  const providerList = (providerID) => local.model.list().filter((x) => x.provider.id === providerID);
  const providerVisible = (providerID) => providerList(providerID).every((x) => local.model.visible({
    modelID: x.id,
    providerID: x.provider.id
  }));
  const setProviderVisibility = (providerID, checked) => {
    providerList(providerID).forEach((x) => {
      local.model.setVisibility({
        modelID: x.id,
        providerID: x.provider.id
      }, checked);
    });
  };
  return createComponent(Dialog, {
    get title() {
      return language.t("dialog.model.manage");
    },
    get description() {
      return language.t("dialog.model.manage.description");
    },
    get action() {
      return createComponent(Button, {
        "class": "h-7 -my-1 text-14-medium",
        icon: "plus-small",
        tabIndex: -1,
        onClick: handleConnectProvider,
        get children() {
          return language.t("command.provider.connect");
        }
      });
    },
    get children() {
      return createComponent(List, {
        "class": "px-3",
        get search() {
          return {
            placeholder: language.t("dialog.model.search.placeholder"),
            autofocus: true
          };
        },
        get emptyMessage() {
          return language.t("dialog.model.empty");
        },
        key: (x) => `${x?.provider?.id}:${x?.id}`,
        get items() {
          return local.model.list();
        },
        filterKeys: ["provider.name", "name", "id"],
        sortBy: (a, b) => a.name.localeCompare(b.name),
        groupBy: (x) => x.provider.id,
        groupHeader: (group) => {
          const provider = group.items[0].provider;
          return [(() => {
            var _el$ = _tmpl$();
            insert(_el$, () => provider.name);
            return _el$;
          })(), createComponent(Tooltip, {
            placement: "top",
            get value() {
              return language.t("dialog.model.manage.provider.toggle", {
                provider: provider.name
              });
            },
            get children() {
              return createComponent(Switch, {
                "class": "-mr-1",
                get checked() {
                  return providerVisible(provider.id);
                },
                onChange: (checked) => setProviderVisibility(provider.id, checked),
                hideLabel: true,
                get children() {
                  return provider.name;
                }
              });
            }
          })];
        },
        sortGroupsBy: (a, b) => {
          const aRank = providerRank(a.items[0].provider.id);
          const bRank = providerRank(b.items[0].provider.id);
          const aPopular = aRank >= 0;
          const bPopular = bRank >= 0;
          if (aPopular && !bPopular) return -1;
          if (!aPopular && bPopular) return 1;
          return aRank - bRank;
        },
        onSelect: (x) => {
          if (!x) return;
          const key = {
            modelID: x.id,
            providerID: x.provider.id
          };
          local.model.setVisibility(key, !local.model.visible(key));
        },
        children: (i) => (() => {
          var _el$2 = _tmpl$2(), _el$3 = _el$2.firstChild, _el$4 = _el$3.nextSibling;
          insert(_el$3, () => i.name);
          _el$4.$$click = (e) => e.stopPropagation();
          insert(_el$4, createComponent(Switch, {
            get checked() {
              return !!local.model.visible({
                modelID: i.id,
                providerID: i.provider.id
              });
            },
            onChange: (checked) => {
              local.model.setVisibility({
                modelID: i.id,
                providerID: i.provider.id
              }, checked);
            }
          }));
          return _el$2;
        })()
      });
    }
  });
};
delegateEvents(["click"]);
export {
  DialogManageModels
};
//# sourceMappingURL=dialog-manage-models-BMSsKwsG.js.map
