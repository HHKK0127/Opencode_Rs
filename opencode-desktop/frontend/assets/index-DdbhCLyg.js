const __vite__mapDeps=(i,m=__vite__mapDeps,d=(m.f||(m.f=["./dialog-settings-BAWf-Z5x.js","./main-D_cwiNV1.js","./main-CqjwCdp1.css","./switch-C568Vph6.js","./LROKH5N7-r-w1NcZW.js","./settings-keybinds-BXGE5p-w.js","./dialog-connect-provider-BF5IVAo-.js","./dialog-select-provider-bKjtPeTG.js"])))=>i.map(i=>d[i]);
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
      n && (e._sentryDebugIds = e._sentryDebugIds || {}, e._sentryDebugIds[n] = "eb2d2548-63fd-489f-9558-ec1071df7eff", e._sentryDebugIdIdentifier = "sentry-dbid-eb2d2548-63fd-489f-9558-ec1071df7eff");
    })();
  } catch (e) {
  }
}
;
import { bf as splitProps, ad as createComponent, aR as mergeProps, U as Tabs, aL as insert, bg as spread, O as Show, ao as createRenderEffect, bc as setAttribute, a9 as classList, bh as template, az as delegateEvents, aY as onCleanup, am as createMemo, t as Select, aP as memo, bR as useTheme, bw as useLanguage, bD as usePermission, bE as usePlatform, br as useDialog, bC as useParams, bP as useSettings, al as createMediaQuery, ay as decode64, bM as useServerSync, bL as useServerSDK, ap as createResource, aZ as onMount, $ as __vitePreload, Y as TextInputV2, b8 as sansDefault, b9 as sansFontFamily, aT as monoDefault, aU as monoFontFamily, bi as terminalDefault, bj as terminalFontFamily, a as ButtonV2, ba as sansInput, aV as monoInput, bk as terminalInput, S as SOUND_OPTIONS, b4 as playSoundById, bG as useProviders, b5 as popularProviders, h as For, s as ProviderIcon, bd as showToast, bz as useModels, bt as useFilteredList, o as IconButtonV2, m as Icon, bS as useWslServers, au as createStore, ag as createEffect, T as Switch$2, M as Match, B as Button, R as Spinner, be as showToast$1, aE as fuzzysort, p as MenuV2, bA as useMutation, v as ServerConnection, y as ServerHealthIndicator, b as Dialog, bK as useServerManagementController, bb as serverName, C as ServerRowMenu, d as DialogServerV2, I as Icon$1 } from "./main-D_cwiNV1.js";
import { S as Switch$1 } from "./LROKH5N7-r-w1NcZW.js";
import { u as useUpdaterAction, S as SettingsKeybinds } from "./settings-keybinds-BXGE5p-w.js";
import { L as Link, D as DialogConnectProvider } from "./dialog-connect-provider-BF5IVAo-.js";
import { SettingsListV2 } from "./list-Ux16SPGo.js";
import { D as DialogCustomProvider, a as DialogSelectProvider } from "./dialog-select-provider-bKjtPeTG.js";
var _tmpl$$a = /* @__PURE__ */ template(`<span class="inline-flex items-center gap-2"data-slot=tabs-v2-trigger-content>`), _tmpl$2$6 = /* @__PURE__ */ template(`<div data-slot=tabs-v2-trigger-wrapper>`), _tmpl$3$6 = /* @__PURE__ */ template(`<span data-slot=tabs-v2-subtext class="ml-2 text-xs text-text-weak">`), _tmpl$4$6 = /* @__PURE__ */ template(`<div role=button tabindex=0 aria-label="Close tab"data-slot=tabs-v2-close-button><svg width=14 height=14 viewBox="0 0 14 14"fill=none xmlns=http://www.w3.org/2000/svg><path d="M10.8889 3.11108L3.11108 10.8889"stroke=currentColor stroke-linejoin=round></path><path d="M3.11108 3.11108L10.8889 10.8889"stroke=currentColor stroke-linejoin=round>`), _tmpl$5$6 = /* @__PURE__ */ template(`<div data-slot=tabs-v2-section-title>`);
function TabsV2Root(props) {
  const [split, rest] = splitProps(props, ["class", "classList", "variant", "orientation"]);
  return createComponent(Tabs, mergeProps(rest, {
    get orientation() {
      return split.orientation;
    },
    "data-component": "tabs-v2",
    get ["data-variant"]() {
      return split.variant || "normal";
    },
    get ["data-orientation"]() {
      return split.orientation || "horizontal";
    },
    get classList() {
      return {
        ...split.classList,
        [split.class ?? ""]: !!split.class
      };
    }
  }));
}
function TabsV2List(props) {
  const [split, rest] = splitProps(props, ["class", "classList"]);
  return createComponent(Tabs.List, mergeProps(rest, {
    "data-slot": "tabs-v2-list",
    get classList() {
      return {
        ...split.classList,
        [split.class ?? ""]: !!split.class
      };
    }
  }));
}
function TabsV2Trigger(props) {
  const [split, rest] = splitProps(props, ["class", "classList", "children", "onMiddleClick", "subtext"]);
  return (() => {
    var _el$ = _tmpl$2$6();
    _el$.addEventListener("auxclick", (e) => {
      if (e.button === 1 && split.onMiddleClick) {
        e.preventDefault();
        split.onMiddleClick();
      }
    });
    _el$.$$mousedown = (e) => {
      if (e.button === 1 && split.onMiddleClick) {
        e.preventDefault();
      }
    };
    insert(_el$, createComponent(Tabs.Trigger, mergeProps(rest, {
      "data-slot": "tabs-v2-trigger",
      get ["data-value"]() {
        return props.value;
      },
      get children() {
        var _el$2 = _tmpl$$a();
        insert(_el$2, () => split.children, null);
        insert(_el$2, createComponent(Show, {
          get when() {
            return split.subtext;
          },
          children: (subtext) => (() => {
            var _el$3 = _tmpl$3$6();
            insert(_el$3, subtext);
            return _el$3;
          })()
        }), null);
        return _el$2;
      }
    })));
    createRenderEffect((_p$) => {
      var _v$ = props.value, _v$2 = {
        ...split.classList,
        [split.class ?? ""]: !!split.class
      };
      _v$ !== _p$.e && setAttribute(_el$, "data-value", _p$.e = _v$);
      _p$.t = classList(_el$, _v$2, _p$.t);
      return _p$;
    }, {
      e: void 0,
      t: void 0
    });
    return _el$;
  })();
}
function TabsV2CloseButton(props) {
  const [split, rest] = splitProps(props, ["class", "classList", "onClick"]);
  return (() => {
    var _el$4 = _tmpl$4$6();
    spread(_el$4, mergeProps(rest, {
      get classList() {
        return {
          [split.class ?? ""]: !!split.class,
          ...split.classList
        };
      },
      "onClick": (e) => {
        e.preventDefault();
        e.stopPropagation();
        if (typeof split.onClick === "function") {
          split.onClick(e);
        }
      },
      "onMouseDown": (e) => {
        e.preventDefault();
        e.stopPropagation();
      }
    }), false, true);
    return _el$4;
  })();
}
function TabsV2Content(props) {
  const [split, rest] = splitProps(props, ["class", "classList", "children"]);
  return createComponent(Tabs.Content, mergeProps(rest, {
    "data-slot": "tabs-v2-content",
    get classList() {
      return {
        ...split.classList,
        [split.class ?? ""]: !!split.class
      };
    },
    get children() {
      return split.children;
    }
  }));
}
const TabsV2SectionTitle = (props) => {
  return (() => {
    var _el$5 = _tmpl$5$6();
    insert(_el$5, () => props.children);
    return _el$5;
  })();
};
const TabsV2 = Object.assign(TabsV2Root, {
  List: TabsV2List,
  Trigger: TabsV2Trigger,
  CloseButton: TabsV2CloseButton,
  Content: TabsV2Content,
  SectionTitle: TabsV2SectionTitle
});
delegateEvents(["mousedown"]);
var _tmpl$$9 = /* @__PURE__ */ template(`<svg width=16 height=16 viewBox="0 0 16 16"fill=none xmlns=http://www.w3.org/2000/svg aria-hidden=true><path d="M11 9.5L8 6.5L5 9.5"stroke=currentColor stroke-width=1 stroke-linecap=round stroke-linejoin=round>`), _tmpl$2$5 = /* @__PURE__ */ template(`<svg width=14 height=14 viewBox="0 0 16 16"fill=none xmlns=http://www.w3.org/2000/svg aria-hidden=true><path d="M3.53564 8.17857L6.39279 11.75L12.4642 4.25"stroke=currentColor stroke-width=1 stroke-linecap=round stroke-linejoin=round>`), _tmpl$3$5 = /* @__PURE__ */ template(`<div data-slot=select-v2-value>`), _tmpl$4$5 = /* @__PURE__ */ template(`<span data-slot=select-v2-chevron aria-hidden=true>`), _tmpl$5$5 = /* @__PURE__ */ template(`<div data-slot=menu-v2-group-label>`);
function groupOptions(options, groupBy) {
  if (!groupBy) {
    return [{
      category: "",
      options
    }];
  }
  const map = /* @__PURE__ */ new Map();
  for (const opt of options) {
    const key = groupBy(opt);
    const arr = map.get(key);
    if (arr) arr.push(opt);
    else map.set(key, [opt]);
  }
  return [...map.entries()].map(([category, opts]) => ({
    category,
    options: opts
  }));
}
const ChevronDown = () => _tmpl$$9();
const CheckSmall = () => _tmpl$2$5();
function SelectV2(props) {
  const [local, others] = splitProps(props, ["class", "classList", "placeholder", "options", "current", "value", "label", "groupBy", "onSelect", "onHighlight", "onOpenChange", "children", "appearance", "invalid", "numeric", "disabled", "valueClass", "placement", "gutter", "sameWidth", "flip", "slide", "fitViewport"]);
  const inline = () => (local.appearance ?? "base") === "inline";
  const state = {};
  const stop = () => {
    state.cleanup?.();
    state.cleanup = void 0;
    state.key = void 0;
  };
  const keyFor = (item) => local.value ? local.value(item) : String(item);
  const move = (item) => {
    if (!local.onHighlight) return;
    if (!item) {
      stop();
      return;
    }
    const key = keyFor(item);
    if (state.key === key) return;
    state.cleanup?.();
    state.cleanup = local.onHighlight(item);
    state.key = key;
  };
  onCleanup(stop);
  const grouped = createMemo(() => groupOptions(local.options, local.groupBy));
  return createComponent(Select, mergeProps(others, {
    multiple: false,
    get disabled() {
      return local.disabled;
    },
    "data-component": "select-v2-root",
    get placement() {
      return local.placement ?? (inline() ? "bottom-end" : "bottom-start");
    },
    get gutter() {
      return local.gutter ?? 4;
    },
    get sameWidth() {
      return local.sameWidth ?? !inline();
    },
    get flip() {
      return local.flip ?? true;
    },
    get slide() {
      return local.slide ?? true;
    },
    get fitViewport() {
      return local.fitViewport ?? false;
    },
    get value() {
      return local.current;
    },
    get options() {
      return grouped();
    },
    optionValue: (x) => local.value ? local.value(x) : String(x),
    optionTextValue: (x) => local.label ? local.label(x) : String(x),
    optionGroupChildren: "options",
    get placeholder() {
      return local.placeholder;
    },
    sectionComponent: (sectionProps) => createComponent(Select.Section, {
      get children() {
        return createComponent(Show, {
          get when() {
            return sectionProps.section.rawValue.category;
          },
          get children() {
            var _el$5 = _tmpl$5$5();
            insert(_el$5, () => sectionProps.section.rawValue.category);
            return _el$5;
          }
        });
      }
    }),
    itemComponent: (itemProps) => createComponent(Select.Item, mergeProps(itemProps, {
      "data-component": "menu-v2-item",
      onPointerEnter: () => move(itemProps.item.rawValue),
      onPointerMove: () => move(itemProps.item.rawValue),
      onFocus: () => move(itemProps.item.rawValue),
      get children() {
        return [createComponent(Select.ItemLabel, {
          "data-slot": "menu-v2-item-content",
          as: "span",
          get children() {
            return memo(() => !!local.children)() ? local.children(itemProps.item.rawValue) : memo(() => !!local.label)() ? local.label(itemProps.item.rawValue) : String(itemProps.item.rawValue);
          }
        }), createComponent(Select.ItemIndicator, {
          "data-slot": "menu-v2-item-indicator",
          forceMount: true,
          get children() {
            return createComponent(CheckSmall, {});
          }
        })];
      }
    })),
    onChange: (next) => {
      const v = next == null ? null : Array.isArray(next) ? next[0] ?? null : next;
      local.onSelect?.(v);
      stop();
    },
    onOpenChange: (open) => {
      local.onOpenChange?.(open);
      if (!open) stop();
    },
    get children() {
      return [createComponent(Select.Trigger, {
        as: "div",
        "data-component": "select-v2",
        get ["data-appearance"]() {
          return local.appearance ?? "base";
        },
        get ["data-invalid"]() {
          return local.invalid ? "" : void 0;
        },
        get ["data-numeric"]() {
          return local.numeric ? "" : void 0;
        },
        get disabled() {
          return local.disabled;
        },
        get ["data-disabled"]() {
          return local.disabled ? "" : void 0;
        },
        get classList() {
          return {
            ...local.classList,
            [local.class ?? ""]: !!local.class
          };
        },
        get children() {
          return [(() => {
            var _el$3 = _tmpl$3$5();
            insert(_el$3, createComponent(Select.Value, {
              "data-slot": "select-v2-value-text",
              get ["class"]() {
                return local.valueClass;
              },
              children: (st) => {
                const selected = st.selectedOption();
                if (local.label && selected != null) return local.label(selected);
                return selected != null ? selected : "";
              }
            }));
            return _el$3;
          })(), (() => {
            var _el$4 = _tmpl$4$5();
            insert(_el$4, createComponent(ChevronDown, {}));
            return _el$4;
          })()];
        }
      }), createComponent(Select.Portal, {
        get children() {
          return createComponent(Select.Content, {
            "data-component": "menu-v2-content",
            "data-slot": "select-v2-content",
            get children() {
              return createComponent(Select.Listbox, {
                "data-slot": "select-v2-listbox"
              });
            }
          });
        }
      })];
    }
  }));
}
function Switch(props) {
  const [local, others] = splitProps(props, ["children", "class", "hideLabel"]);
  return createComponent(Switch$1, mergeProps(others, {
    get ["class"]() {
      return local.class;
    },
    "data-component": "switch",
    get children() {
      return [createComponent(Switch$1.Input, {
        "data-slot": "switch-input"
      }), createComponent(Show, {
        get when() {
          return local.children;
        },
        children: (label) => createComponent(Switch$1.Label, {
          "data-slot": "switch-label",
          get classList() {
            return {
              "sr-only": local.hideLabel
            };
          },
          get children() {
            return label();
          }
        })
      }), createComponent(Switch$1.Control, {
        "data-slot": "switch-control",
        get children() {
          return createComponent(Switch$1.Thumb, {
            "data-slot": "switch-thumb"
          });
        }
      }), createComponent(Switch$1.ErrorMessage, {
        "data-slot": "switch-error"
      })];
    }
  }));
}
var _tmpl$$8 = /* @__PURE__ */ template(`<div data-component=settings-v2-row><div data-slot=settings-v2-row-copy><div data-slot=settings-v2-row-title></div><div data-slot=settings-v2-row-description></div></div><div data-slot=settings-v2-row-control>`);
const SettingsRowV2 = (props) => {
  return (() => {
    var _el$ = _tmpl$$8(), _el$2 = _el$.firstChild, _el$3 = _el$2.firstChild, _el$4 = _el$3.nextSibling, _el$5 = _el$2.nextSibling;
    insert(_el$3, () => props.title);
    insert(_el$4, () => props.description);
    insert(_el$5, () => props.children);
    return _el$;
  })();
};
var _tmpl$$7 = /* @__PURE__ */ template(`<div data-action=settings-auto-accept-permissions>`), _tmpl$2$4 = /* @__PURE__ */ template(`<div data-action=settings-feed-reasoning-summaries>`), _tmpl$3$4 = /* @__PURE__ */ template(`<div data-action=settings-feed-shell-tool-parts-expanded>`), _tmpl$4$4 = /* @__PURE__ */ template(`<div data-action=settings-feed-edit-tool-parts-expanded>`), _tmpl$5$4 = /* @__PURE__ */ template(`<div data-action=settings-new-layout-designs>`), _tmpl$6$4 = /* @__PURE__ */ template(`<div data-action=settings-mobile-titlebar-bottom>`), _tmpl$7$3 = /* @__PURE__ */ template(`<div class=settings-v2-section>`), _tmpl$8$2 = /* @__PURE__ */ template(`<div data-action=settings-show-file-tree>`), _tmpl$9$1 = /* @__PURE__ */ template(`<div data-action=settings-show-search>`), _tmpl$0$1 = /* @__PURE__ */ template(`<div data-action=settings-show-status>`), _tmpl$1$1 = /* @__PURE__ */ template(`<div data-action=settings-show-custom-agents>`), _tmpl$10$1 = /* @__PURE__ */ template(`<div class=settings-v2-section><h3 class=settings-v2-section-title>`), _tmpl$11$1 = /* @__PURE__ */ template(`<div class="w-full sm:w-[220px]">`), _tmpl$12$1 = /* @__PURE__ */ template(`<div data-action=settings-notifications-agent>`), _tmpl$13$1 = /* @__PURE__ */ template(`<div data-action=settings-notifications-permissions>`), _tmpl$14$1 = /* @__PURE__ */ template(`<div data-action=settings-notifications-errors>`), _tmpl$15$1 = /* @__PURE__ */ template(`<div data-action=settings-release-notes>`), _tmpl$16 = /* @__PURE__ */ template(`<div data-action=settings-pinch-zoom>`), _tmpl$17 = /* @__PURE__ */ template(`<div class=settings-v2-tab-header><h2 class=settings-v2-tab-title>`), _tmpl$18 = /* @__PURE__ */ template(`<div class=settings-v2-tab-body>`);
let demoSoundState = {
  cleanup: void 0,
  timeout: void 0,
  run: 0
};
const stopDemoSound = () => {
  demoSoundState.run += 1;
  if (demoSoundState.cleanup) {
    demoSoundState.cleanup();
  }
  clearTimeout(demoSoundState.timeout);
  demoSoundState.cleanup = void 0;
};
const playDemoSound = (id) => {
  stopDemoSound();
  if (!id) return;
  const run = ++demoSoundState.run;
  demoSoundState.timeout = setTimeout(() => {
    void playSoundById(id).then((cleanup) => {
      if (demoSoundState.run !== run) {
        cleanup?.();
        return;
      }
      demoSoundState.cleanup = cleanup;
    });
  }, 100);
};
const SettingsGeneralV2 = () => {
  const theme = useTheme();
  const language = useLanguage();
  const permission = usePermission();
  const platform = usePlatform();
  const dialog = useDialog();
  const params = useParams();
  const settings = useSettings();
  const mobile = createMediaQuery("(max-width: 767px)");
  const updater = useUpdaterAction();
  const dir = createMemo(() => decode64(params.dir));
  const accepting = createMemo(() => {
    const value = dir();
    if (!value) return false;
    if (!params.id) return permission.isAutoAcceptingDirectory(value);
    return permission.isAutoAccepting(params.id, value);
  });
  const toggleAccept = (checked) => {
    const value = dir();
    if (!value) return;
    if (!params.id) {
      if (permission.isAutoAcceptingDirectory(value) === checked) return;
      permission.toggleAutoAcceptDirectory(value);
      return;
    }
    if (checked) {
      permission.enableAutoAccept(params.id, value);
      return;
    }
    permission.disableAutoAccept(params.id, value);
  };
  const desktop = createMemo(() => platform.platform === "desktop");
  const themeOptions = createMemo(() => theme.ids().map((id) => ({
    id,
    name: theme.name(id)
  })));
  const serverSync = useServerSync();
  const serverSdk = useServerSDK();
  const [shells] = createResource(() => serverSdk().client.pty.shells().then((res) => res.data ?? []).catch(() => []), {
    initialValue: []
  });
  const [pinchZoom, {
    mutate: setPinchZoom
  }] = createResource(() => desktop() && platform.getPinchZoomEnabled ? true : false, () => Promise.resolve(platform.getPinchZoomEnabled?.() ?? false).catch(() => false), {
    initialValue: false
  });
  onMount(() => {
    void theme.loadThemes();
  });
  const autoOption = {
    id: "auto",
    value: "",
    label: language.t("settings.general.row.shell.autoDefault")
  };
  const currentShell = createMemo(() => serverSync().data.config.shell ?? "");
  const shellOptions = createMemo(() => {
    const list = shells.latest;
    const current = serverSync().data.config.shell;
    const nameCounts = /* @__PURE__ */ new Map();
    for (const s of list) {
      nameCounts.set(s.name, (nameCounts.get(s.name) || 0) + 1);
    }
    const options = [autoOption, ...list.map((s) => {
      const ambiguousName = (nameCounts.get(s.name) || 0) > 1;
      const text = ambiguousName ? s.path : s.name;
      const label = s.acceptable ? text : `${text} (${language.t("settings.general.row.shell.terminalOnly")})`;
      return {
        id: s.path,
        // Prefer name over path - "bash" is much cleaner than the explicit full route even when it may change due to PATH.
        value: ambiguousName ? s.path : s.name,
        label
      };
    })];
    if (current && !options.some((o) => o.value === current)) {
      options.push({
        id: current,
        value: current,
        label: current
      });
    }
    return options;
  });
  const onPinchZoomChange = (checked) => {
    setPinchZoom(checked);
    const update = platform.setPinchZoomEnabled?.(checked);
    if (!update) return;
    void update.catch(() => setPinchZoom(!checked));
  };
  const colorSchemeOptions = createMemo(() => [{
    value: "system",
    label: language.t("theme.scheme.system")
  }, {
    value: "light",
    label: language.t("theme.scheme.light")
  }, {
    value: "dark",
    label: language.t("theme.scheme.dark")
  }]);
  const languageOptions = createMemo(() => language.locales.map((locale) => ({
    value: locale,
    label: language.label(locale)
  })));
  const noneSound = {
    id: "none",
    label: "sound.option.none"
  };
  const soundOptions = [noneSound, ...SOUND_OPTIONS];
  const mono = () => monoInput(settings.appearance.font());
  const sans = () => sansInput(settings.appearance.uiFont());
  const terminal = () => terminalInput(settings.appearance.terminalFont());
  const soundSelectProps = (enabled, current, setEnabled, set) => ({
    options: soundOptions,
    current: enabled() ? soundOptions.find((o) => o.id === current()) ?? noneSound : noneSound,
    value: (o) => o.id,
    label: (o) => language.t(o.label),
    onHighlight: (option) => {
      if (!option) return;
      playDemoSound(option.id === "none" ? void 0 : option.id);
    },
    onSelect: (option) => {
      if (!option) return;
      if (option.id === "none") {
        setEnabled(false);
        stopDemoSound();
        return;
      }
      setEnabled(true);
      set(option.id);
      playDemoSound(option.id);
    }
  });
  const GeneralSection = () => (() => {
    var _el$ = _tmpl$7$3();
    insert(_el$, createComponent(SettingsListV2, {
      get children() {
        return [createComponent(SettingsRowV2, {
          get title() {
            return language.t("settings.general.row.language.title");
          },
          get description() {
            return language.t("settings.general.row.language.description");
          },
          get children() {
            return createComponent(SelectV2, {
              appearance: "inline",
              "data-action": "settings-language",
              get options() {
                return languageOptions();
              },
              placement: "bottom-end",
              gutter: 6,
              get current() {
                return languageOptions().find((o) => o.value === language.locale());
              },
              value: (o) => o.value,
              label: (o) => o.label,
              onSelect: (option) => option && language.setLocale(option.value)
            });
          }
        }), createComponent(SettingsRowV2, {
          get title() {
            return language.t("command.permissions.autoaccept.enable");
          },
          get description() {
            return language.t("toast.permissions.autoaccept.on.description");
          },
          get children() {
            var _el$2 = _tmpl$$7();
            insert(_el$2, createComponent(Switch, {
              get checked() {
                return accepting();
              },
              get disabled() {
                return !dir();
              },
              onChange: toggleAccept
            }));
            return _el$2;
          }
        }), createComponent(SettingsRowV2, {
          get title() {
            return language.t("settings.general.row.shell.title");
          },
          get description() {
            return language.t("settings.general.row.shell.description");
          },
          get children() {
            return createComponent(SelectV2, {
              appearance: "inline",
              "data-action": "settings-shell",
              get options() {
                return shellOptions();
              },
              get current() {
                return shellOptions().find((o) => o.value === currentShell()) ?? autoOption;
              },
              placement: "bottom-end",
              gutter: 6,
              value: (o) => o.id,
              label: (o) => o.label,
              onSelect: (option) => {
                if (!option) return;
                if (option.value === currentShell()) return;
                serverSync().updateConfig({
                  shell: option.value
                });
              }
            });
          }
        }), createComponent(SettingsRowV2, {
          get title() {
            return language.t("settings.general.row.reasoningSummaries.title");
          },
          get description() {
            return language.t("settings.general.row.reasoningSummaries.description");
          },
          get children() {
            var _el$3 = _tmpl$2$4();
            insert(_el$3, createComponent(Switch, {
              get checked() {
                return settings.general.showReasoningSummaries();
              },
              onChange: (checked) => settings.general.setShowReasoningSummaries(checked)
            }));
            return _el$3;
          }
        }), createComponent(SettingsRowV2, {
          get title() {
            return language.t("settings.general.row.shellToolPartsExpanded.title");
          },
          get description() {
            return language.t("settings.general.row.shellToolPartsExpanded.description");
          },
          get children() {
            var _el$4 = _tmpl$3$4();
            insert(_el$4, createComponent(Switch, {
              get checked() {
                return settings.general.shellToolPartsExpanded();
              },
              onChange: (checked) => settings.general.setShellToolPartsExpanded(checked)
            }));
            return _el$4;
          }
        }), createComponent(SettingsRowV2, {
          get title() {
            return language.t("settings.general.row.editToolPartsExpanded.title");
          },
          get description() {
            return language.t("settings.general.row.editToolPartsExpanded.description");
          },
          get children() {
            var _el$5 = _tmpl$4$4();
            insert(_el$5, createComponent(Switch, {
              get checked() {
                return settings.general.editToolPartsExpanded();
              },
              onChange: (checked) => settings.general.setEditToolPartsExpanded(checked)
            }));
            return _el$5;
          }
        }), createComponent(SettingsRowV2, {
          get title() {
            return language.t("settings.general.row.newLayoutDesigns.title");
          },
          get description() {
            return language.t("settings.general.row.newLayoutDesigns.description");
          },
          get children() {
            var _el$6 = _tmpl$5$4();
            insert(_el$6, createComponent(Switch, {
              get checked() {
                return settings.general.newLayoutDesigns();
              },
              onChange: (checked) => {
                settings.general.setNewLayoutDesigns(checked);
                if (checked) return;
                void __vitePreload(() => import("./dialog-settings-BAWf-Z5x.js"), true ? __vite__mapDeps([0,1,2,3,4,5,6,7]) : void 0, import.meta.url).then((module) => {
                  dialog.show(() => createComponent(module.DialogSettings, {}));
                });
              }
            }));
            return _el$6;
          }
        }), createComponent(Show, {
          get when() {
            return mobile();
          },
          get children() {
            return createComponent(SettingsRowV2, {
              get title() {
                return language.t("settings.general.row.mobileTitlebarBottom.title");
              },
              get description() {
                return language.t("settings.general.row.mobileTitlebarBottom.description");
              },
              get children() {
                var _el$7 = _tmpl$6$4();
                insert(_el$7, createComponent(Switch, {
                  get checked() {
                    return settings.general.mobileTitlebarPosition() === "bottom";
                  },
                  onChange: (checked) => settings.general.setMobileTitlebarPosition(checked ? "bottom" : "top")
                }));
                return _el$7;
              }
            });
          }
        })];
      }
    }));
    return _el$;
  })();
  const AdvancedSection = () => (() => {
    var _el$8 = _tmpl$10$1(), _el$9 = _el$8.firstChild;
    insert(_el$9, () => language.t("settings.general.section.advanced"));
    insert(_el$8, createComponent(SettingsListV2, {
      get children() {
        return [createComponent(SettingsRowV2, {
          get title() {
            return language.t("settings.general.row.showFileTree.title");
          },
          get description() {
            return language.t("settings.general.row.showFileTree.description");
          },
          get children() {
            var _el$0 = _tmpl$8$2();
            insert(_el$0, createComponent(Switch, {
              get checked() {
                return settings.general.showFileTree();
              },
              onChange: (checked) => settings.general.setShowFileTree(checked)
            }));
            return _el$0;
          }
        }), createComponent(SettingsRowV2, {
          get title() {
            return language.t("settings.general.row.showSearch.title");
          },
          get description() {
            return language.t("settings.general.row.showSearch.description");
          },
          get children() {
            var _el$1 = _tmpl$9$1();
            insert(_el$1, createComponent(Switch, {
              get checked() {
                return settings.general.showSearch();
              },
              onChange: (checked) => settings.general.setShowSearch(checked)
            }));
            return _el$1;
          }
        }), createComponent(SettingsRowV2, {
          get title() {
            return language.t("settings.general.row.showStatus.title");
          },
          get description() {
            return language.t("settings.general.row.showStatus.description");
          },
          get children() {
            var _el$10 = _tmpl$0$1();
            insert(_el$10, createComponent(Switch, {
              get checked() {
                return settings.general.showStatus();
              },
              onChange: (checked) => settings.general.setShowStatus(checked)
            }));
            return _el$10;
          }
        }), createComponent(SettingsRowV2, {
          get title() {
            return language.t("settings.general.row.showCustomAgents.title");
          },
          get description() {
            return language.t("settings.general.row.showCustomAgents.description");
          },
          get children() {
            var _el$11 = _tmpl$1$1();
            insert(_el$11, createComponent(Switch, {
              get checked() {
                return settings.general.showCustomAgents();
              },
              onChange: (checked) => settings.general.setShowCustomAgents(checked)
            }));
            return _el$11;
          }
        })];
      }
    }), null);
    return _el$8;
  })();
  const AppearanceSection = () => (() => {
    var _el$12 = _tmpl$10$1(), _el$13 = _el$12.firstChild;
    insert(_el$13, () => language.t("settings.general.section.appearance"));
    insert(_el$12, createComponent(SettingsListV2, {
      get children() {
        return [createComponent(SettingsRowV2, {
          get title() {
            return language.t("settings.general.row.colorScheme.title");
          },
          get description() {
            return language.t("settings.general.row.colorScheme.description");
          },
          get children() {
            return createComponent(SelectV2, {
              appearance: "inline",
              "data-action": "settings-color-scheme",
              get options() {
                return colorSchemeOptions();
              },
              get current() {
                return colorSchemeOptions().find((o) => o.value === theme.colorScheme());
              },
              placement: "bottom-end",
              gutter: 6,
              value: (o) => o.value,
              label: (o) => o.label,
              onSelect: (option) => option && theme.setColorScheme(option.value),
              onHighlight: (option) => {
                if (!option) return;
                theme.previewColorScheme(option.value);
                return () => theme.cancelPreview();
              }
            });
          }
        }), createComponent(SettingsRowV2, {
          get title() {
            return language.t("settings.general.row.theme.title");
          },
          get description() {
            return [memo(() => language.t("settings.general.row.theme.description")), " ", createComponent(Link, {
              "class": "settings-v2-link",
              href: "https://opencode.ai/docs/themes/",
              get children() {
                return language.t("common.learnMore");
              }
            })];
          },
          get children() {
            return createComponent(SelectV2, {
              appearance: "inline",
              "data-action": "settings-theme",
              get options() {
                return themeOptions();
              },
              get current() {
                return themeOptions().find((o) => o.id === theme.themeId());
              },
              placement: "bottom-end",
              gutter: 6,
              value: (o) => o.id,
              label: (o) => o.name,
              onSelect: (option) => {
                if (!option) return;
                theme.setTheme(option.id);
              },
              onHighlight: (option) => {
                if (!option) return;
                theme.previewTheme(option.id);
                return () => theme.cancelPreview();
              }
            });
          }
        }), createComponent(SettingsRowV2, {
          get title() {
            return language.t("settings.general.row.uiFont.title");
          },
          get description() {
            return language.t("settings.general.row.uiFont.description");
          },
          get children() {
            var _el$14 = _tmpl$11$1();
            insert(_el$14, createComponent(TextInputV2, {
              "data-action": "settings-ui-font",
              type: "text",
              appearance: "base",
              get value() {
                return sans();
              },
              onInput: (event) => settings.appearance.setUIFont(event.currentTarget.value),
              placeholder: sansDefault,
              spellcheck: false,
              autocorrect: "off",
              autocomplete: "off",
              autocapitalize: "off",
              get ["aria-label"]() {
                return language.t("settings.general.row.uiFont.title");
              },
              get style() {
                return {
                  "font-family": sansFontFamily(settings.appearance.uiFont())
                };
              }
            }));
            return _el$14;
          }
        }), createComponent(SettingsRowV2, {
          get title() {
            return language.t("settings.general.row.font.title");
          },
          get description() {
            return language.t("settings.general.row.font.description");
          },
          get children() {
            var _el$15 = _tmpl$11$1();
            insert(_el$15, createComponent(TextInputV2, {
              "data-action": "settings-code-font",
              type: "text",
              appearance: "base",
              get value() {
                return mono();
              },
              onInput: (event) => settings.appearance.setFont(event.currentTarget.value),
              placeholder: monoDefault,
              spellcheck: false,
              autocorrect: "off",
              autocomplete: "off",
              autocapitalize: "off",
              get ["aria-label"]() {
                return language.t("settings.general.row.font.title");
              },
              get style() {
                return {
                  "font-family": monoFontFamily(settings.appearance.font())
                };
              }
            }));
            return _el$15;
          }
        }), createComponent(SettingsRowV2, {
          get title() {
            return language.t("settings.general.row.terminalFont.title");
          },
          get description() {
            return language.t("settings.general.row.terminalFont.description");
          },
          get children() {
            var _el$16 = _tmpl$11$1();
            insert(_el$16, createComponent(TextInputV2, {
              "data-action": "settings-terminal-font",
              type: "text",
              appearance: "base",
              get value() {
                return terminal();
              },
              onInput: (event) => settings.appearance.setTerminalFont(event.currentTarget.value),
              placeholder: terminalDefault,
              spellcheck: false,
              autocorrect: "off",
              autocomplete: "off",
              autocapitalize: "off",
              get ["aria-label"]() {
                return language.t("settings.general.row.terminalFont.title");
              },
              get style() {
                return {
                  "font-family": terminalFontFamily(settings.appearance.terminalFont())
                };
              }
            }));
            return _el$16;
          }
        })];
      }
    }), null);
    return _el$12;
  })();
  const NotificationsSection = () => (() => {
    var _el$17 = _tmpl$10$1(), _el$18 = _el$17.firstChild;
    insert(_el$18, () => language.t("settings.general.section.notifications"));
    insert(_el$17, createComponent(SettingsListV2, {
      get children() {
        return [createComponent(SettingsRowV2, {
          get title() {
            return language.t("settings.general.notifications.agent.title");
          },
          get description() {
            return language.t("settings.general.notifications.agent.description");
          },
          get children() {
            var _el$19 = _tmpl$12$1();
            insert(_el$19, createComponent(Switch, {
              get checked() {
                return settings.notifications.agent();
              },
              onChange: (checked) => settings.notifications.setAgent(checked)
            }));
            return _el$19;
          }
        }), createComponent(SettingsRowV2, {
          get title() {
            return language.t("settings.general.notifications.permissions.title");
          },
          get description() {
            return language.t("settings.general.notifications.permissions.description");
          },
          get children() {
            var _el$20 = _tmpl$13$1();
            insert(_el$20, createComponent(Switch, {
              get checked() {
                return settings.notifications.permissions();
              },
              onChange: (checked) => settings.notifications.setPermissions(checked)
            }));
            return _el$20;
          }
        }), createComponent(SettingsRowV2, {
          get title() {
            return language.t("settings.general.notifications.errors.title");
          },
          get description() {
            return language.t("settings.general.notifications.errors.description");
          },
          get children() {
            var _el$21 = _tmpl$14$1();
            insert(_el$21, createComponent(Switch, {
              get checked() {
                return settings.notifications.errors();
              },
              onChange: (checked) => settings.notifications.setErrors(checked)
            }));
            return _el$21;
          }
        })];
      }
    }), null);
    return _el$17;
  })();
  const SoundsSection = () => (() => {
    var _el$22 = _tmpl$10$1(), _el$23 = _el$22.firstChild;
    insert(_el$23, () => language.t("settings.general.section.sounds"));
    insert(_el$22, createComponent(SettingsListV2, {
      get children() {
        return [createComponent(SettingsRowV2, {
          get title() {
            return language.t("settings.general.sounds.agent.title");
          },
          get description() {
            return language.t("settings.general.sounds.agent.description");
          },
          get children() {
            return createComponent(SelectV2, mergeProps({
              appearance: "inline",
              "data-action": "settings-sounds-agent"
            }, () => soundSelectProps(() => settings.sounds.agentEnabled(), () => settings.sounds.agent(), (value) => settings.sounds.setAgentEnabled(value), (id) => settings.sounds.setAgent(id)), {
              placement: "bottom-end",
              gutter: 6
            }));
          }
        }), createComponent(SettingsRowV2, {
          get title() {
            return language.t("settings.general.sounds.permissions.title");
          },
          get description() {
            return language.t("settings.general.sounds.permissions.description");
          },
          get children() {
            return createComponent(SelectV2, mergeProps({
              appearance: "inline",
              "data-action": "settings-sounds-permissions"
            }, () => soundSelectProps(() => settings.sounds.permissionsEnabled(), () => settings.sounds.permissions(), (value) => settings.sounds.setPermissionsEnabled(value), (id) => settings.sounds.setPermissions(id)), {
              placement: "bottom-end",
              gutter: 6
            }));
          }
        }), createComponent(SettingsRowV2, {
          get title() {
            return language.t("settings.general.sounds.errors.title");
          },
          get description() {
            return language.t("settings.general.sounds.errors.description");
          },
          get children() {
            return createComponent(SelectV2, mergeProps({
              appearance: "inline",
              "data-action": "settings-sounds-errors"
            }, () => soundSelectProps(() => settings.sounds.errorsEnabled(), () => settings.sounds.errors(), (value) => settings.sounds.setErrorsEnabled(value), (id) => settings.sounds.setErrors(id)), {
              placement: "bottom-end",
              gutter: 6
            }));
          }
        })];
      }
    }), null);
    return _el$22;
  })();
  const UpdatesSection = () => (() => {
    var _el$24 = _tmpl$10$1(), _el$25 = _el$24.firstChild;
    insert(_el$25, () => language.t("settings.general.section.updates"));
    insert(_el$24, createComponent(SettingsListV2, {
      get children() {
        return [createComponent(SettingsRowV2, {
          get title() {
            return language.t("settings.general.row.releaseNotes.title");
          },
          get description() {
            return language.t("settings.general.row.releaseNotes.description");
          },
          get children() {
            var _el$26 = _tmpl$15$1();
            insert(_el$26, createComponent(Switch, {
              get checked() {
                return settings.general.releaseNotes();
              },
              onChange: (checked) => settings.general.setReleaseNotes(checked)
            }));
            return _el$26;
          }
        }), createComponent(SettingsRowV2, {
          get title() {
            return language.t("settings.updates.row.check.title");
          },
          get description() {
            return language.t("settings.updates.row.check.description");
          },
          get children() {
            return createComponent(ButtonV2, {
              size: "normal",
              variant: "neutral",
              get disabled() {
                return !updater.action().run;
              },
              get onClick() {
                return updater.run;
              },
              get children() {
                return language.t(updater.action().label);
              }
            });
          }
        })];
      }
    }), null);
    return _el$24;
  })();
  const DisplaySection = () => createComponent(Show, {
    get when() {
      return desktop();
    },
    get children() {
      var _el$27 = _tmpl$10$1(), _el$28 = _el$27.firstChild;
      insert(_el$28, () => language.t("settings.general.section.display"));
      insert(_el$27, createComponent(SettingsListV2, {
        get children() {
          return createComponent(SettingsRowV2, {
            get title() {
              return language.t("settings.general.row.pinchZoom.title");
            },
            get description() {
              return language.t("settings.general.row.pinchZoom.description");
            },
            get children() {
              var _el$29 = _tmpl$16();
              insert(_el$29, createComponent(Switch, {
                get checked() {
                  return pinchZoom.latest;
                },
                onChange: onPinchZoomChange
              }));
              return _el$29;
            }
          });
        }
      }), null);
      return _el$27;
    }
  });
  return [(() => {
    var _el$30 = _tmpl$17(), _el$31 = _el$30.firstChild;
    insert(_el$31, () => language.t("settings.tab.general"));
    return _el$30;
  })(), (() => {
    var _el$32 = _tmpl$18();
    insert(_el$32, createComponent(GeneralSection, {}), null);
    insert(_el$32, createComponent(AppearanceSection, {}), null);
    insert(_el$32, createComponent(NotificationsSection, {}), null);
    insert(_el$32, createComponent(SoundsSection, {}), null);
    insert(_el$32, createComponent(Show, {
      get when() {
        return desktop();
      },
      get children() {
        return createComponent(UpdatesSection, {});
      }
    }), null);
    insert(_el$32, createComponent(DisplaySection, {}), null);
    insert(_el$32, createComponent(AdvancedSection, {}), null);
    return _el$32;
  })()];
};
var _tmpl$$6 = /* @__PURE__ */ template(`<span>`);
function Tag(props) {
  const [split, rest] = splitProps(props, ["class", "classList", "children"]);
  return (() => {
    var _el$ = _tmpl$$6();
    spread(_el$, mergeProps(rest, {
      "data-component": "tag",
      get classList() {
        return {
          ...split.classList,
          [split.class ?? ""]: !!split.class
        };
      }
    }), false, true);
    insert(_el$, () => split.children);
    return _el$;
  })();
}
var _tmpl$$5 = /* @__PURE__ */ template(`<div class=settings-v2-tab-header><h2 class=settings-v2-tab-title>`), _tmpl$2$3 = /* @__PURE__ */ template(`<div class=settings-v2-provider-row data-component=custom-provider-section><div class=settings-v2-provider-lead><div class=settings-v2-provider-copy><div class=settings-v2-provider-main><span class=settings-v2-provider-name></span></div><p class=settings-v2-provider-description>`), _tmpl$3$3 = /* @__PURE__ */ template(`<div class="settings-v2-tab-body settings-v2-providers"><div class=settings-v2-section data-component=connected-providers-section><h3 class=settings-v2-section-title></h3></div><div class=settings-v2-section><h3 class=settings-v2-section-title></h3><button type=button class=settings-v2-providers-view-all>`), _tmpl$4$3 = /* @__PURE__ */ template(`<div class=settings-v2-provider-empty>`), _tmpl$5$3 = /* @__PURE__ */ template(`<div class="settings-v2-provider-row group"><div class=settings-v2-provider-lead><div class=settings-v2-provider-main><span class="settings-v2-provider-name truncate">`), _tmpl$6$3 = /* @__PURE__ */ template(`<span class=settings-v2-provider-env-hint>`), _tmpl$7$2 = /* @__PURE__ */ template(`<div class=settings-v2-provider-row><div class=settings-v2-provider-lead><div class=settings-v2-provider-copy><div class=settings-v2-provider-main><span class=settings-v2-provider-name>`), _tmpl$8$1 = /* @__PURE__ */ template(`<p class=settings-v2-provider-description>`);
const PROVIDER_NOTES = [{
  match: (id) => id === "opencode",
  key: "dialog.provider.opencode.note"
}, {
  match: (id) => id === "opencode-go",
  key: "dialog.provider.opencodeGo.tagline"
}, {
  match: (id) => id === "anthropic",
  key: "dialog.provider.anthropic.note"
}, {
  match: (id) => id.startsWith("github-copilot"),
  key: "dialog.provider.copilot.note"
}, {
  match: (id) => id === "openai",
  key: "dialog.provider.openai.note"
}, {
  match: (id) => id === "google",
  key: "dialog.provider.google.note"
}, {
  match: (id) => id === "openrouter",
  key: "dialog.provider.openrouter.note"
}, {
  match: (id) => id === "vercel",
  key: "dialog.provider.vercel.note"
}];
const PROVIDER_ICON_SIZE$1 = 16;
const SettingsProvidersV2 = () => {
  const dialog = useDialog();
  const language = useLanguage();
  const serverSdk = useServerSDK();
  const serverSync = useServerSync();
  const providers = useProviders();
  const connected = createMemo(() => {
    return providers.connected().filter((p) => p.id !== "opencode" || Object.values(p.models).find((m) => m.cost?.input));
  });
  const popular = createMemo(() => {
    const connectedIDs = new Set(connected().map((p) => p.id));
    const items = providers.popular().filter((p) => !connectedIDs.has(p.id)).slice();
    items.sort((a, b) => popularProviders.indexOf(a.id) - popularProviders.indexOf(b.id));
    return items;
  });
  const source = (item) => {
    if (!("source" in item)) return;
    const value = item.source;
    if (value === "env" || value === "api" || value === "config" || value === "custom") return value;
    return;
  };
  const type = (item) => {
    const current = source(item);
    if (current === "env") return language.t("settings.providers.tag.environment");
    if (current === "api") return language.t("provider.connect.method.apiKey");
    if (current === "config") {
      if (isConfigCustom(item.id)) return language.t("settings.providers.tag.custom");
      return language.t("settings.providers.tag.config");
    }
    if (current === "custom") return language.t("settings.providers.tag.custom");
    return language.t("settings.providers.tag.other");
  };
  const canDisconnect = (item) => source(item) !== "env";
  const note = (id) => PROVIDER_NOTES.find((item) => item.match(id))?.key;
  const isConfigCustom = (providerID) => {
    const provider = serverSync().data.config.provider?.[providerID];
    if (!provider) return false;
    if (provider.npm !== "@ai-sdk/openai-compatible") return false;
    if (!provider.models || Object.keys(provider.models).length === 0) return false;
    return true;
  };
  const disableProvider = async (providerID, name) => {
    const before = serverSync().data.config.disabled_providers ?? [];
    const next = before.includes(providerID) ? before : [...before, providerID];
    serverSync().set("config", "disabled_providers", next);
    await serverSync().updateConfig({
      disabled_providers: next
    }).then(() => {
      showToast({
        variant: "success",
        icon: "circle-check",
        title: language.t("provider.disconnect.toast.disconnected.title", {
          provider: name
        }),
        description: language.t("provider.disconnect.toast.disconnected.description", {
          provider: name
        })
      });
    }).catch((err) => {
      serverSync().set("config", "disabled_providers", before);
      const message = err instanceof Error ? err.message : String(err);
      showToast({
        title: language.t("common.requestFailed"),
        description: message
      });
    });
  };
  const disconnect = async (providerID, name) => {
    if (isConfigCustom(providerID)) {
      await serverSdk().client.auth.remove({
        providerID
      }).catch(() => void 0);
      await disableProvider(providerID, name);
      return;
    }
    await serverSdk().client.auth.remove({
      providerID
    }).then(async () => {
      await serverSdk().client.global.dispose();
      showToast({
        variant: "success",
        icon: "circle-check",
        title: language.t("provider.disconnect.toast.disconnected.title", {
          provider: name
        }),
        description: language.t("provider.disconnect.toast.disconnected.description", {
          provider: name
        })
      });
    }).catch((err) => {
      const message = err instanceof Error ? err.message : String(err);
      showToast({
        title: language.t("common.requestFailed"),
        description: message
      });
    });
  };
  return [(() => {
    var _el$ = _tmpl$$5(), _el$2 = _el$.firstChild;
    insert(_el$2, () => language.t("settings.providers.title"));
    return _el$;
  })(), (() => {
    var _el$3 = _tmpl$3$3(), _el$4 = _el$3.firstChild, _el$5 = _el$4.firstChild, _el$6 = _el$4.nextSibling, _el$7 = _el$6.firstChild, _el$12 = _el$7.nextSibling;
    insert(_el$5, () => language.t("settings.providers.section.connected"));
    insert(_el$4, createComponent(SettingsListV2, {
      get children() {
        return createComponent(Show, {
          get when() {
            return connected().length > 0;
          },
          get fallback() {
            return (() => {
              var _el$13 = _tmpl$4$3();
              insert(_el$13, () => language.t("settings.providers.connected.empty"));
              return _el$13;
            })();
          },
          get children() {
            return createComponent(For, {
              get each() {
                return connected();
              },
              children: (item) => (() => {
                var _el$14 = _tmpl$5$3(), _el$15 = _el$14.firstChild, _el$16 = _el$15.firstChild, _el$17 = _el$16.firstChild;
                insert(_el$15, createComponent(ProviderIcon, {
                  get id() {
                    return item.id;
                  },
                  width: PROVIDER_ICON_SIZE$1,
                  height: PROVIDER_ICON_SIZE$1,
                  "class": "settings-v2-provider-icon shrink-0"
                }), _el$16);
                insert(_el$17, () => item.name);
                insert(_el$16, createComponent(Tag, {
                  get children() {
                    return type(item);
                  }
                }), null);
                insert(_el$14, createComponent(Show, {
                  get when() {
                    return canDisconnect(item);
                  },
                  get fallback() {
                    return (() => {
                      var _el$18 = _tmpl$6$3();
                      insert(_el$18, () => language.t("settings.providers.connected.environmentDescription"));
                      return _el$18;
                    })();
                  },
                  get children() {
                    return createComponent(ButtonV2, {
                      size: "normal",
                      variant: "ghost-muted",
                      onClick: () => void disconnect(item.id, item.name),
                      get children() {
                        return language.t("common.disconnect");
                      }
                    });
                  }
                }), null);
                return _el$14;
              })()
            });
          }
        });
      }
    }), null);
    insert(_el$7, () => language.t("settings.providers.section.popular"));
    insert(_el$6, createComponent(SettingsListV2, {
      get children() {
        return [createComponent(For, {
          get each() {
            return popular();
          },
          children: (item) => (() => {
            var _el$19 = _tmpl$7$2(), _el$20 = _el$19.firstChild, _el$21 = _el$20.firstChild, _el$22 = _el$21.firstChild, _el$23 = _el$22.firstChild;
            insert(_el$20, createComponent(ProviderIcon, {
              get id() {
                return item.id;
              },
              width: PROVIDER_ICON_SIZE$1,
              height: PROVIDER_ICON_SIZE$1,
              "class": "settings-v2-provider-icon shrink-0"
            }), _el$21);
            insert(_el$23, () => item.name);
            insert(_el$22, createComponent(Show, {
              get when() {
                return item.id === "opencode" || item.id === "opencode-go";
              },
              get children() {
                return createComponent(Tag, {
                  get children() {
                    return language.t("dialog.provider.tag.recommended");
                  }
                });
              }
            }), null);
            insert(_el$21, createComponent(Show, {
              get when() {
                return note(item.id);
              },
              children: (key) => (() => {
                var _el$24 = _tmpl$8$1();
                insert(_el$24, () => language.t(key()));
                return _el$24;
              })()
            }), null);
            insert(_el$19, createComponent(ButtonV2, {
              size: "normal",
              variant: "neutral",
              icon: "plus",
              onClick: () => {
                dialog.show(() => createComponent(DialogConnectProvider, {
                  get provider() {
                    return item.id;
                  }
                }));
              },
              get children() {
                return language.t("common.connect");
              }
            }), null);
            return _el$19;
          })()
        }), (() => {
          var _el$8 = _tmpl$2$3(), _el$9 = _el$8.firstChild, _el$0 = _el$9.firstChild, _el$1 = _el$0.firstChild, _el$10 = _el$1.firstChild, _el$11 = _el$1.nextSibling;
          insert(_el$9, createComponent(ProviderIcon, {
            id: "synthetic",
            width: PROVIDER_ICON_SIZE$1,
            height: PROVIDER_ICON_SIZE$1,
            "class": "settings-v2-provider-icon shrink-0"
          }), _el$0);
          insert(_el$10, () => language.t("provider.custom.title"));
          insert(_el$1, createComponent(Tag, {
            get children() {
              return language.t("settings.providers.tag.custom");
            }
          }), null);
          insert(_el$11, () => language.t("settings.providers.custom.description"));
          insert(_el$8, createComponent(ButtonV2, {
            size: "normal",
            variant: "neutral",
            icon: "plus",
            onClick: () => {
              dialog.show(() => createComponent(DialogCustomProvider, {
                back: "close"
              }));
            },
            get children() {
              return language.t("common.connect");
            }
          }), null);
          return _el$8;
        })()];
      }
    }), _el$12);
    _el$12.$$click = () => {
      dialog.show(() => createComponent(DialogSelectProvider, {}));
    };
    insert(_el$12, () => language.t("dialog.provider.viewAll"));
    return _el$3;
  })()];
};
delegateEvents(["click"]);
var _tmpl$$4 = /* @__PURE__ */ template(`<div class="settings-v2-tab-header settings-v2-tab-header--stacked"><h2 class=settings-v2-tab-title></h2><div class=settings-v2-tab-search>`), _tmpl$2$2 = /* @__PURE__ */ template(`<div class="settings-v2-tab-body settings-v2-models">`), _tmpl$3$2 = /* @__PURE__ */ template(`<div class=settings-v2-models-status>`), _tmpl$4$2 = /* @__PURE__ */ template(`<span class=settings-v2-models-status-filter>&quot;<!>&quot;`), _tmpl$5$2 = /* @__PURE__ */ template(`<div class=settings-v2-models-status><span>`), _tmpl$6$2 = /* @__PURE__ */ template(`<div class=settings-v2-section data-component=settings-models-provider><div class=settings-v2-models-group-header><h3 class=settings-v2-section-title>`), _tmpl$7$1 = /* @__PURE__ */ template(`<div>`);
const PROVIDER_ICON_SIZE = 16;
const SettingsModelsV2 = () => {
  const language = useLanguage();
  const models = useModels();
  const list = useFilteredList({
    items: (_filter) => models.list(),
    key: (x) => `${x.provider.id}:${x.id}`,
    filterKeys: ["provider.name", "name", "id"],
    sortBy: (a, b) => a.name.localeCompare(b.name),
    groupBy: (x) => x.provider.id,
    sortGroupsBy: (a, b) => {
      const aIndex = popularProviders.indexOf(a.category);
      const bIndex = popularProviders.indexOf(b.category);
      const aPopular = aIndex >= 0;
      const bPopular = bIndex >= 0;
      if (aPopular && !bPopular) return -1;
      if (!aPopular && bPopular) return 1;
      if (aPopular && bPopular) return aIndex - bIndex;
      const aName = a.items[0].provider.name;
      const bName = b.items[0].provider.name;
      return aName.localeCompare(bName);
    }
  });
  return [(() => {
    var _el$ = _tmpl$$4(), _el$2 = _el$.firstChild, _el$3 = _el$2.nextSibling;
    insert(_el$2, () => language.t("settings.models.title"));
    insert(_el$3, createComponent(TextInputV2, {
      type: "search",
      appearance: "base",
      get value() {
        return list.filter();
      },
      onInput: (event) => list.onInput(event.currentTarget.value),
      get placeholder() {
        return language.t("dialog.model.search.placeholder");
      },
      spellcheck: false,
      autocorrect: "off",
      autocomplete: "off",
      autocapitalize: "off",
      get ["aria-label"]() {
        return language.t("dialog.model.search.placeholder");
      }
    }), null);
    insert(_el$3, createComponent(Show, {
      get when() {
        return list.filter();
      },
      get children() {
        return createComponent(IconButtonV2, {
          type: "button",
          variant: "ghost-muted",
          size: "small",
          "class": "settings-v2-tab-search-clear",
          get icon() {
            return createComponent(Icon, {
              name: "close",
              size: "large",
              "class": "text-v2-icon-icon-muted"
            });
          },
          onClick: () => list.clear()
        });
      }
    }), null);
    return _el$;
  })(), (() => {
    var _el$4 = _tmpl$2$2();
    insert(_el$4, createComponent(Show, {
      get when() {
        return !list.grouped.loading;
      },
      get fallback() {
        return (() => {
          var _el$5 = _tmpl$3$2();
          insert(_el$5, () => language.t("common.loading"), null);
          insert(_el$5, () => language.t("common.loading.ellipsis"), null);
          return _el$5;
        })();
      },
      get children() {
        return createComponent(Show, {
          get when() {
            return list.flat().length > 0;
          },
          get fallback() {
            return (() => {
              var _el$6 = _tmpl$5$2(), _el$7 = _el$6.firstChild;
              insert(_el$7, () => language.t("dialog.model.empty"));
              insert(_el$6, createComponent(Show, {
                get when() {
                  return list.filter();
                },
                get children() {
                  var _el$8 = _tmpl$4$2(), _el$9 = _el$8.firstChild, _el$1 = _el$9.nextSibling;
                  _el$1.nextSibling;
                  insert(_el$8, () => list.filter(), _el$1);
                  return _el$8;
                }
              }), null);
              return _el$6;
            })();
          },
          get children() {
            return createComponent(For, {
              get each() {
                return list.grouped.latest;
              },
              children: (group) => (() => {
                var _el$10 = _tmpl$6$2(), _el$11 = _el$10.firstChild, _el$12 = _el$11.firstChild;
                insert(_el$11, createComponent(ProviderIcon, {
                  get id() {
                    return group.category;
                  },
                  width: PROVIDER_ICON_SIZE,
                  height: PROVIDER_ICON_SIZE,
                  "class": "settings-v2-models-provider-icon shrink-0"
                }), _el$12);
                insert(_el$12, () => group.items[0].provider.name);
                insert(_el$10, createComponent(SettingsListV2, {
                  get children() {
                    return createComponent(For, {
                      get each() {
                        return group.items;
                      },
                      children: (item) => {
                        const key = {
                          providerID: item.provider.id,
                          modelID: item.id
                        };
                        return createComponent(SettingsRowV2, {
                          get title() {
                            return item.name;
                          },
                          description: "",
                          get children() {
                            var _el$13 = _tmpl$7$1();
                            insert(_el$13, createComponent(Switch, {
                              get checked() {
                                return models.visible(key);
                              },
                              onChange: (checked) => {
                                models.setVisibility(key, checked);
                              },
                              hideLabel: true,
                              get children() {
                                return item.name;
                              }
                            }));
                            return _el$13;
                          }
                        });
                      }
                    });
                  }
                }), null);
                return _el$10;
              })()
            });
          }
        });
      }
    }));
    return _el$4;
  })()];
};
const wslRuntimeRetryable = (runtime) => runtime.kind === "failed" || runtime.kind === "stopped";
async function enterWslOpencodeStep(distro, probe, select) {
  await probe(distro);
  select("opencode");
}
function wslOpencodeAction(check) {
  if (!check) return;
  if (!check.resolvedPath) return "Install OpenCode";
  if (check.matchesDesktop === false) return "Update OpenCode";
}
var _tmpl$$3 = /* @__PURE__ */ template(`<div class="flex gap-2 pb-1">`), _tmpl$2$1 = /* @__PURE__ */ template(`<div class="rounded-md border border-border-weak-base px-3 py-3"><div class="text-12-regular text-text-warning-base">`), _tmpl$3$1 = /* @__PURE__ */ template(`<div class="rounded-md bg-surface-base p-4 flex flex-col gap-3"><div class="flex items-center justify-between gap-3"><div class="text-14-medium text-text-strong"></div></div><div class="text-12-regular text-text-weak whitespace-pre-wrap break-words"></div><div class="flex items-center justify-end">`), _tmpl$4$1 = /* @__PURE__ */ template(`<div class="rounded-md border border-border-weak-base p-2 flex flex-col gap-2"><div class="px-1 flex items-center justify-between gap-3"><div class="text-12-medium text-text-weak"></div><div class="flex items-center gap-2 shrink-0"></div></div><div role=radiogroup class="max-h-52 overflow-y-auto rounded-md bg-background-base">`), _tmpl$5$1 = /* @__PURE__ */ template(`<div class="text-12-regular text-text-warning-base">`), _tmpl$6$1 = /* @__PURE__ */ template(`<div class="rounded-md border border-border-weak-base px-3 py-3 flex flex-col gap-1">`), _tmpl$7 = /* @__PURE__ */ template(`<div class="rounded-md bg-surface-base p-4 flex flex-col gap-3"><div class="flex items-center justify-between gap-3"><div class="text-14-medium text-text-strong"></div></div><div class="text-12-regular text-text-weak whitespace-pre-wrap break-words"></div><div class="flex flex-col gap-2"></div><div class="flex items-center gap-2"></div><div class="flex items-center justify-end">`), _tmpl$8 = /* @__PURE__ */ template(`<div class="rounded-md bg-surface-base p-4 flex flex-col gap-3"><div class="flex items-center justify-between gap-3"><div class="text-14-medium text-text-strong"></div><div class="flex items-center gap-2"></div></div><div class="text-12-regular text-text-weak whitespace-pre-wrap break-words">`), _tmpl$9 = /* @__PURE__ */ template(`<div class="flex items-center justify-end gap-2">`), _tmpl$0 = /* @__PURE__ */ template(`<div class="px-5 pb-5 flex flex-col gap-4">`), _tmpl$1 = /* @__PURE__ */ template(`<div class="px-1 py-6 text-14-regular text-text-weak">`), _tmpl$10 = /* @__PURE__ */ template(`<button type=button class="basis-0 flex-1 min-w-0 rounded-md border px-3 py-2 text-left transition-colors"><div class="text-13-medium text-text-strong">`), _tmpl$11 = /* @__PURE__ */ template(`<div class="text-12-regular text-text-weak">`), _tmpl$12 = /* @__PURE__ */ template(`<button type=button class="rounded-md border border-border-weak-base px-3 py-2 text-left transition-colors"><div class="text-13-medium text-text-strong">`), _tmpl$13 = /* @__PURE__ */ template(`<button type=button role=radio class="w-full px-3 py-2 flex items-center gap-3 text-left border-b border-border-weak-base last:border-b-0 transition-colors"><div class="mt-0.5 h-4 w-4 rounded-full border border-border-strong-base flex items-center justify-center shrink-0"><div class="h-2 w-2 rounded-full bg-text-strong"></div></div><div class="min-w-0 flex-1 text-13-medium text-text-strong truncate">`), _tmpl$14 = /* @__PURE__ */ template(`<div class="rounded-md border border-border-weak-base px-3 py-3 flex flex-col gap-1"><div class="text-12-regular text-text-weak"></div><div class="text-12-regular text-text-weak"></div><div class="text-12-regular text-text-warning-base">`), _tmpl$15 = /* @__PURE__ */ template(`<span>`);
const STEPS = ["wsl", "distro", "opencode"];
function isHiddenDistro(name) {
  return /^docker-desktop(?:-data)?$/i.test(name);
}
function DialogAddWslServer(props = {}) {
  const language = useLanguage();
  const platform = usePlatform();
  const dialog = useDialog();
  const wslServers = useWslServers();
  const api = platform.wslServers;
  const [store, setStore] = createStore({
    step: void 0,
    selectedDistro: null,
    installTarget: void 0,
    adding: false
  });
  const current = () => wslServers.data;
  let disposed = false;
  onCleanup(() => {
    disposed = true;
  });
  const busy = createMemo(() => !!current()?.job || store.adding);
  const visibleInstalledDistros = createMemo(() => (current()?.installed ?? []).filter((item) => !isHiddenDistro(item.name)));
  const visibleOnlineDistros = createMemo(() => (current()?.online ?? []).filter((item) => !isHiddenDistro(item.name)));
  const defaultInstalledDistro = createMemo(() => visibleInstalledDistros().find((item) => item.isDefault) ?? null);
  const existingServerDistros = createMemo(() => new Set((current()?.servers ?? []).map((item) => item.config.distro)));
  const addableInstalledDistros = createMemo(() => {
    return visibleInstalledDistros().filter((item) => !existingServerDistros().has(item.name));
  });
  const selectedDistro = createMemo(() => {
    if (store.selectedDistro && addableInstalledDistros().some((item) => item.name === store.selectedDistro)) {
      return store.selectedDistro;
    }
    const distro = defaultInstalledDistro();
    if (distro && !existingServerDistros().has(distro.name)) return distro.name;
    return null;
  });
  const selectedProbe = createMemo(() => {
    const distro = selectedDistro();
    if (!distro) return null;
    return current()?.distroProbes[distro] ?? null;
  });
  const selectedInstalled = createMemo(() => {
    const distro = selectedDistro();
    if (!distro) return null;
    return (current()?.installed ?? []).find((item) => item.name === distro) ?? null;
  });
  const opencodeCheck = createMemo(() => {
    const distro = selectedDistro();
    if (!distro) return null;
    return current()?.opencodeChecks[distro] ?? null;
  });
  const wslReady = createMemo(() => !!current()?.runtime?.available && !current()?.pendingRestart);
  const distroReady = createMemo(() => {
    const probe = selectedProbe();
    if (!probe || !selectedDistro()) return false;
    if (selectedInstalled()?.version === 1) return false;
    return probe.canExecute && probe.hasBash && probe.hasCurl;
  });
  const opencodeReady = createMemo(() => {
    const check = opencodeCheck();
    return !!check?.resolvedPath && !check.error;
  });
  const distroWarningProbe = createMemo(() => {
    const probe = selectedProbe();
    if (!probe) return null;
    if (distroReady()) return null;
    return probe;
  });
  const distroUnavailableMessage = createMemo(() => {
    const probe = distroWarningProbe();
    const distro = selectedDistro();
    if (!probe || probe.canExecute || !distro) return null;
    if (!selectedInstalled()) return language.t("wsl.onboarding.distroNotInstalled", {
      distro
    });
    return language.t("wsl.onboarding.openDistroOnce", {
      distro
    });
  });
  const distroMissingTools = createMemo(() => {
    const probe = distroWarningProbe();
    if (!probe?.canExecute) return null;
    if (probe.hasBash && probe.hasCurl) return null;
    return probe;
  });
  const installableDistros = createMemo(() => {
    const online = visibleOnlineDistros();
    const installed = new Set(visibleInstalledDistros().map((item) => item.name));
    const hasVersionedUbuntu = online.some((item) => /^Ubuntu-\d/.test(item.name));
    return online.filter((item) => !installed.has(item.name)).filter((item) => !(item.name === "Ubuntu" && hasVersionedUbuntu));
  });
  const installTarget = createMemo(() => installableDistros().find((item) => item.name === store.installTarget) ?? installableDistros()[0] ?? null);
  const installingDistro = createMemo(() => current()?.job?.kind === "install-distro");
  const installingOpencode = createMemo(() => {
    const job = current()?.job;
    return job?.kind === "install-opencode" && job.distro === selectedDistro();
  });
  const allReady = createMemo(() => wslReady() && distroReady() && opencodeReady());
  const addDisabled = createMemo(() => {
    const job = current()?.job;
    if (!job) return store.adding;
    return store.adding || job.kind !== "probe-opencode";
  });
  const recommendedStep = createMemo(() => {
    if (!wslReady()) return "wsl";
    if (!distroReady()) return "distro";
    return "opencode";
  });
  const activeStep = createMemo(() => store.step ?? recommendedStep());
  const autoProbe = createMemo(() => {
    const state = current();
    if (!state || busy()) return null;
    if (state.pendingRestart) return null;
    if (!state.runtime) return {
      key: "runtime",
      run: () => api.probeRuntime()
    };
    if (!wslReady()) return null;
    if (!state.installed.length && !state.online.length) {
      return {
        key: "distros",
        run: () => api.refreshDistros()
      };
    }
    const distro = selectedDistro();
    if (distro && !state.distroProbes[distro]) {
      return {
        key: `probe-distro:${distro}`,
        run: () => api.probeDistro(distro)
      };
    }
    if (!distro || !distroReady()) return null;
    if (!state.opencodeChecks[distro]) {
      return {
        key: `probe-opencode:${distro}`,
        run: () => api.probeOpencode(distro)
      };
    }
    return null;
  });
  let lastAutoProbe = null;
  createEffect(() => {
    const probe = autoProbe();
    if (!probe || probe.key === lastAutoProbe) return;
    const key = probe.key;
    lastAutoProbe = key;
    void (async () => {
      try {
        await probe.run();
      } catch (err) {
        if (disposed) return;
        if (lastAutoProbe === key) lastAutoProbe = null;
        requestError(language, err);
      }
    })();
  });
  const wslMessage = createMemo(() => {
    const state = current();
    if (!state || state.job?.kind === "runtime") return language.t("wsl.onboarding.checkingRuntime");
    if (state.pendingRestart) return language.t("wsl.onboarding.restartRequired");
    if (state.runtime?.available) return state.runtime.version ?? language.t("wsl.onboarding.ready");
    return state.runtime?.error ?? language.t("wsl.onboarding.required");
  });
  const distroMessage = createMemo(() => {
    const state = current();
    if (!state) return language.t("wsl.onboarding.checkingDistros");
    const distro = selectedDistro();
    if (state.job?.kind === "install-distro") return language.t("wsl.onboarding.installingDistro", {
      distro: state.job.distro
    });
    if (state.job?.kind === "probe-distro") return language.t("wsl.onboarding.checkingDistro", {
      distro: state.job.distro
    });
    if (state.job?.kind === "distros") return language.t("wsl.onboarding.listingDistros");
    if (distroUnavailableMessage()) return distroUnavailableMessage();
    if (selectedProbe() && distroReady()) return language.t("wsl.onboarding.distroReady", {
      distro: selectedProbe().name
    });
    if (distro) return language.t("wsl.onboarding.finishingDistro", {
      distro
    });
    return language.t("wsl.onboarding.pickDistro");
  });
  const opencodeMessage = createMemo(() => {
    const state = current();
    if (!state) return language.t("wsl.onboarding.checkingOpencode");
    const distro = selectedDistro();
    if (state.job?.kind === "install-opencode") {
      return distro ? language.t("wsl.onboarding.updatingOpencodeIn", {
        distro
      }) : language.t("wsl.onboarding.updatingOpencode");
    }
    if (state.job?.kind === "probe-opencode") {
      return distro ? language.t("wsl.onboarding.checkingOpencodeIn", {
        distro
      }) : language.t("wsl.onboarding.checkingOpencode");
    }
    if (opencodeCheck()?.error) return opencodeCheck().error;
    if (opencodeCheck()?.matchesDesktop === false) {
      return distro ? language.t("wsl.onboarding.updateOpencodeIn", {
        distro
      }) : language.t("wsl.onboarding.updateOpencode");
    }
    if (opencodeReady()) {
      return distro ? language.t("wsl.onboarding.opencodeReadyIn", {
        distro
      }) : language.t("wsl.onboarding.opencodeReady");
    }
    return distro ? language.t("wsl.onboarding.installOpencodeIn", {
      distro
    }) : language.t("wsl.onboarding.chooseDistroFirst");
  });
  const run = async (action) => {
    try {
      await action();
    } catch (err) {
      requestError(language, err);
    }
  };
  const runSelectedDistro = (action) => {
    const distro = selectedDistro();
    if (!distro) return;
    void run(() => action(distro));
  };
  const selectDistro = (name) => {
    setStore("selectedDistro", name);
    setStore("step", void 0);
  };
  const openOpencodeStep = () => {
    const distro = selectedDistro();
    if (!distro) return;
    void run(() => enterWslOpencodeStep(distro, api.probeOpencode, (step) => setStore("step", step)));
  };
  const finish = async () => {
    const distro = selectedDistro();
    if (!distro) return;
    setStore("adding", true);
    try {
      await api.addServer(distro);
      if (props.onAdded) {
        await props.onAdded(distro);
      } else {
        dialog.close();
      }
    } catch (err) {
      requestError(language, err);
    } finally {
      setStore("adding", false);
    }
  };
  const steps = createMemo(() => {
    const active = activeStep();
    const activeIndex = STEPS.indexOf(active);
    const recommendedIndex = STEPS.indexOf(recommendedStep());
    return STEPS.map((step) => {
      const index = STEPS.indexOf(step);
      return {
        step,
        title: step === "wsl" ? language.t("wsl.server.label") : step === "distro" ? language.t("wsl.onboarding.step.distro") : language.t("wsl.onboarding.step.opencode"),
        state: active === step ? "current" : step === "wsl" ? wslReady() ? "done" : "warning" : step === "distro" ? distroReady() ? "done" : index > activeIndex ? "locked" : "warning" : opencodeCheck()?.matchesDesktop === false ? "warning" : opencodeReady() ? "done" : index > activeIndex ? "locked" : "warning",
        locked: index > recommendedIndex
      };
    });
  });
  const loadError = createMemo(() => {
    const error = wslServers.error;
    if (!error) return language.t("wsl.onboarding.loadFailed");
    return error instanceof Error ? error.message : String(error);
  });
  return (() => {
    var _el$ = _tmpl$0();
    insert(_el$, createComponent(Show, {
      get when() {
        return !wslServers.isPending;
      },
      get fallback() {
        return (() => {
          var _el$29 = _tmpl$1();
          insert(_el$29, () => language.t("wsl.onboarding.loading"));
          return _el$29;
        })();
      },
      get children() {
        return createComponent(Show, {
          get when() {
            return !wslServers.isError;
          },
          get fallback() {
            return (() => {
              var _el$30 = _tmpl$1();
              insert(_el$30, loadError);
              return _el$30;
            })();
          },
          get children() {
            return [(() => {
              var _el$2 = _tmpl$$3();
              insert(_el$2, createComponent(For, {
                get each() {
                  return steps();
                },
                children: (item) => (() => {
                  var _el$31 = _tmpl$10(), _el$32 = _el$31.firstChild;
                  _el$31.$$click = () => setStore("step", item.step);
                  insert(_el$32, () => item.title);
                  createRenderEffect((_p$) => {
                    var _v$ = {
                      "border-border-strong-base bg-surface-base-hover": item.state === "current",
                      "border-icon-success-base/40 bg-surface-base": item.state === "done",
                      "border-border-weak-base bg-background-base opacity-60": item.state === "locked",
                      "border-icon-warning-base/40 bg-surface-base": item.state === "warning"
                    }, _v$2 = item.locked;
                    _p$.e = classList(_el$31, _v$, _p$.e);
                    _v$2 !== _p$.t && (_el$31.disabled = _p$.t = _v$2);
                    return _p$;
                  }, {
                    e: void 0,
                    t: void 0
                  });
                  return _el$31;
                })()
              }));
              return _el$2;
            })(), createComponent(Switch$2, {
              get children() {
                return [createComponent(Match, {
                  get when() {
                    return activeStep() === "wsl";
                  },
                  get children() {
                    var _el$3 = _tmpl$3$1(), _el$4 = _el$3.firstChild, _el$5 = _el$4.firstChild, _el$6 = _el$4.nextSibling, _el$9 = _el$6.nextSibling;
                    insert(_el$5, () => language.t("wsl.server.label"));
                    insert(_el$4, createComponent(Show, {
                      get when() {
                        return memo(() => !!(current()?.runtime && !wslReady()))() && !current()?.pendingRestart;
                      },
                      get children() {
                        return createComponent(Button, {
                          variant: "secondary",
                          size: "large",
                          get disabled() {
                            return busy();
                          },
                          onClick: () => void run(() => api.installWsl()),
                          get children() {
                            return language.t("wsl.onboarding.installWsl");
                          }
                        });
                      }
                    }), null);
                    insert(_el$6, wslMessage);
                    insert(_el$3, createComponent(Show, {
                      get when() {
                        return current()?.pendingRestart;
                      },
                      get children() {
                        var _el$7 = _tmpl$2$1(), _el$8 = _el$7.firstChild;
                        insert(_el$8, () => language.t("wsl.onboarding.windowsRestartRequired"));
                        return _el$7;
                      }
                    }), _el$9);
                    insert(_el$9, createComponent(Button, {
                      variant: "secondary",
                      size: "large",
                      get disabled() {
                        return busy() || !wslReady();
                      },
                      onClick: () => setStore("step", "distro"),
                      get children() {
                        return language.t("wsl.onboarding.next");
                      }
                    }));
                    return _el$3;
                  }
                }), createComponent(Match, {
                  get when() {
                    return activeStep() === "distro";
                  },
                  get children() {
                    var _el$0 = _tmpl$7(), _el$1 = _el$0.firstChild, _el$10 = _el$1.firstChild, _el$11 = _el$1.nextSibling, _el$12 = _el$11.nextSibling, _el$21 = _el$12.nextSibling, _el$22 = _el$21.nextSibling;
                    insert(_el$10, () => language.t("wsl.onboarding.step.distro"));
                    insert(_el$1, createComponent(Show, {
                      get when() {
                        return selectedDistro();
                      },
                      get children() {
                        return createComponent(Button, {
                          variant: "ghost",
                          size: "small",
                          get disabled() {
                            return busy();
                          },
                          onClick: () => runSelectedDistro((distro) => api.probeDistro(distro)),
                          get children() {
                            return language.t("wsl.onboarding.refresh");
                          }
                        });
                      }
                    }), null);
                    insert(_el$11, distroMessage);
                    insert(_el$12, createComponent(Show, {
                      get when() {
                        return addableInstalledDistros().length > 0;
                      },
                      get fallback() {
                        return (() => {
                          var _el$33 = _tmpl$11();
                          insert(_el$33, (() => {
                            var _c$ = memo(() => !!visibleInstalledDistros().length);
                            return () => _c$() ? language.t("wsl.onboarding.allDistrosAdded") : memo(() => !!current()?.runtime?.available)() ? language.t("wsl.onboarding.noDistros") : language.t("wsl.onboarding.checkingDistros");
                          })());
                          return _el$33;
                        })();
                      },
                      get children() {
                        return createComponent(For, {
                          get each() {
                            return addableInstalledDistros();
                          },
                          children: (item) => (() => {
                            var _el$34 = _tmpl$12(), _el$35 = _el$34.firstChild;
                            _el$34.$$click = () => selectDistro(item.name);
                            insert(_el$35, () => item.name);
                            insert(_el$34, createComponent(Show, {
                              get when() {
                                return item.isDefault;
                              },
                              get children() {
                                var _el$36 = _tmpl$11();
                                insert(_el$36, () => language.t("common.default"));
                                return _el$36;
                              }
                            }), null);
                            createRenderEffect(() => _el$34.classList.toggle("bg-surface-raised-base", !!(selectedDistro() === item.name)));
                            return _el$34;
                          })()
                        });
                      }
                    }));
                    insert(_el$0, createComponent(Show, {
                      get when() {
                        return installableDistros().length > 0;
                      },
                      get children() {
                        var _el$13 = _tmpl$4$1(), _el$14 = _el$13.firstChild, _el$15 = _el$14.firstChild, _el$16 = _el$15.nextSibling, _el$17 = _el$14.nextSibling;
                        insert(_el$15, () => language.t("wsl.onboarding.install"));
                        insert(_el$16, createComponent(Show, {
                          get when() {
                            return installingDistro();
                          },
                          get children() {
                            return createComponent(Spinner, {
                              "class": "h-4 w-4 text-icon-info-base shrink-0"
                            });
                          }
                        }), null);
                        insert(_el$16, createComponent(Button, {
                          variant: "secondary",
                          size: "small",
                          get disabled() {
                            return busy() || !installTarget();
                          },
                          onClick: () => void run(() => api.installDistro(installTarget().name)),
                          get children() {
                            return memo(() => !!installingDistro())() ? language.t("wsl.onboarding.installing") : language.t("wsl.onboarding.install");
                          }
                        }), null);
                        insert(_el$17, createComponent(For, {
                          get each() {
                            return installableDistros();
                          },
                          children: (item) => {
                            const selected = () => installTarget()?.name === item.name;
                            return (() => {
                              var _el$37 = _tmpl$13(), _el$38 = _el$37.firstChild, _el$39 = _el$38.firstChild, _el$40 = _el$38.nextSibling;
                              _el$37.$$click = () => setStore("installTarget", item.name);
                              insert(_el$40, () => item.label);
                              createRenderEffect((_p$) => {
                                var _v$3 = selected(), _v$4 = busy(), _v$5 = {
                                  "bg-surface-raised-base": selected(),
                                  "hover:bg-surface-base": !selected()
                                }, _v$6 = !!selected(), _v$7 = !selected();
                                _v$3 !== _p$.e && setAttribute(_el$37, "aria-checked", _p$.e = _v$3);
                                _v$4 !== _p$.t && (_el$37.disabled = _p$.t = _v$4);
                                _p$.a = classList(_el$37, _v$5, _p$.a);
                                _v$6 !== _p$.o && _el$38.classList.toggle("border-text-strong", _p$.o = _v$6);
                                _v$7 !== _p$.i && _el$39.classList.toggle("hidden", _p$.i = _v$7);
                                return _p$;
                              }, {
                                e: void 0,
                                t: void 0,
                                a: void 0,
                                o: void 0,
                                i: void 0
                              });
                              return _el$37;
                            })();
                          }
                        }));
                        createRenderEffect(() => setAttribute(_el$17, "aria-label", language.t("wsl.onboarding.installDistro")));
                        return _el$13;
                      }
                    }), _el$21);
                    insert(_el$0, createComponent(Show, {
                      get when() {
                        return selectedInstalled()?.version === 1 || distroUnavailableMessage() || distroMissingTools();
                      },
                      get children() {
                        var _el$18 = _tmpl$6$1();
                        insert(_el$18, createComponent(Show, {
                          get when() {
                            return selectedInstalled()?.version === 1;
                          },
                          get children() {
                            var _el$19 = _tmpl$5$1();
                            insert(_el$19, () => language.t("wsl.onboarding.wsl2Required"));
                            return _el$19;
                          }
                        }), null);
                        insert(_el$18, createComponent(Show, {
                          get when() {
                            return distroUnavailableMessage();
                          },
                          children: (message) => (() => {
                            var _el$41 = _tmpl$5$1();
                            insert(_el$41, message);
                            return _el$41;
                          })()
                        }), null);
                        insert(_el$18, createComponent(Show, {
                          get when() {
                            return distroMissingTools();
                          },
                          get children() {
                            var _el$20 = _tmpl$5$1();
                            insert(_el$20, () => language.t("wsl.onboarding.toolsRequired"));
                            return _el$20;
                          }
                        }), null);
                        return _el$18;
                      }
                    }), _el$21);
                    insert(_el$21, createComponent(Button, {
                      variant: "secondary",
                      size: "large",
                      get disabled() {
                        return busy() || !selectedInstalled();
                      },
                      onClick: () => runSelectedDistro((distro) => api.openTerminal(distro)),
                      get children() {
                        return language.t("wsl.onboarding.openTerminal");
                      }
                    }), null);
                    insert(_el$21, createComponent(Button, {
                      variant: "ghost",
                      size: "large",
                      get disabled() {
                        return busy() || !selectedDistro();
                      },
                      onClick: () => runSelectedDistro((distro) => api.probeDistro(distro)),
                      get children() {
                        return language.t("wsl.onboarding.refresh");
                      }
                    }), null);
                    insert(_el$22, createComponent(Button, {
                      variant: "secondary",
                      size: "large",
                      get disabled() {
                        return busy() || !selectedDistro() || !distroReady();
                      },
                      onClick: openOpencodeStep,
                      get children() {
                        return language.t("wsl.onboarding.next");
                      }
                    }));
                    return _el$0;
                  }
                }), createComponent(Match, {
                  get when() {
                    return activeStep() === "opencode";
                  },
                  get children() {
                    var _el$23 = _tmpl$8(), _el$24 = _el$23.firstChild, _el$25 = _el$24.firstChild, _el$26 = _el$25.nextSibling, _el$27 = _el$24.nextSibling;
                    insert(_el$25, () => language.t("wsl.onboarding.step.opencode"));
                    insert(_el$26, createComponent(Show, {
                      get when() {
                        return selectedDistro();
                      },
                      get children() {
                        return createComponent(Button, {
                          variant: "ghost",
                          size: "large",
                          get disabled() {
                            return busy();
                          },
                          onClick: () => runSelectedDistro((distro) => api.probeOpencode(distro)),
                          get children() {
                            return language.t("wsl.onboarding.refresh");
                          }
                        });
                      }
                    }), null);
                    insert(_el$26, createComponent(Show, {
                      get when() {
                        return !opencodeReady() || opencodeCheck()?.matchesDesktop === false;
                      },
                      get children() {
                        return createComponent(Button, {
                          variant: "secondary",
                          size: "large",
                          get disabled() {
                            return busy();
                          },
                          onClick: () => runSelectedDistro((distro) => api.installOpencode(distro)),
                          get children() {
                            return [createComponent(Show, {
                              get when() {
                                return installingOpencode();
                              },
                              get children() {
                                return createComponent(Spinner, {
                                  "class": "size-4 shrink-0"
                                });
                              }
                            }), memo(() => memo(() => !!opencodeCheck()?.resolvedPath)() ? language.t("wsl.onboarding.updateOpencode") : language.t("wsl.onboarding.installOpencode"))];
                          }
                        });
                      }
                    }), null);
                    insert(_el$27, opencodeMessage);
                    insert(_el$23, createComponent(Show, {
                      get when() {
                        return memo(() => opencodeCheck()?.matchesDesktop === false)() ? opencodeCheck() : null;
                      },
                      children: (check) => (() => {
                        var _el$42 = _tmpl$14(), _el$43 = _el$42.firstChild, _el$44 = _el$43.nextSibling, _el$45 = _el$44.nextSibling;
                        insert(_el$43, () => language.t("wsl.onboarding.path", {
                          path: check().resolvedPath ?? language.t("wsl.onboarding.notFound")
                        }));
                        insert(_el$44, () => language.t("wsl.onboarding.version", {
                          version: check().version ?? language.t("wsl.onboarding.unknown")
                        }), null);
                        insert(_el$44, createComponent(Show, {
                          get when() {
                            return check().expectedVersion;
                          },
                          children: (expected) => (() => {
                            var _el$46 = _tmpl$15();
                            insert(_el$46, () => ` · ${language.t("wsl.onboarding.desktopVersion", {
                              version: expected()
                            })}`);
                            return _el$46;
                          })()
                        }), null);
                        insert(_el$45, () => language.t("wsl.onboarding.versionMismatch"));
                        return _el$42;
                      })()
                    }), null);
                    return _el$23;
                  }
                })];
              }
            }), createComponent(Show, {
              get when() {
                return memo(() => !!(activeStep() === "opencode" && allReady()))() && selectedDistro();
              },
              get children() {
                var _el$28 = _tmpl$9();
                insert(_el$28, createComponent(Button, {
                  variant: "ghost",
                  size: "large",
                  get disabled() {
                    return store.adding;
                  },
                  onClick: () => dialog.close(),
                  get children() {
                    return language.t("common.cancel");
                  }
                }), null);
                insert(_el$28, createComponent(Button, {
                  variant: "primary",
                  size: "large",
                  get disabled() {
                    return addDisabled();
                  },
                  onClick: () => void finish(),
                  get children() {
                    return memo(() => !!store.adding)() ? language.t("wsl.onboarding.adding") : language.t("wsl.server.add");
                  }
                }), null);
                return _el$28;
              }
            })];
          }
        });
      }
    }));
    return _el$;
  })();
}
function requestError(language, err) {
  console.error("WSL servers request failed", err instanceof Error ? err.stack ?? err.message : String(err));
  showToast$1({
    variant: "error",
    title: language.t("common.requestFailed"),
    description: err instanceof Error ? err.message : String(err)
  });
}
delegateEvents(["click"]);
var _tmpl$$2 = /* @__PURE__ */ template(`<div class=settings-v2-servers-row><div class=settings-v2-servers-lead><div class=settings-v2-servers-copy><span class="flex min-w-0 items-center gap-1"><span class=settings-v2-servers-name></span><span class="shrink-0 rounded-[3px] border border-v2-border-border-base px-1 py-0.5 text-[9px] leading-none text-v2-text-text-muted"></span></span><span class=settings-v2-servers-meta></span></div></div><div class=settings-v2-servers-actions>`);
function isWslServer(server) {
  return server.type === "sidecar" && server.variant === "wsl";
}
function AddServerMenu(props) {
  const platform = usePlatform();
  const dialog = useDialog();
  const language = useLanguage();
  const openAddWsl = () => {
    dialog.push(() => createComponent(Dialog, {
      get title() {
        return language.t("wsl.server.add");
      },
      size: "large",
      fit: true,
      "class": "settings-v2-wsl-dialog",
      get children() {
        return createComponent(DialogAddWslServer, {});
      }
    }));
  };
  return createComponent(Show, {
    get when() {
      return platform.wslServers;
    },
    get fallback() {
      return createComponent(ButtonV2, {
        variant: "ghost-muted",
        icon: "plus",
        get onClick() {
          return props.onAddServer;
        },
        get children() {
          return language.t("dialog.server.add.button");
        }
      });
    },
    get children() {
      return createComponent(MenuV2, {
        gutter: 4,
        modal: false,
        placement: "bottom-end",
        get children() {
          return [createComponent(MenuV2.Trigger, {
            as: ButtonV2,
            variant: "ghost-muted",
            icon: "plus",
            get children() {
              return language.t("dialog.server.add.button");
            }
          }), createComponent(MenuV2.Portal, {
            get children() {
              return createComponent(MenuV2.Content, {
                get children() {
                  return [createComponent(MenuV2.Item, {
                    get onSelect() {
                      return props.onAddServer;
                    },
                    get children() {
                      return language.t("dialog.server.add.button");
                    }
                  }), createComponent(MenuV2.Item, {
                    onSelect: openAddWsl,
                    get children() {
                      return language.t("wsl.server.add");
                    }
                  })];
                }
              });
            }
          })];
        }
      });
    }
  });
}
function useFilteredWslServers(filter) {
  const wsl = useWslServers();
  return createMemo(() => {
    const servers = wsl.data?.servers ?? [];
    const query = filter().trim();
    if (!query) return servers;
    return fuzzysort.go(query, servers, {
      keys: [(item) => item.config.distro, (item) => item.config.id]
    }).map((x) => x.obj);
  });
}
function WslServerSettings(props) {
  const platform = usePlatform();
  const language = useLanguage();
  const wsl = useWslServers();
  const api = platform.wslServers;
  const request = useMutation(() => ({
    mutationFn: (action) => action(),
    onError: (error) => showToast({
      variant: "error",
      title: language.t("common.requestFailed"),
      description: error instanceof Error ? error.message : String(error)
    })
  }));
  const remove = (key) => {
    request.mutate(() => props.controller.handleRemove(key));
  };
  return createComponent(Show, {
    when: api,
    get children() {
      return createComponent(For, {
        get each() {
          return props.servers();
        },
        children: (item) => {
          const key = ServerConnection.Key.make(item.config.id);
          const check = () => wsl.data?.opencodeChecks[item.config.distro];
          const opencodeAction = () => wslOpencodeAction(check());
          const busy = () => wsl.data?.job?.kind === "install-opencode" && wsl.data.job.distro === item.config.distro;
          return (() => {
            var _el$ = _tmpl$$2(), _el$2 = _el$.firstChild, _el$3 = _el$2.firstChild, _el$4 = _el$3.firstChild, _el$5 = _el$4.firstChild, _el$6 = _el$5.nextSibling, _el$7 = _el$4.nextSibling, _el$8 = _el$2.nextSibling;
            insert(_el$2, createComponent(ServerHealthIndicator, {
              get health() {
                return props.controller.status()[key];
              }
            }), _el$3);
            insert(_el$5, () => item.config.distro);
            insert(_el$6, () => language.t("wsl.server.label"));
            insert(_el$7, createComponent(Show, {
              get when() {
                return check()?.version;
              },
              children: (version) => `v${version()}`
            }));
            insert(_el$8, createComponent(Show, {
              get when() {
                return memo(() => !!props.controller.canDefault())() && props.controller.defaultKey() === key;
              },
              get children() {
                return createComponent(Tag, {
                  get children() {
                    return language.t("dialog.server.status.default");
                  }
                });
              }
            }), null);
            insert(_el$8, createComponent(Show, {
              get when() {
                return opencodeAction();
              },
              children: (label) => createComponent(ButtonV2, {
                size: "small",
                get disabled() {
                  return busy() || request.isPending;
                },
                onClick: () => api && request.mutate(() => api.installOpencode(item.config.distro)),
                get children() {
                  return memo(() => !!busy())() ? language.t("wsl.server.updating") : label();
                }
              })
            }), null);
            insert(_el$8, createComponent(MenuV2, {
              gutter: 4,
              modal: false,
              placement: "bottom-end",
              get children() {
                return [createComponent(MenuV2.Trigger, {
                  as: IconButtonV2,
                  variant: "ghost-muted",
                  size: "small",
                  get icon() {
                    return createComponent(Icon, {
                      name: "outline-dots"
                    });
                  },
                  get ["aria-label"]() {
                    return language.t("common.moreOptions");
                  }
                }), createComponent(MenuV2.Portal, {
                  get children() {
                    return createComponent(MenuV2.Content, {
                      get children() {
                        return createComponent(MenuV2.Group, {
                          get children() {
                            return [createComponent(MenuV2.GroupLabel, {
                              get children() {
                                return language.t("wsl.server.menu.label");
                              }
                            }), createComponent(Show, {
                              get when() {
                                return wslRuntimeRetryable(item.runtime);
                              },
                              get children() {
                                return createComponent(MenuV2.Item, {
                                  onSelect: () => api && request.mutate(() => api.startServer(key)),
                                  get children() {
                                    return language.t("wsl.server.retryStart");
                                  }
                                });
                              }
                            }), createComponent(Show, {
                              get when() {
                                return memo(() => !!props.controller.canDefault())() && props.controller.defaultKey() !== key;
                              },
                              get children() {
                                return createComponent(MenuV2.Item, {
                                  onSelect: () => props.controller.setDefault(key),
                                  get children() {
                                    return language.t("dialog.server.menu.default");
                                  }
                                });
                              }
                            }), createComponent(Show, {
                              get when() {
                                return memo(() => !!props.controller.canDefault())() && props.controller.defaultKey() === key;
                              },
                              get children() {
                                return createComponent(MenuV2.Item, {
                                  onSelect: () => props.controller.setDefault(null),
                                  get children() {
                                    return language.t("dialog.server.menu.defaultRemove");
                                  }
                                });
                              }
                            }), createComponent(MenuV2.Separator, {}), createComponent(MenuV2.Item, {
                              onSelect: () => remove(key),
                              get children() {
                                return language.t("dialog.server.menu.delete");
                              }
                            })];
                          }
                        });
                      }
                    });
                  }
                })];
              }
            }), null);
            return _el$;
          })();
        }
      });
    }
  });
}
var _tmpl$$1 = /* @__PURE__ */ template(`<div class=settings-v2-tab-search>`), _tmpl$2 = /* @__PURE__ */ template(`<div class="settings-v2-tab-header settings-v2-servers-header"><div class=settings-v2-tab-header-row><h2 class=settings-v2-tab-title>`), _tmpl$3 = /* @__PURE__ */ template(`<div class="settings-v2-tab-body settings-v2-servers">`), _tmpl$4 = /* @__PURE__ */ template(`<span class=settings-v2-servers-status-filter>&quot;<!>&quot;`), _tmpl$5 = /* @__PURE__ */ template(`<div class=settings-v2-servers-status><span>`), _tmpl$6 = /* @__PURE__ */ template(`<div class=settings-v2-servers-row><div class=settings-v2-servers-lead><div class=settings-v2-servers-copy><span class=settings-v2-servers-name></span><span class=settings-v2-servers-meta></span></div></div><div class=settings-v2-servers-actions>`);
const SettingsServersV2 = () => {
  const dialog = useDialog();
  const language = useLanguage();
  const controller = useServerManagementController();
  const [store, setStore] = createStore({
    filter: ""
  });
  const wslServers = useFilteredWslServers(() => store.filter);
  const showSearch = createMemo(() => controller.sortedItems().filter((item) => !isWslServer(item)).length + wslServers().length > 1);
  const filtered = createMemo(() => {
    const items = controller.sortedItems().filter((item) => !isWslServer(item));
    const query = store.filter.trim();
    if (!query) return items;
    return fuzzysort.go(query, items, {
      keys: [(item) => serverName(item), (item) => item.http.url]
    }).map((result) => result.obj);
  });
  const openAdd = () => {
    dialog.push(() => createComponent(DialogServerV2, {
      mode: "add"
    }));
  };
  const openEdit = (server) => {
    dialog.push(() => createComponent(DialogServerV2, {
      mode: "edit",
      server
    }));
  };
  return [(() => {
    var _el$ = _tmpl$2(), _el$2 = _el$.firstChild, _el$3 = _el$2.firstChild;
    insert(_el$3, () => language.t("status.popover.tab.servers"));
    insert(_el$2, createComponent(AddServerMenu, {
      onAddServer: openAdd
    }), null);
    insert(_el$, createComponent(Show, {
      get when() {
        return showSearch();
      },
      get children() {
        var _el$4 = _tmpl$$1();
        insert(_el$4, createComponent(TextInputV2, {
          type: "search",
          appearance: "base",
          get value() {
            return store.filter;
          },
          onInput: (event) => setStore("filter", event.currentTarget.value),
          get placeholder() {
            return language.t("dialog.server.search.placeholder");
          },
          spellcheck: false,
          autocorrect: "off",
          autocomplete: "off",
          autocapitalize: "off",
          get ["aria-label"]() {
            return language.t("dialog.server.search.placeholder");
          }
        }), null);
        insert(_el$4, createComponent(Show, {
          get when() {
            return store.filter;
          },
          get children() {
            return createComponent(IconButtonV2, {
              type: "button",
              variant: "ghost-muted",
              size: "small",
              "class": "settings-v2-tab-search-clear",
              get icon() {
                return createComponent(Icon, {
                  name: "close",
                  size: "large",
                  "class": "text-v2-icon-icon-muted"
                });
              },
              onClick: () => setStore("filter", "")
            });
          }
        }), null);
        return _el$4;
      }
    }), null);
    createRenderEffect(() => _el$.classList.toggle("settings-v2-tab-header--stacked", !!showSearch()));
    return _el$;
  })(), (() => {
    var _el$5 = _tmpl$3();
    insert(_el$5, createComponent(Show, {
      get when() {
        return filtered().length > 0 || wslServers().length > 0;
      },
      get fallback() {
        return (() => {
          var _el$6 = _tmpl$5(), _el$7 = _el$6.firstChild;
          insert(_el$7, (() => {
            var _c$ = memo(() => !!store.filter);
            return () => _c$() ? language.t("palette.empty") : language.t("dialog.server.empty");
          })());
          insert(_el$6, createComponent(Show, {
            get when() {
              return store.filter;
            },
            get children() {
              var _el$8 = _tmpl$4(), _el$9 = _el$8.firstChild, _el$1 = _el$9.nextSibling;
              _el$1.nextSibling;
              insert(_el$8, () => store.filter, _el$1);
              return _el$8;
            }
          }), null);
          return _el$6;
        })();
      },
      get children() {
        return createComponent(SettingsListV2, {
          get children() {
            return [createComponent(WslServerSettings, {
              controller,
              servers: wslServers
            }), createComponent(For, {
              get each() {
                return filtered();
              },
              children: (item) => {
                const key = ServerConnection.key(item);
                const health = () => controller.status()[key];
                const isDefault = () => controller.defaultKey() === key;
                return (() => {
                  var _el$10 = _tmpl$6(), _el$11 = _el$10.firstChild, _el$12 = _el$11.firstChild, _el$13 = _el$12.firstChild, _el$14 = _el$13.nextSibling, _el$15 = _el$11.nextSibling;
                  insert(_el$11, createComponent(ServerHealthIndicator, {
                    get health() {
                      return health();
                    }
                  }), _el$12);
                  insert(_el$13, () => serverName(item));
                  insert(_el$14, createComponent(Show, {
                    get when() {
                      return health()?.version;
                    },
                    get children() {
                      return ["v", memo(() => health()?.version)];
                    }
                  }), null);
                  insert(_el$14, createComponent(Show, {
                    get when() {
                      return memo(() => !!health()?.version)() && item.type === "http";
                    },
                    children: " • "
                  }), null);
                  insert(_el$14, createComponent(Show, {
                    get when() {
                      return memo(() => item.type === "http")() && item.http.username;
                    },
                    get fallback() {
                      return createComponent(Show, {
                        get when() {
                          return item.type === "http";
                        },
                        get children() {
                          return language.t("server.row.noUsername");
                        }
                      });
                    },
                    get children() {
                      return item.http.username;
                    }
                  }), null);
                  insert(_el$15, createComponent(Show, {
                    get when() {
                      return memo(() => !!controller.canDefault())() && isDefault();
                    },
                    get children() {
                      return createComponent(Tag, {
                        get children() {
                          return language.t("dialog.server.status.default");
                        }
                      });
                    }
                  }), null);
                  insert(_el$15, createComponent(ServerRowMenu, {
                    server: item,
                    controller,
                    onEdit: openEdit
                  }), null);
                  return _el$10;
                })();
              }
            })];
          }
        });
      }
    }));
    return _el$5;
  })()];
};
var _tmpl$ = /* @__PURE__ */ template(`<div class="flex flex-col justify-between h-full w-full"><div class="flex flex-col gap-3 w-full"><div class="flex flex-col gap-3"><div class="flex flex-col gap-1.5"><div class="flex flex-col gap-1.5 w-full"></div></div><div class="flex flex-col gap-1.5"><div class="flex flex-col gap-1.5 w-full"></div></div></div></div><div class=settings-v2-nav-footer><span></span><span>v`);
const DialogSettings = () => {
  const language = useLanguage();
  const platform = usePlatform();
  return createComponent(Dialog, {
    size: "x-large",
    variant: "settings",
    "class": "settings-v2-dialog",
    get children() {
      return createComponent(TabsV2, {
        orientation: "vertical",
        variant: "settings",
        defaultValue: "general",
        "class": "settings-v2",
        get children() {
          return [createComponent(TabsV2.List, {
            get children() {
              var _el$ = _tmpl$(), _el$2 = _el$.firstChild, _el$3 = _el$2.firstChild, _el$4 = _el$3.firstChild, _el$5 = _el$4.firstChild, _el$6 = _el$4.nextSibling, _el$7 = _el$6.firstChild, _el$8 = _el$2.nextSibling, _el$9 = _el$8.firstChild, _el$0 = _el$9.nextSibling;
              _el$0.firstChild;
              insert(_el$4, createComponent(TabsV2.SectionTitle, {
                get children() {
                  return language.t("settings.section.desktop");
                }
              }), _el$5);
              insert(_el$5, createComponent(TabsV2.Trigger, {
                value: "general",
                get children() {
                  return [createComponent(Icon$1, {
                    name: "sliders"
                  }), memo(() => language.t("settings.tab.general"))];
                }
              }), null);
              insert(_el$5, createComponent(TabsV2.Trigger, {
                value: "shortcuts",
                get children() {
                  return [createComponent(Icon$1, {
                    name: "keyboard"
                  }), memo(() => language.t("settings.tab.shortcuts"))];
                }
              }), null);
              insert(_el$6, createComponent(TabsV2.SectionTitle, {
                get children() {
                  return language.t("settings.section.server");
                }
              }), _el$7);
              insert(_el$7, createComponent(TabsV2.Trigger, {
                value: "servers",
                get children() {
                  return [createComponent(Icon$1, {
                    name: "server"
                  }), memo(() => language.t("status.popover.tab.servers"))];
                }
              }), null);
              insert(_el$7, createComponent(TabsV2.Trigger, {
                value: "providers",
                get children() {
                  return [createComponent(Icon$1, {
                    name: "providers"
                  }), memo(() => language.t("settings.providers.title"))];
                }
              }), null);
              insert(_el$7, createComponent(TabsV2.Trigger, {
                value: "models",
                get children() {
                  return [createComponent(Icon$1, {
                    name: "models"
                  }), memo(() => language.t("settings.models.title"))];
                }
              }), null);
              insert(_el$9, () => language.t("app.name.desktop"));
              insert(_el$0, () => platform.version, null);
              return _el$;
            }
          }), createComponent(TabsV2.Content, {
            value: "general",
            "class": "settings-v2-panel",
            get children() {
              return createComponent(SettingsGeneralV2, {});
            }
          }), createComponent(TabsV2.Content, {
            value: "shortcuts",
            "class": "settings-v2-panel",
            get children() {
              return createComponent(SettingsKeybinds, {
                v2: true
              });
            }
          }), createComponent(TabsV2.Content, {
            value: "servers",
            "class": "settings-v2-panel",
            get children() {
              return createComponent(SettingsServersV2, {});
            }
          }), createComponent(TabsV2.Content, {
            value: "providers",
            "class": "settings-v2-panel",
            get children() {
              return createComponent(SettingsProvidersV2, {});
            }
          }), createComponent(TabsV2.Content, {
            value: "models",
            "class": "settings-v2-panel",
            get children() {
              return createComponent(SettingsModelsV2, {});
            }
          })];
        }
      });
    }
  });
};
export {
  DialogSettings
};
//# sourceMappingURL=index-DdbhCLyg.js.map
