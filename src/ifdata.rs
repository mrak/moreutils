use std::env;
use std::io;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::ops::BitAnd;
use std::ops::BitOr;
use std::process;
use std::thread;
use std::time;
use sysinfo::MacAddr;
use sysinfo::NetworkData;
use sysinfo::Networks;

#[derive(Debug)]
enum Op {
    ExitExistence,
    PrintConfiguration,
    PrintExistence,
    PrintAddress,
    PrintNetmask,
    PrintNetworkAddress,
    PrintBroadcast,
    PrintMtu,
    PrintMac,
    PrintInputStats,
    PrintInputPackets,
    PrintInputBytes,
    PrintInputErrors,
    PrintOutputStats,
    PrintOutputPackets,
    PrintOutputBytes,
    PrintOutputErrors,
    PrintInputBytesOverSecond,
    PrintOutputBytesOverSecond,
}

#[derive(Default, Debug)]
struct Args {
    ops: Vec<Op>,
    interface: String,
    refresh_one_second: bool,
}

fn usage() {
    println!("Usage: ifdata [OPTION]... INTERFACE");
    println!("Get network interface info without parsing ifconfig output");
    println!();
    println!("  -h    Display this help text and exit");
    println!("  -e    Reports interface existence via return code");
    println!("  -p    Print out the whole config of interface");
    println!("  -pe   Print out yes or no according to existence");
    println!("  -pa   Print out the address");
    println!("  -pn   Print out the netmask");
    println!("  -pN   Print out the network address");
    println!("  -pb   Print out the broadcast address");
    println!("  -pm   Print out the MTU");
    println!("  -ph   Print out the hardware address");

    println!("  -sip  Print out the number of input packets");
    println!("  -sib  Print out the number of input bytes");
    println!("  -sie  Print out the number of input errors");
    println!("  -sop  Print out the number of input packets");
    println!("  -sob  Print out the number of input bytes");
    println!("  -soe  Print out the number of input errors");
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args::default();
    let mut iface = None;
    for arg in env::args().skip(1) {
        match arg.as_str() {
            "-h" => {
                usage();
                process::exit(0);
            }
            "-e" => args.ops.push(Op::ExitExistence),
            "-p" => args.ops.push(Op::PrintConfiguration),
            "-pe" => args.ops.push(Op::PrintExistence),
            "-pa" => args.ops.push(Op::PrintAddress),
            "-pn" => args.ops.push(Op::PrintNetmask),
            "-pN" => args.ops.push(Op::PrintNetworkAddress),
            "-pb" => args.ops.push(Op::PrintBroadcast),
            "-pm" => args.ops.push(Op::PrintMtu),
            "-ph" => args.ops.push(Op::PrintMac),
            "-si" => args.ops.push(Op::PrintInputStats),
            "-sip" => args.ops.push(Op::PrintInputPackets),
            "-sib" => args.ops.push(Op::PrintInputBytes),
            "-sie" => args.ops.push(Op::PrintInputErrors),
            "-so" => args.ops.push(Op::PrintOutputStats),
            "-sop" => args.ops.push(Op::PrintOutputPackets),
            "-sob" => args.ops.push(Op::PrintOutputBytes),
            "-soe" => args.ops.push(Op::PrintOutputErrors),
            "-bips" => {
                args.ops.push(Op::PrintInputBytesOverSecond);
                args.refresh_one_second = true;
            }
            "-bops" => {
                args.ops.push(Op::PrintOutputBytesOverSecond);
                args.refresh_one_second = true;
            }
            _ => iface = Some(arg),
        }
    }

    if let Some(x) = iface {
        args.interface = x;
        Ok(args)
    } else {
        Err(String::from("INTERFACE is required"))
    }
}

fn fail(name: &str) {
    eprintln!("No such network interface: {}", name);
    process::exit(1);
}

