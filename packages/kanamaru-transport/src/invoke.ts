import { Deferred, RpcError, RpcMetadata } from "@protobuf-ts/runtime-rpc";
import { IpcMessageBase, KanamaruStatus, RawReqwest } from "./commons";
import { invoke } from "@tauri-apps/api/core";
import { Code } from "./status-code";

export enum InvokeType {
    Unary,
    ServerStreaming,
    ClientStreaming,
    Duplex,
}

export function invokeTypeCommand(type: InvokeType): string {
    switch (type) {
        case InvokeType.Unary:
            return "unary"
        case InvokeType.ServerStreaming:
            return "server_streaming"
        case InvokeType.ClientStreaming:
            return "client_streaming"
        case InvokeType.Duplex:
            return "duplex"

        default:
            return "";
    }
}

export default async function invokeCall({ pluginName, type, args }: { pluginName: string; type: InvokeType; args: RawReqwest; }): Promise<IpcMessageBase | null> {
    return invoke<IpcMessageBase | null>(`plugin:${pluginName}|${invokeTypeCommand(type)}`, args)
        .catch((err: string | KanamaruStatus) => {
            if (typeof err == "string") {
                throw new RpcError(err, Code[Code.Internal]);
            } else {
                throw new RpcError(err.message, Code[err.code], err.metadata);
            }
        });
}