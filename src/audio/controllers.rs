use anyhow::Result;
use crossbeam_channel;
use jack::{AsyncClient, AudioIn, AudioOut, Port, PortSpec};
use realfft::RealFftPlanner;
use ringbuf::RingBuffer;
use std::thread;
use std::time::Duration;

pub const DEFAULT_FFT_SIZE: u32 = 8;
// const DEFAULT_FREQ_SCALE: i64 = 1; // log10
// const DEFAULT_MAXFREQ: i64 = 20000;
// const DEFAULT_MINFREQ: i64 = 20;
// const DEFAULT_SPEC_MIN: i64 = -100;
// const DEFAULT_SPEC_MAX: i64 = -20;
// const DEFAULT_WEIGHTING: i64 = 1; // A
// const DEFAULT_SHOW_FREQ_LABELS: bool = true;
// const DEFAULT_RESPONSE_TIME: f64 = 0.025;
// const DEFAULT_RESPONSE_TIME_INDEX: i32 = 0;

fn port_name(port_basename: &str, port_index: i64) -> String {
    return format!("{}_{}", port_basename, port_index);
}

fn make_client<T>(
    client_name: &str,
    port_basename: &str,
    port_spec_1: T,
    port_spec_2: T,
) -> Result<(jack::Client, Port<T>, Port<T>)>
where
    T: PortSpec,
{
    let (client, _status) = jack::Client::new(client_name, jack::ClientOptions::NO_START_SERVER)?;
    let port_1 = client.register_port(&port_name(port_basename, 1), port_spec_1)?;
    let port_2 = client.register_port(&port_name(port_basename, 2), port_spec_2)?;

    return Ok((client, port_1, port_2));
}

pub struct SineGen<'a> {
    pub name: &'a str,
    pub port_basename: &'a str,
    pub sample_rate: usize,
    jack_client: AsyncClient<(), SineProcessor>,
}

impl<'a> SineGen<'a> {
    pub fn new(
        name: &'a str,
        port_spec_1: AudioOut,
        port_spec_2: AudioOut,
        freq_1: f32,
        freq_2: f32,
    ) -> Result<SineGen<'a>> {
        let port_basename = "out";
        let (client, port_1, port_2) = make_client(name, port_basename, port_spec_1, port_spec_2)?;

        let sample_rate = client.sample_rate();
        let process = SineProcessor {
            port_1,
            port_2,
            frame_t: 1.0 / sample_rate as f32,
            freq_1,
            freq_2,
            time: 0.,
        };

        let jack_client = client.activate_async((), process)?;

        Ok(SineGen {
            name,
            port_basename,
            sample_rate,
            jack_client,
        })
    }
}

struct SineProcessor {
    port_1: jack::Port<AudioOut>,
    port_2: jack::Port<AudioOut>,
    frame_t: f32,
    freq_1: f32,
    freq_2: f32,
    time: f32,
}

impl jack::ProcessHandler for SineProcessor {
    fn process(&mut self, _: &jack::Client, ps: &jack::ProcessScope) -> jack::Control {
        // Get output buffer
        let out1 = self.port_1.as_mut_slice(ps);
        let out2 = self.port_2.as_mut_slice(ps);

        for (a, b) in out1.iter_mut().zip(out2.iter_mut()) {
            let x1 = self.freq_1 * self.time * 2.0 * std::f32::consts::PI;
            let y1 = x1.sin();
            let x2 = self.freq_2 * self.time * 2.0 * std::f32::consts::PI;
            let y2 = x2.sin();
            *a = y1 as f32;
            *b = y2 as f32;
            self.time += self.frame_t;
        }

        // Continue as normal
        jack::Control::Continue
    }
}

impl<'a> PortConnector for SineGen<'a> {
    fn connect_to<P: PortName + PortConnector>(&self, client: &P) -> Result<()> {
        for i in 1..3 {
            self.jack_client
                .as_client()
                .connect_ports_by_name(&self.client_port_name(i), &client.client_port_name(i))?
        }
        Ok(())
    }
}

impl<'a> PortName for SineGen<'a> {
    fn client_port_name(&self, port_index: i64) -> String {
        let p_name = port_name(self.port_basename, port_index);
        return format!("{}:{}", self.name, p_name);
    }
}

pub struct FFTProc<'a> {
    pub name: &'a str,
    pub port_basename: &'a str,
    pub sample_rate: usize,
    pub fft_size: usize,
    jack_client: AsyncClient<(), FFTProcessor>,
}

