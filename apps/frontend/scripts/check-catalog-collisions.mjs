/**
 * Reports message-id collisions *between* Lingui catalogs.
 *
 * Why this exists: catalogs are per-module, but at runtime they are all merged
 * into ONE flat map (see `shared/presentation/utils/i18n.ts`), keyed by a hash
 * of the message + its context. `Object.assign` in glob order means the last
 * catalog loaded wins, so two modules that translate the same source string
 * differently silently overwrite each other — app-wide, not just on their own
 * pages. When the loser is a real translation and the winner is empty (an
 * untranslated entry compiles to its English source), the UI simply shows
 * English with every catalog looking perfectly translated. That is how
 * "Start your agent" survived next to a filled-in "Démarrez votre agent".
 *
 * Two ways out of a collision, both fine:
 *   - unify the wording, when the two uses genuinely mean the same thing;
 *   - give one of them a `context` (`<Trans context="date-prefix">`,
 *     `msg({ context, message })`), when the same English word needs different
 *     grammar in French — "Created" as a column header vs. as "Créé le" before
 *     a date. Context is part of the hash, so it separates them cleanly.
 *
 * Usage:
 *   node scripts/check-catalog-collisions.mjs [--report-only]
 */

import { readdirSync, readFileSync } from 'node:fs';

const reportOnly = process.argv.includes('--report-only');

const ROOT = 'src/modules';
const MODULE_ID = /^(features\/[^/]+|platform\/[^/]+|[^/]+)/;

/** `features/jobs/locales/fr/messages.po` -> `features/jobs`. */
const moduleOf = relPath => MODULE_ID.exec(relPath)?.[1] ?? relPath;

/** `features/jobs/locales/fr/messages.po` -> `fr`. */
const localeOf = relPath => /\/locales\/([^/]+)\/messages\.po$/.exec(relPath)?.[1] ?? null;

/**
 * Entries look like:
 *   msgctxt "date-prefix"      (optional)
 *   msgid "some text"
 *   msgstr "translation"
 * and any of them may be split over several quoted continuation lines.
 * Obsolete entries are prefixed `#~` and are deliberately not matched — they
 * carry no context and must not be compared.
 */
const parsePo = source => {
  const entries = [];
  const buffer = { msgctxt: '', msgid: '', msgstr: '' };
  let field = null;

  const flush = () => {
    if (buffer.msgid !== '') {
      entries.push({ ctx: buffer.msgctxt, id: buffer.msgid, str: buffer.msgstr });
    }
    buffer.msgctxt = '';
    buffer.msgid = '';
    buffer.msgstr = '';
    field = null;
  };

  for (const line of source.split('\n')) {
    const msgctxt = /^msgctxt "(.*)"$/.exec(line);
    const msgid = /^msgid "(.*)"$/.exec(line);
    const msgstr = /^msgstr "(.*)"$/.exec(line);
    const cont = /^"(.*)"$/.exec(line);

    if (msgctxt) {
      flush();
      field = 'msgctxt';
      buffer.msgctxt = msgctxt[1];
    } else if (msgid) {
      if (field !== 'msgctxt') flush();
      field = 'msgid';
      buffer.msgid = msgid[1];
    } else if (msgstr && field === 'msgid') {
      field = 'msgstr';
      buffer.msgstr = msgstr[1];
    } else if (cont && field) {
      buffer[field] += cont[1];
    } else if (line.trim() === '' && field === 'msgstr') {
      flush();
    }
  }
  flush();
  return entries;
};

// Sorted, because that is the order the runtime merges them in: later wins.
const catalogPaths = readdirSync(ROOT, { recursive: true, encoding: 'utf8' })
  .map(p => p.split('\\').join('/'))
  .filter(p => /\/locales\/[^/]+\/messages\.po$/.test(p))
  .sort();

/** locale -> "ctx\u001Fid" -> [{ module, str }, ...] */
const byLocale = new Map();

for (const relPath of catalogPaths) {
  const locale = localeOf(relPath);
  if (!locale) continue;
  if (!byLocale.has(locale)) byLocale.set(locale, new Map());
  const index = byLocale.get(locale);

  for (const { ctx, id, str } of parsePo(readFileSync(`${ROOT}/${relPath}`, 'utf8'))) {
    const key = `${ctx}\u001F${id}`;
    if (!index.has(key)) index.set(key, []);
    index.get(key).push({ module: moduleOf(relPath), str });
  }
}

const collisions = [];
for (const [locale, index] of byLocale) {
  for (const [key, holders] of index) {
    if (holders.length < 2) continue;
    if (new Set(holders.map(h => h.str)).size === 1) continue; // same wording: harmless
    const [ctx, id] = key.split('\u001F');
    collisions.push({ locale, ctx, id, holders });
  }
}

const catalogCount = catalogPaths.length;

if (collisions.length === 0) {
  console.log(`\n✔ no catalog collisions (${catalogCount} catalogs checked)\n`);
  process.exit(0);
}

collisions.sort((a, b) => a.locale.localeCompare(b.locale) || a.id.localeCompare(b.id));

console.log(
  `\n${reportOnly ? '⚠' : '✖'} ${collisions.length} catalog collision(s) ` +
    `across ${catalogCount} catalogs:\n`,
);

for (const { locale, ctx, id, holders } of collisions) {
  const label = ctx ? `${JSON.stringify(id)} (context ${JSON.stringify(ctx)})` : JSON.stringify(id);
  console.log(`  [${locale}] ${label}`);
  holders.forEach(({ module, str }, i) => {
    const value = str === '' ? '<untranslated → falls back to English>' : JSON.stringify(str);
    const marker = i === holders.length - 1 ? '  <-- WINS app-wide' : '';
    console.log(`      ${module.padEnd(28)} ${value}${marker}`);
  });
  console.log('');
}

console.log(
  '  Fix by unifying the wording, or by giving one use a `context` so the two\n' +
    '  hash to different ids. See the header of this script.\n',
);

process.exit(reportOnly ? 0 : 1);
