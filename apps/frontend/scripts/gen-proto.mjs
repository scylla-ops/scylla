import { execSync } from 'node:child_process';
import { rmSync, mkdirSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = resolve(__dirname, '..');
const protoDir = resolve(root, '..', '..', 'libs', 'protocol', 'proto');
const outDir = resolve(root, 'src', 'generated');

rmSync(outDir, { recursive: true, force: true });
mkdirSync(outDir, { recursive: true });

const cmd = `protoc -I="${protoDir}" --ts_out="${outDir}" "${protoDir}"/*.proto`;

execSync(cmd, { stdio: 'inherit', shell: true, cwd: root });

console.log('Protobuf generation done.');
