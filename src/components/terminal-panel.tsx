import { useEffect, useRef } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import { closeTerminalSession, startTerminalSession, writeTerminalInput } from "@/lib/tauri";

type TerminalOutputEvent = {
  sessionId: string;
  data: string;
};

export function TerminalPanel({ mode }: { mode: string }) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const sessionIdRef = useRef<string>("");

  useEffect(() => {
    if (!containerRef.current) {
      return;
    }

    const terminal = new Terminal({
      fontFamily: "Consolas, 'Courier New', monospace",
      fontSize: 12,
      lineHeight: 1.25,
      cursorBlink: true,
      cursorStyle: "block",
      convertEol: true,
      allowTransparency: false,
      theme: {
        background: "#0d1117",
        foreground: "#d7e2f0",
        cursor: "#7dd3fc",
        selectionBackground: "#24415f",
        black: "#0d1117",
        brightBlack: "#4b5563",
        red: "#f87171",
        brightRed: "#fca5a5",
        green: "#4ade80",
        brightGreen: "#86efac",
        yellow: "#facc15",
        brightYellow: "#fde68a",
        blue: "#60a5fa",
        brightBlue: "#93c5fd",
        magenta: "#c084fc",
        brightMagenta: "#d8b4fe",
        cyan: "#22d3ee",
        brightCyan: "#67e8f9",
        white: "#cbd5e1",
        brightWhite: "#f8fafc",
      },
    });
    const fitAddon = new FitAddon();
    terminal.loadAddon(fitAddon);
    terminal.open(containerRef.current);
    fitAddon.fit();
    terminal.focus();
    containerRef.current.addEventListener("click", () => terminal.focus());

    let unlisten: UnlistenFn | null = null;
    const disposeInput = terminal.onData((data) => {
      if (sessionIdRef.current) {
        void writeTerminalInput(sessionIdRef.current, data);
      }
    });

    const boot = async () => {
      const session = await startTerminalSession(mode);
      sessionIdRef.current = session.sessionId;
      terminal.writeln(`\x1b[36m[terminal]\x1b[0m ${session.shell}`);
      terminal.write("\r\n");

      unlisten = await listen<TerminalOutputEvent>("terminal-output", (event) => {
        if (event.payload.sessionId === sessionIdRef.current) {
          terminal.write(event.payload.data);
        }
      });
    };

    void boot();

    const handleResize = () => fitAddon.fit();
    window.addEventListener("resize", handleResize);

    return () => {
      window.removeEventListener("resize", handleResize);
      if (unlisten) {
        void unlisten();
      }
      if (sessionIdRef.current) {
        void closeTerminalSession(sessionIdRef.current);
      }
      disposeInput.dispose();
      terminal.dispose();
    };
  }, [mode]);

  return <div ref={containerRef} className="h-full w-full cursor-text bg-[#0d1117]" />;
}
