import { execSync } from 'node:child_process';
import { rmSync, mkdirSync, readdirSync, readFileSync, writeFileSync, statSync } from 'node:fs';
import { resolve, dirname, delimiter, join } from 'node:path';
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

// protobuf-ts emits well-known-type helpers (e.g. google/protobuf/timestamp.ts)
// whose method signatures carry parameters they don't use. Our tsconfig enables
// `noUnusedParameters`, which would flag this *generated* code. Prepend
// `// @ts-nocheck` to every generated file so machine output stays out of the
// strict app lint while its type declarations remain importable. Re-applied on
// every regeneration, so the generated tree never needs hand-editing.
function prependTsNoCheck(dir) {
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) {
      prependTsNoCheck(full);
    } else if (entry.endsWith('.ts')) {
      const source = readFileSync(full, 'utf8');
      if (!source.startsWith('// @ts-nocheck')) {
        writeFileSync(full, `// @ts-nocheck\n${source}`);
      }
    }
  }
}

prependTsNoCheck(outDir);

console.log('Protobuf generation done.');
