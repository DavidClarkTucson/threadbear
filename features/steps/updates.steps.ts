// The Updates panel (features/updates.feature). Its lines are what it draws:
// "# App", "# Runner", "# From Mike" and "# CLIs" for its headings, a CLI as
// "<what it has> (<its update setting>) [<its buttons>]", and what's under a
// line indented by two spaces.
import { When } from '@cucumber/cucumber';
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
