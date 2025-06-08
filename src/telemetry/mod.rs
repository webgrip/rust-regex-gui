use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use tracing::{info, subscriber::set_global_default};
use tracing_subscriber::{
    Registry,
    fmt::MakeWriter,
    layer::SubscriberExt,
    EnvFilter,
    filter::LevelFilter,
};

/// Abstraction over logging so application code can remain decoupled from
/// specific logging frameworks.
pub trait Logger: Send + Sync {
    fn log(&self, message: &str);
}

/// Logger implementation that forwards logs to the [`tracing`] facade.
pub struct TracingLogger;

impl Logger for TracingLogger {
    fn log(&self, message: &str) {
        info!("{message}");
    }
}

/// Writer that stores formatted log lines in memory so they can be displayed
/// in the GUI.
#[derive(Default, Clone)]
pub struct MemoryWriter {
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl MemoryWriter {
    pub fn logs(&self) -> Vec<String> {
        let buf = self.buffer.lock().unwrap();
        String::from_utf8_lossy(&buf)
            .lines()
            .map(|s| s.to_string())
            .collect()
    }
}

impl Write for MemoryWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for MemoryWriter {
    type Writer = MemoryWriter;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

/// Initializes global tracing with an in-memory writer. Returns the writer so
/// callers can read the collected logs.
pub fn init_tracing(level: LevelFilter) -> MemoryWriter {
    let writer = MemoryWriter::default();
    let layer = tracing_subscriber::fmt::layer().with_writer(writer.clone());
    let filter = EnvFilter::new(format!("{}={}", env!("CARGO_PKG_NAME"), level.to_string().to_lowercase()));
    let subscriber = Registry::default().with(filter).with(layer);
    set_global_default(subscriber).expect("set tracing subscriber");
    writer
}
