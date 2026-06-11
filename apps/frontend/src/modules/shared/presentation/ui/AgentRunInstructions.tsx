import { Trans } from '@lingui/react/macro';
import { Container, Terminal } from 'lucide-react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@shadcn/tabs.tsx';
import { CodeSnippet } from '@shadcn/code-snippet.tsx';

interface AgentRunInstructionsProps {
  /** The agent's app id (its identity towards the control plane). */
  appId: string;
  /** One-time plaintext secret (creation flow). Omitted → placeholder. */
  secret?: string;
}

const SECRET_PLACEHOLDER = '<APP_SECRET>';
const URL_PLACEHOLDER = '<CONTROL_PLANE_URL>';
const DOCKER_IMAGE = 'godlyjaaaaj/scylla-agent:latest';

/**
 * "How do I start this worker?" — the copy-paste answer, in the two supported
 * flavors: `cargo run` from a repo checkout (first — no extra tooling concepts
 * beyond Rust itself), or the published Docker image. Shared by the one-time
 * secret dialog (shown after the reveal) and the agent detail page
 * (placeholder secret).
 *
 * The control-plane URL is deliberately a placeholder, not a guessed value:
 * the UI only knows how the BROWSER reaches the API, which is rarely the
 * address a worker on some other machine (or inside Docker) should dial.
 * Better an explicit "fill this in" than a copied command that half-works.
 */
export const AgentRunInstructions = ({ appId, secret }: AgentRunInstructionsProps) => {
  const secretValue = secret ?? SECRET_PLACEHOLDER;

  const cargoCommand = [
    'cargo run --release --bin scylla-agent -- \\',
    `  --control-plane-url ${URL_PLACEHOLDER} \\`,
    `  --app-id ${appId} \\`,
    `  --app-secret ${secretValue} \\`,
    '  --workspace-root "$HOME/.scylla/workspaces"',
  ].join('\n');

  const dockerCommand = [
    `docker run ${DOCKER_IMAGE} \\`,
    `  --control-plane-url ${URL_PLACEHOLDER} \\`,
    `  --app-id ${appId} \\`,
    `  --app-secret ${secretValue}`,
  ].join('\n');

  const triggerClass =
    'h-auto flex-none gap-1.5 rounded-none border-0 border-b-2 border-transparent bg-transparent ' +
    'px-0 pb-2 text-xs text-muted-foreground shadow-none transition-colors hover:text-foreground ' +
    'data-[state=active]:border-primary data-[state=active]:text-primary ' +
    'data-[state=active]:bg-transparent data-[state=active]:shadow-none';

  return (
    <div className='space-y-2'>
      <Tabs defaultValue='cargo' className='w-full'>
        <TabsList className='h-auto w-full justify-start gap-4 rounded-none border-b bg-transparent p-0'>
          <TabsTrigger value='cargo' className={triggerClass}>
            <Terminal className='h-3.5 w-3.5' />
            <Trans>From source</Trans>
          </TabsTrigger>
          <TabsTrigger value='docker' className={triggerClass}>
            <Container className='h-3.5 w-3.5' />
            Docker
          </TabsTrigger>
        </TabsList>

        <TabsContent value='cargo' className='space-y-2 pt-2'>
          <CodeSnippet
            multiline
            value={cargoCommand}
            copyToast='Command copied'
            label={<Trans>Run from a repo checkout</Trans>}
          />
          <p className='text-xs text-muted-foreground'>
            <Trans>
              Requires the Rust toolchain, from a clone of{' '}
              <a
                href='https://github.com/scylla-ops/scylla'
                target='_blank'
                rel='noreferrer'
                className='underline underline-offset-2 hover:text-foreground'
              >
                the scylla repo
              </a>{' '}
              (crate scylla-agent). The worker runs in the foreground and Ctrl-C stops it.
            </Trans>
          </p>
        </TabsContent>

        <TabsContent value='docker' className='space-y-2 pt-2'>
          <CodeSnippet
            multiline
            value={dockerCommand}
            copyToast='Command copied'
            label={<Trans>Run with Docker</Trans>}
          />
          <p className='text-xs text-muted-foreground'>
            <Trans>
              The worker starts in the foreground: its logs show up right in the terminal and Ctrl-C
              stops it.
            </Trans>
          </p>
        </TabsContent>
      </Tabs>

      <p className='text-xs text-muted-foreground'>
        <Trans>
          Replace CONTROL_PLANE_URL with the address your worker reaches the control plane at — for
          example http://localhost:50051 on the same machine, or http://host.docker.internal:50051
          from inside Docker.
        </Trans>
      </p>

      {!secret && (
        <p className='rounded-md border bg-muted/40 p-2.5 text-xs text-muted-foreground'>
          <Trans>
            Replace APP_SECRET with the secret revealed when this agent was created — it cannot be
            shown again. Lost it? Delete this agent and create a new one.
          </Trans>
        </p>
      )}
    </div>
  );
};

export default AgentRunInstructions;
