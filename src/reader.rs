use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use crate::app::ReceivedPacket;
use crate::xbee::{RxPacket, XbeePacket};
use anyhow::Result;

/// Telem
pub struct TelemetryReader {
    tx: Sender<ReceivedPacket>,
}

impl TelemetryReader {
    pub fn new(tx: Sender<ReceivedPacket>) -> Self {
        Self { tx }
    }

    pub fn run(&mut self) -> Result<()> {
        // start the reader thread
        let file = File::open("test_data/test_data.txt")?;
        let buf_reader = BufReader::new(file);

        // collect all the lines so we can cycle them
        let lines: Vec<_> = buf_reader.lines().collect();

        for line in lines.iter().cycle() {
            let line = match line {
                Err(e) => {
                    tracing::warn!("Encountered error while reading line: {e:?}");
                    continue;
                }
                Ok(line) => line,
            };
            tracing::trace!("line = {:?}", line);

            match line.parse() {
                Ok(telem) => {
                    let packet = ReceivedPacket::Telemetry {
                        packet: XbeePacket {
                            frame_type: 0x81,
                            data: vec![],
                            checksum: 0,
                        },
                        frame: RxPacket {
                            src_addr: 0xFFFF,
                            rssi: 0,
                            options: 0,
                            data: vec![],
                        },
                        telem,
                    };
                    if let Err(e) = self.tx.send(packet) {
                        tracing::warn!(
                            "Encountered error sending telemtry over the channel: {e:?}"
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to parse received telemetry: {e:?}");
                }
            }

            thread::sleep(Duration::from_secs(1));
            // thread::sleep(Duration::from_millis(50));
        }

        Ok(())
    }
}
