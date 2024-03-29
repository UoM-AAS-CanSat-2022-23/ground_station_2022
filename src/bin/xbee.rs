use anyhow::Result;
use chrono::{Timelike, Utc};
use ground_station::telemetry::Telemetry;
use ground_station::telemetry::*;
use ground_station::xbee::{TxRequest, XbeePacket};
use rand::{
    distributions::{Open01, Slice, Uniform},
    prelude::*,
};
use std::{
    io::{self, Write},
    thread,
    time::Duration,
};
use tracing::Level;

fn main() -> Result<()> {
    let usb_port = std::env::args()
        .nth(1)
        .expect("Need at least 2 arguments - first is port, second is the telemetry file");
    let mut sport = serialport::new(dbg!(usb_port), 230400).open()?;

    // real team number
    const TEAM_ID: u16 = 1047;

    // made up sea level constant
    const SEA_LEVEL: f64 = 1600.0;

    // failure rate of packet sending
    const ARTIFICIAL_FAILURE_RATE: f64 = 0.001;

    // define the distributions of various variables
    let modes = [Mode::Flight, Mode::Simulation];
    let alt_dist = Uniform::new(0.0, 750.0);
    let mode_dist = Slice::new(&modes)?;
    let temp_dist = Uniform::new(12.0, 70.0);
    let volt_dist = Uniform::new(4.8, 5.6);
    let press_dist = Uniform::new(80.0, 101.325);
    let lat_dist = Uniform::new(37.0, 37.4);
    let long_dist = Uniform::new(-90.0, 80.0);
    let sat_dist = Uniform::new(8, 35);
    let tilt_dist = Uniform::new(-45.0, 45.0);
    let delay_dist = Uniform::new(0.5, 1.5);

    // define the mutable state of the system
    let mut rng = thread_rng();
    let mut packet_count = 0;
    let mut frame_id = 0;

    // setup logging
    tracing_subscriber::fmt()
        .with_ansi(true)
        .with_max_level(Level::DEBUG)
        .with_writer(io::stderr)
        .init();

    loop {
        let now = Utc::now();
        let altitude = rng.sample(alt_dist);
        let telem = Telemetry {
            team_id: TEAM_ID,
            mission_time: MissionTime {
                h: now.hour() as u8,
                m: now.minute() as u8,
                s: now.second() as u8,
                cs: (now.timestamp_millis().rem_euclid(1000) / 10) as u8,
            },
            packet_count,
            mode: *rng.sample(mode_dist),
            state: State::Yeeted,
            altitude,
            hs_deployed: HsDeployed::Deployed,
            pc_deployed: PcDeployed::Deployed,
            mast_raised: MastRaised::Raised,
            temperature: rng.sample(temp_dist),
            voltage: rng.sample(volt_dist),
            pressure: rng.sample(press_dist),
            gps_time: GpsTime {
                h: now.hour() as u8,
                m: now.minute() as u8,
                s: now.second() as u8,
            },
            gps_altitude: SEA_LEVEL + altitude,
            gps_latitude: rng.sample(lat_dist),
            gps_longitude: rng.sample(long_dist),
            gps_sats: rng.sample(sat_dist),
            tilt_x: rng.sample(tilt_dist),
            tilt_y: rng.sample(tilt_dist),
            cmd_echo: "CXON".to_string(),
        };
        tracing::trace!("Generated telem = {telem}");

        // artificially fail some packets
        let fail_packet: f64 = rng.sample(Open01);
        if fail_packet < ARTIFICIAL_FAILURE_RATE {
            tracing::info!("Artificially failed a packet: {telem}");
        } else {
            let req = TxRequest::new(frame_id, 0xFFFF, format!("{telem}"));
            let packet: XbeePacket = req.try_into().unwrap();
            let ser = packet.serialise()?;
            sport.write_all(&ser)?;
        }

        frame_id = frame_id.wrapping_add(1);
        packet_count += 1;

        // wait to send the next packet
        thread::sleep(Duration::from_secs_f32(rng.sample(delay_dist)));
    }
}
