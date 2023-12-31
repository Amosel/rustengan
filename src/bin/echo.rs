use rustengan::*;

use anyhow::{bail, Context};
use serde::{Deserialize, Serialize};
use std::io::{StdoutLock, Write};

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

impl Node<Payload> for EchoNode {
    fn step(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
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
    main_loop(EchoNode { id: 0 })
}
