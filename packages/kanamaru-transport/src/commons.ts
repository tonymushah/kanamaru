import { RpcMetadata } from "@protobuf-ts/runtime-rpc";
import { Channel } from "@tauri-apps/api/core";

export type KanamaruStatus = {
    code: number;
    metadata: Record<string, string>;
    message: string;
};

export function isStatus(param: unknown): param is KanamaruStatus {
    return (
        typeof param === "object" &&
        param !== null &&
        "code" in param &&
        "metadata" in param &&
        "message" in param &&
        typeof (param as KanamaruStatus).code === "number" &&
        typeof (param as KanamaruStatus).message === "string" &&
        typeof (param as KanamaruStatus).metadata === "object" &&
        param.metadata !== null &&
        Object.values((param as KanamaruStatus).metadata).every(
            (value) => typeof value === "string"
        )
    );
}

export type IpcMessageBase = {
    metadata: Record<string, string>;
    body?: string | null;
};

export function isMessage(param: unknown): param is IpcMessageBase {
    return (
        typeof param === "object" &&
        param !== null &&
        "metadata" in param &&
        typeof (param as any).metadata === "object" &&
        (typeof (param as any).body === "string" ||
            (param as any).body === null ||
            typeof (param as any).body === "undefined")
    );
}

export type RawReqwest = {
    route: string;
    cancel_token_event_id: string;
    payload?: IpcMessageBase;
    client_streaming_event_id?: string;
    server_streaming_event_id?: string;
    status_channel: Channel<KanamaruStatus>;
};

export function convertGrpcMeta(meta: RpcMetadata): Record<string, string> {
    let map = new Map<string, string>();
    for (const key in meta) {
        if (Object.prototype.hasOwnProperty.call(meta, key)) {
            const element = meta[key];
            if (typeof element == "string") {
                map.set(key, element);
            } else {
                const value = element.reduce((prev, next) => `${prev}||${next}`, "");
                if (value.length != 0) map.set(key, value);
            }
        }
    }
    return Object.fromEntries(map.entries());
}

export type ServerStreamingResponse = {
    Ok?: IpcMessageBase,
    Err?: KanamaruStatus
}

export function generate_event_id(): string {
    return `${Math.floor(Math.random() * 10000000)}`;
}

export class DeferredFunction<F extends (...args: any) => any> {
    private _func?: F
    constructor(func?: F) {
        this._func = func;
    }

    public set func(v: F) {
        this._func = v;
    }

    public call(args: Parameters<F>): ReturnType<F> | undefined {
        return this._func?.(args)
    }
}