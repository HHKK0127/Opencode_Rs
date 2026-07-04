const __vite__mapDeps=(i,m=__vite__mapDeps,d=(m.f||(m.f=["./dialog-connect-provider-BF5IVAo-.js","./main-D_cwiNV1.js","./main-CqjwCdp1.css","./dialog-select-provider-bKjtPeTG.js"])))=>i.map(i=>d[i]);
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
      n && (e._sentryDebugIds = e._sentryDebugIds || {}, e._sentryDebugIds[n] = "a6ce5192-9108-4c12-beaf-4812e8bee1a1", e._sentryDebugIdIdentifier = "sentry-dbid-a6ce5192-9108-4c12-beaf-4812e8bee1a1");
    })();
  } catch (e) {
  }
}
;
import { by as useLocal, br as useDialog, bG as useProviders, bw as useLanguage, ad as createComponent, aL as insert, L as List, W as Tag, O as Show, Z as Tooltip, q as ModelTooltip, aP as memo, s as ProviderIcon, b5 as popularProviders, B as Button, D as Dialog, ay as decode64, bh as template, $ as __vitePreload, az as delegateEvents } from "./main-D_cwiNV1.js";
var _tmpl$ = /* @__PURE__ */ template(`<div class="flex flex-col gap-3 px-2.5"><div class="text-14-medium text-text-base px-2.5">`), _tmpl$2 = /* @__PURE__ */ template(`<div class="px-1.5 pb-1.5"><div class="w-full rounded-sm border border-border-weak-base bg-surface-raised-base"><div class="w-full flex flex-col items-start gap-4 px-1.5 pt-4 pb-4"><div class="px-2 text-14-medium text-text-base"></div><div class=w-full>`), _tmpl$3 = /* @__PURE__ */ template(`<div class="w-full flex items-center gap-x-2.5"><span>`), _tmpl$4 = /* @__PURE__ */ template(`<div class="text-14-regular text-text-weak">`), _tmpl$5 = /* @__PURE__ */ template(`<div class="w-full flex items-center gap-x-3"><span>`);
const DialogSelectModelUnpaid = (props) => {
  const local = useLocal();
  const model = props.model ?? local.model;
  const dialog = useDialog();
  const directory = () => decode64(local.slug());
  const providers = useProviders(directory);
  const language = useLanguage();
  const connect = (provider) => {
    void __vitePreload(() => import("./dialog-connect-provider-BF5IVAo-.js").then((n) => n.d), true ? __vite__mapDeps([0,1,2]) : void 0, import.meta.url).then((x) => {
      dialog.show(() => createComponent(x.DialogConnectProvider, {
        provider,
        directory
      }));
    });
  };
  const all = () => {
    void __vitePreload(() => import("./dialog-select-provider-bKjtPeTG.js").then((n) => n.d), true ? __vite__mapDeps([3,1,2,0]) : void 0, import.meta.url).then((x) => {
      dialog.show(() => createComponent(x.DialogSelectProvider, {
        directory
      }));
    });
  };
  let listRef;
  const handleKeyDown = (e) => {
    if (e.key === "Escape") return;
    listRef?.onKeyDown(e);
  };
  return createComponent(Dialog, {
    get title() {
      return language.t("dialog.model.select.title");
    },
    "class": "overflow-y-auto [&_[data-slot=dialog-body]]:overflow-visible [&_[data-slot=dialog-body]]:flex-none",
    get children() {
      return [(() => {
        var _el$ = _tmpl$(), _el$2 = _el$.firstChild;
        _el$.$$keydown = handleKeyDown;
        insert(_el$2, () => language.t("dialog.model.unpaid.freeModels.title"));
        insert(_el$, createComponent(List, {
          "class": "px-3 [&_[data-slot=list-scroll]]:overflow-visible",
          ref: (ref) => listRef = ref,
          get items() {
            return model.list;
          },
          get current() {
            return model.current();
          },
          key: (x) => `${x.provider.id}:${x.id}`,
          itemWrapper: (item, node) => createComponent(Tooltip, {
            "class": "w-full",
            placement: "right-start",
            gutter: 12,
            get value() {
              return createComponent(ModelTooltip, {
                model: item,
                get latest() {
                  return item.latest;
                },
                get free() {
                  return memo(() => item.provider.id === "opencode")() && (!item.cost || item.cost.input === 0);
                }
              });
            },
            children: node
          }),
          onSelect: (x) => {
            model.set(x ? {
              modelID: x.id,
              providerID: x.provider.id
            } : void 0, {
              recent: true
            });
            dialog.close();
          },
          children: (i) => (() => {
            var _el$8 = _tmpl$3(), _el$9 = _el$8.firstChild;
            insert(_el$9, () => i.name);
            insert(_el$8, createComponent(Tag, {
              get children() {
                return language.t("model.tag.free");
              }
            }), null);
            insert(_el$8, createComponent(Show, {
              get when() {
                return i.latest;
              },
              get children() {
                return createComponent(Tag, {
                  get children() {
                    return language.t("model.tag.latest");
                  }
                });
              }
            }), null);
            return _el$8;
          })()
        }), null);
        return _el$;
      })(), (() => {
        var _el$3 = _tmpl$2(), _el$4 = _el$3.firstChild, _el$5 = _el$4.firstChild, _el$6 = _el$5.firstChild, _el$7 = _el$6.nextSibling;
        insert(_el$6, () => language.t("dialog.model.unpaid.addMore.title"));
        insert(_el$7, createComponent(List, {
          "class": "w-full px-3",
          key: (p) => p.id,
          get items() {
            return providers.popular;
          },
          activeIcon: "plus-small",
          sortBy: (a, b) => {
            if (popularProviders.includes(a.id) && popularProviders.includes(b.id)) return popularProviders.indexOf(a.id) - popularProviders.indexOf(b.id);
            return a.name.localeCompare(b.name);
          },
          onSelect: (x) => {
            if (!x) return;
            connect(x.id);
          },
          children: (i) => (() => {
            var _el$0 = _tmpl$5(), _el$1 = _el$0.firstChild;
            insert(_el$0, createComponent(ProviderIcon, {
              "data-slot": "list-item-extra-icon",
              get id() {
                return i.id;
              }
            }), _el$1);
            insert(_el$1, () => i.name);
            insert(_el$0, createComponent(Show, {
              get when() {
                return i.id === "opencode";
              },
              get children() {
                var _el$10 = _tmpl$4();
                insert(_el$10, () => language.t("dialog.provider.opencode.tagline"));
                return _el$10;
              }
            }), null);
            insert(_el$0, createComponent(Show, {
              get when() {
                return i.id === "opencode";
              },
              get children() {
                return createComponent(Tag, {
                  get children() {
                    return language.t("dialog.provider.tag.recommended");
                  }
                });
              }
            }), null);
            insert(_el$0, createComponent(Show, {
              get when() {
                return i.id === "opencode-go";
              },
              get children() {
                return [(() => {
                  var _el$11 = _tmpl$4();
                  insert(_el$11, () => language.t("dialog.provider.opencodeGo.tagline"));
                  return _el$11;
                })(), createComponent(Tag, {
                  get children() {
                    return language.t("dialog.provider.tag.recommended");
                  }
                })];
              }
            }), null);
            insert(_el$0, createComponent(Show, {
              get when() {
                return i.id === "anthropic";
              },
              get children() {
                var _el$12 = _tmpl$4();
                insert(_el$12, () => language.t("dialog.provider.anthropic.note"));
                return _el$12;
              }
            }), null);
            return _el$0;
          })()
        }), null);
        insert(_el$7, createComponent(Button, {
          variant: "ghost",
          "class": "w-full justify-start px-[11px] py-3.5 gap-4.5 text-14-medium",
          icon: "dot-grid",
          onClick: all,
          get children() {
            return language.t("dialog.provider.viewAll");
          }
        }), null);
        return _el$3;
      })()];
    }
  });
};
delegateEvents(["keydown"]);
export {
  DialogSelectModelUnpaid
};
//# sourceMappingURL=dialog-select-model-unpaid-DRuc3Egw.js.map
