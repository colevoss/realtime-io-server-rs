use crossbeam::channel::{Receiver, RecvError, SendError, Sender};

pub trait Client<Request, Response> {
    // Receiver channel tx to send requests/messages through
    fn server_tx(&self) -> Sender<(Request, Sender<Response>)>;

    // Response sender to send to server so it can reply
    fn response_channel(&self) -> Sender<Response>;

    // Channel rx to receive server replies from
    fn response_receiver(&self) -> &Receiver<Response>;

    fn handle_response(&self, response: Response) -> Result<(), ()>;
    fn handle_response_mut(&mut self, response: Response) -> Result<(), ()>;

    fn send(&self, message: Request) -> Result<(), SendError<(Request, Sender<Response>)>> {
        self.server_tx().send((message, self.response_channel()))
    }

    fn poll(&self) -> Result<(), ()> {
        let rx = self.response_receiver();

        if rx.is_empty() {
            return Ok(());
        }

        match rx.try_recv() {
            Ok(response) => self.handle_response(response),
            Err(crossbeam::channel::TryRecvError::Empty) => Ok(()),
            Err(crossbeam::channel::TryRecvError::Disconnected) => Err(()),
        }
    }

    fn poll_mut(&mut self) -> Result<(), ()> {
        let rx = self.response_receiver();

        if rx.is_empty() {
            return Ok(());
        }

        match rx.try_recv() {
            Ok(response) => self.handle_response_mut(response),
            Err(crossbeam::channel::TryRecvError::Empty) => Ok(()),
            Err(crossbeam::channel::TryRecvError::Disconnected) => Err(()),
        }
    }

    fn recv(&self) -> Result<Response, RecvError> {
        self.response_receiver().recv()
    }
}
