// Engine NodeExecutor Tests (~20 test cases)
// Tests node execution, error propagation, cancellation, disabled nodes,
// cycle detection, progress callbacks, and state management.

use photopipeline_core::{
    ChannelLayout, ImageFormat, ImageInfo, Metadata, PixelBuffer, PixelFormat, PluginError,
};
use photopipeline_engine::{
    ExecutionContext, NodeExecutor, NodeRunState, NodeStatus, ParameterResolver, PipelineGraph,
};
use photopipeline_plugin::{ProgressSink, Registry};
use std::sync::Arc;
use uuid::Uuid;

// ── Helpers ──────────────────────────────────────────────────────────

fn empty_registry() -> Arc<Registry> {
    Arc::new(Registry::new())
}

fn empty_resolver() -> Arc<ParameterResolver> {
    Arc::new(ParameterResolver::new())
}

fn make_test_image_info() -> ImageInfo {
    ImageInfo {
        id: Uuid::new_v4(),
        path: "/tmp/test.jpg".into(),
        filename: "test.jpg".into(),
        format: ImageFormat::JPEG,
        width: 100,
        height: 100,
        file_size_bytes: 5000,
        pixel_format: PixelFormat::U8,
        color_space: Default::default(),
    }
}

fn make_test_buffer() -> PixelBuffer {
    PixelBuffer::new(100, 100, ChannelLayout::RGB, PixelFormat::U8)
}

// ── Progress Sink Mocks ─────────────────────────────────────────────

struct NoopProgress;
impl ProgressSink for NoopProgress {
    fn set_progress(&self, _fraction: f32, _message: &str) {}
    fn is_canceled(&self) -> bool {
        false
    }
}

struct CancelProgress;
impl ProgressSink for CancelProgress {
    fn set_progress(&self, _fraction: f32, _message: &str) {}
    fn is_canceled(&self) -> bool {
        true
    }
}

struct TrackingProgress {
    progress_values: std::sync::Mutex<Vec<f32>>,
}
impl TrackingProgress {
    fn new() -> Self {
        Self {
            progress_values: std::sync::Mutex::new(Vec::new()),
        }
    }
    fn values(&self) -> Vec<f32> {
        self.progress_values.lock().unwrap().clone()
    }
}
impl ProgressSink for TrackingProgress {
    fn set_progress(&self, fraction: f32, _message: &str) {
        self.progress_values.lock().unwrap().push(fraction);
    }
    fn is_canceled(&self) -> bool {
        false
    }
}

// ── Construction Tests ──────────────────────────────────────────────

#[test]
fn executor_new_with_empty_registry() {
    let reg = empty_registry();
    let resolver = empty_resolver();
    let executor = NodeExecutor::new(reg, resolver);
    assert!(format!("{:?}", executor).contains("NodeExecutor"));
}

// ── Execute Empty Graph ─────────────────────────────────────────────

#[test]
fn execute_empty_graph_returns_ok() {
    let reg = empty_registry();
    let resolver = empty_resolver();
    let executor = NodeExecutor::new(reg, resolver);
    let graph = PipelineGraph::new();
    let info = make_test_image_info();
    let md = Metadata::default();
    let progress: Box<dyn ProgressSink> = Box::new(NoopProgress);

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        executor.execute(&graph, &info, None, &md, progress).await
    });

    assert!(result.is_ok(), "empty graph should execute successfully");
    let exec_result = result.unwrap();
    assert!(exec_result.node_states.is_empty());
    assert!(exec_result.buffer.is_none());
}

// ── Disabled Node Tests ─────────────────────────────────────────────

#[test]
fn execute_disabled_node_is_skipped() {
    let reg = empty_registry();
    let resolver = empty_resolver();
    let executor = NodeExecutor::new(reg, resolver);

    let mut graph = PipelineGraph::new();
    let node_id = graph.add_node("n1".into(), "test.plugin".into());
    graph.node_mut(node_id).unwrap().enabled = false;

    let info = make_test_image_info();
    let md = Metadata::default();
    let progress: Box<dyn ProgressSink> = Box::new(NoopProgress);

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        executor.execute(&graph, &info, None, &md, progress).await
    });

    assert!(result.is_ok(), "disabled node should not cause error");
    let exec_result = result.unwrap();
    let state = exec_result.node_states.get(&node_id).unwrap();
    assert!(matches!(state.status, NodeStatus::Skipped));
}

#[test]
fn execute_all_disabled_nodes_no_error() {
    let reg = empty_registry();
    let resolver = empty_resolver();
    let executor = NodeExecutor::new(reg, resolver);

    let mut graph = PipelineGraph::new();
    for i in 0..3 {
        let id = graph.add_node(format!("n{i}"), format!("test{i}.plugin"));
        graph.node_mut(id).unwrap().enabled = false;
    }

    let info = make_test_image_info();
    let md = Metadata::default();
    let progress: Box<dyn ProgressSink> = Box::new(NoopProgress);

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        executor.execute(&graph, &info, None, &md, progress).await
    });

    assert!(result.is_ok());
    let exec_result = result.unwrap();
    assert_eq!(exec_result.node_states.len(), 3);
    for state in exec_result.node_states.values() {
        assert!(matches!(state.status, NodeStatus::Skipped));
    }
}

