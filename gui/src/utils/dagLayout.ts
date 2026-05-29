/**
 * Simple layered DAG layout using topological sort.
 * Places nodes in columns based on their depth in the DAG.
 */
export interface LayoutNode {
  id: string;
  width?: number;
  height?: number;
}

export interface LayoutEdge {
  from: string;
  to: string;
}

export function autoLayout(
  nodes: LayoutNode[],
  edges: LayoutEdge[],
  options?: { nodeWidth?: number; nodeHeight?: number; hGap?: number; vGap?: number }
): Map<string, { x: number; y: number }> {
  const nodeW = options?.nodeWidth ?? 170;
  const nodeH = options?.nodeHeight ?? 70;
  const hGap = options?.hGap ?? 220;
  const vGap = options?.vGap ?? 100;

  const positions = new Map<string, { x: number; y: number }>();

  if (nodes.length === 0) return positions;

  // Build adjacency and compute in-degree
  const adj = new Map<string, string[]>();
  const inDegree = new Map<string, number>();
  for (const n of nodes) {
    adj.set(n.id, []);
    inDegree.set(n.id, 0);
  }
  for (const e of edges) {
    adj.get(e.from)?.push(e.to);
    inDegree.set(e.to, (inDegree.get(e.to) ?? 0) + 1);
  }

  // Topological sort into layers
  const layers: string[][] = [];
  const queue: string[] = [];
  inDegree.forEach((deg, id) => { if (deg === 0) queue.push(id); });
  const processed = new Set<string>();

  while (queue.length > 0) {
    const layer: string[] = [];
    const nextQueue: string[] = [];
    for (const id of queue) {
      if (processed.has(id)) continue;
      processed.add(id);
      layer.push(id);
      for (const next of adj.get(id) ?? []) {
        const deg = (inDegree.get(next) ?? 1) - 1;
        inDegree.set(next, deg);
        if (deg === 0 && !processed.has(next)) nextQueue.push(next);
      }
    }
    if (layer.length > 0) layers.push(layer);
    queue.length = 0;
    queue.push(...nextQueue);
  }

  // Position nodes by layer and index within layer
  for (let li = 0; li < layers.length; li++) {
    const layer = layers[li];
    const totalHeight = layer.length * nodeH + (layer.length - 1) * vGap;
    const startY = -totalHeight / 2;
    for (let ni = 0; ni < layer.length; ni++) {
      positions.set(layer[ni], {
        x: li * hGap + 20,
        y: startY + ni * (nodeH + vGap) + 60,
      });
    }
  }

  return positions;
}
