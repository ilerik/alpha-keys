"use client";

import { useState } from "react";
import { flipOneByte, PUBLISHED_DIGEST, recordDigest, RELAYED_CANONICAL, verificationVerdict } from "../../lib/verify";

export function FlipByte() {
  const [status, setStatus] = useState<"VERIFIED" | "NOT VERIFIED" | "">("");
  const [digest, setDigest] = useState("");
  const [flipped, setFlipped] = useState(false);

  async function check(bytes: string) {
    const next = await recordDigest(bytes);
    setDigest(next);
    setStatus(verificationVerdict(next, PUBLISHED_DIGEST));
  }

  return (
    <section className="card">
      <h2>Flip one byte</h2>
      <p>
        The published digest is sha256 of the fixture line below. The check reads that line again.
        One flipped byte makes the digest disagree.
      </p>
      <pre>{flipped ? flipOneByte(RELAYED_CANONICAL) : RELAYED_CANONICAL}</pre>
      <div className="row">
        <button type="button" onClick={() => check(RELAYED_CANONICAL)}>
          Check record
        </button>
        <button
          className="ghost"
          type="button"
          onClick={() => {
            setFlipped(true);
            void check(flipOneByte(RELAYED_CANONICAL));
          }}
        >
          Flip one byte
        </button>
      </div>
      {status ? (
        <p className={status === "VERIFIED" ? "badge" : "refused"}>{status}</p>
      ) : (
        <p className="muted">Check the record, then flip one byte.</p>
      )}
      {digest ? (
        <p>
          recomputed <code>{digest}</code>
          <br />
          published <code>{PUBLISHED_DIGEST}</code>
        </p>
      ) : null}
      <p className="muted">
        Reproduce the published digest: <code>printf &apos;%s&apos; &apos;{RELAYED_CANONICAL}&apos; | sha256sum</code>
      </p>
    </section>
  );
}
