import {
	ClientStreamingCall,
	Deferred,
	DuplexStreamingCall,
	mergeRpcOptions,
	RpcError,
	RpcMetadata,
	RpcStatus,
	ServerStreamingCall,
	UnaryCall,
	type MethodInfo,
	type RpcOptions,
	type RpcTransport
} from "@protobuf-ts/runtime-rpc";
import { Channel } from "@tauri-apps/api/core";
import { UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import "core-js/actual/typed-array/from-base64";
import "core-js/actual/typed-array/to-base64";
import ClientStreamingStreamController from "./client-stream-controller";
import { convertGrpcMeta, DeferredFunction, generate_event_id, IpcMessageBase, RawReqwest } from "./commons";
import invokeCall, { InvokeType } from "./invoke";
import make_cancel from "./make_cancel";
import make_server_streaming from "./make_server_streaming";
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


export class KanamaruTransport implements RpcTransport {
	private readonly defaultOptions: RpcOptions;
	private readonly pluginName: string;
	constructor(pluginName: string, defaultOptions: RpcOptions) {
		this.pluginName = pluginName;
		this.defaultOptions = defaultOptions;

	}
	invokeCall(type: InvokeType, arg: RawReqwest): Promise<IpcMessageBase | null> {
		return invokeCall({
			type,
			args: arg,
			pluginName: this.pluginName
		})
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

		const { unlistenAll, cancel_token_event_id } = make_cancel({
			appWebview,
			abortSignal: opt.abort
		})

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

		this.invokeCall(InvokeType.Unary, invokeArgs)
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
			.finally(() => {
				unlistenAll()
			});
		return call;
	}


	serverStreaming<I extends object, O extends object>(
		method: MethodInfo<I, O>,
		input: I,
		options: RpcOptions
	): ServerStreamingCall<I, O> {
		const opt = this.mergeOptions(options),
			meta = opt.meta ?? {},
			defHeader = new Deferred<RpcMetadata>(),
			defStatus = new Deferred<RpcStatus>(),
			defTrailer = new Deferred<RpcMetadata>();

		const appWebview = getCurrentWebview();

		const cancel = new DeferredFunction<UnlistenFn>();
		const stream_unlistener = new DeferredFunction<UnlistenFn>();

		const { cancel_token_event_id, cancel_fn, unlistenAll } = make_cancel({
			appWebview,
			abortSignal: opt.abort,
			other: () => {
				stream_unlistener.call([]);
			}
		});
		cancel.func = cancel_fn;

		const { unlisten, server_streaming_event_id, outStream } = make_server_streaming({
			appWebview,
			defHeader,
			defStatus,
			defTrailer,
			method,
			cancel: () => {
				cancel.call([])
			}
		});
		stream_unlistener.func = unlisten;

		const invokeArgs: RawReqwest = {
			route: `${method.service.typeName}/${method.name}`,
			cancel_token_event_id,
			payload: {
				metadata: convertGrpcMeta(meta),
				body: method.I.toBinary(input, opt.binaryOptions).toBase64(),
			},
			status_channel: new Channel(),
			server_streaming_event_id,
		};

		invokeArgs.status_channel.onmessage = (status) => {
			defStatus.resolvePending({
				code: Code[status.code],
				detail: status.message,
			});
			defTrailer.resolvePending(status.metadata);
		};

		this.invokeCall(InvokeType.ServerStreaming, invokeArgs)
			.then(() => {
				if (!outStream.closed) {
					outStream.notifyComplete();
				}
				defStatus.resolvePending({
					code: Code[Code.Ok],
					detail: "Done!",
				});
				defTrailer.resolvePending({});
				defHeader.resolvePending({});
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
				if (!outStream.closed) {
					outStream.notifyError(error);
				}
				defStatus.rejectPending(error);
				defTrailer.rejectPending(error);
			})
			.finally(() => {
				unlistenAll()
			});

		return new ServerStreamingCall<I, O>(
			method,
			meta,
			input,
			defHeader.promise,
			outStream,
			defStatus.promise,
			defTrailer.promise
		);
	}


	clientStreaming<I extends object, O extends object>(
		method: MethodInfo<I, O>,
		options: RpcOptions
	): ClientStreamingCall<I, O> {
		const opt = this.mergeOptions(options),
			meta = opt.meta ?? {},
			defHeader = new Deferred<RpcMetadata>(),
			defMessage = new Deferred<O>(),
			defStatus = new Deferred<RpcStatus>(),
			defTrailer = new Deferred<RpcMetadata>();

		const appWebview = getCurrentWebview();

		const client_streaming_event_id = generate_event_id();

		const inStream = new ClientStreamingStreamController(method, appWebview, client_streaming_event_id);

		const { cancel_token_event_id, unlistenAll } = make_cancel({
			appWebview,
			abortSignal: opt.abort
		})

		const invokeArgs: RawReqwest = {
			route: `${method.service.typeName}/${method.name}`,
			cancel_token_event_id,
			status_channel: new Channel(),
			client_streaming_event_id,
			payload: {
				metadata: convertGrpcMeta(meta)
			}
		};

		invokeArgs.status_channel.onmessage = (status) => {
			defStatus.resolvePending({
				code: Code[status.code],
				detail: status.message,
			});
			defTrailer.resolvePending(status.metadata);
		};

		this.invokeCall(InvokeType.ClientStreaming, invokeArgs)
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
			.finally(() => {
				unlistenAll()
			});

		return new ClientStreamingCall<I, O>(method, meta, inStream, defHeader.promise, defMessage.promise, defStatus.promise, defTrailer.promise);
	}


	duplex<I extends object, O extends object>(
		method: MethodInfo<I, O>,
		options: RpcOptions
	): DuplexStreamingCall<I, O> {
		const opt = this.mergeOptions(options),
			meta = opt.meta ?? {},
			defHeader = new Deferred<RpcMetadata>(),
			defStatus = new Deferred<RpcStatus>(),
			defTrailer = new Deferred<RpcMetadata>();
		const appWebview = getCurrentWebview();

		const client_streaming_event_id = generate_event_id();

		const inStream = new ClientStreamingStreamController(method, appWebview, client_streaming_event_id);

		const cancel = new DeferredFunction<UnlistenFn>();
		const stream_unlistener = new DeferredFunction<UnlistenFn>();

		const { cancel_token_event_id, cancel_fn, unlistenAll } = make_cancel({
			appWebview,
			abortSignal: opt.abort,
			other: () => {
				stream_unlistener.call([]);
			}
		});
		cancel.func = cancel_fn;

		const { unlisten, server_streaming_event_id, outStream } = make_server_streaming({
			appWebview,
			defHeader,
			defStatus,
			defTrailer,
			method,
			cancel: () => {
				cancel.call([])
			}
		});
		stream_unlistener.func = unlisten;

		const invokeArgs: RawReqwest = {
			route: `${method.service.typeName}/${method.name}`,
			cancel_token_event_id,
			payload: {
				metadata: convertGrpcMeta(meta),
			},
			status_channel: new Channel(),
			server_streaming_event_id,
			client_streaming_event_id
		};

		invokeArgs.status_channel.onmessage = (status) => {
			defStatus.resolvePending({
				code: Code[status.code],
				detail: status.message,
			});
			defTrailer.resolvePending(status.metadata);
		};

		this.invokeCall(InvokeType.Duplex, invokeArgs)
			.then(() => {
				if (!outStream.closed) {
					outStream.notifyComplete();
				}
				defStatus.resolvePending({
					code: Code[Code.Ok],
					detail: "Done!",
				});
				defTrailer.resolvePending({});
				defHeader.resolvePending({});
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
				if (!outStream.closed) {
					outStream.notifyError(error);
				}
				defStatus.rejectPending(error);
				defTrailer.rejectPending(error);
			})
			.finally(() => {
				unlistenAll()
			});

		return new DuplexStreamingCall(method, meta, inStream, defHeader.promise, outStream, defStatus.promise, defTrailer.promise)
	}
}
