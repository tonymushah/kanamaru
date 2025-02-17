import { UnlistenFn } from "@tauri-apps/api/event"
import { Webview } from "@tauri-apps/api/webview"
import { generate_event_id } from "./commons"
import { onEvent, onWindowUnload } from "./onUnload"

type MakeCancelReturnType = {
    cancel_token_event_id: string,
    abortUnlisten: UnlistenFn,
    windowUnlisten: UnlistenFn,
    cancel_fn: UnlistenFn,
    unlistenAll: UnlistenFn
}

type MakeCancelArgs = {
    appWebview: Webview
    abortSignal?: AbortSignal,
    other?: UnlistenFn
}

export default function make_cancel({ appWebview, abortSignal, other }: MakeCancelArgs): MakeCancelReturnType {
    const cancel_token_event_id = generate_event_id();
    const cancel_fn = () => {
        appWebview.emitTo({
            label: appWebview.label,
            kind: "Webview"
        }, cancel_token_event_id);
        other?.();
    }
    const abortUnlisten = abortSignal ? onEvent({
        target: abortSignal,
        event: cancel_fn,
        name: "abort"
    }) : () => { };
    const windowUnlisten = onWindowUnload(cancel_fn);
    return {
        cancel_token_event_id,
        abortUnlisten,
        windowUnlisten,
        cancel_fn,
        unlistenAll: () => {
            abortUnlisten();
            windowUnlisten();
            other?.();
        }
    }
}