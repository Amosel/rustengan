use anyhow::{bail, Context};
use serde::{Deserialize, Serialize};
use std::io::{StdoutLock, Write};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Message {
    src: String,
    #[serde(rename = "dest")]
    dst: String,
    body: Body,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
struct Body {
    #[serde(rename = "msg_id")]
    id: usize,
    in_reply_to: Option<usize>,
    #[serde(flatten)]
    payload: Payload,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
}

struct EchoNode {
    id: usize,
}

impl EchoNode {
    pub fn step(&mut self, input: Message, output: &mut StdoutLock) -> anyhow::Result<()> {
        match input.body.payload {
            Payload::Init { .. } => {
                let reply = Message {
                    dst: input.src,
                    src: input.dst,
                    body: Body {
                        id: self.id,
                        in_reply_to: Some(input.body.id),
                        payload: Payload::InitOk,
                    },
                };
                serde_json::to_writer(&mut *output, &reply)
                    .context("serialize response to init")?;
                output.write_all(b"\n").context("write newline")?;
                self.id += 1;
            }
            Payload::InitOk { .. } => bail!("received init_ok message"),
            Payload::Echo { echo } => {
                let reply = Message {
                    dst: input.src,
                    src: input.dst,
                    body: Body {
                        id: self.id,
                        in_reply_to: Some(input.body.id),
                        payload: Payload::EchoOk { echo },
                    },
                };
                serde_json::to_writer(&mut *output, &reply)
                    .context("serialize response to init")?;
                output.write_all(b"\n").context("write newline")?;
                self.id += 1;
            }
            Payload::EchoOk { .. } => {}
        }
        Ok(())
    }
}
fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin().lock();
    let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message>();

    let mut stdout = std::io::stdout().lock();

    let mut node = EchoNode { id: 0 };
    for input in inputs {
        let input = input.context("Maelstrom input from STDIN could not be serializd")?;
        node.step(input, &mut stdout)
            .context("Node step function failed")?;
    }

    Ok(())
}
