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
