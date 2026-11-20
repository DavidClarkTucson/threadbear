import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { tmpDir } from '../helpers/fakes.ts';
import { checkPatchFile } from '../../src/patch-check.ts';

// tb patch-check: a patch file's load error without waiting for the
// runner to read it again (a duplicate id was only found by sleep and
// tb actions, 20 Nov).
const write = (body: string) => {
  const file = path.join(tmpDir(), 'demo.ts');
  fs.writeFileSync(file, body);
  return file;
};

test('a good patch lists what it gives', async () => {
  const r = await checkPatchFile(write(`export default { actions: [{ title: 'Say hi', run: 'echo hi' }], panels: [{ title: 'Docs', url: 'https://example.com' }] };`));
  assert.equal(r.error, undefined);
  assert.deepEqual(r.ids, ['demo.say-hi', 'demo.docs']);
});

test('two entries with one title are the error the runner gives', async () => {
  const r = await checkPatchFile(write(`export default { actions: [{ title: 'Jira', run: 'echo' }], panels: [{ title: 'Jira', url: 'https://example.com' }] };`));
  assert.match(r.error ?? '', /Two entries would both be called demo\.jira/);
});

test('a file that does not parse, or is not there, says so', async () => {
  assert.ok((await checkPatchFile(write(`export default { actions: [ };`))).error);
  assert.match((await checkPatchFile('/nonexistent/x.ts')).error ?? '', /can't read/);
});
