import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface UseTauriCommandResult<TResult> {
  execute: <TArgs extends Record<string, unknown>>(args: TArgs) => Promise<TResult>;
  data: TResult | null;
  loading: boolean;
  error: string | null;
}

export function useTauriCommand<TResult = unknown>(commandName: string): UseTauriCommandResult<TResult> {
  const [data, setData] = useState<TResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const execute = useCallback(
    async <TArgs extends Record<string, unknown>>(args: TArgs): Promise<TResult> => {
      setLoading(true);
      setError(null);
      try {
        const result = await invoke<TResult>(commandName, args);
        setData(result);
        return result;
      } catch (e) {
        const msg = String(e);
        setError(msg);
        throw new Error(msg);
      } finally {
        setLoading(false);
      }
    },
    [commandName]
  );

  return { execute, data, loading, error };
}
