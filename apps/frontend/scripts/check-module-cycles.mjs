/**
 * Reports dependency cycles *between modules*, reading dependency-cruiser JSON
 * from stdin.
 *
 * Why this exists: dependency-cruiser's `no-circular` rule works on the file
 * graph, and its `collapse` option is applied after validation. Our cycles are
 * folder-level — `features/jobs/hooks/a.ts` imports `features/pipeline`, and
 * `features/pipeline/hooks/b.ts` imports `features/jobs`. No single file is in
 * a loop, so `no-circular` sees nothing while the two modules are hopelessly
 * entangled. This collapses to module granularity first, then looks for
 * strongly connected components (so A->B->C->A is caught, not just A<->B).
 *
 * This is scaffolding. Once every cross-module import goes through a module's
 * `index.ts`, a folder cycle *becomes* a file cycle (a/index.ts -> b/index.ts ->
 * a/index.ts) and plain `no-circular` catches it natively — at which point this
 * script can be deleted.
 *
 * Usage:
 *   depcruise src --output-type json | node scripts/check-module-cycles.mjs [--report-only]
 */

const reportOnly = process.argv.includes('--report-only');

const MODULE_ID = /^src\/modules\/(features\/[^/]+|[^/]+)/;

/** `src/modules/features/jobs/presentation/x.ts` -> `features/jobs`. */
const moduleOf = filePath => MODULE_ID.exec(filePath)?.[1] ?? null;

const readStdin = async () => {
  const chunks = [];
  for await (const chunk of process.stdin) chunks.push(chunk);
  return Buffer.concat(chunks).toString('utf8');
};

const cruise = JSON.parse(await readStdin());

/** module -> Set<module>, plus one example file edge per pair for the report. */
const edges = new Map();
const examples = new Map();

for (const module of cruise.modules) {
  const from = moduleOf(module.source);
  if (!from) continue;

  for (const dependency of module.dependencies) {
    const to = moduleOf(dependency.resolved);
    if (!to || to === from) continue;

    if (!edges.has(from)) edges.set(from, new Set());
    edges.get(from).add(to);

    const key = `${from} -> ${to}`;
    if (!examples.has(key)) examples.set(key, `${module.source} -> ${dependency.resolved}`);
  }
}

// Tarjan's strongly connected components: every component with more than one
// module is a cycle, and every module in it can reach every other one.
const index = new Map();
const lowlink = new Map();
const onStack = new Set();
const stack = [];
const components = [];
let counter = 0;

const strongConnect = node => {
  index.set(node, counter);
  lowlink.set(node, counter);
  counter += 1;
  stack.push(node);
  onStack.add(node);

  for (const next of edges.get(node) ?? []) {
    if (!index.has(next)) {
      strongConnect(next);
      lowlink.set(node, Math.min(lowlink.get(node), lowlink.get(next)));
    } else if (onStack.has(next)) {
      lowlink.set(node, Math.min(lowlink.get(node), index.get(next)));
    }
  }

  if (lowlink.get(node) === index.get(node)) {
    const component = [];
    let member;
    do {
      member = stack.pop();
      onStack.delete(member);
      component.push(member);
    } while (member !== node);
    if (component.length > 1) components.push(component.sort());
  }
};

for (const node of edges.keys()) if (!index.has(node)) strongConnect(node);

if (components.length === 0) {
  console.log(`\n✔ no module cycles (${edges.size} modules cruised)\n`);
  process.exit(0);
}

// Within a component every module reaches every other, but the actionable unit
// is the pair that imports both ways — that is what someone has to go fix.
const pairs = [];
for (const component of components) {
  for (const a of component) {
    for (const b of edges.get(a) ?? []) {
      if (a < b && edges.get(b)?.has(a)) pairs.push([a, b]);
    }
  }
}

console.log(
  `\n${reportOnly ? '⚠' : '✖'} ${components.length} module cycle(s), ` +
    `${pairs.length} mutually dependent pair(s):\n`,
);

for (const component of components) {
  console.log(`  ${component.join(' <-> ')}`);
}

console.log('\n  offending edges:\n');
for (const [a, b] of pairs.sort()) {
  console.log(`    ${a} -> ${b}`);
  console.log(`      ${examples.get(`${a} -> ${b}`)}`);
  console.log(`    ${b} -> ${a}`);
  console.log(`      ${examples.get(`${b} -> ${a}`)}`);
  console.log('');
}

process.exit(reportOnly ? 0 : 1);
