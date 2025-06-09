use std::net::SocketAddr;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::spawn;

use statig::prelude::*;

struct Receiver {
    connection: TcpStream,
}

enum Event {
    Step,
}

#[state_machine(initial = "State::on()")]
impl Receiver {
    #[action]
    async fn enter_on(&mut self) {
        self.connection
            .write_all("Entering `On`\n".as_bytes())
            .await
            .unwrap();
    }

    #[state(entry_action = "enter_on")]
    async fn on(&mut self, event: &Event) -> Outcome<State> {
        match event {
            Event::Step => {
                self.connection
                    .write_all("Received `Step`\n".as_bytes())
                    .await
                    .unwrap();
                Transition(State::off())
            }
        }
    }

    #[state]
    async fn off(&mut self, event: &Event) -> Outcome<State> {
        match event {
            Event::Step => Transition(State::off()),
        }
    }
}

// Will receive two messages and return.
async fn receiver(addr: SocketAddr) {
    let listener = TcpListener::bind(addr).await.unwrap();

    let (receiver, _) = listener.accept().await.unwrap();

    let mut buf_reader = BufReader::new(receiver);

    let mut message = String::new();
    buf_reader.read_line(&mut message).await.unwrap();
    println!("{message}");

    let mut message = String::new();
    buf_reader.read_line(&mut message).await.unwrap();
    println!("{message}");
}

#[tokio::main]
async fn main() {
    let addr: SocketAddr = ([127, 0, 0, 1], 3000).into();

    let receiver_task = spawn(receiver(addr));

    let sender = TcpStream::connect(addr).await.unwrap();

    let mut state_machine = Receiver { connection: sender }.state_machine();

    state_machine.handle(&Event::Step).await;
    state_machine.handle(&Event::Step).await;

    receiver_task.await.unwrap();
}
