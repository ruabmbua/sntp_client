extern crate byteorder;
extern crate time;
#[macro_use]
extern crate clap;

use std::net::UdpSocket;
use std::process;
use byteorder::{BigEndian, ByteOrder};

fn main() {
    // Setting up command line argument parser
    let matches = clap_app!(sntp =>
        (version: "1.1.0")
        (author: "Roland Ruckerbauer <roland.rucky@gmail.com>")
        (about: "Fetches time from the given time server and outputs it formatted")
        (@arg HOST: +required "Sets the host of the sntp server")
        (@arg PORT: "Sets the port of the sntp server")
        (@arg format: -f --format +takes_value "Sets a custom format for printing the time")
        (@arg pure: -p --pure "Only output the time")
    ).get_matches();

    // Extract port value
    let port_result = matches.value_of("PORT").unwrap_or("123").parse::<u16>();
    if let Err(error) = port_result {
        println!("Error in port argument: {}", error);
        process::exit(-1);
    }
    let port = port_result.unwrap();

    // Setting up the buffer for the request
    let mut buf = [0u8; 48];
    buf[0]  = 0xe3;
    buf[2]  = 0x06;
    buf[3]  = 0xec;
    buf[12] = 0x5e;
    buf[13] = 0x4e;
    buf[14] = 0x31;
    buf[15] = 0x34;

    // Create a socket and bind it to any interface
    let socket_result = UdpSocket::bind("0.0.0.0:0");
    if let Err(error) = socket_result {
        println!("Error: {}", error);
        process::exit(-1);
    }
    let socket = socket_result.unwrap();

    // Send the request to the server
    match socket.send_to(&buf, (matches.value_of("HOST").unwrap(), port)) {
        Err(error) => {
            println!("Error in address: {}", error);
            process::exit(-1);
        },
        Ok(sent) => {
            if sent == 0 {
                println!("Error: Can not send request");
                process::exit(-1);
            }
        }
    }

    // Receive the response from the sntp sever
    match socket.recv_from(&mut buf) {
        Err(error) => {
            println!("Error: {}", error);
            process::exit(-1);
        },
        Ok((rec, _)) => {
            if rec < 44 {
                println!("Error: To few bytes received");
                process::exit(-1);
            }
        }
    }

    // Transmute the byte array into an unsigned integer and apply byte order
    let raw_time = BigEndian::read_u32(&buf[40..44]);

    // Substract static amount to convert into UNIX timestamp
    let unix_timestamp = raw_time - 2208988800;

    // Convert into time crate format, and parse into single components
    let timespec = time::Timespec::new(unix_timestamp as i64, 0);
    let time = time::at(timespec);

    // Print out time components with formatters
    let hide_extra = matches.is_present("pure");
    match matches.value_of("format") {
        None => {
            if !hide_extra { print!("Time (asctime): "); }
            println!("{}", time.asctime());
        },
        Some(format) => {
            match time.strftime(format) {
                Err(error) => println!("Error in time format: {}", error),
                Ok(formatter) => {
                    if !hide_extra { print!("Time (custom): "); }
                    println!("{}", formatter);
                }
            }
        }
    }
}