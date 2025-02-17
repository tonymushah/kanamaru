import { Deferred, MethodInfo, RpcError, RpcInputStream, RpcMetadata, RpcOutputStream, RpcOutputStreamController, RpcStatus } from "@protobuf-ts/runtime-rpc"
import { UnlistenFn } from "@tauri-apps/api/event"
import { Webview } from "@tauri-apps/api/webview"
import { generate_event_id, ServerStreamingResponse } from "./commons"
import { Code } from "./status-code"

type MakeServerStreamingArgs<I extends object, O extends object> = {
    appWebview: Webview,
    defHeader: Deferred<RpcMetadata>,
    defStatus: Deferred<RpcStatus>,
    defTrailer: Deferred<RpcMetadata>,
    method: MethodInfo<I, O>,
    cancel?: UnlistenFn
}

type MakeServerStreamingRet<O extends object> = {
    unlisten: UnlistenFn,
    server_streaming_event_id: string,
    outStream: RpcOutputStreamController<O>
}

export default function make_server_streaming<I extends object, O extends object>({ appWebview, defHeader, defStatus, defTrailer, method, cancel }: MakeServerStreamingArgs<I, O>): MakeServerStreamingRet<O> {
    const server_streaming_event_id = generate_event_id();

    const outStream = new RpcOutputStreamController<O>()

    const stream_listener = appWebview.listen<ServerStreamingResponse | null>(server_streaming_event_id, (ev) => {
        if (ev.payload != null) {
            const _payload = ev.payload;
            if (_payload.Err != undefined) {
                const payload = _payload.Err;
                const e = new RpcError(
                    payload.message,
                    Code[payload.code],
                    payload.metadata
                );
                e.methodName = method.name;
                e.serviceName = method.service.typeName;
                defHeader.rejectPending(e);
                if (!outStream.closed) {
                    outStream.notifyError(e);
                }
                defStatus.rejectPending(e);
                defTrailer.rejectPending(e);
                cancel?.();
            } else if (_payload.Ok != undefined) {
                const payload = _payload.Ok;
                if (payload.body != null && payload.body != undefined) {
                    outStream.notifyMessage(
                        method.O.fromBinary(Uint8Array.fromBase64(payload.body))
                    );
                } else {
                    outStream.notifyMessage(method.O.create());
                }
            }
        }
    });

    const unlisten_stream = () => stream_listener.then((f) => f());

    return {
        unlisten: unlisten_stream,
        server_streaming_event_id,
        outStream
    }
}