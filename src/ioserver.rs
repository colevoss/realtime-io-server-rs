use std::{
    borrow::Cow,
    fs::File,
    result,
    sync::{Arc, Mutex},
    thread,
};

use crate::{
    file_handle::FileHandle,
    ioclient::{IoClient, IoClientRepsonse, IoClientRequest},
};
use crossbeam::channel::{unbounded, Receiver, Sender};
use rubato::{FftFixedInOut, Resampler};
use symphonia::core::{
    audio::{AudioBuffer, AudioBufferRef, Signal},
    conv::IntoSample,
    sample::Sample,
};

pub struct IoServer {
    request_tx: Sender<(IoClientRequest, Sender<IoClientRepsonse>)>,
    request_rx: Receiver<(IoClientRequest, Sender<IoClientRepsonse>)>,
}

impl IoServer {
    fn start(&self) {
        while let Ok((request, tx)) = self.request_rx.recv() {
            match request {
                IoClientRequest::OpenFile { handle } => self.open_file_handle(handle, tx),
                IoClientRequest::ReadBlock {
                    file_handle,
                    // position: _,
                } => {
                    self.prefetch_data(file_handle, tx);
                }
            }
        }
    }

    pub fn open_file_handle(&self, file_path: String, tx: Sender<IoClientRepsonse>) {
        match FileHandle::new(file_path) {
            Ok(file_handle) => {
                // let file_handle = Arc::new(file_handle);
                tx.send(IoClientRepsonse::BufferOpened(file_handle))
                    .unwrap();
            }
            Err(_) => {
                eprintln!("Error opening file handle")
            }
        }
    }

    pub fn prefetch_data(&self, handle: Arc<Mutex<FileHandle>>, tx: Sender<IoClientRepsonse>) {
        tx.send(IoClientRepsonse::ReadingAhead);
        let mut file_handle = handle.lock().unwrap();
        let sample_rate = file_handle.sample_rate;
        let channel_count = file_handle.channel_count;

        println!("HELLO!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");

        let decoded = loop {
            let packet = file_handle.format_reader.next_packet().unwrap();
            match file_handle.decoder.decode(&packet) {
                Ok(decoded) => break decoded,
                Err(_) => {
                    tx.send(IoClientRepsonse::Error);
                }
            }
        };

        let frame_count = decoded.frames();

        let resampler =
            FftFixedInOut::<f32>::new(sample_rate as usize, 44100, frame_count, channel_count)
                .unwrap();
        // resampler.input_buffer_allocate()
        // let frames = self.input_frames_max();
        // let channels = self.nbr_channels();
        // let mut buffer = Vec::with_capacity(channels);
        // for _ in 0..channels {
        //     buffer.push(Vec::with_capacity(frames));
        // }

        let mut input: Vec<Vec<f32>> = Vec::with_capacity(channel_count);

        for i in 0..channel_count {
            input.push(Vec::<f32>::with_capacity(frame_count));
        }

        match decoded {
            AudioBufferRef::U8(buf) => self.test(channel_count, &mut input, &buf),
            AudioBufferRef::U16(buf) => self.test(channel_count, &mut input, &buf),
            AudioBufferRef::U24(buf) => self.test(channel_count, &mut input, &buf),
            AudioBufferRef::U32(buf) => self.test(channel_count, &mut input, &buf),
            AudioBufferRef::S8(buf) => self.test(channel_count, &mut input, &buf),
            AudioBufferRef::S16(buf) => self.test(channel_count, &mut input, &buf),
            AudioBufferRef::S24(buf) => self.test(channel_count, &mut input, &buf),
            AudioBufferRef::S32(buf) => self.test(channel_count, &mut input, &buf),
            AudioBufferRef::F32(buf) => self.test(channel_count, &mut input, &buf),
            AudioBufferRef::F64(buf) => self.test(channel_count, &mut input, &buf),
        }

        println!("{:?}", input);
    }

    pub fn test<T, S: Sample + IntoSample<T>>(
        &self,
        channel_count: usize,
        input: &mut Vec<Vec<T>>,
        buf: &AudioBuffer<S>,
    ) {
        for chan_idx in 0..channel_count {
            // let sample_idx = 0;
            for &sample in buf.chan(chan_idx) {
                input[chan_idx].push(sample.into_sample());
            }
        }
    }
}

pub struct IoServerController {
    ioserver: Arc<IoServer>,
}

impl IoServerController {
    pub fn new() -> Self {
        let (request_tx, request_rx) = unbounded::<(IoClientRequest, Sender<IoClientRepsonse>)>();

        let server = IoServer {
            request_tx,
            request_rx,
        };

        let controller = IoServerController {
            ioserver: Arc::new(server),
        };

        controller
    }

    pub fn start(&self) -> thread::JoinHandle<()> {
        let server = self.ioserver.clone();

        thread::spawn(move || {
            server.start();
        })
    }

    pub fn open_stream(&self, handle: String) -> IoClient {
        let server_tx = self.ioserver.request_tx.clone();

        let client = IoClient::new(handle, server_tx);

        client
    }
}
