import { execSync } from 'node:child_process';
import { rmSync, mkdirSync, readdirSync } from 'node:fs';
import { resolve, dirname, delimiter } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = resolve(__dirname, '..');
const protoDir = resolve(root, '..', '..', 'crates', 'scylla-protocol', 'proto');
const outDir = resolve(root, 'src', 'generated');

rmSync(outDir, { recursive: true, force: true });
mkdirSync(outDir, { recursive: true });

// Use the project-local @protobuf-ts protoc wrapper (which auto-wires the
// protoc-gen-ts plugin and emits the *.client.ts layout the source imports).
// A bare `protoc` on PATH may resolve to a different plugin (e.g. a homebrew
// protoc-gen-ts) and produce an incompatible single-file layout.
const protoc = resolve(root, 'node_modules', '.bin', 'protoc');

// Enumerate .proto files in JS rather than relying on a shell glob: cmd.exe on
// Windows does not expand `*.proto`, so the glob would be passed to protoc
// verbatim and fail.
const protoFiles = readdirSync(protoDir)
  .filter((f) => f.endsWith('.proto'))
  .map((f) => `"${resolve(protoDir, f)}"`)
  .join(' ');

const cmd = `"${protoc}" -I="${protoDir}" --ts_out="${outDir}" ${protoFiles}`;

// protoc resolves the protoc-gen-ts plugin from PATH; ensure node_modules/.bin
// is on it so the local plugin is found (cross-platform).
const binDir = resolve(root, 'node_modules', '.bin');
execSync(cmd, {
  stdio: 'inherit',
  shell: true,
  cwd: root,
  env: { ...process.env, PATH: `${binDir}${delimiter}${process.env.PATH ?? ''}` },
});

console.log('Protobuf generation done.');
