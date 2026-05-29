import { Sidebar } from "./Sidebar";
import { Content } from "./Content";
import { Panel } from "./Panel";

export function MainLayout() {
  return (
    <div className="main-layout">
      <Sidebar />
      <Content />
      <Panel />
    </div>
  );
}