pub fn ifdata() -> io::Result<()> {
    let args = parse_args().unwrap_or_else(|e| {
        eprintln!("{e}");
        usage();
        process::exit(1);
    });

    let mut networks = Networks::new_with_refreshed_list();
    if args.refresh_one_second {
        thread::sleep(time::Duration::from_secs(1));
        networks.refresh(true);
    }
    let maybe_interface = networks.get(&args.interface);

    for op in args.ops.iter() {
        match op {
            Op::ExitExistence => match maybe_interface {
                None => process::exit(1),
                _ => process::exit(0),
            },
            Op::PrintExistence => {
                match maybe_interface {
                    None => println!("no"),
                    _ => println!("yes"),
                }
                process::exit(0);
            }
            Op::PrintAddress => match maybe_interface {
                None => fail(&args.interface),
                Some(x) => {
                    if let Some(a) = get_address(x) {
                        println!("{a}");
                    }
                }
            },
            Op::PrintNetmask => match maybe_interface {
                None => fail(&args.interface),
                Some(x) => {
                    if let Some(n) = get_netmask(x) {
                        println!("{n}");
                    }
                }
            },
            Op::PrintNetworkAddress => match maybe_interface {
                None => fail(&args.interface),
                Some(x) => {
                    if let Some(n) = get_network_address(x) {
                        println!("{n}");
                    }
                }
            },
            Op::PrintBroadcast => match maybe_interface {
                None => fail(&args.interface),
                Some(x) => {
                    if let Some(b) = get_broadcast(x) {
                        println!("{b}");
                    }
                }
            },
            Op::PrintMtu => match maybe_interface {
                None => fail(&args.interface),
                Some(x) => println!("{}", x.mtu()),
            },
            Op::PrintMac => match maybe_interface {
                None => fail(&args.interface),
                Some(x) => println!("{}", get_mac(&args.interface, x)),
            },
            Op::PrintConfiguration => match maybe_interface {
                None => fail(&args.interface),
                Some(x) => {
                    let mut infos: Vec<String> = vec![];
                    if let Some(a) = get_address(x) {
                        infos.push(a.to_string());
                    }
                    if let Some(a) = get_netmask(x) {
                        infos.push(a.to_string());
                    }
                    if let Some(a) = get_broadcast(x) {
                        infos.push(a.to_string());
                    }
                    infos.push(x.mtu().to_string());
                    println!("{}", infos.join(" "));
                }
            },
            Op::PrintInputPackets => match maybe_interface {
                None => fail(&args.interface),
                Some(x) => println!("{}", x.total_packets_received()),
            },
            Op::PrintInputBytes => match maybe_interface {
                None => fail(&args.interface),
                Some(x) => println!("{}", x.total_received()),
            },
            Op::PrintInputErrors => match maybe_interface {
                None => fail(&args.interface),
                Some(x) => println!("{}", x.total_errors_on_received()),
            },
            Op::PrintInputStats => match maybe_interface {
                None => fail(&args.interface),
                Some(x) => println!(
                    "{} {} {}",
                    x.total_received(),
                    x.total_packets_received(),
                    x.total_errors_on_received(),
                ),
            },
            Op::PrintOutputPackets => match maybe_interface {
                None => fail(&args.interface),
                Some(x) => println!("{}", x.total_packets_transmitted()),
            },
            Op::PrintOutputBytes => match maybe_interface {
                None => fail(&args.interface),
                Some(x) => println!("{}", x.total_transmitted()),
            },
            Op::PrintOutputErrors => match maybe_interface {
                None => fail(&args.interface),
                Some(x) => println!("{}", x.total_errors_on_transmitted()),
            },
            Op::PrintOutputStats => match maybe_interface {
                None => fail(&args.interface),
                Some(x) => println!(
                    "{} {} {}",
                    x.total_transmitted(),
                    x.total_packets_transmitted(),
                    x.total_errors_on_transmitted(),
                ),
            },
            Op::PrintInputBytesOverSecond => match maybe_interface {
                None => fail(&args.interface),
                Some(x) => {
                    println!("{}", x.received(),)
                }
            },
            Op::PrintOutputBytesOverSecond => match maybe_interface {
                None => fail(&args.interface),
                Some(x) => {
                    println!("{}", x.transmitted(),)
                }
            },
        }
    }

    Ok(())
}

fn get_address(network_data: &NetworkData) -> Option<IpAddr> {
    for ip_network in network_data.ip_networks() {
        if ip_network.addr.is_ipv4() {
            return Some(ip_network.addr);
        }
    }
    None
}

fn ipv4_prefix_to_mask(prefix: u8) -> u32 {
    u32::MAX.checked_shl((32 - prefix) as u32).unwrap_or(0)
}

fn ipv4_netmask_addr(prefix: u8) -> Ipv4Addr {
    Ipv4Addr::from(ipv4_prefix_to_mask(prefix))
}

fn get_netmask(network_data: &NetworkData) -> Option<Ipv4Addr> {
    for ip_network in network_data.ip_networks() {
        if ip_network.addr.is_ipv4() {
            return Some(ipv4_netmask_addr(ip_network.prefix));
        }
    }
    None
}

fn get_network_address(network_data: &NetworkData) -> Option<Ipv4Addr> {
    for ip_network in network_data.ip_networks() {
        match ip_network.addr {
            std::net::IpAddr::V4(addr) => {
                return Some(addr.bitand(ipv4_netmask_addr(ip_network.prefix)));
            }
            _ => continue,
        }
    }
    None
}

fn get_broadcast(network_data: &NetworkData) -> Option<Ipv4Addr> {
    for ip_network in network_data.ip_networks() {
        match ip_network.addr {
            std::net::IpAddr::V4(addr) => {
                let broadcast_mask = u32::MAX ^ ipv4_prefix_to_mask(ip_network.prefix);
                return Some(addr.bitor(Ipv4Addr::from(broadcast_mask)));
            }
            _ => continue,
        }
    }
    None
}

fn get_mac(name: &str, network_data: &NetworkData) -> MacAddr {
    match network_data.mac_address() {
        MacAddr::UNSPECIFIED => {
            eprintln!("interface \"{name}\" does not have a hardware address");
            process::exit(1);
        }
        x => x,
    }
}
