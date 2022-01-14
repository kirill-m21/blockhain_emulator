//! Simple websocket client.
use std::time::Duration;
use std::{io, thread};

mod blockchain;
use blockchain::Blockchain;

use actix::io::SinkWrite;
use actix::*;
use actix_codec::Framed;
use awc::{
    error::WsProtocolError,
    ws::{Codec, Frame, Message},
    BoxedSocket, Client,
};
use bytes::Bytes;
use futures::stream::{SplitSink, StreamExt};

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let sys = System::new("websocket-client");

    Arbiter::spawn(async {
        let (response1, framed1) = Client::new()
            .ws("http://127.0.0.1:8081/ws/")
            .connect()
            .await
            .map_err(|e| {
                println!("Error: {}", e);
            })
            .unwrap();
        // let (response2, framed2) = Client::new()
        //     .ws("http://127.0.0.1:8082/ws/")
        //     .connect()
        //     .await
        //     .map_err(|e| {
        //         println!("Error: {}", e);
        //     })
        //     .unwrap();
        // let (response3, framed3) = Client::new()
        //     .ws("http://127.0.0.1:8083/ws/")
        //     .connect()
        //     .await
        //     .map_err(|e| {
        //         println!("Error: {}", e);
        //     })
        //     .unwrap();

        println!("{:?}\n", response1);
        // println!("{:?}\n", response2);
        // println!("{:?}\n", response3);

        let (sink1, stream1) = framed1.split();
        // let (sink2, stream2) = framed2.split();
        // let (sink3, stream3) = framed3.split();
        let addr1 = ChatClient::create(|ctx| {
            ChatClient::add_stream(stream1, ctx);
            ChatClient(SinkWrite::new(sink1, ctx))
        });
        // let addr2 = ChatClient::create(|ctx| {
        //     ChatClient::add_stream(stream2, ctx);
        //     ChatClient(SinkWrite::new(sink2, ctx))
        // });
        // let addr3 = ChatClient::create(|ctx| {
        //     ChatClient::add_stream(stream3, ctx);
        //     ChatClient(SinkWrite::new(sink3, ctx))
        // });
        // start console loop
        thread::spawn(move || loop {
            let mut cmd = String::new();
            if io::stdin().read_line(&mut cmd).is_err() {
                println!("error");
                return;
            }
            let blnch = Blockchain::new();
            let cmd = blnch.chain.front().unwrap();

            //addr1.try_send(cmd)
            addr1.do_send(ClientCommand(cmd.header.timestamp.clone()));
            addr1.do_send(ClientCommand(cmd.header.nonce.to_string()));

            addr1.do_send(ClientCommand(cmd.tr_data.from.clone()));
            addr1.do_send(ClientCommand(cmd.tr_data.to.clone()));
            addr1.do_send(ClientCommand(cmd.tr_data.amount.to_string()));

            addr1.do_send(ClientCommand(cmd.hash.clone()));
            addr1.do_send(ClientCommand(cmd.prev_hash.clone()));
            // addr2.do_send(ClientCommand(cmd.clone()));
            // addr3.do_send(ClientCommand(cmd.clone()));
        });
    });

    sys.run().unwrap();
}

struct ChatClient(SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>);

#[derive(Message)]
#[rtype(result = "()")]
struct ClientCommand(String);

impl Actor for ChatClient {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        // start heartbeats otherwise server will disconnect after 10 seconds
        self.hb(ctx)
    }

    fn stopped(&mut self, _: &mut Context<Self>) {
        println!("Disconnected");

        // Stop application on disconnect
        System::current().stop();
    }
}

impl ChatClient {
    fn hb(&self, ctx: &mut Context<Self>) {
        ctx.run_later(Duration::new(1, 0), |act, ctx| {
            act.0.write(Message::Ping(Bytes::from_static(b"")));
            act.hb(ctx);

            // client should also check for a timeout here, similar to the
            // server code
        });
    }
}

/// Handle stdin commands
impl Handler<ClientCommand> for ChatClient {
    type Result = ();

    fn handle(&mut self, msg: ClientCommand, _ctx: &mut Context<Self>) {
        self.0.write(Message::Text(msg.0));
    }
}

/// Handle server websocket messages
impl StreamHandler<Result<Frame, WsProtocolError>> for ChatClient {
    fn handle(&mut self, msg: Result<Frame, WsProtocolError>, _: &mut Context<Self>) {
        if let Ok(Frame::Text(txt)) = msg {
            println!("Server: {:?}", txt)
        }
    }

    fn started(&mut self, _ctx: &mut Context<Self>) {
        println!("Connected");
    }

    fn finished(&mut self, ctx: &mut Context<Self>) {
        println!("Server disconnected");
        ctx.stop()
    }
}

impl actix::io::WriteHandler<WsProtocolError> for ChatClient {}
