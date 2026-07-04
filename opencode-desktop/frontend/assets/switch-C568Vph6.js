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
      n && (e._sentryDebugIds = e._sentryDebugIds || {}, e._sentryDebugIds[n] = "802d216c-8f4d-4d25-a553-08f6fe84775a", e._sentryDebugIdIdentifier = "sentry-dbid-802d216c-8f4d-4d25-a553-08f6fe84775a");
    })();
  } catch (e) {
  }
}
;
import { S as Switch$1 } from "./LROKH5N7-r-w1NcZW.js";
import { bf as splitProps, ad as createComponent, aR as mergeProps, O as Show } from "./main-D_cwiNV1.js";
function Switch(props) {
  const [local, others] = splitProps(props, ["children", "class", "hideLabel", "description"]);
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
        get children() {
          return createComponent(Switch$1.Label, {
            "data-slot": "switch-label",
            get classList() {
              return {
                "sr-only": local.hideLabel
              };
            },
            get children() {
              return local.children;
            }
          });
        }
      }), createComponent(Show, {
        get when() {
          return local.description;
        },
        get children() {
          return createComponent(Switch$1.Description, {
            "data-slot": "switch-description",
            get children() {
              return local.description;
            }
          });
        }
      }), createComponent(Switch$1.ErrorMessage, {
        "data-slot": "switch-error"
      }), createComponent(Switch$1.Control, {
        "data-slot": "switch-control",
        get children() {
          return createComponent(Switch$1.Thumb, {
            "data-slot": "switch-thumb"
          });
        }
      })];
    }
  }));
}
export {
  Switch as S
};
//# sourceMappingURL=switch-C568Vph6.js.map
