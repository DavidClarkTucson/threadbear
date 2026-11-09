// A fixture folder for the long-thread scenarios: "A very long thread" (longer
// than one page of events, so paging has somewhere to go) and one short demo
// thread to go to and come back from. Made once per run.
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
let folder: string | undefined;

export function longThreadFolder(): string {
  if (folder) return folder;
  const dir = fs.mkdtempSync(path.join(fs.realpathSync(os.tmpdir()), 'tb-long-'));
  // 250 turns, about 1,750 events: three pages' worth.
  execFileSync(process.execPath, [path.join(root, 'desktop/scripts/make-long-thread.mjs'), dir, '250'], { stdio: 'ignore' });
  fs.cpSync(path.join(root, 'app/Tools/mock-runner/fixtures/demo-dark-mode'), path.join(dir, 'demo-dark-mode'), { recursive: true });
  folder = dir;
  return dir;
}
