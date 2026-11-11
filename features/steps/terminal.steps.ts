// The Terminal panel (terminal.feature). Both apps take the `terminal` command
// and say what the panel shows in the dump's `terminal`; the GPUI app's `keys`
// and `press` go through its keyboard and input method, the Swift app's to the
// shell as SwiftTerm's `send` does.
import { After, Given, Then, When } from '@cucumber/cucumber';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { okValue, type Dump } from '../support/driver.ts';
import type { ThreadbearWorld } from '../support/world.ts';

/** Each scenario's folders, by thread id, the threads' ids by title, and what it noted. */
const scenario = new WeakMap<ThreadbearWorld, { folders: Map<string, string>; ids: Map<string, string>; width?: number; cols?: number }>();

function state(world: ThreadbearWorld) {
  let s = scenario.get(world);
  if (!s) {
    s = { folders: new Map(), ids: new Map() };
    scenario.set(world, s);
  }
  return s;
}

/** The folder the open thread was given. */
function openFolder(world: ThreadbearWorld, d: Dump): string {
  const folder = d.selection ? state(world).folders.get(d.selection.id) : undefined;
  assert.ok(folder, 'the open thread has no folder of its own');
  return folder;
}

function screen(d: Dump): string[] {
  return d.terminal?.screen ?? [];
}

/** `until`, saying what the Terminal's screen had when it never came. */
async function untilScreen(world: ThreadbearWorld, what: string, check: (lines: string[], d: Dump) => boolean): Promise<void> {
  try {
    await world.until(what, (d) => check(screen(d), d));
  } catch (e) {
    const d = await world.app.dump();
    throw new Error(`${(e as Error).message.split('\n')[0]}\n\nThe Terminal (${d.terminal?.cols}x${d.terminal?.rows}, ${d.terminal?.shell} in ${d.terminal?.cwd}) showed:\n${screen(d).join('\n')}`);
  }
}

After(function (this: ThreadbearWorld) {
  for (const folder of state(this).folders.values()) fs.rmSync(folder, { recursive: true, force: true });
});

// A real folder, as the demo threads' are Mike's and on no other machine. A
// temp one, made for the scenario, so the shell never starts in anyone's.
Given('the thread {string} works in a folder of its own', async function (this: ThreadbearWorld, title: string) {
  const thread = await this.until(`a thread called "${title}"`, (d) => this.thread(d, title));
  const folder = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), 'tb-terminal-')));
  state(this).folders.set(thread.id, folder);
  state(this).ids.set(title, thread.id);
  await this.app.do({ cmd: 'mock', method: 'terminal', params: { threadId: thread.id, cwd: folder } });
});

When('I press ⌘J', async function (this: ThreadbearWorld) {
  await this.app.do({ cmd: 'menu-key', key: 'j', modifiers: ['cmd'] });
});

// The empty panel and New Shell, by their tooltips' start, as each app's
// tip names the folder.
When('I start a shell from the empty Terminal', async function (this: ThreadbearWorld) {
  await this.app.do({ cmd: 'click', tipPrefix: 'Start your login shell in ' });
});

When('I press New Shell in the Terminal', async function (this: ThreadbearWorld) {
  await this.app.do({ cmd: 'click', tipPrefix: 'Start a new shell in ' });
});

When('I bring up the Terminal without starting a shell', async function (this: ThreadbearWorld) {
  await this.app.do({ cmd: 'terminal', show: true });
  await this.until('the Terminal on show', (d) => d.terminal?.shown === true);
});

When('I type {string} and press Return in the Terminal', async function (this: ThreadbearWorld, text: string) {
  // As the feature file has it: `\033` and `\n` stay as the shell's printf reads them.
  await this.until('a shell running', (d) => d.terminal?.shell === 'running');
  await this.app.do({ cmd: 'terminal', keys: text });
  await this.app.do({ cmd: 'terminal', press: 'enter' });
});

When('I scroll the Terminal back {int} lines', async function (this: ThreadbearWorld, lines: number) {
  await this.app.do({ cmd: 'terminal', scroll: -lines });
});

When('I select everything in the Terminal and copy it', async function (this: ThreadbearWorld) {
  await this.app.do({ cmd: 'terminal', select: 'all' });
  await this.app.do({ cmd: 'terminal', copy: true });
});

When('I paste {string} into the Terminal and press Return', async function (this: ThreadbearWorld, text: string) {
  await this.app.do({ cmd: 'terminal', paste: text });
  await this.app.do({ cmd: 'terminal', press: 'enter' });
});

