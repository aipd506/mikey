#![cfg(windows)]

use super::client::handle_bt_client;
use crate::audio::pipeline::JitterBuffer;
use crate::session::SessionManager;
use std::io;
use std::os::windows::io::FromRawSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const AF_BTH: i32 = 32;
const BTHPROTO_RFCOMM: i32 = 3;
const SOCK_STREAM: i32 = 1;
const INVALID_SOCKET: usize = !0;

#[link(name = "ws2_32")]
extern "system" {
    fn socket(af: i32, socket_type: i32, protocol: i32) -> usize;
    fn closesocket(s: usize) -> i32;
    fn bind(s: usize, name: *const u8, namelen: i32) -> i32;
    fn listen(s: usize, backlog: i32) -> i32;
    fn accept(s: usize, addr: *mut u8, addrlen: *mut i32) -> usize;
    fn WSAGetLastError() -> i32;
}

#[repr(C)]
struct SockAddrBth {
    address_family: u16,
    bt_addr: u64,
    service_class_id: [u8; 16],
    port: u32,
}

pub fn run_windows_rfcomm_listener(
    session_manager: SessionManager,
    jitter_buffer: Arc<JitterBuffer>,
    running: Arc<AtomicBool>,
) -> io::Result<()> {
    let sock = unsafe { socket(AF_BTH, SOCK_STREAM, BTHPROTO_RFCOMM) };

    if sock == INVALID_SOCKET {
        let err = unsafe { WSAGetLastError() };
        println!(
            "[bt] Bluetooth RFCOMM not supported or adapter absent (WSA error {})",
            err
        );
        return Ok(());
    }

    let addr = SockAddrBth {
        address_family: AF_BTH as u16,
        bt_addr: 0,
        service_class_id: [0u8; 16],
        port: 0,
    };

    let bind_res = unsafe {
        bind(
            sock,
            &addr as *const _ as *const u8,
            std::mem::size_of::<SockAddrBth>() as i32,
        )
    };

    if bind_res != 0 {
        let err = unsafe { WSAGetLastError() };
        unsafe { closesocket(sock) };
        if err == 10050 {
            println!("[bt] Bluetooth radio is turned off or disabled (WSA error 10050)");
        } else {
            println!("[bt] Bluetooth RFCOMM unavailable (error {})", err);
        }
        return Ok(());
    }

    let listen_res = unsafe { listen(sock, 1) };
    if listen_res != 0 {
        let err = unsafe { WSAGetLastError() };
        unsafe { closesocket(sock) };
        println!(
            "[bt] Failed to listen on Bluetooth RFCOMM socket (error {})",
            err
        );
        return Ok(());
    }

    println!("[bt] RFCOMM server listening for Level 4 connections");

    while running.load(Ordering::Relaxed) {
        let mut client_addr = SockAddrBth {
            address_family: 0,
            bt_addr: 0,
            service_class_id: [0u8; 16],
            port: 0,
        };
        let mut addr_len = std::mem::size_of::<SockAddrBth>() as i32;

        let client_sock =
            unsafe { accept(sock, &mut client_addr as *mut _ as *mut u8, &mut addr_len) };

        if client_sock == INVALID_SOCKET {
            thread::sleep(Duration::from_millis(100));
            continue;
        }

        let stream = unsafe { std::net::TcpStream::from_raw_socket(client_sock as _) };
        let sm = session_manager.clone();
        let jb = Arc::clone(&jitter_buffer);

        thread::spawn(move || {
            handle_bt_client(stream, sm, jb);
        });
    }

    unsafe { closesocket(sock) };
    Ok(())
}
