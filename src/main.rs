use std::{
    net::{SocketAddr, UdpSocket},
    path::PathBuf,
    rc::Rc,
    time::SystemTime,
};

use clap::{Parser, Subcommand};
use flute::{
    core::UDPEndpoint,
    receiver::{writer, MultiReceiver},
    sender::{Cenc, ObjectDesc, Sender},
};
use utils::ExtractIoErrorFromFlute;

/// Send and receive files over UDP
#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    cmd: Command,
}

/// Send file
#[derive(Subcommand, Debug)]
enum Command {
    Send(SendCmd),
    Recv(ReceiveCmd),
}

#[derive(Parser, Debug)]
struct SendCmd {
    addr: SocketAddr,
    files: Vec<PathBuf>,

    /// How much time to wait between sending UDP datagrams.
    ///
    /// Affects file transfer speed.
    #[clap(long, short = 'i', default_value = "2000")]
    delay_between_datagrams_us: u64,
}

#[derive(Parser, Debug)]
struct ReceiveCmd {
    bind_addr: SocketAddr,
    dir: PathBuf,

    /// Maximum cache size to keep file
    #[clap(long, default_value = "100_000_000")]
    maximum_file_size: usize,
}

mod utils;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Args::parse();

    match args.cmd {
        Command::Send(SendCmd {
            addr,
            files,
            delay_between_datagrams_us,
        }) => {
            let dest = addr;
            let endpoint = UDPEndpoint::new(None, format!("{addr}"), 3400);

            let args: Vec<String> = std::env::args().collect();
            if args.len() == 1 {
                println!("Send a list of files over UDP/FLUTE to {}", dest);
                println!("Usage: {} path/to/file1 path/to/file2 ...", args[0]);
                std::process::exit(0);
            }

            log::info!("Create UDP Socket");

            let udp_socket = UdpSocket::bind("0.0.0.0:0")?;

            log::info!("Create FLUTE Sender");
            let tsi = 1;
            let mut sender = Sender::new(endpoint, tsi, &Default::default(), &Default::default());

            log::info!("Connect to {}", dest);
            udp_socket.connect(dest)?;

            for file in &files {
                let path = std::path::Path::new(file);

                if !path.is_file() {
                    log::error!("{:?} is not a file", file);
                    std::process::exit(-1);
                }

                log::info!("Insert file {:?} to FLUTE sender", file);
                let obj = ObjectDesc::create_from_file(
                    path,
                    None,
                    "application/octet-stream",
                    true,
                    1,
                    None,
                    None,
                    None,
                    Cenc::Null,
                    true,
                    None,
                    true,
                )
                .ee()?;
                sender.add_object(0, obj).ee()?;
            }

            log::info!("Publish FDT update");
            sender.publish(SystemTime::now()).ee()?;

            while let Some(pkt) = sender.read(SystemTime::now()) {
                udp_socket.send(&pkt)?;
                std::thread::sleep(std::time::Duration::from_micros(delay_between_datagrams_us));
            }
        }
        Command::Recv(ReceiveCmd { bind_addr, dir, maximum_file_size }) => {
            let endpoint = UDPEndpoint::new(None, format!("{}", bind_addr.ip()), bind_addr.port());

            let dest_dir = dir;
            if !dest_dir.is_dir() {
                log::error!("{:?} is not a directory", dest_dir);
                std::process::exit(-1);
            }

            log::info!("Create FLUTE, write objects to {:?}", dest_dir);

            let writer = Rc::new(writer::ObjectWriterFSBuilder::new(&dest_dir).ee()?);

            let mut config = flute::receiver::Config::default();
            config.object_max_cache_size = Some(maximum_file_size);

            let mut receiver = MultiReceiver::new(writer, Some(config), false);

            // Receive from 224.0.0.1:3400 on 127.0.0.1 (lo) interface
            let socket = UdpSocket::bind(bind_addr)?;

            let mut buf = [0; 2048];
            loop {
                let (n, _src) = socket.recv_from(&mut buf)?;

                let now = std::time::SystemTime::now();
                match receiver.push(&endpoint, &buf[..n], now) {
                    Err(_) => log::error!("Wrong ALC/LCT packet"),
                    _ => {}
                };
                receiver.cleanup(now);
            }
        }
    }
    Ok(())
}
