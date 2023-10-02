use anyhow::Context;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::io::StdoutLock;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message<Payload> {
    pub src: String,
    #[serde(rename = "dest")]
    pub dst: String,
    pub body: Body<Payload>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Body<Payload> {
    #[serde(rename = "msg_id")]
    pub id: usize,
    pub in_reply_to: Option<usize>,
    #[serde(flatten)]
    pub payload: Payload,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Init {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

// #[derive(Serialize, Deserialize, Debug, Clone)]
// #[serde(tag = "type")]
// #[serde(rename_all = "snake_case")]
// enum Payload {
//     Init {
//         node_id: String,
//         node_ids: Vec<String>,
//     },
//     InitOk,
//     Echo {
//         echo: String,
//     },
//     EchoOk {
//         echo: String,
//     },
// }

// struct EchoNode {
//     id: usize,
// }

// impl EchoNode {
//     pub fn step(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
//         match input.body.payload {
//             Payload::Init { .. } => {
//                 let reply = Message {
//                     dst: input.src,
//                     src: input.dst,
//                     body: Body {
//                         id: self.id,
//                         in_reply_to: Some(input.body.id),
//                         payload: Payload::InitOk,
//                     },
//                 };
//                 serde_json::to_writer(&mut *output, &reply)
//                     .context("serialize response to init")?;
//                 output.write_all(b"\n").context("write newline")?;
//                 self.id += 1;
//             }
//             Payload::InitOk { .. } => bail!("received init_ok message"),
//             Payload::Echo { echo } => {
//                 let reply = Message {
//                     dst: input.src,
//                     src: input.dst,
//                     body: Body {
//                         id: self.id,
//                         in_reply_to: Some(input.body.id),
//                         payload: Payload::EchoOk { echo },
//                     },
//                 };
//                 serde_json::to_writer(&mut *output, &reply)
//                     .context("serialize response to init")?;
//                 output.write_all(b"\n").context("write newline")?;
//                 self.id += 1;
//             }
//             Payload::EchoOk { .. } => {}
//         }
//         Ok(())
//     }
// }

pub trait Node<Payload> {
    fn step(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()>;
}

pub fn main_loop<S, Payload>(mut state: S) -> anyhow::Result<()>
where
    S: Node<Payload>,
    Payload: DeserializeOwned,
{
    let stdin = std::io::stdin().lock();
    let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message<Payload>>();

    let mut stdout = std::io::stdout().lock();

    for input in inputs {
        let input = input.context("Maelstrom input from STDIN could not be serializd")?;
        state
            .step(input, &mut stdout)
            .context("Node step function failed")?;
    }

    Ok(())
}
