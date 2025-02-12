import {
  Deferred,
  mergeRpcOptions,
  RpcError,
  RpcMetadata,
  RpcStatus,
  UnaryCall,
  type ClientStreamingCall,
  type DuplexStreamingCall,
  type MethodInfo,
  type RpcOptions,
  type RpcTransport,
  type ServerStreamingCall,
} from "@protobuf-ts/runtime-rpc";
import { Channel, invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import "core-js/actual/typed-array/from-base64";
import "core-js/actual/typed-array/to-base64";
import { Code } from "./status-code";

declare global {
  interface Uint8ArrayConstructor {
    fromBase64(
      string: string,
      options?: {
        alphabet?: "base64" | "base64url";
        lastChunkHandling?: "loose" | "strict" | "stop-before-partial";
      }
    ): Uint8Array;
  }
  interface Uint8Array {
    toBase64(options?: {
      alphabet?: "base64" | "base64url";
      omitPadding?: boolean;
    }): string;
  }
}

type KanamaruStatus = {
  code: number;
  metadata: Record<string, string>;
  message: string;
};

type IpcMessageBase = {
  metadata: Record<string, string>;
  body?: string | null;
};

type RawReqwest = {
  route: string;
  cancel_token_event_id: string;
  payload?: IpcMessageBase;
  client_streaming_event_id?: string;
  server_streaming_event_id?: string;
  status_channel: Channel<KanamaruStatus>;
};

function convertGrpcMeta(meta: RpcMetadata): Record<string, string> {
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

function generate_event_id(): string {
  return `${Math.floor(Math.random() * 10000000)}`;
}

export class KanamaruTransport implements RpcTransport {
  private readonly defaultOptions: RpcOptions;
  private readonly pluginName: string;
  constructor(pluginName: string, defaultOptions: RpcOptions) {
    this.pluginName = pluginName;
    this.defaultOptions = defaultOptions;
  }
  mergeOptions(options?: Partial<RpcOptions>): RpcOptions {
    return mergeRpcOptions(this.defaultOptions, options);
  }
  unary<I extends object, O extends object>(
    method: MethodInfo<I, O>,
    input: I,
    options: RpcOptions
  ): UnaryCall<I, O> {
    const opt = this.mergeOptions(options),
      meta = opt.meta ?? {},
      defHeader = new Deferred<RpcMetadata>(),
      defMessage = new Deferred<O>(),
      defStatus = new Deferred<RpcStatus>(),
      defTrailer = new Deferred<RpcMetadata>(),
      call = new UnaryCall<I, O>(
        method,
        meta,
        input,
        defHeader.promise,
        defMessage.promise,
        defStatus.promise,
        defTrailer.promise
      );
    const appWebview = getCurrentWebview();

    let cancel_token_event_id = generate_event_id();
    let cancel_fn = () => {
      appWebview.emitTo(
        {
          kind: "Webview",
          label: appWebview.label,
        },
        cancel_token_event_id
      );
    };
    window.addEventListener("unload", cancel_fn);
    if (opt.abort) {
      opt.abort.addEventListener("abort", cancel_fn);
    }
    const invokeArgs: RawReqwest = {
      route: `${method.service.typeName}/${method.name}`,
      cancel_token_event_id,
      payload: {
        metadata: convertGrpcMeta(meta),
        body: method.I.toBinary(input, opt.binaryOptions).toBase64(),
      },
      status_channel: new Channel(),
    };
    invokeArgs.status_channel.onmessage = (status) => {
      defStatus.resolvePending({
        code: Code[status.code],
        detail: status.message,
      });
      defTrailer.resolvePending(status.metadata);
    };
    invoke<IpcMessageBase | null>(`plugin:${this.pluginName}|unary`, invokeArgs)
      .catch((err: string | KanamaruStatus) => {
        if (typeof err == "string") {
          throw new RpcError(err, Code[Code.Internal]);
        } else {
          throw new RpcError(err.message, Code[err.code], err.metadata);
        }
      })
      .then((res) => {
        if (res == null) {
          throw new RpcError("Invalid response", Code[Code.DataLoss]);
        }
        defHeader.resolvePending(res.metadata);
        if (res.body == undefined || res.body == null) {
          throw new RpcError("Invalid response", Code[Code.DataLoss]);
        }
        defMessage.resolvePending(
          method.O.fromBinary(
            Uint8Array.fromBase64(res.body),
            opt.binaryOptions
          )
        );
        defStatus.resolvePending({
          code: Code[Code.Ok],
          detail: "Done!",
        });
        defTrailer.resolvePending({});
      })
      .catch((err) => {
        let error: RpcError;
        if (err instanceof RpcError) {
          error = err;
        } else if (typeof err == "string") {
          error = new RpcError(err, Code[Code.Internal]);
        } else {
          error = new RpcError(
            err instanceof Error ? err.message : "",
            Code[Code.Internal]
          );
        }
        error.methodName = method.name;
        error.serviceName = method.service.typeName;
        defHeader.rejectPending(error);
        defMessage.rejectPending(error);
        defStatus.rejectPending(error);
        defTrailer.rejectPending(error);
      })
      .finally(() => window.removeEventListener("unload", cancel_fn));
    return call;
  }
  serverStreaming<I extends object, O extends object>(
    method: MethodInfo<I, O>,
    input: I,
    options: RpcOptions
  ): ServerStreamingCall<I, O> {
    throw new Error("Method not implemented.");
  }
  clientStreaming<I extends object, O extends object>(
    method: MethodInfo<I, O>,
    options: RpcOptions
  ): ClientStreamingCall<I, O> {
    throw new Error("Method not implemented.");
  }
  duplex<I extends object, O extends object>(
    method: MethodInfo<I, O>,
    options: RpcOptions
  ): DuplexStreamingCall<I, O> {
    throw new Error("Method not implemented.");
  }
}
