// Editing one of Mike's messages (features/editing.feature).
import { Then, When } from '@cucumber/cucumber';
import assert from 'node:assert/strict';
import { sleep } from '../support/driver.ts';
import type { ThreadbearWorld } from '../support/world.ts';

const edited = new WeakMap<ThreadbearWorld, { seq: number; text: string }>();

When('I edit my first message', async function (this: ThreadbearWorld) {
  const d = await this.until('a message of mine', (d) => d.transcript.rows.find((r) => r.kind === 'user' && r.id.startsWith('u:')));
  const out = await this.app.do({ cmd: 'edit-message', row: d.id });
  assert.match(out, /ok editing/);
  edited.set(this, { seq: d.seq ?? 0, text: d.text });
});

Then('the composer holds my first message to edit', async function (this: ThreadbearWorld) {
  const want = edited.get(this)!;
  await this.until('the message in the box, being edited', (d) => d.composer.editing === want.seq && d.composer.text.trim().length > 0 && want.text.startsWith(d.composer.text.trim().slice(0, 20)));
});

Then("the composer isn't editing", async function (this: ThreadbearWorld) {
  await this.until('no edit under way', (d) => d.composer.editing == null, 20_000);
});

When("I replace what's in the composer with {string}", async function (this: ThreadbearWorld, text: string) {
  await this.app.do({ cmd: 'type', text, replace: true });
});

Then('the transcript has my message {string}', async function (this: ThreadbearWorld, text: string) {
  await this.until(`my message "${text}"`, (d) => d.transcript.rows.some((r) => r.kind === 'user' && r.text.startsWith(text)), 30_000);
});

Then('the composer is still editing my first message', async function (this: ThreadbearWorld) {
  const want = edited.get(this)!;
  await this.until('the edit still under way', (d) => d.composer.editing === want.seq);
});

// Long enough for an allow to have reached the mock runner and the card to
// have gone, as it does in a second after a click on Allow.
Then('the approval for {string} is still waiting', async function (this: ThreadbearWorld, command: string) {
  await sleep(2_000);
  const d = await this.app.dump();
  // An answered one stays a row for a moment, its text then ending in
  // "(allow)" or "(deny)", before the work it let run folds it away.
  const row = d.transcript.rows.find((r) => r.kind === 'approval' && r.text.includes(command));
  assert.ok(row, `the approval for "${command}" was answered and has gone`);
  assert.doesNotMatch(row.text, /\((allow|deny|answered)\)\s*$/, `the approval for "${command}" was answered`);
});
