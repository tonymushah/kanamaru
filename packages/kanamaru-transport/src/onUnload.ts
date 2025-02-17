import { UnlistenFn } from "@tauri-apps/api/event";

export function onEvent({ target, event, name }: { target: EventTarget; event: EventListenerOrEventListenerObject; name: string; }): UnlistenFn {
    target.addEventListener(name, event);
    return () => {
        target.removeEventListener(name, event);
    }
}

export function onWindowUnload(event: EventListenerOrEventListenerObject): UnlistenFn {
    return onEvent({
        target: window,
        event,
        name: "unload"
    });
}
