import { BatchLeftPanel } from "./BatchLeftPanel";
import { BatchCenterPanel } from "./BatchCenterPanel";
import { BatchRightPanel } from "./BatchRightPanel";

export function BatchMode() {
  return (
    <div className="batch-layout">
      <BatchLeftPanel />
      <BatchCenterPanel />
      <BatchRightPanel />
    </div>
  );
}