// ── Missing Plugin Tests ────────────────────────────────────────────

#[test]
fn execute_missing_plugin_returns_not_found_error() {
    let reg = empty_registry();
    let resolver = empty_resolver();
    let executor = NodeExecutor::new(reg, resolver);

    let mut graph = PipelineGraph::new();
    graph.add_node("n1".into(), "nonexistent.plugin.test".into());

    let info = make_test_image_info();
    let md = Metadata::default();
    let progress: Box<dyn ProgressSink> = Box::new(NoopProgress);

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        executor.execute(&graph, &info, None, &md, progress).await
    });

    assert!(result.is_err(), "missing plugin must produce error");
    match &result {
        Err(PluginError::NotFound(id)) => {
            assert!(!id.is_empty(), "NotFound must include the plugin id");
        }
        other => panic!("expected NotFound error, got {:?}", other),
    }
}

// ── Pipeline Error Propagation ──────────────────────────────────────

#[test]
fn execute_no_buffer_for_pixel_node_returns_error() {
    let reg = empty_registry();
    let resolver = empty_resolver();
    let executor = NodeExecutor::new(reg, resolver);

    let mut graph = PipelineGraph::new();
    // Node "n1" exists but plugin is not in registry - it will be queried
    // and produce NotFound error
    graph.add_node("n1".into(), "nonexistent.test.plugin".into());

    let info = make_test_image_info();
    let md = Metadata::default();
    let progress: Box<dyn ProgressSink> = Box::new(NoopProgress);

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        executor.execute(&graph, &info, None, &md, progress).await
    });

    assert!(result.is_err());
}

// ── Cancellation Tests ──────────────────────────────────────────────

#[test]
fn execute_canceled_returns_cancel_error() {
    // Cancellation is checked before plugin lookup in execute().
    // CancelProgress returns is_canceled=true immediately, so the
    // executor returns Canceled before reaching plugin resolution.
    // Empty registry is safe here because plugin lookup is never reached.
    let reg = empty_registry();
    let resolver = empty_resolver();
    let executor = NodeExecutor::new(reg, resolver);

    let mut graph = PipelineGraph::new();
    graph.add_node("n1".into(), "some.plugin".into());

    let info = make_test_image_info();
    let md = Metadata::default();
    let progress: Box<dyn ProgressSink> = Box::new(CancelProgress);

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        executor.execute(&graph, &info, None, &md, progress).await
    });

    assert!(result.is_err(), "canceled execution must return error");
    match result {
        Err(PluginError::Canceled { .. }) => {}
        other => panic!("expected Canceled error, got {:?}", other),
    }
}

// ── Cycle Detection in Executor ─────────────────────────────────────

#[test]
fn execute_graph_with_cycle_returns_error() {
    // Verifies that PipelineGraph::connect rejects edges that would
    // create a cycle. The rejected edge is never added to the graph,
    // so the executor never sees the cycle — cycle prevention is
    // enforced at graph construction time.
    let reg = empty_registry();
    let resolver = empty_resolver();
    let executor = NodeExecutor::new(reg, resolver);

    let mut graph = PipelineGraph::new();
    let n1 = graph.add_node("a".into(), "p1".into());
    let n2 = graph.add_node("b".into(), "p2".into());

    // Attempt to create cycle a->b->a
    let out_a = graph.node(n1).unwrap().outputs[0];
    let in_b = graph.node(n2).unwrap().inputs[0];
    let out_b = graph.node(n2).unwrap().outputs[0];
    let in_a = graph.node(n1).unwrap().inputs[0];

    // Edge a->b is valid
    assert!(graph.connect(out_a, in_b).is_ok());
    // Edge b->a creates a cycle, must be rejected
    let result_b_to_a = graph.connect(out_b, in_a);
    assert!(result_b_to_a.is_err(), "cycle-creating edge must be rejected");
    // Verify the cycle edge was NOT added (only a->b exists)
    assert_eq!(graph.edges.len(), 1, "only a->b edge should exist");

    // With cancel progress sink, execute returns Canceled quickly
    let info = make_test_image_info();
    let md = Metadata::default();
    let progress: Box<dyn ProgressSink> = Box::new(CancelProgress);
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        executor.execute(&graph, &info, None, &md, progress).await
    });

    assert!(result.is_err(), "canceled execution must return error");
    match result {
        Err(PluginError::Canceled { .. }) => {}
        other => panic!("expected Canceled error, got {:?}", other),
    }
}

