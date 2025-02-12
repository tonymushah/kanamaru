import {
	ClientStreamingCall,
	Deferred,
	DuplexStreamingCall,
	mergeRpcOptions,
	RpcError,
	RpcInputStream,
	RpcMetadata,
	RpcOutputStreamController,
	RpcStatus,
	ServerStreamingCall,
	UnaryCall,
	type MethodInfo,
	type RpcOptions,
	type RpcTransport,
} from "@protobuf-ts/runtime-rpc";
import { Channel, invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import "core-js/actual/typed-array/from-base64";
import "core-js/actual/typed-array/to-base64";
import { Code } from "./status-code";
import { convertGrpcMeta, IpcMessageBase, isMessage, isStatus, KanamaruStatus, RawReqwest, ServerStreamingResponse } from "./commons";
import ClientStreamingStreamController from "./client-stream-controller";

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

		const cancel_token_event_id = generate_event_id();

		const cancel_fn = () => {
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
			.finally(() => {
				window.removeEventListener("unload", cancel_fn);
				if (opt.abort) {
					opt.abort.removeEventListener("abort", cancel_fn);
				}
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
			outStream = new RpcOutputStreamController<O>(),
			defStatus = new Deferred<RpcStatus>(),
			defTrailer = new Deferred<RpcMetadata>(),
			call = new ServerStreamingCall<I, O>(
				method,
				meta,
				input,
				defHeader.promise,
				outStream,
				defStatus.promise,
				defTrailer.promise
			);



		const appWebview = getCurrentWebview();

		const cancel_token_event_id = generate_event_id();

		const server_streaming_event_id = generate_event_id();

		function cancel() {
			appWebview.emitTo(
				{
					kind: "Webview",
					label: appWebview.label,
				},
				cancel_token_event_id
			);
		}

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
					cancel();
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
		const cancel_fn = () => {
			cancel();
			unlisten_stream();
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
			server_streaming_event_id,
		};

		invokeArgs.status_channel.onmessage = (status) => {
			defStatus.resolvePending({
				code: Code[status.code],
				detail: status.message,
			});
			defTrailer.resolvePending(status.metadata);
		};

		invoke(`plugin:${this.pluginName}|server_streaming`, invokeArgs)
			.catch((err: string | KanamaruStatus) => {
				if (typeof err == "string") {
					throw new RpcError(err, Code[Code.Internal]);
				} else {
					throw new RpcError(err.message, Code[err.code], err.metadata);
				}
			})
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
				window.removeEventListener("unload", cancel_fn);
				unlisten_stream();
				if (opt.abort) {
					opt.abort.removeEventListener("abort", cancel_fn);
				}
			});

		return call;
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

		const cancel_token_event_id = generate_event_id();

		const client_streaming_event_id = generate_event_id();

		const inStream = new ClientStreamingStreamController(method, appWebview, client_streaming_event_id);

		const cancel_fn = () => {
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

		invoke<IpcMessageBase | null>(`plugin:${this.pluginName}|client_streaming`, invokeArgs)
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
			.finally(() => {
				window.removeEventListener("unload", cancel_fn);
				if (opt.abort) {
					opt.abort.removeEventListener("abort", cancel_fn);
				}
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
			outStream = new RpcOutputStreamController<O>(),
			defStatus = new Deferred<RpcStatus>(),
			defTrailer = new Deferred<RpcMetadata>();
		const appWebview = getCurrentWebview();

		const cancel_token_event_id = generate_event_id();

		const server_streaming_event_id = generate_event_id();

		const client_streaming_event_id = generate_event_id();

		const inStream = new ClientStreamingStreamController(method, appWebview, client_streaming_event_id);

		function cancel() {
			appWebview.emitTo(
				{
					kind: "Webview",
					label: appWebview.label,
				},
				cancel_token_event_id
			);
		}

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
					cancel();
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
		const cancel_fn = () => {
			cancel();
			unlisten_stream();
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

		invoke(`plugin:${this.pluginName}|duplex`, invokeArgs)
			.catch((err: string | KanamaruStatus) => {
				if (typeof err == "string") {
					throw new RpcError(err, Code[Code.Internal]);
				} else {
					throw new RpcError(err.message, Code[err.code], err.metadata);
				}
			})
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
				window.removeEventListener("unload", cancel_fn);
				unlisten_stream();
				if (opt.abort) {
					opt.abort.removeEventListener("abort", cancel_fn);
				}
			});

		return new DuplexStreamingCall(method, meta, inStream, defHeader.promise, outStream, defStatus.promise, defTrailer.promise)
	}
}
