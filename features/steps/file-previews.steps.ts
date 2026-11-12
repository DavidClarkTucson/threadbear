import { Then, When } from '@cucumber/cucumber';
import assert from 'node:assert/strict';
import type { ThreadbearWorld } from '../support/world.ts';
import { pane, previewLine } from '../support/pane.ts';

When('I click the highlight {string}', async function (this: ThreadbearWorld, title: string) {
  await this.until(`a highlight called "${title}"`, (d) => pane(d)?.sections?.some((s) => s.rows.some((r) => r.title === title)));
  await this.app.do({ cmd: 'highlight', open: title });
});

When('I open {string} in the thread\'s folder in the pane', async function (this: ThreadbearWorld, relative: string) {
  const d = await this.until('the composer to know its folder', (x) => (x.composer as unknown as { cwd?: string }).cwd && x);
  const cwd = (d.composer as unknown as { cwd: string }).cwd.replace(/\/$/, '');
  await this.app.do({ cmd: 'preview', path: `${cwd}/${relative}` });
});

Then('the pane is showing the file {string}', async function (this: ThreadbearWorld, relative: string) {
  await this.until(`the preview line to name ${relative}`, (d) => previewLine(d)?.startsWith('file ') && previewLine(d)?.endsWith(relative));
});

When('I press Back in the pane', async function (this: ThreadbearWorld) {
  await this.app.do({ cmd: 'preview', back: true });
});

Then('the pane shows Highlights', async function (this: ThreadbearWorld) {
  await this.until('Highlights to be showing, with no preview over it', (d) => !previewLine(d) && pane(d)?.section === 'threadbear.highlights');
});

Then('the pane shows a {string} page with the words {string}', async function (this: ThreadbearWorld, kind: string, words: string) {
  await this.until(`a ${kind} preview with "${words}"`, (d) => pane(d)?.previewKind === kind && pane(d)?.previewText?.includes(words));
});

Then('the pane\'s page has the words {string}', async function (this: ThreadbearWorld, words: string) {
  await this.until(`the page to have "${words}"`, (d) => pane(d)?.previewText?.includes(words));
});

Then('the pane\'s Markdown menu says {string}', async function (this: ThreadbearWorld, mode: string) {
  await this.until(`the bar's Markdown menu on ${mode}`, (d) => pane(d)?.previewMode === mode);
});

When('I pick {string} from the pane\'s Markdown menu', async function (this: ThreadbearWorld, view: string) {
  await this.until('the bar\'s Markdown menu', (d) => pane(d)?.previewMode);
  await this.app.do({ cmd: 'preview', markdownView: view });
});

When('I look for {string} in the pane\'s file', async function (this: ThreadbearWorld, words: string) {
  await this.until('a text file in the pane', (d) => ['markdown', 'code', 'text'].includes(pane(d)?.previewKind ?? '') && pane(d)?.previewText);
  await this.app.do({ cmd: 'preview', find: words });
});

Then('the find bar says {string}', async function (this: ThreadbearWorld, line: string) {
  await this.until(`the find bar to say "${line}"`, (d) => pane(d)?.previewFind === line);
});

When('I find the next match', async function (this: ThreadbearWorld) {
  await this.app.do({ cmd: 'preview', findNext: true });
});

When('I find the match before', async function (this: ThreadbearWorld) {
  await this.app.do({ cmd: 'preview', findPrevious: true });
});

When('I close the find bar', async function (this: ThreadbearWorld) {
  await this.app.do({ cmd: 'preview', findDone: true });
});

Then('the find bar is gone', async function (this: ThreadbearWorld) {
  await this.until('no find bar', (d) => pane(d)?.previewKind && !pane(d)?.previewFind);
});

Then('the pane shows a picture', async function (this: ThreadbearWorld) {
  await this.until('a picture in the pane', (d) => pane(d)?.previewKind === 'picture' && /\d+x\d+/.test(pane(d)?.previewText ?? ''));
});

// Both apps say "video" for the player in the pane, with the film's size
// once its track has said it.
Then('the pane shows a video {int} by {int}', async function (this: ThreadbearWorld, width: number, height: number) {
  await this.until(`a ${width}x${height} video in the pane`, (d) => pane(d)?.previewKind === 'video' && pane(d)?.previewText === `${width}x${height}`);
});

// The Swift app says "quickLook" for whatever Quick Look draws, so it's that
// there; the GPUI app's PDFKit view says "pdf" and its pages.
Then('the pane shows the PDF', async function (this: ThreadbearWorld) {
  await this.until('a PDF in the pane', (d) => pane(d)?.previewKind === 'quickLook' || (pane(d)?.previewKind === 'pdf' && pane(d)?.previewText === '1 page'));
});

When('I copy the picture in the pane', async function (this: ThreadbearWorld) {
  this.lastAnswer = await this.app.do({ cmd: 'preview', copyImage: true });
});

Then('it was copied as a picture', function (this: ThreadbearWorld) {
  assert.match(this.lastAnswer ?? '', /copied (png|public\.png)/);
});
