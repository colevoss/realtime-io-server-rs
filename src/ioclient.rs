use std::{
    fs::File,
    io::ErrorKind,
    sync::{Arc, Mutex},
};

use crossbeam::channel::{bounded, Receiver, SendError, Sender};

use crate::file_handle::FileHandle;

#[derive(Debug, Clone)]
pub enum IoClientState {
    Idle,
    Opening,
    OpenedIdle,
    Buffering,
    Streaming,
    Error(ErrorKind),
}

pub enum IoClientRequest {
    OpenFile {
        handle: String,
    },
    ReadBlock {
        file_handle: Arc<Mutex<FileHandle>>,
        // position: usize,
    },
}

pub enum IoClientRepsonse {
    BufferOpening,
    BufferOpened(FileHandle),
    ReadingAhead,
    Error,
}

pub struct IoClient {
    file_path: String,
    // TODO: Change this to a parkinglot mutext
    file_handle: Option<Arc<Mutex<FileHandle>>>,
    state: IoClientState,
    server_tx: Sender<(IoClientRequest, Sender<IoClientRepsonse>)>,
    response_tx: Sender<IoClientRepsonse>,
    response_rx: Receiver<IoClientRepsonse>,
}

impl IoClient {
    pub fn new(
        file_path: String,
        server_tx: Sender<(IoClientRequest, Sender<IoClientRepsonse>)>,
    ) -> Self {
        // let (response_tx, response_rx) = unbounded::<IoClientRepsonse>();
        let (response_tx, response_rx) = bounded::<IoClientRepsonse>(1);

        IoClient {
            file_handle: None,
            state: IoClientState::Idle,
            file_path,
            server_tx,
            response_rx,
            response_tx,
        }
    }

    pub fn state(&self) -> &IoClientState {
        &self.state
    }

    pub fn poll(&mut self) -> Result<(), ()> {
        if self.response_rx.is_empty() {
            return Ok(());
        }

        // match self.response_rx.try_recv() {
        match self.response_rx.try_recv() {
            Ok(response) => {
                self.handle_server_response(response);
                Ok(())
            }
            Err(crossbeam::channel::TryRecvError::Empty) => Ok(()),
            Err(crossbeam::channel::TryRecvError::Disconnected) => {
                eprintln!("Disconnected");
                Err(())
            }
        }
    }

    pub fn handle_server_response(&mut self, response: IoClientRepsonse) {
        match response {
            IoClientRepsonse::BufferOpening => self.set_state(IoClientState::Opening),
            IoClientRepsonse::BufferOpened(file_handle) => {
                println!("Got file handle");
                self.file_handle = Some(Arc::new(Mutex::new(file_handle)));
                self.set_state(IoClientState::OpenedIdle);
                self.read_block();
                self.read_block();
            }
            IoClientRepsonse::ReadingAhead => self.set_state(IoClientState::Buffering),
            IoClientRepsonse::Error => {}
        }
    }

    pub fn read_block(&self) {
        if let Some(file_handle) = &self.file_handle {
            println!("Sending File Read");
            self.send(IoClientRequest::ReadBlock {
                file_handle: file_handle.clone(),
                // position: 1,
            })
            .unwrap()
        }
    }

    pub fn set_state(&mut self, state: IoClientState) {
        self.state = state;
    }

    pub fn open(&self) {
        self.send(IoClientRequest::OpenFile {
            handle: self.file_path.clone(),
        })
        // TODO: Handle errors
        .expect("Client send should work");
    }

    #[inline]
    pub fn send(
        &self,
        request: IoClientRequest,
    ) -> Result<(), SendError<(IoClientRequest, Sender<IoClientRepsonse>)>> {
        self.server_tx.send((request, self.response_tx.clone()))
    }
}
