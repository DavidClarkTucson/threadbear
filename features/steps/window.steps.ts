// The window's panels, the zoom and what's remembered between launches.
import { Given, Then, When } from '@cucumber/cucumber';
import assert from 'node:assert/strict';
import type { ThreadbearWorld } from '../support/world.ts';

Then('the panels are laid out as {string}', async function (this: ThreadbearWorld, layout: string) {
  await this.until(`the layout "${layout}"`, (d) => d.dock === layout);
});

Then("the panels aren't laid out as {string}", async function (this: ThreadbearWorld, layout: string) {
  await this.until('the layout to change', (d) => d.dock !== undefined && d.dock !== layout);
});

When('I move the {string} panel to the {string} edge', async function (this: ThreadbearWorld, panel: string, edge: string) {
  await this.app.do({ cmd: 'dock-move', panel, edge });
});

When('I choose {string} from the menu bar', async function (this: ThreadbearWorld, title: string) {
  await this.app.do({ cmd: 'menu-item', title });
});

When('I move the window and quit the app', function () {
  // By hand (see the feature file): this step only exists so the scenario reads.
  return 'pending';
});

When('I open the app again', function () {
  return 'pending';
});

Then('the window is where it was', function () {
  return 'pending';
});

When('the app is quit and opened again', async function (this: ThreadbearWorld) {
  await this.app.relaunch();
  await this.until('the app connected again, with its threads', (d) => d.connection === 'connected' && d.sidebar.sections.some((s) => s.threads.length > 0), 30_000);
});

When('I press {string} with {string}', async function (this: ThreadbearWorld, key: string, modifiers: string) {
  await this.app.do({ cmd: 'menu-key', key, modifiers: modifiers.split(',').map((m) => m.trim()).filter(Boolean) });
});

When('I set the zoom to {int} percent', async function (this: ThreadbearWorld, percent: number) {
  await this.app.do({ cmd: 'zoom', percent });
});

When('I choose Actual Size', async function (this: ThreadbearWorld) {
  await this.app.do({ cmd: 'zoom', to: 'reset' });
});

Then('the zoom is {int} percent', async function (this: ThreadbearWorld, percent: number) {
  await this.until(`the zoom at ${percent}%`, (d) => d.zoom === percent);
});

Given("I note how tall the transcript's first row is", async function (this: ThreadbearWorld) {
  this.notedHeight = await this.until('a drawn first row', (d) => d.transcript.rows[0]?.height);
});

Then("the transcript's first row is taller than it was", async function (this: ThreadbearWorld) {
  const before = this.notedHeight ?? 0;
  const after = await this.until('the first row to have grown', (d) => {
    const h = d.transcript.rows[0]?.height;
    return h && h > before ? h : undefined;
  });
  assert.ok(after > before, `${after} is not taller than ${before}`);
});
