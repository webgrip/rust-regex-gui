use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt};

/// Abstraction over logging so application code can remain decoupled from
/// specific logging frameworks.
pub trait Logger: Send + Sync {
    fn log(&self, message: &str);
}

/// Logger implementation that forwards logs to the [`tracing`] facade.
pub struct TracingLogger;

impl Logger for TracingLogger {
    fn log(&self, message: &str) {
        tracing::info!(target: "app", "{}", message);
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

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for MemoryWriter {
    type Writer = MemoryWriter;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

/// Initializes global tracing with an in-memory writer. Returns the writer so
/// callers can read the collected logs.
pub fn init_tracing(level: tracing_subscriber::filter::LevelFilter) -> MemoryWriter {
    let writer = MemoryWriter::default();

    #[cfg(target_arch = "wasm32")]
    let layer = tracing_subscriber::fmt::layer()
        .with_writer(writer.clone())
        .with_ansi(false)
        // ⬇️ avoid std::time::SystemTime, which panics on wasm32-unknown-unknown
        .without_time();

    #[cfg(not(target_arch = "wasm32"))]
    let layer = tracing_subscriber::fmt::layer()
        .with_writer(writer.clone())
        .with_ansi(false);

    let filter = EnvFilter::new(format!(
        "{}={}",
        env!("CARGO_PKG_NAME").replace('-', "_"),
        level.to_string().to_lowercase()
    ));

    let subscriber = Registry::default().with(filter).with(layer);
    tracing::subscriber::set_global_default(subscriber).expect("set tracing subscriber");
    writer
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tracing_subscriber::fmt::MakeWriter;

    #[test]
    fn logs_returns_written_lines() {
        let mut writer = MemoryWriter::default();
        writer.write_all(b"first\nsecond\n").unwrap();
        assert_eq!(writer.logs(), vec!["first", "second"]);
    }

    #[test]
    fn make_writer_produces_shared_buffer() {
        let writer = MemoryWriter::default();
        let mut other = writer.make_writer();
        other.write_all(b"line\n").unwrap();
        assert_eq!(writer.logs(), vec!["line"]);
    }
}
