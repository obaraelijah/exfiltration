use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use etherparse::TransportSlice::Icmpv6;
use etherparse::{Icmpv6Type, SlicedPacket};
use log::{debug, error, info, warn};
use messages::Messages;
use pcap::{Capture, Error};

use crate::config::Config;

/// Start capturing
pub async fn start_capture(config: &Config) -> Result<(), String> {
    let mut capture = Capture::from_device(config.interface.interface.as_str())
        .map_err(|e| {
            format!(
                "Couldn't open device {} for capturing: {e}",
                config.interface.interface
            )
        })?
        .timeout(0)
        .open()
        .map_err(|e| {
            format!(
                "Couldn't activate device {} for capturing: {e}",
                config.interface.interface
            )
        })?;
    info!("Device {} open for capturing", config.interface.interface);

    let filter = "icmp6";
    capture
        .filter(filter, true)
        .map_err(|e| format!("Couldn't apply filter: {e}"))?;
    info!("Applied filter for capture: {filter}");

    let mut file_map: HashMap<u16, (String, u64, Vec<u8>)> = HashMap::new();

    loop {
        match capture.next_packet() {
            Ok(packet) => {
                let packet = SlicedPacket::from_ethernet(packet.data).unwrap();
                if let Some(Icmpv6(slice)) = packet.transport {
                    if let Icmpv6Type::EchoRequest(req) = slice.icmp_type() {
                        if req.seq == 1337 {
                            if let Ok(msg) = serde_json::from_slice(slice.payload()) {
                                let msg: Messages = msg;
                                match msg {
                                    Messages::Initial {
                                        file_name,
                                        file_size,
                                        identifier,
                                    } => {
                                        info!(
                                            "Received new file start: {file_name} with size {file_size} and ID {identifier}"
                                        );
                                        file_map
                                            .insert(identifier, (file_name, file_size, Vec::new()));
                                    }
                                    Messages::EndOfTransmission { identifier } => {
                                        if let Some((file_name, file_size, data)) =
                                            file_map.remove(&identifier)
                                        {
                                            info!(
                                                "Received end of transmission for ID {identifier}"
                                            );
                                            match lz4_flex::decompress(
                                                data.as_slice(),
                                                file_size as usize,
                                            ) {
                                                Ok(decompressed) => {
                                                    File::create(file_name)
                                                        .map_err(|e| {
                                                            format!("Error creating file: {e}")
                                                        })?
                                                        .write_all(&decompressed)
                                                        .map_err(|e| {
                                                            format!("Error writing to file: {e}")
                                                        })?;
                                                }
                                                Err(err) => {
                                                    warn!("Error decompressing data: {err}")
                                                }
                                            }
                                        }
                                    }
                                    _ => {
                                        debug!("Unexpected packet")
                                    }
                                }
                            } else {
                                debug!("Could not decode message in payload");
                            }
                        } else if file_map.contains_key(&req.seq) {
                            if let Ok(msg) = serde_json::from_slice(slice.payload()) {
                                let msg: Messages = msg;
                                match msg {
                                    Messages::Data { identifier, data } => {
                                        file_map
                                            .entry(identifier)
                                            .and_modify(|(_, _, d)| d.extend(data));
                                    }
                                    _ => {
                                        debug!("Unexpected packet");
                                    }
                                }
                            }
                        } else {
                            debug!(
                                "Received id: {}, but there's no associated entry in our db",
                                req.seq
                            );
                        }
                    } else {
                        debug!("No ICMP echo request");
                    }
                } else {
                    debug!("No icmpv6 received")
                }
            }
            Err(err) => {
                error!("Error while capturing packet: {err}");
                match err {
                    Error::MalformedError(_) => {}
                    Error::InvalidString => {}
                    Error::PcapError(_) => {}
                    Error::InvalidLinktype => {}
                    Error::TimeoutExpired => {}
                    Error::NoMorePackets => {}
                    Error::NonNonBlock => {}
                    Error::InsufficientMemory => {}
                    Error::InvalidInputString => {}
                    Error::IoError(_) => {}
                    Error::InvalidRawFd => {}
                    Error::ErrnoError(_) => {}
                    Error::BufferOverflow => {}
                }
            }
        }
    }

    Ok(())
}