impl<'a> FFTProc<'a> {
    pub fn new(
        name: &'a str,
        port_spec_1: AudioIn,
        port_spec_2: AudioIn,
        fft_size: usize,
        thread_tx: crossbeam_channel::Sender<Vec<f32>>,
    ) -> Result<FFTProc<'a>> {
        let port_basename = "in";
        let (client, port_1, port_2) = make_client(name, port_basename, port_spec_1, port_spec_2)?;

        let sample_rate = client.sample_rate();
        let frame_size = client.buffer_size();

        let ring_buf_size = fft_size * 10;
        let rb = RingBuffer::<f32>::new(ring_buf_size);
        let (prod, cons) = rb.split();

        let process = FFTProcessor {
            port_1,
            port_2,
            tmp_buff: Vec::with_capacity(frame_size as usize),
            ring_buf: prod,
        };

        let jack_client = client.activate_async((), process)?;

        let fft_proc = FFTProc {
            name,
            port_basename,
            sample_rate,
            jack_client,
            fft_size,
        };

        fft_proc.run(cons, thread_tx);

        return Ok(fft_proc);
    }

    fn run(
        &self,
        mut ring_buf: ringbuf::Consumer<f32>,
        thread_tx: crossbeam_channel::Sender<Vec<f32>>,
    ) {
        let sleep_millis = Duration::from_millis(5);
        let mut planner = RealFftPlanner::new();
        let fft = planner.plan_fft_forward(self.fft_size);
        let mut spec_buf = fft.make_output_vec();
        let mut sig_buf = Vec::with_capacity(self.fft_size);
        let spec_buf_len = spec_buf.len() as f32;
        let fft_size = self.fft_size;

        thread::spawn(move || loop {
            while ring_buf.len() < fft_size {
                thread::sleep(sleep_millis);
            }
            ring_buf.pop_slice(&mut sig_buf);
            fft.process(&mut sig_buf, &mut spec_buf).unwrap();
            let out: Vec<f32> = spec_buf
                .iter()
                .into_iter()
                .map(|x| x.norm_sqr().sqrt() / spec_buf_len)
                .collect();

            thread_tx.send(out).unwrap();
        });
    }
}

struct FFTProcessor {
    port_1: jack::Port<AudioIn>,
    port_2: jack::Port<AudioIn>,
    tmp_buff: Vec<f32>,
    ring_buf: ringbuf::Producer<f32>,
}

impl jack::ProcessHandler for FFTProcessor {
    fn process(&mut self, _: &jack::Client, ps: &jack::ProcessScope) -> jack::Control {
        let in_a_p = self.port_1.as_slice(ps);
        let in_b_p = self.port_2.as_slice(ps);
        for i in 0..=1023 {
            self.tmp_buff[i] = in_a_p[i] + in_b_p[i];
        }
        self.ring_buf.push_slice(&self.tmp_buff);
        jack::Control::Continue
    }
}

impl<'a> PortConnector for FFTProc<'a> {
    fn connect_to<P: PortName + PortConnector>(&self, client: &P) -> Result<()> {
        for i in 1..3 {
            self.jack_client
                .as_client()
                .connect_ports_by_name(&self.client_port_name(i), &client.client_port_name(i))?
        }
        Ok(())
    }
}

impl<'a> PortName for FFTProc<'a> {
    fn client_port_name(&self, port_index: i64) -> String {
        let p_name = port_name(self.port_basename, port_index);
        return format!("{}:{}", self.name, p_name);
    }
}

pub struct PlaybackSystem<'a> {
    name: &'a str,
    port_basename: &'a str,
}

impl<'a> PlaybackSystem<'a> {
    pub fn new() -> PlaybackSystem<'a> {
        PlaybackSystem {
            name: "system",
            port_basename: "playback",
        }
    }
}

impl<'a> PortConnector for PlaybackSystem<'a> {
    fn connect_to<P: PortName + PortConnector>(&self, client: &P) -> Result<()> {
        client.connect_to(self)?;
        Ok(())
    }
}

impl<'a> PortName for PlaybackSystem<'a> {
    fn client_port_name(&self, port_index: i64) -> String {
        let p_name = port_name(self.port_basename, port_index);
        return format!("{}:{}", self.name, p_name);
    }
}

pub trait PortName {
    fn client_port_name(&self, port_index: i64) -> String;
}

pub trait PortConnector {
    fn connect_to<P: PortName + PortConnector>(&self, client: &P) -> Result<()>;
}
