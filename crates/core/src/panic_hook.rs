use std::fmt;
use std::sync::Mutex;

#[derive(Debug, Clone, Default)]
pub struct ExecutionDump {
    pub pipeline_id: Option<String>,
    pub image_path: Option<String>,
    pub current_node_id: Option<String>,
    pub current_plugin: Option<String>,
    pub node_history: Vec<String>,
    pub buffer_size_bytes: u64,
    pub gpu_backend: Option<String>,
    pub gpu_memory_mb: u64,
}

impl fmt::Display for ExecutionDump {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "ExecutionDump:")?;
        if let Some(ref pid) = self.pipeline_id {
            writeln!(f, "  pipeline_id: {}", pid)?;
        }
        if let Some(ref img) = self.image_path {
            writeln!(f, "  image_path: {}", img)?;
        }
        if let Some(ref node) = self.current_node_id {
            writeln!(f, "  current_node_id: {}", node)?;
        }
        if let Some(ref plugin) = self.current_plugin {
            writeln!(f, "  current_plugin: {}", plugin)?;
        }
        if !self.node_history.is_empty() {
            writeln!(f, "  node_history ({} nodes):", self.node_history.len())?;
            for (i, n) in self.node_history.iter().enumerate() {
                writeln!(f, "    {}. {}", i + 1, n)?;
            }
        }
        writeln!(f, "  buffer_size_bytes: {}", self.buffer_size_bytes)?;
        if let Some(ref gpu) = self.gpu_backend {
            writeln!(f, "  gpu_backend: {}", gpu)?;
            writeln!(f, "  gpu_memory_mb: {}", self.gpu_memory_mb)?;
        }
        Ok(())
    }
}

pub static CURRENT_EXECUTION_CTX: Mutex<Option<ExecutionDump>> = Mutex::new(None);

pub fn install_panic_hook() {
    let prev_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |info| {
        let backtrace = std::backtrace::Backtrace::capture();

        let location = info
            .location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
            .unwrap_or_else(|| "<unknown location>".to_string());

        let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Box<dyn Any>".to_string()
        };

        tracing::error!(
            panic_location = %location,
            panic_payload = %payload,
            "PANIC at {}: {}",
            location,
            payload,
        );

        if let Ok(ctx) = CURRENT_EXECUTION_CTX.lock()
            && let Some(ref dump) = *ctx
        {
            tracing::error!(
                execution_dump = %dump,
                "Panic occurred during pipeline execution:\n{}",
                dump,
            );
        }

        tracing::error!("Backtrace:\n{}", backtrace);

        prev_hook(info);
    }));
}
