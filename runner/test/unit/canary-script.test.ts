// runner/scripts/canary.ts runs its checks at the top level and exits before
// the end of the file, so a top-level const or let declared below the exit is
// still uninitialised when the checks reach it. Daybreak, 10 Nov (finding
// 81): BLIP sat down there, so the first failed read threw "Cannot access
// 'BLIP' before initialization" instead of being tried again.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const file = path.join(path.dirname(fileURLToPath(import.meta.url)), '../../scripts/canary.ts');

test('the canary declares every top-level const and let before it exits', () => {
  const lines = fs.readFileSync(file, 'utf8').split('\n');
  const exit = lines.findIndex((l) => /^process\.exit\(/.test(l));
  assert.ok(exit > 0, 'the top-level exit');
  const late = lines.slice(exit + 1).filter((l) => /^(export )?(const|let) /.test(l));
  assert.deepEqual(late, []);
  assert.ok(lines.slice(0, exit).some((l) => l.startsWith('const BLIP = ')), 'BLIP is up top');
});
