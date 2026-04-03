// use std::{cmp::Ordering, net::{IpAddr, Ipv4Addr}};

// use if_addrs::{IfAddr, Ifv4Addr};

// #[derive(Debug)]
// struct NetworkInterface {
//     name: String,
//     ip: Ipv4Addr,
//     is_up: bool,
//     is_loopback: bool,
// }

// pub fn get_local_ip() -> Option<Ipv4Addr> {
//     for iface in if_addrs::get_if_addrs().unwrap() 
//     {
//         println!("{:#?}", iface);
//         match &iface.addr
//         {
//             IfAddr::V4(ip) =>
//             {
//                 if !is_private_ip(&ip.ip) 
//                 {
//                     continue;
//                 }
//                 if !ip.is_loopback() && iface.is_oper_up()
//                 {
//                     return Some(ip.ip)
//                 }
//             }
//             _ => ()
//         }
//     }
//     None
// }

// fn is_private_ip(ip: &Ipv4Addr) -> bool {
//     match ip.octets() 
//     {
//         [10, ..] => true,                         // 10.0.0.0/8
//         [172, b, ..] if (16..=31).contains(&b) => true, // 172.16.0.0/12
//         [192, 168, ..] => true,                   // 192.168.0.0/16
//         [169, 254, ..] => true,                   // APIPA/Link-local
//         [182, 5, ..] => true,          
//         _ => false
//     }
// }
