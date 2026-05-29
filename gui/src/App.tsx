import { useEffect, useState } from "react";
import { FluentProvider, webDarkTheme, webLightTheme } from "@fluentui/react-components";
import { useAppStore } from "./stores/useAppStore";
import { usePluginStore } from "./stores/usePluginStore";
import { useFilmstripStore } from "./stores/useFilmstripStore";
import { usePipelineStore } from "./stores/usePipelineStore";
import { useKeyboard } from "./hooks/useKeyboard";
import { TitleBar } from "./components/layout/TitleBar";
import { MainLayout } from "./components/layout/MainLayout";
import { StatusBar } from "./components/layout/StatusBar";
import { BatchMode } from "./components/batch/BatchMode";
import { SettingsDialog } from "./components/settings/SettingsDialog";
import { ToastContainer } from "./components/common/Toast";
import { MOCK_IMAGES } from "./data/mockImages";
import "./App.css";

export function App() {
  const mode = useAppStore((s) => s.mode);
  const theme = useAppStore((s) => s.theme);
  const initMockData = usePluginStore((s) => s.initMockData);
  const importImages = useFilmstripStore((s) => s.importImages);
  const selectAll = useFilmstripStore((s) => s.selectAll);
  const clearSelection = useFilmstripStore((s) => s.clearSelection);
  const removeImages = useFilmstripStore((s) => s.removeImages);
  const selectedIndices = useFilmstripStore((s) => s.selectedIndices);
  const removeNode = usePipelineStore((s) => s.removeNode);
  const selectedNodeId = usePipelineStore((s) => s.selectedNodeId);
  const undo = usePipelineStore((s) => s.undo);
  const redo = usePipelineStore((s) => s.redo);
  const executePipeline = usePipelineStore((s) => s.executePipeline);
  const [settingsOpen, setSettingsOpen] = useState(false);

  useEffect(() => { initMockData(); importImages(MOCK_IMAGES); }, [initMockData, importImages]);

  useKeyboard([
    { key: "a", ctrl: true, handler: selectAll, scope: "filmstrip" },
    { key: "Escape", handler: () => { clearSelection(); setSettingsOpen(false); }, scope: "global" },
    { key: "Delete", handler: () => {
      if (selectedNodeId && mode === "edit") removeNode(selectedNodeId);
      else if (selectedIndices.size > 0) removeImages([...selectedIndices]);
    }, scope: "global" },
    { key: "z", ctrl: true, handler: undo, scope: "dag" },
    { key: "y", ctrl: true, handler: redo, scope: "dag" },
    { key: "e", ctrl: true, handler: () => { if (mode === "edit") executePipeline(); }, scope: "global" },
    { key: ",", ctrl: true, handler: () => setSettingsOpen(true), scope: "global" },
  ]);

  const currentTheme = theme === "light" ? webLightTheme : webDarkTheme;

  return (
    <FluentProvider theme={currentTheme}>
      <div id="app-root" className="app-shell">
        <TitleBar onOpenSettings={() => setSettingsOpen(true)} />
        {mode === "edit" ? <MainLayout /> : <BatchMode />}
        <StatusBar />
        <SettingsDialog isOpen={settingsOpen} onClose={() => setSettingsOpen(false)} />
        <ToastContainer />
      </div>
    </FluentProvider>
  );
}
