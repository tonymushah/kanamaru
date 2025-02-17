pub mod protos;

use std::{pin::Pin, time::Duration};

use async_stream::try_stream;
use kanamaru::{
    ipc::IpcMessage, RequestBase, Status, StreamingRequest, StreamingResponse, UnaryRequest,
    UnaryResponse,
};
use protos::{
    example1::{
        hello_service_server::{HelloService, HelloServiceResponder},
        HelloRequest, HelloResponse,
    },
    Empty,
};
use tauri::Runtime;
use tokio::sync::watch::Sender;
use tokio_stream::{wrappers::WatchStream, Stream, StreamExt};

pub struct HelloServiceInternal {
    event_sender: Sender<String>,
}

#[kanamaru::async_trait]
impl HelloService for HelloServiceInternal {
    async fn say_hello<R: Runtime>(
        &self,
        request: UnaryRequest<R, HelloRequest>,
    ) -> Result<UnaryResponse<HelloResponse>, Status> {
        let resp = format!("Hello {}!", request.message().name);
        let _ = self.event_sender.send_replace(resp.clone());
        Ok(UnaryResponse::new(HelloResponse { response: resp }))
    }

    type ListenToHellosStream = WatchStream<Result<IpcMessage<HelloResponse>, Status>>;
    async fn listen_to_hellos<R: Runtime>(
        &self,
        request: UnaryRequest<R, Empty>,
    ) -> Result<StreamingResponse<HelloResponse, Self::ListenToHellosStream>, Status> {
        let self_recv = self.event_sender.subscribe();
        let cancel = request.cancel_token();
        let (sender, recv) = tokio::sync::watch::channel(Ok(IpcMessage::new(HelloResponse {
            response: (*self_recv.borrow()).clone(),
        })));
        tokio::spawn(async move {
            let mut stream = WatchStream::new(self_recv);
            loop {
                tokio::select! {
                    _ = cancel.cancelled() => {
                        break;
                    }
                    Some(response) = stream.next() => {
                        let maybe_send = sender.send(Ok(IpcMessage::new(HelloResponse { response })));
                        if maybe_send.is_err() {
                            break;
                        }
                    }
                    else => break
                }
            }
        });

        Ok(StreamingResponse::new(WatchStream::from_changes(recv)))
    }
    async fn say_hellos<R: Runtime>(
        &self,
        mut request: StreamingRequest<R, HelloRequest>,
    ) -> Result<UnaryResponse<Empty>, Status> {
        while let Some(data) = request.stream_mut().next().await {
            let resp = format!("Hello {}!", data?.body.name);
            let _ = self.event_sender.send_replace(resp.clone());
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
        println!("finished stream!!");
        Ok(UnaryResponse::new(Empty {}))
    }

    type SayHelloWithResponsesStream = Pin<
        Box<dyn Stream<Item = Result<IpcMessage<HelloResponse>, Status>> + Send + Sync + 'static>,
    >;

    async fn say_hello_with_responses<R: Runtime>(
        &self,
        mut request: StreamingRequest<R, HelloRequest>,
    ) -> Result<StreamingResponse<HelloResponse, Self::SayHelloWithResponsesStream>, Status> {
        let sender = self.event_sender.clone();

        let stream = try_stream! {
            while let Some(data) = request.stream_mut().next().await {
                let data = data?;
                let resp = format!("Hello {}!", data.body.name);
                let _ = sender.send_replace(resp.clone());
                yield IpcMessage::new(HelloResponse { response: resp });
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        };

        Ok(StreamingResponse::new(
            Box::pin(stream) as Self::SayHelloWithResponsesStream
        ))
    }
}

pub fn init<R: Runtime>() -> kanamaru::KanamaruPlugin<R> {
    kanamaru::Builder::new("example-1-plugin")
        .add_route(HelloServiceResponder::new(HelloServiceInternal {
            event_sender: Default::default(),
        }))
        .build()
}
