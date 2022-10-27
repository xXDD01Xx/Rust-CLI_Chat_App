use std::io::{ErrorKind, Read, Write};          //io, Error_message, read, write
use std::net::{TcpListener, TcpStream};         //create server, listen on port
use std::sync::mpsc;                            //spawn a channel
use std::thread;                                // work with multiple threads

const LOCAL: &str = "127.0.0.1:6000";           //ip port
const MSG_SIZE: usize = 32;                     //buffer size of msgs

fn loop_sleep()                                 //thread sleeps for 100ms between loops
{
    thread::sleep(::std::time::Duration::from_millis(100));
}

fn main()
{
    let server = TcpListener::bind(LOCAL).expect("Listener failed to bind!");
    server.set_nonblocking(true).expect("Failed to initialize non-blocking mode!");                          //allows server to constantly check for msgs

    let mut clients = vec![];
    let (transmitter_x, receiver_x) = mpsc::channel::<String>();
    loop
    {
        if let Ok((mut socket, address)) = server.accept()
        {
            println!("Client {} connected...", address);

            let transmitter_x = transmitter_x.clone();
            clients.push(socket.try_clone().expect("Failed cloning client!"));                          //push to thread

            thread::spawn(move || loop
            {
                let mut buff = vec![0; MSG_SIZE];

                match socket.read_exact(&mut buff)
                {
                    Ok(_) =>
                        {
                            let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();     //convert msg to iterator and take all !whitespace, add to vec
                            let msg = String::from_utf8(msg).expect("Invalid UTF8 Message");           //convert to utf8

                            println!("{}: {:?}", address, msg);
                            transmitter_x.send(msg).expect("Failed to send msg to receiver_x");
                        }
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) =>
                        {
                            println!("Closing connection with: {}", address);
                            break;
                        }
                }
                loop_sleep();
            });
        }

        if let Ok(msg) = receiver_x.try_recv()              //server receives a message and we push it to channel
        {
            clients = clients.into_iter().filter_map(|mut client|
                {
                    let mut buff = msg.clone().into_bytes();
                    buff.resize(MSG_SIZE, 0);

                    client.write_all(&buff).map(|_| client).ok()
                }).collect::<Vec<_>>();
        }
        loop_sleep();
    }
}
