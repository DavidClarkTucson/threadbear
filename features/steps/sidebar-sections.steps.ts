import { Given, Then, When } from '@cucumber/cucumber';
import type { Dump } from '../support/driver.ts';
import type { ThreadbearWorld } from '../support/world.ts';

const section = (d: Dump, name: string) => d.sidebar.sections.find((s) => s.name === name);

Then('the {string} section isn\'t folded', async function (this: ThreadbearWorld, name: string) {
  await this.until(`${name} open`, (d) => section(d, name)?.collapsed === false);
});

Then('the {string} heading says {string}', async function (this: ThreadbearWorld, name: string, count: string) {
  await this.until(`${name} heading saying ${count}`, (d) => String(section(d, name)?.count ?? '') === count);
});

Then('the {string} heading says nothing', async function (this: ThreadbearWorld, name: string) {
  await this.until(`${name} heading with no count`, (d) => section(d, name) && (section(d, name)?.count ?? null) === null);
});

// As a person would: the heading's button, whose tooltip says what it'll do.
When('I click the {string} heading', async function (this: ThreadbearWorld, name: string) {
  const d = await this.app.dump();
  await this.app.do({ cmd: 'click', tip: `${section(d, name)?.collapsed ? 'Show' : 'Hide'} ${name}` });
});

Given('the Inbox is sorted by what needs me first', async function (this: ThreadbearWorld) {
  await this.until('the default sort', (d) => d.sidebar.sort === 'needsYou');
});

When('I sort the Inbox oldest first', async function (this: ThreadbearWorld) {
  await this.app.do({ cmd: 'sort', order: 'oldest' });
});

Then('the Inbox is sorted oldest first', async function (this: ThreadbearWorld) {
  await this.until('the oldest-first sort', (d) => d.sidebar.sort === 'oldest');
});
