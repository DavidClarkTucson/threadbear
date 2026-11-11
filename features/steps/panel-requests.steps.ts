// An agent's show_panel (features/panel-requests.feature), through the
// mock's showPanel and panelState (docs/protocol.md, "Showing panels").
import { Then, When } from '@cucumber/cucumber';
import assert from 'node:assert/strict';
import type { ThreadbearWorld } from '../support/world.ts';

const requested = new WeakMap<ThreadbearWorld, { threadId: string; requestId: string }>();

When("I'm looking at the app", async function (this: ThreadbearWorld) {
  // A scripted app is never in front: this says Mike is looking.
  await this.app.do({ cmd: 'focus' });
});

When('the agent asks to show the {string} panel', async function (this: ThreadbearWorld, panel: string) {
  const d = await this.app.dump();
  const threadId = d.selection?.id;
  assert.ok(threadId, 'no thread open');
  const out = await this.app.do({ cmd: 'mock', method: 'showPanel', params: { threadId, panel } });
  const requestId = JSON.parse(out.replace(/^(\w+ )?ok /, '')).request.id as string;
  requested.set(this, { threadId, requestId });
});

Then('the dock has the {string} panel', async function (this: ThreadbearWorld, panel: string) {
  await this.until(`the ${panel} panel in the dock`, (d) => new RegExp(`\\b${panel}\\b`).test(d.dock ?? ''));
});

async function panelState(world: ThreadbearWorld): Promise<{ request: { id: string } | null; shown: string[] }> {
  const asked = requested.get(world);
  assert.ok(asked, 'no panel was asked for');
  const out = await world.app.do({ cmd: 'mock', method: 'panelState', params: { threadId: asked.threadId } });
  return JSON.parse(out.replace(/^(\w+ )?ok /, ''));
}

Then('the runner heard the panel was shown', async function (this: ThreadbearWorld) {
  const asked = requested.get(this)!;
  for (let i = 0; i < 100; i += 1) {
    const state = await panelState(this);
    if (state.shown.includes(asked.requestId) || state.request === null) return;
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  assert.fail('the app never said it showed the panel');
});

Then('the runner still has the request waiting', async function (this: ThreadbearWorld) {
  const asked = requested.get(this)!;
  const state = await panelState(this);
  assert.equal(state.request?.id, asked.requestId, 'the request went');
  assert.ok(!state.shown.includes(asked.requestId), 'the app said it showed a panel the thread hasn\'t got');
});
