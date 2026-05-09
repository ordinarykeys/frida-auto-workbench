import type { PropsWithChildren, ReactNode } from "react";

type AppShellProps = PropsWithChildren<{
  sidebar: ReactNode;
  rightPanel: ReactNode;
}>;

export function AppShell({ children, sidebar, rightPanel }: AppShellProps) {
  return (
    <div className="h-screen bg-[#f3f3f3]">
      <main className="mx-auto grid h-full max-w-[1800px] grid-rows-[auto_auto_1fr] gap-3 p-3">
        <section>{sidebar}</section>
        <section>{children}</section>
        <section>{rightPanel}</section>
      </main>
    </div>
  );
}
