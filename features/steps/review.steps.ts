// The design review window (review.feature). Both apps take the Swift app's
// `review` command, which opens the open thread's review as its card does and
// says where the window has got to; `"open": false` only reads one that's
// open. Clicks go to the review's own window by tooltip, as a person's would.
import { Then, When } from '@cucumber/cucumber';
import assert from 'node:assert/strict';
import { okValue, sleep } from '../support/driver.ts';
import type { ThreadbearWorld } from '../support/world.ts';

/** The demo thread's review (app/Tools/mock-runner/fixtures/demo-design-review). */
const REVIEW = 'review_account_picker';

type ReviewState = {
  id: string;
  status: string;
  mode: string;
  focused: string;
  shot: number;
  pickedIds: string[];
  none: boolean;
  reply: string;
  sendLabel: string;
  editable: boolean;
  actual: boolean;
  peeking: boolean;
  minimap: boolean;
  error: string;
};

async function review(world: ThreadbearWorld, cmd: Record<string, unknown> = {}): Promise<ReviewState> {
  return JSON.parse(okValue(await world.app.do({ cmd: 'review', ...cmd }, 30))) as ReviewState;
}

/** Waits for the open review window to pass `check`, without opening one. */
async function untilReview(world: ThreadbearWorld, what: string, check: (s: ReviewState) => boolean, ms = 15_000): Promise<ReviewState> {
  const deadline = Date.now() + ms;
  let last: ReviewState | string = 'no answer yet';
  for (;;) {
    try {
      last = await review(world, { open: false });
      if (check(last)) return last;
    } catch (e) {
      last = (e as Error).message;
    }
    if (Date.now() > deadline) throw new Error(`Never saw: ${what}\n\nThe review said: ${JSON.stringify(last)}`);
    await sleep(150);
  }
}

/** Waits until no window is open on the review. */
async function untilClosed(world: ThreadbearWorld): Promise<void> {
  const deadline = Date.now() + 15_000;
  for (;;) {
    try {
      await review(world, { open: false });
    } catch {
      return;
    }
    if (Date.now() > deadline) throw new Error('The review window stayed open');
    await sleep(150);
  }
}

/** A control in the review's window, by its tooltip. */
async function press(world: ThreadbearWorld, tip: string): Promise<void> {
  await world.app.do({ cmd: 'click', tip, review: REVIEW });
}

When('I open the review', async function (this: ThreadbearWorld) {
  await review(this);
});

When('I close the review', async function (this: ThreadbearWorld) {
  await this.app.do({ cmd: 'review', close: true });
  await untilClosed(this);
});

Then('the review window is open on option {string}, one at a time', async function (this: ThreadbearWorld, id: string) {
  await untilReview(this, `the review on ${id}, one at a time`, (s) => s.mode === 'Single' && s.focused === id && s.status === 'pending');
});

Then('the bar over the thread says {string}', async function (this: ThreadbearWorld, words: string) {
  await this.until(`the bar over the thread saying "${words}"`, (d) => (d.header as { pendingReviews?: string | null } | undefined)?.pendingReviews === words);
});

When('I press the bar\'s {string}', async function (this: ThreadbearWorld, words: string) {
  await this.until(`the bar saying "${words}"`, (d) => (d.header as { pendingReviews?: string | null } | undefined)?.pendingReviews === words);
  await this.app.do({ cmd: 'click', tip: 'Compare the images the agent made side by side, and reply' });
});

When('I press the review {string} in the bar\'s list', async function (this: ThreadbearWorld, _title: string) {
  // Its list comes from the runner, a moment after the click.
  const deadline = Date.now() + 10_000;
  for (;;) {
    try {
      await this.app.do({ cmd: 'click', tip: 'Open this review' });
      return;
    } catch (e) {
      if (Date.now() > deadline) throw e;
      await sleep(150);
    }
  }
});

When('I tick {string} in the review', async function (this: ThreadbearWorld, id: string) {
  await press(this, `Pick ${id}`);
  await untilReview(this, `${id} ticked`, (s) => s.pickedIds.includes(id));
});

When('I choose None in the review', async function (this: ThreadbearWorld) {
  await press(this, 'None of them will do');
  await untilReview(this, 'none of them chosen', (s) => s.none);
});

When('I write {string} in the review', async function (this: ThreadbearWorld, words: string) {
  await review(this, { open: false, reply: words });
});

When('I press Send in the review', async function (this: ThreadbearWorld) {
  await press(this, 'Send your picks and reply to the agent (⌘↩). Close the window to finish it later.');
});

Then('the review\'s Send says {string}', async function (this: ThreadbearWorld, words: string) {
  await untilReview(this, `Send saying "${words}"`, (s) => s.sendLabel === words);
});

Then('no option is ticked in the review', async function (this: ThreadbearWorld) {
  const s = await untilReview(this, 'no picks', (s) => s.pickedIds.length === 0);
  assert.deepEqual(s.pickedIds, []);
});

Then('{string} has no tick in the review', async function (this: ThreadbearWorld, id: string) {
  await untilReview(this, 'the review open', () => true);
  await assert.rejects(press(this, `Pick ${id}`), `${id} has a tick to press`);
  const s = await review(this, { open: false });
  assert.equal(s.pickedIds.includes(id), false);
});

Then('the review has {string} ticked and the reply {string}', async function (this: ThreadbearWorld, id: string, words: string) {
  await untilReview(this, `${id} ticked with "${words}"`, (s) => s.pickedIds.join(',') === id && s.reply === words);
});

Then('the review window has closed', async function (this: ThreadbearWorld) {
  await untilClosed(this);
});

Then('the review window shows what was sent: {string} and {string}', async function (this: ThreadbearWorld, id: string, words: string) {
  await untilReview(this, `the answered review with ${id} and "${words}"`, (s) => s.status === 'answered' && s.pickedIds.join(',') === id && s.reply === words && s.mode === 'All');
});

Then('nothing can be changed in the review', async function (this: ThreadbearWorld) {
  const s = await review(this, { open: false });
  assert.equal(s.editable, false, 'it can still be changed');
  await assert.rejects(press(this, 'Pick A'), 'a tick is still there to press');
});
