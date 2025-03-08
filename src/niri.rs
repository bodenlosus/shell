use std::{fmt::format, io::{self, Error}, rc::Rc};

use niri_ipc::{socket::Socket, Event, Reply, Response, Workspace};
pub use niri_ipc::Request;

pub struct Niri {
    socket: Socket
}

impl Niri {
    pub fn connect() -> Result<Self, String>{
        let soc = match Socket::connect() {
            Ok(soc) => soc,
            Err(err) => return Err(format!("{err}")),
        };
        Ok(Self { socket: soc })
    }
    pub fn stream(self, callback: Box<dyn Fn(Event) -> ()>) -> Result<(), String>{
        let (reply, mut block) = self.socket.send(Request::EventStream)
            .map_err(|err| format!("Error communicating with NIRI: {}", err))?;
        let res = reply.map_err(|err| format!("Error from NIRI: {}", err))?;
        match res {
            Response::Handled => (),
            _ => return Err(format!("unhandled stream request")),
        };

        loop {
            let event = match block() {
                Err(e) => {eprintln!("{e}");break;},
                Ok(ev) => ev,
            };
            callback(event);
        }

        Ok(())

    }
    pub fn request(self, request: Request) -> Result<Response, String>// Ensure Response can be converted into T// Ensure conversion error can be formatted
    {
        let reply = self.socket.send(request)
            .map_err(|err| format!("Error communicating with NIRI: {}", err))?;

        let res = reply.0.map_err(|err| format!("Error from NIRI: {}", err))?;

        Ok(res)
    }

}

struct IPCResponse {
    response: Response,
    blocker: dyn FnMut() -> io::Result<Event>
}