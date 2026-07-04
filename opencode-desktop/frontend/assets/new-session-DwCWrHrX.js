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
      n && (e._sentryDebugIds = e._sentryDebugIds || {}, e._sentryDebugIds[n] = "c143fd5e-f703-41e6-af03-2a9878d43589", e._sentryDebugIdIdentifier = "sentry-dbid-c143fd5e-f703-41e6-af03-2a9878d43589");
    })();
  } catch (e) {
  }
}
;
import { aw as createUniqueId, bc as setAttribute, ao as createRenderEffect, a9 as classList, bh as template, aa as className, N as NEW_SESSION_CONTENT_WIDTH, aL as insert, ad as createComponent, bF as usePrompt, bH as useSDK, bQ as useSync, bM as useServerSync, bp as useComments, bN as useSessionKey, bI as useSearchParams, ar as createSessionComposerState, aq as createSessionComposerControls, au as createStore, am as createMemo, ag as createEffect, bm as untrack, aZ as onMount, J as SessionComposerRegion } from "./main-D_cwiNV1.js";
var _tmpl$$2 = /* @__PURE__ */ template(`<svg xmlns=http://www.w3.org/2000/svg viewBox="0 0 720.002 129.001"fill=none preserveAspectRatio=none><g opacity=0.16><path opacity=0.7 d="M55.3846 36.8583H18.4615V92.144H55.3846V36.8583ZM73.8462 110.573H0V18.4297H73.8462V110.573Z"fill=currentColor></path><path opacity=0.7 d="M110.774 92.144H147.697V36.8583H110.774V92.144ZM166.159 110.573H110.774V129.001H92.3125V18.4297H166.159V110.573Z"fill=currentColor></path><path opacity=0.7 d="M258.463 73.7154H203.079V92.144H258.463V110.573H184.617V18.4297H258.463V73.7154ZM203.079 55.2868H240.002V36.8583H203.079V55.2868Z"fill=currentColor></path><path opacity=0.7 d="M332.306 36.8583H295.383V110.573H276.922V18.4297H332.306V36.8583ZM350.768 110.573H332.306V36.8583H350.768V110.573Z"fill=currentColor></path><path opacity=0.7 d="M443.081 36.8583H387.696V92.144H443.081V110.573H369.234V18.4297H443.081V36.8583Z"fill=currentColor></path><path opacity=0.7 d="M516.924 36.8583H480.001V92.144H516.924V36.8583ZM535.385 110.573H461.539V18.4297H535.385V110.573Z"fill=currentColor></path><path opacity=0.7 d="M609.228 36.8571H572.305V92.1429H609.228V36.8571ZM627.69 110.571H553.844V18.4286H609.228V0H627.69V110.571Z"fill=currentColor></path><path opacity=0.7 d="M664.618 36.8583V55.2868H701.541V36.8583H664.618ZM720.002 73.7154H664.618V92.144H720.002V110.573H646.156V18.4297H720.002V73.7154Z"fill=currentColor></path></g><defs><mask maskUnits=userSpaceOnUse x=0 y=0 width=720 height=129><rect width=720 height=129></rect></mask><linearGradient x1=360 y1=0 x2=360 y2=112 gradientUnits=userSpaceOnUse><stop stop-color=white stop-opacity=0.7></stop><stop offset=1 stop-color=white stop-opacity=0></stop></linearGradient><filter x=0 y=0 width=720.002 height=130.001 filterUnits=userSpaceOnUse color-interpolation-filters=sRGB><feFlood flood-opacity=0 result=BackgroundImageFix></feFlood><feBlend mode=normal in=SourceGraphic in2=BackgroundImageFix result=shape></feBlend><feColorMatrix in=SourceAlpha type=matrix values="0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 127 0"result=hardAlpha></feColorMatrix><feOffset dy=1></feOffset><feGaussianBlur stdDeviation=1></feGaussianBlur><feComposite in2=hardAlpha operator=arithmetic k2=-1 k3=1></feComposite><feColorMatrix type=matrix values="0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 0"></feColorMatrix><feBlend mode=normal in2=shape result=effect1_innerShadow_4938_16028>`);
function WordmarkV2(props) {
  const filter = createUniqueId();
  const mask = createUniqueId();
  const maskGradient = createUniqueId();
  return (() => {
    var _el$ = _tmpl$$2(), _el$2 = _el$.firstChild, _el$3 = _el$2.nextSibling, _el$4 = _el$3.firstChild, _el$5 = _el$4.firstChild, _el$6 = _el$4.nextSibling, _el$7 = _el$6.nextSibling;
    setAttribute(_el$2, "filter", `url(#${filter})`);
    setAttribute(_el$2, "mask", `url(#${mask})`);
    setAttribute(_el$4, "id", mask);
    setAttribute(_el$5, "fill", `url(#${maskGradient})`);
    setAttribute(_el$6, "id", maskGradient);
    setAttribute(_el$7, "id", filter);
    createRenderEffect((_$p) => classList(_el$, {
      [props.class ?? ""]: !!props.class
    }, _$p));
    return _el$;
  })();
}
var _tmpl$$1 = /* @__PURE__ */ template(`<div data-component=session-new-design class="relative size-full overflow-hidden bg-v2-background-bg-deep "><div class="absolute inset-x-0 top-[25.375%] flex justify-center px-6"><div><div class=mt-8>`);
function NewSessionDesignView(props) {
  return (() => {
    var _el$ = _tmpl$$1(), _el$2 = _el$.firstChild, _el$3 = _el$2.firstChild, _el$4 = _el$3.firstChild;
    className(_el$3, NEW_SESSION_CONTENT_WIDTH);
    insert(_el$3, createComponent(WordmarkV2, {
      "class": "h-auto w-full text-v2-icon-icon-base"
    }), _el$4);
    insert(_el$4, () => props.children);
    return _el$;
  })();
}
var _tmpl$ = /* @__PURE__ */ template(`<div class="relative size-full overflow-hidden flex flex-col"><div class="flex-1 min-h-0 flex flex-col gap-2 p-2"><div class="@container relative flex flex-col min-h-0 h-full bg-background-stronger flex-1"><div class="flex-1 min-h-0 overflow-hidden rounded-[10px]">`);
function NewSessionPage() {
  const prompt = usePrompt();
  const sdk = useSDK();
  const sync = useSync();
  const serverSync = useServerSync();
  const comments = useComments();
  const route = useSessionKey();
  const [searchParams, setSearchParams] = useSearchParams();
  let inputRef;
  const composer = createSessionComposerState();
  const composerControls = createSessionComposerControls({
    sessionKey: route.sessionKey,
    sessionID: () => route.params.id,
    queryOptions: serverSync().queryOptions
  });
  const [store, setStore] = createStore({
    worktree: "main"
  });
  const newSessionWorktree = createMemo(() => {
    if (store.worktree === "create") return "create";
    const project = sync().project;
    if (project && sdk().directory !== project.worktree) return sdk().directory;
    return "main";
  });
  createEffect(() => {
    if (!prompt.ready()) return;
    untrack(() => {
      const text = searchParams.prompt;
      if (!text) return;
      prompt.set([{
        type: "text",
        content: text,
        start: 0,
        end: text.length
      }], text.length);
      setSearchParams({
        ...searchParams,
        prompt: void 0
      });
    });
  });
  onMount(() => {
    requestAnimationFrame(() => inputRef?.focus());
  });
  return (() => {
    var _el$ = _tmpl$(), _el$2 = _el$.firstChild, _el$3 = _el$2.firstChild, _el$4 = _el$3.firstChild;
    insert(_el$4, createComponent(NewSessionDesignView, {
      get children() {
        return createComponent(SessionComposerRegion, {
          state: composer,
          get sessionKey() {
            return route.sessionKey();
          },
          get sessionID() {
            return route.params.id;
          },
          get controls() {
            return composerControls();
          },
          get promptInput() {
            return {
              ref: (el) => {
                inputRef = el;
              },
              newSessionWorktree: newSessionWorktree(),
              onNewSessionWorktreeReset: () => setStore("worktree", "main"),
              onSubmit: () => comments.clear()
            };
          },
          todo: {
            collapsed: false,
            onToggle: () => {
            }
          },
          ready: true,
          centered: false,
          placement: "inline",
          onResponseSubmit: () => {
          },
          setPromptDockRef: () => {
          }
        });
      }
    }));
    return _el$;
  })();
}
export {
  NewSessionPage as default
};
//# sourceMappingURL=new-session-DwCWrHrX.js.map
