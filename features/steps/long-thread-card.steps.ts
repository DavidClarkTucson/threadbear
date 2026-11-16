// The long thread card above the message box (protocol "Long threads"):
// its words are the dump's composer.longThread, its buttons found by tip.
import { Then, When } from '@cucumber/cucumber';
import type { ThreadbearWorld } from '../support/world.ts';

const TIPS: Record<string, string> = {
  Compact: 'Send /compact',
  'Continue in a new thread': 'Ask the agent for a short handoff',
  'Not now': 'Hide this for this thread',
  Open: 'Go to the thread this one carried on in',
};

const words = (d: { composer?: unknown }) => (d.composer as { longThread?: string | null } | undefined)?.longThread ?? null;

Then('the box has the long thread card saying {string}', async function (this: ThreadbearWorld, start: string) {
  await this.until(`the long thread card saying "${start}"`, (d) => words(d)?.startsWith(start) === true, 10_000);
});

Then('the box has no long thread card', async function (this: ThreadbearWorld) {
  await this.until('no long thread card', (d) => words(d) === null, 10_000);
});

When('I press {string} on the long thread card', async function (this: ThreadbearWorld, button: string) {
  const tip = TIPS[button];
  if (!tip) throw new Error(`The long thread card has no "${button}"`);
  await this.until(`the long thread card`, (d) => words(d) !== null, 10_000);
  await this.app.do({ cmd: 'click', tipPrefix: tip });
});
