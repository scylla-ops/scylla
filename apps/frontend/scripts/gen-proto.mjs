import { execSync } from 'node:child_process';
import { rmSync, mkdirSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
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

const cmd = `"${protoc}" -I="${protoDir}" --ts_out="${outDir}" "${protoDir}"/*.proto`;

execSync(cmd, { stdio: 'inherit', shell: true, cwd: root });

console.log('Protobuf generation done.');
