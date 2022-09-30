use std::fs::File;

use symphonia::core::{
    audio::{AudioBufferRef, Signal, SignalSpec},
    codecs::{Decoder, DecoderOptions},
    conv::IntoSample,
    formats::{FormatOptions, FormatReader},
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
};

use rubato::FftFixedInOut;

pub struct FileHandle {
    pub sample_rate: u32,
    pub channel_count: usize,
    pub format_reader: Box<dyn FormatReader>,
    pub decoder: Box<dyn Decoder>,
    // pub spec: SignalSpec,
    // pub frames_read: usize;
    // Can we make this generic at the `FileHandle` level?
    // pub resampler: FftFixedInOut<f32>,
}

impl FileHandle {
    pub fn new(file_path: String) -> Result<Self, ()> {
        let file = Box::new(File::open(file_path).unwrap());
        let media_source_stream = MediaSourceStream::new(file, Default::default());
        let hint = Hint::new();

        let format_opts: FormatOptions = Default::default();
        let metadata_opts: MetadataOptions = Default::default();
        let decoder_opts: DecoderOptions = Default::default();

        let probed = symphonia::default::get_probe()
            .format(&hint, media_source_stream, &format_opts, &metadata_opts)
            .expect("Cannot get media probe");

        let mut format_reader = probed.format;
        let track = match format_reader.default_track() {
            Some(track) => track,
            None => return Err(()),
        };

        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &decoder_opts)
            .expect("Could not get decoder");

        let codec_params = decoder.codec_params();
        let channel_count = codec_params.channels.unwrap().count();
        let sample_rate = codec_params.sample_rate.unwrap();
        // let frame_total = codec_params.n_frames.unwrap();
        // let bits_per_sample = codec_params.bits_per_sample.unwrap();
        // let fpp = codec_params.start_ts;

        // println!(
        //     "Channel Count: {}, Sample Rate: {}, frames: {}, bits per sample: {}, layout: {}",
        //     channel_count, sample_rate, frame_total, bits_per_sample, fpp
        // );

        // let decoded = loop {
        //     let packet = format_reader.next_packet().unwrap();
        //     match decoder.decode(&packet) {
        //         Ok(decoded) => break decoded,
        //         Err(_) => return Err(()),
        //     }
        // };
        //
        // let spec = decoded.spec().to_owned();
        //
        // let sample_rate = spec.rate;
        // let channel_count = decoded.frames();

        // I think we can skip the sample symphonia buffer step when resampling, by
        // manually iterating over samples in each channel and reading it into the resample
        // input buffer and doing *input_sample = symp_sample.into_sample(). This should type case
        // the sample to whatever type the resample input buffer needs
        //
        // The sample buffer copy interleaved step will just be unnecessary iterations over the sample,
        // becuase in order to resample, we have to deinterleave the samples _again_
        // This is essentially reimplmenting the SampleBuffer::copy_interleaved_ref method just wiht
        // a different sample destination.

        // // TODO: Make this configurable by passing the stream config from the server
        // let target_sample_rate = 44100 as usize;
        // let target_channel_count = 2; // This will probably always be 2 but we can make this configurable too

        // let source_sample_rate = spec.rate as usize;
        // let source_channel_count = spec.channels.count();

        // let resampler = FftFixedInOut::<f32>::new(
        //     sample_rate as usize,
        //     target_sample_rate,
        //     2048,
        //     channel_count,
        // )
        // .unwrap();

        let file_handle = FileHandle {
            decoder,
            // resampler,
            format_reader,
            sample_rate,
            channel_count,
        };

        Ok(file_handle)
    }

    pub fn test(&self) {}
    pub fn next_frame(&self) {}
}
