import { msg } from '@lingui/core/macro';

/**
 * `lingui extract` does not read `.svelte` — see `refacto_svelte.md` §4.5.
 *
 * One message of the React original could not be carried across as one id.
 * `<Trans>Requires the Rust toolchain, from a clone of <0>the scylla repo</0>…`
 * encoded the anchor as an element slot, and a `t()` call returns a string,
 * which cannot carry one. It is split into the three pieces below — three new
 * ids, whose French forms are carried over by hand in the catalogs. The rest of
 * the file keeps the original ids, placeholder names included.
 */
export const agentRunInstructionsMessages = {
  fromSource: msg`From source`,
  runFromCheckout: msg`Run from a repo checkout`,
  runWithDocker: msg`Run with Docker`,
  commandCopied: msg`Command copied`,

  requiresRustBefore: msg`Requires the Rust toolchain, from a clone of`,
  scyllaRepoLink: msg`the scylla repo`,
  requiresRustAfter: msg`(crate scylla-agent). The agent runs in the foreground and Ctrl-C stops it.`,

  dockerForeground: msg`The agent starts in the foreground: its logs show up right in the terminal and Ctrl-C stops it.`,
  replaceUrl: msg`The URL above is prefilled with this page's own address. Replace it if your agent reaches the control plane another way — for example http://host.docker.internal:8080 from inside Docker, or a private-network address.`,
  replaceSecret: msg`Replace APP_SECRET with the secret revealed when this agent was created — it cannot be shown again. Lost it? Delete this agent and create a new one.`,
};
