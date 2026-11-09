import { Then, When } from '@cucumber/cucumber';
import assert from 'node:assert/strict';
import type { ThreadbearWorld } from '../support/world.ts';

// The mock's threads on "bruce" (@remote-threads turns them on, world.ts).

Then('the row {string} is on the runner {string}', async function (this: ThreadbearWorld, title: string, runner: string) {
  await this.until(`"${title}" on ${runner}`, (d) => this.thread(d, title)?.runner === runner);
});

Then('the row {string} has the machine tile', async function (this: ThreadbearWorld, title: string) {
  await this.until(`"${title}" with the machine tile`, (d) => this.thread(d, title)?.tile === 'runner');
});

Then('the row {string} isn\'t on another runner', async function (this: ThreadbearWorld, title: string) {
  const d = await this.app.dump();
  const t = this.thread(d, title);
  assert.ok(t, `no row ${title}`);
  assert.ok(!t.runner, `"${title}" says it's on ${t.runner}`);
});

Then('the composer is on the runner {string}', async function (this: ThreadbearWorld, runner: string) {
  await this.until(`the composer on ${runner}`, (d) => d.composer.runner === runner);
});

When('I make {string} the default runner', async function (this: ThreadbearWorld, runner: string) {
  await this.app.do({ cmd: 'composer-menu', menu: 'runner', setDefault: runner });
});
