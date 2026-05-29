import { useEffect, useRef } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export function useTauriEvent<T>(eventName: string, handler: (payload: T) => void): void {
  const handlerRef = useRef(handler);
  handlerRef.current = handler;

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;

    const setup = async () => {
      unlisten = await listen<T>(eventName, (event) => {
        handlerRef.current(event.payload);
      });
    };

    setup();

    return () => {
      if (unlisten) unlisten();
    };
  }, [eventName]);
}
