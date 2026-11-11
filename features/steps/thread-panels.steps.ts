// The panels round the thread: the Threads panel folded away with ⌘/, and a
// thread in a panel of its own.
import { Then } from '@cucumber/cucumber';
import type { ThreadbearWorld } from '../support/world.ts';

Then('the Threads panel is folded away', async function (this: ThreadbearWorld) {
  await this.until('the Threads panel off screen', (d) => d.threadsShowing === false);
});

Then('the Threads panel is showing', async function (this: ThreadbearWorld) {
  await this.until('the Threads panel on screen', (d) => d.threadsShowing === true);
});
