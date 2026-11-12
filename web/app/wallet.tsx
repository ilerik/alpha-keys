"use client";

import { useState } from "react";

export function WalletButton() {
  const [account, setAccount] = useState("");
  const [error, setError] = useState("");

  async function connect() {
    setError("");
    try {
      const { NearConnector } = await import("@hot-labs/near-connect");
      const connector = new NearConnector({ network: "testnet" });
      const wallet = await connector.connect();
      const accounts = await wallet.getAccounts();
      setAccount(accounts[0]?.accountId ?? "");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "wallet unavailable");
    }
  }

  async function disconnect() {
    setAccount("");
  }

  return (
    <span className="row">
      {account ? (
        <>
          <span>{account}</span>
          <button type="button" onClick={disconnect}>Log out</button>
        </>
      ) : (
        <button type="button" onClick={connect}>Connect testnet wallet</button>
      )}
      {error ? <span className="muted">{error}</span> : null}
    </span>
  );
}
