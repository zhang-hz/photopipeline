export interface DAGNodeData {
  id: string;
  pluginId: string;
  label: string;
  enabled: boolean;
  position: { x: number; y: number };
  params: Record<string, unknown>;
  inputs: string[];
  outputs: string[];
}

export interface DAGEdgeData {
  id: string;
  fromNode: string;
  toNode: string;
}