When('I note how many columns the Terminal has', async function (this: ThreadbearWorld) {
  // From a known width, so "wider" is the same step on both apps.
  state(this).width = 1000;
  await this.app.do({ cmd: 'resize', w: 1000, h: 800 });
  const cols = await this.until('the Terminal drawn at 1000 wide', (d) => d.terminal?.cols ?? undefined);
  // A frame or two for the grid to follow the window.
  await new Promise((done) => setTimeout(done, 400));
  state(this).cols = (await this.app.dump()).terminal?.cols ?? cols;
});

When('the window is made {int} points wider', async function (this: ThreadbearWorld, more: number) {
  const width = (state(this).width ?? 1000) + more;
  await this.app.do({ cmd: 'resize', w: width, h: 800 });
});

Then('the Terminal has more columns than before', async function (this: ThreadbearWorld) {
  const before = state(this).cols ?? 0;
  await this.until(`more than ${before} columns`, (d) => (d.terminal?.cols ?? 0) > before);
});

Then('the Terminal shows its own number of columns', async function (this: ThreadbearWorld) {
  // `tput cols` asks the pty, so this is the shell hearing the new size.
  await untilScreen(this, "tput cols answering the grid's width", (lines, d) => d.terminal?.cols != null && lines.includes(String(d.terminal.cols)));
});

Then("the Terminal is showing a shell in the thread's folder", async function (this: ThreadbearWorld) {
  const d0 = await this.app.dump();
  const folder = openFolder(this, d0);
  await this.until(`a shell running in ${folder}`, (d) => d.terminal?.shown && d.terminal.shell === 'running' && d.terminal.cwd === folder && !d.terminal.note);
});

Then('the Terminal has the keyboard', async function (this: ThreadbearWorld) {
  await this.until('the Terminal focused', (d) => d.terminal?.focused === true);
});

Then("the Terminal shows the thread's folder", async function (this: ThreadbearWorld) {
  const folder = openFolder(this, await this.app.dump());
  // A long folder wraps onto the next line, so the screen is read whole.
  await untilScreen(this, `the folder "${folder}"`, (lines) => lines.join('').includes(folder));
});

Then('the Terminal shows the line {string}', async function (this: ThreadbearWorld, line: string) {
  await untilScreen(this, `the line "${line}"`, (lines) => lines.includes(line));
});

Then('the Terminal shows earlier lines and not the line {string}', async function (this: ThreadbearWorld, line: string) {
  await untilScreen(this, `earlier output, without "${line}"`, (lines) => !lines.includes(line) && lines.some((l) => /^\d+$/.test(l) && Number(l) < 350));
});

Then('the clipboard holds {string} among other lines', async function (this: ThreadbearWorld, words: string) {
  // The copy lands a moment after the key, on both apps.
  let text = '';
  for (let i = 0; i < 30; i++) {
    text = JSON.parse(okValue(await this.app.do({ cmd: 'clipboard' }))) as string;
    if (text.split('\n').some((l) => l.trim() === words)) return;
    await new Promise((done) => setTimeout(done, 100));
  }
  assert.fail(`the clipboard had ${JSON.stringify(text)}`);
});

Then('the Terminal is put away', async function (this: ThreadbearWorld) {
  await this.until('the Terminal hidden', (d) => d.terminal?.shown === false);
});

Then("the thread's shell is still running", async function (this: ThreadbearWorld) {
  const d0 = await this.app.dump();
  const id = d0.selection?.id;
  await this.until("the thread's shell running", (d) => d.terminal?.shells.some((s) => s.startsWith(`thread:${id} running`)));
});

Then('the shell of {string} has stopped', async function (this: ThreadbearWorld, title: string) {
  const id = state(this).ids.get(title);
  assert.ok(id, `"${title}" wasn't given a folder`);
  await this.until(`no shell for "${title}"`, (d) => !d.terminal?.shells.some((s) => s.startsWith(`thread:${id} `)));
});

Then('the Terminal says {string}', async function (this: ThreadbearWorld, note: string) {
  await this.until(`the note "${note}"`, (d) => d.terminal?.note === note);
});

Then('the Terminal offers to start a shell', async function (this: ThreadbearWorld) {
  await this.until('no shell yet, with Start a shell', (d) => d.terminal?.shown === true && d.terminal.shell === 'none');
});

Then('no shell is running', async function (this: ThreadbearWorld) {
  const d = await this.app.dump();
  assert.deepEqual(d.terminal?.shells ?? [], []);
});

Then('{int} shell(s) is/are running', async function (this: ThreadbearWorld, count: number) {
  await this.until(`${count} shells`, (d) => d.terminal?.shells.filter((s) => s.includes(' running in ')).length === count);
});

Then('the Terminal says the shell has exited', async function (this: ThreadbearWorld) {
  await this.until('the shell exited', (d) => d.terminal?.shell === 'exited');
});
