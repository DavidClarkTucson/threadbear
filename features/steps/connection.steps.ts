import { Given, Then, When } from '@cucumber/cucumber';
import type { ThreadbearWorld } from '../support/world.ts';

When('the runner restarts', async function (this: ThreadbearWorld) {
  await this.app.do({ cmd: 'mock', method: 'restart', params: {} });
});

Then('the app is connected again', async function (this: ThreadbearWorld) {
  // The mock answers `restart` first and drops every connection about 100 ms
  // later, then stays down for a few seconds. A dump taken straight after the
  // step above still says "connected", so wait for the app to see the drop
  // before waiting for it to come back, or this passes without a reconnect.
  await this.until('the connection to drop', (d) => d.connection !== 'connected');
  await this.until('the app connected again', (d) => d.connection === 'connected', 30_000);
});

Then('the transcript still shows the thread {string}', async function (this: ThreadbearWorld, title: string) {
  await this.until(`"${title}" still open with its rows`, (d) => d.selection?.title === title && d.transcript.rows.length > 0);
});

When('the runner goes away without a word for {int} seconds', async function (this: ThreadbearWorld, seconds: number) {
  // No "restarting" notice first, as a runner that crashed or was stopped.
  await this.app.do({ cmd: 'mock', method: 'goAway', params: { downMs: seconds * 1000 } });
});

Then('the bar says {string}', async function (this: ThreadbearWorld, text: string) {
  await this.until(`the bar saying "${text}"`, (d) => d.banner?.text === text && d.banner?.style === 'warning', 15_000);
});

Given("the thread {string} can't be read once", async function (this: ThreadbearWorld, title: string) {
  const d = await this.app.dump();
  const id = this.thread(d, title)?.id;
  if (!id) throw new Error(`no thread "${title}"`);
  await this.app.do({ cmd: 'mock', method: 'failEvents', params: { threadId: id, times: 1 } });
});

When('I open the thread {string}, which fails to load', async function (this: ThreadbearWorld, title: string) {
  const t = await this.until(`a thread called "${title}"`, (d) => this.thread(d, title));
  // The Swift app's `open` answers with an error when the thread doesn't
  // load, which is what this scenario is for: what it shows is checked next.
  await this.app.do({ cmd: 'open', id: t.id }).catch(() => undefined);
});

Then("the transcript says it couldn't be loaded", async function (this: ThreadbearWorld) {
  await this.until('the transcript failed', (d) => (d.transcript?.state ?? '').startsWith('failed:'));
});

When('I press Try again on the transcript', async function (this: ThreadbearWorld) {
  await this.app.do({ cmd: 'click', tip: 'Load the thread again' });
});

Then('the transcript shows the thread {string}', async function (this: ThreadbearWorld, title: string) {
  await this.until(`"${title}" loaded with its rows`, (d) => d.selection?.title === title && d.transcript?.state === 'loaded' && d.transcript.rows.length > 0);
});