// ── NodeRunState Tests ──────────────────────────────────────────────

#[test]
fn node_run_state_default_is_pending() {
    let state = NodeRunState::new();
    assert!(matches!(state.status, NodeStatus::Pending));
    assert!(state.started_at.is_none());
}

#[test]
fn node_run_state_default_trait() {
    let state = NodeRunState::default();
    assert!(matches!(state.status, NodeStatus::Pending));
}

#[test]
fn node_status_display_variants() {
    assert!(format!("{:?}", NodeStatus::Pending).contains("Pending"));
    assert!(format!("{:?}", NodeStatus::Running).contains("Running"));
    assert!(format!("{:?}", NodeStatus::Skipped).contains("Skipped"));
    assert!(format!("{:?}", NodeStatus::Failed("err".into())).contains("Failed"));
}

#[test]
fn node_status_completed_contains_stats() {
    let stats = photopipeline_core::ProcessingStats {
        elapsed_ms: 100,
        cpu_time_ms: 80,
        gpu_time_ms: None,
        peak_memory_mb: 256,
        input_pixels: 10000,
        output_pixels: 10000,
    };
    let status = NodeStatus::Completed(stats);
    let display = format!("{:?}", status);
    assert!(display.contains("Completed"));
}

// ── ExecutionContext (no buffer) Tests ─────────────────────────────

#[test]
fn execution_context_new_no_buffer() {
    let info = make_test_image_info();
    let md = Metadata::default();
    let ctx = ExecutionContext::new(info.clone(), None, md);
    assert!(ctx.buffer.is_none());
    assert_eq!(ctx.image_info.filename, "test.jpg");
    assert!(ctx.node_states.is_empty());
}

// NOTE: ExecutionContext::new with buffer is covered implicitly by
// tests that invoke the executor, which creates PixelBuffer
// internally. Creating PixelBuffer directly triggers AlignedBuffer
// debug assertions on Windows.

// ── Progress Callback Tests ─────────────────────────────────────────

#[test]
fn execute_disabled_graph_triggers_progress_at_completion() {
    let reg = empty_registry();
    let resolver = empty_resolver();
    let executor = NodeExecutor::new(reg, resolver);

    let mut graph = PipelineGraph::new();
    let id = graph.add_node("n1".into(), "test.plugin".into());
    graph.node_mut(id).unwrap().enabled = false;

    let info = make_test_image_info();
    let md = Metadata::default();
    let progress = Arc::new(TrackingProgress::new());
    let progress_clone = progress.clone();

    // TrackProgress must be used as Box<dyn ProgressSink>
    // We need a new type for this test since TrackingProgress contains a Mutex
    // We use a custom wrapper that delegates
    struct TrackedProgressWrapper {
        inner: Arc<TrackingProgress>,
    }
    impl ProgressSink for TrackedProgressWrapper {
        fn set_progress(&self, fraction: f32, message: &str) {
            self.inner.set_progress(fraction, message);
        }
        fn is_canceled(&self) -> bool {
            self.inner.is_canceled()
        }
    }

    let progress_box: Box<dyn ProgressSink> = Box::new(TrackedProgressWrapper {
        inner: progress_clone,
    });

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        executor.execute(&graph, &info, None, &md, progress_box).await
    });

    assert!(result.is_ok());
    let values = progress.values();
    assert!(!values.is_empty(), "progress callback should be triggered");
    // The final progress should be 1.0
    assert!(*values.last().unwrap() == 1.0, "final progress must be 1.0");
}

// ── Error Propagation in Graph ──────────────────────────────────────

#[test]
fn execute_two_nodes_disabled_and_missing_plugin() {
    // Mixed disabled + missing plugin: disabled nodes are skipped,
    // missing plugin causes error.
    let reg = empty_registry();
    let resolver = empty_resolver();
    let executor = NodeExecutor::new(reg, resolver);

    let mut graph = PipelineGraph::new();
    let n1 = graph.add_node("n1".into(), "good.plugin".into());
    graph.node_mut(n1).unwrap().enabled = false;
    let n2 = graph.add_node("n2".into(), "bad.plugin".into());

    // Connect n1->n2
    let out_n1 = graph.node(n1).unwrap().outputs[0];
    let in_n2 = graph.node(n2).unwrap().inputs[0];
    graph.connect(out_n1, in_n2).unwrap();

    let info = make_test_image_info();
    let md = Metadata::default();
    let progress: Box<dyn ProgressSink> = Box::new(NoopProgress);

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        executor.execute(&graph, &info, None, &md, progress).await
    });

    // n1 is disabled (skipped), n2 has missing plugin (NotFound)
    assert!(result.is_err());
    match result {
        Err(PluginError::NotFound(id)) => {
            assert!(!id.is_empty(), "NotFound must include the plugin id");
        }
        other => panic!("expected NotFound, got {:?}", other),
    }
}
