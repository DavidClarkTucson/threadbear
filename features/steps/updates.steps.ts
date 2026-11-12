// The Updates panel (features/updates.feature). Its lines are what it draws:
// "# App", "# Runner", "# From Mike" and "# CLIs" for its headings, a CLI as
// "<what it has> (<its update setting>) [<its buttons>]", and what's under a
// line indented by two spaces.
import { Then, When } from '@cucumber/cucumber';
import assert from 'node:assert/strict';
import { okValue } from '../support/driver.ts';
import type { ThreadbearWorld } from '../support/world.ts';

When('the runner starts building main at {string} for {string}', async function (this: ThreadbearWorld, commit: string, subject: string) {
  await this.app.do({ cmd: 'mock', method: 'updates', params: { state: 'building', commit, subject } });
});

When('Mike has a release of 10 Nov with 3 new changes', async function (this: ThreadbearWorld) {
  const release = {
    tag: 'release/2026-11-10',
    commit: '0123456789abcdef0123456789abcdef01234567',
    changes: 3,
    subjects: ['Runners panel in the GPUI app', 'Updates panel', 'Connectors panel'],
    notes: [{ tag: 'release/2026-11-10', lines: ['Runners in the app'] }],
  };
  await this.app.do({ cmd: 'mock', method: 'updates', params: { state: 'idle', upstream: { state: 'available', release } } });
});

// The app's own update, made up (`updates` with `preview`, as both apps'
// screenshots use): staged is def5678 over abc1234.
When("the app's next version, def5678, is ready", async function (this: ThreadbearWorld) {
  await this.app.do({ cmd: 'updates', preview: 'staged' });
});

When("the app's next version is waiting for a Mac to build it on", async function (this: ThreadbearWorld) {
  await this.app.do({ cmd: 'updates', preview: 'waiting' });
});

// The app menu is the first one, whatever it's called. SwiftUI rebuilds its
// menus a moment after what they show changes, so this looks a few times.
Then('the app menu has {string}', async function (this: ThreadbearWorld, title: string) {
  let titles: string[] = [];
  for (let i = 0; i < 20; i += 1) {
    const out = JSON.parse(okValue(await this.app.do({ cmd: 'menus' }))) as { title: string; items: { title: string }[] }[];
    titles = out[0]?.items.map((m) => m.title) ?? [];
    if (titles.includes(title)) return;
    await new Promise((done) => setTimeout(done, 250));
  }
  assert.fail(`${title} isn't in the app menu: ${JSON.stringify(titles)}`);
});

// What a restart onto the staged version by itself waits for, as the GPUI
// app's once-a-second look sees it (`updates` with `restartCheck`). Whether
// the app's in front and how long the desktop's been idle are the machine's,
// so only what holds the restart off is checked.
type RestartCheck = { staged: boolean; loaded: boolean; hasDrafts: boolean; held: boolean; otherWindowInput: boolean };

async function restartWaits(world: ThreadbearWorld, what: string, wanted: (c: RestartCheck) => boolean) {
  let seen: RestartCheck | undefined;
  for (let i = 0; i < 40; i += 1) {
    seen = JSON.parse(await world.app.do({ cmd: 'updates', restartCheck: true })) as RestartCheck;
    if (wanted(seen)) return;
    await new Promise((done) => setTimeout(done, 100));
  }
  assert.fail(`a restart should wait for ${what}, but the app saw ${JSON.stringify(seen)}`);
}

Then('a restart onto it waits for nothing', async function (this: ThreadbearWorld) {
  await restartWaits(this, 'nothing', (c) => c.staged && c.loaded && !c.hasDrafts && !c.held && !c.otherWindowInput);
});

Then('a restart onto it waits for the words typed', async function (this: ThreadbearWorld) {
  await restartWaits(this, 'the words typed', (c) => c.hasDrafts);
});

Then('a restart onto it waits for the open menu', async function (this: ThreadbearWorld) {
  await restartWaits(this, 'the open menu', (c) => c.held && !c.hasDrafts);
});
