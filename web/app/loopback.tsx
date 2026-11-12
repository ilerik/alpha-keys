"use client";

import { createContext, useContext, useState, type ReactNode } from "react";

const LoopbackContext = createContext({ allowed: false, allow: () => {} });

export function LoopbackProvider({ children }: { children: ReactNode }) {
  const [allowed, setAllowed] = useState(false);
  return (
    <LoopbackContext.Provider value={{ allowed, allow: () => setAllowed(true) }}>
      {children}
    </LoopbackContext.Provider>
  );
}

export function useLoopback() {
  return useContext(LoopbackContext);
}
