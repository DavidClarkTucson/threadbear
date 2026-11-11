import assert from 'node:assert/strict';
import { Then, When } from '@cucumber/cucumber';
import type { ThreadbearWorld } from '../support/world.ts';
import { okValue, type Dump, type Row } from '../support/driver.ts';

/** My newest message starting with `words`. */
const mine = (d: Dump, words: string): Row | undefined => [...d.transcript.rows].reverse().find((r) => r.kind === 'user' && r.text.startsWith(words));

async function chipsAre(world: ThreadbearWorld, words: string, want: string[]) {
  let seen: string[] | undefined;
  try {
    await world.until(`my message "${words}" with the chips ${want.join(', ')}`, (d) => {
      seen = mine(d, words)?.chips;
      return JSON.stringify(seen) === JSON.stringify(want);
    });
  } catch (error) {
    assert.fail(`${(error as Error).message.split('\n')[0]}; its chips were ${JSON.stringify(seen)}`);
  }
}

Then('my message {string} shows the chips {string}', async function (this: ThreadbearWorld, words: string, chip: string) {
  await chipsAre(this, words, [chip]);
});

Then('my message {string} shows the chips {string} and {string}', async function (this: ThreadbearWorld, words: string, a: string, b: string) {
  await chipsAre(this, words, [a, b]);
});

/** The row of my message with a chip labelled `label`. */
async function rowWithChip(world: ThreadbearWorld, label: string): Promise<Row> {
  return world.until(`my message with the chip "${label}"`, (d) => [...d.transcript.rows].reverse().find((r) => r.kind === 'user' && r.chips?.some((c) => c.endsWith(`: ${label}`))));
}

// Both apps' `peek`: the card a chip shows after the pointer rests on it.
Then('resting on the chip {string} in my message shows its card', async function (this: ThreadbearWorld, label: string) {
  const row = await rowWithChip(this, label);
  const out = await this.app.do({ cmd: 'peek', row: row.id, words: label });
  assert.match(out, /ok window/);
  await this.app.do({ cmd: 'peek', row: row.id, show: false });
});

// A press on a picture's chip opens it as a click on the pictures over the
// words would: both apps say which of the message's pictures, of how many.
Then('pressing the chip {string} in my message opens picture {int} of {int}', async function (this: ThreadbearWorld, label: string, number: number, count: number) {
  const row = await rowWithChip(this, label);
  const out = okValue(await this.app.do({ cmd: 'peek', row: row.id, words: label, press: true }));
  const found = /^quicklook (\d+) of (\[.*\])$/.exec(out);
  assert.ok(found, out);
  assert.equal(Number(found[1]) + 1, number, out);
  assert.equal((JSON.parse(found[2]) as string[]).length, count, out);
});

// A chip's link followed as a click on it would (both apps' `link`).
When('I follow {string} in my message {string}', async function (this: ThreadbearWorld, url: string, words: string) {
  const row = await this.until(`my message "${words}"`, (d) => mine(d, words));
  await this.app.do({ cmd: 'link', row: row.id, url });
});
