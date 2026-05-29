export interface PipelineProgressPayload {
  node_id: string;
  node_label: string;
  fraction: number;
  message: string;
  elapsed_ms: number;
}

export interface PipelineStagePayload {
  from_stage: string;
  to_stage: string;
}

export interface PipelineErrorPayload {
  node_id?: string;
  code: string;
  message: string;
}

export interface PipelineDonePayload {
  output_paths: string[];
  total_bytes: number;
  total_seconds: number;
}

export interface BatchProgressPayload {
  batch_id: string;
  status: string;
  total_files: number;
  completed_files: number;
  failed_files: number;
  current_file: string;
  fraction: number;
  progress_details: string;
}

export interface BackendStatusPayload {
  connected: boolean;
  gpu_backend?: string;
  memory_mb?: number;
  version?: string;
}
