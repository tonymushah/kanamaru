import { MethodInfo, RpcInputStream } from "@protobuf-ts/runtime-rpc";
import { Webview } from "@tauri-apps/api/webview";
import { IpcMessageBase } from "./commons";

export default class ClientStreamingStreamController<I extends object, O extends object> implements RpcInputStream<I> {
    private _completed: boolean;

    private webview: Webview;
    private event_id: string;
    private method_info: MethodInfo<I, O>


    public get completed(): boolean {
        return this._completed;
    }


    constructor(method_info: MethodInfo<I, O>, webview: Webview, event_id: string) {
        this.webview = webview;
        this.event_id = event_id;
        this._completed = false;
        this.method_info = method_info;
    }
    async send(message: I): Promise<void> {
        if (!this._completed) {
            const toSend: IpcMessageBase = {
                metadata: {},
                body: this.method_info.I.toBinary(message).toBase64()
            };
            await this.webview.emitTo({
                kind: "Webview",
                label: this.webview.label
            }, this.event_id, toSend)
        }
    }
    async complete(): Promise<void> {
        this._completed = true;
        await this.webview.emitTo({
            kind: "Webview",
            label: this.webview.label
        }, this.event_id, null)
    }
}