#!/usr/bin/env node
// Makes sure a build folder is this account's alone before a script builds in it or trusts
// what's in it: appstore-upload.sh's OUT (archives it may upload) and the derived data it and
// install-phone.sh build with. These used to be fixed names under /tmp, where another account
// on the Mac could make the folder first and plant an archive the upload then signed and sent
// to App Store Connect (Daybreak, 8 Nov), or build products Xcode would reuse.
//
//   node scripts/private-dir.mjs <dir>...          make each (mode 700) if missing, then check it
//   node scripts/private-dir.mjs --tree <dir>...   check everything inside too; makes nothing
//
// A folder passes when it and every folder above it belong to this account or root, and none
// can be written by anyone else, except a root-owned sticky one like /tmp (where nobody can
// move or delete what's ours). Symlinks on the way are checked the same, and then the real path
// they lead to. Exits 1 with the reason when one doesn't pass.
import { lstatSync, mkdirSync, readdirSync, realpathSync } from 'node:fs';
import { dirname, join, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

const WRITABLE_BY_OTHERS = 0o022;
const STICKY = 0o1000;

// The folders from / down to `dir`.
function chain(dir) {
  const parts = [];
  for (let p = dir; ; p = dirname(p)) {
    parts.unshift(p);
    if (dirname(p) === p) return parts;
  }
}

function lstat(p) {
  try {
    return lstatSync(p);
  } catch (e) {
    if (e.code === 'ENOENT') return undefined;
    throw e;
  }
}

function checkEntry(p, st, uid, last) {
  if (st.uid !== uid && st.uid !== 0) throw new Error(`${p} belongs to another account (uid ${st.uid})`);
  if (st.isSymbolicLink()) return;
  if (!st.isDirectory()) throw new Error(`${p} isn't a folder`);
  const shared = (st.mode & WRITABLE_BY_OTHERS) !== 0;
  // The folder itself must be ours alone; above it, a root-owned sticky folder (/tmp) is fine.
  if (shared && (last || st.uid !== 0 || (st.mode & STICKY) === 0)) {
    throw new Error(`${p} can be written by other accounts (mode ${(st.mode & 0o7777).toString(8)})`);
  }
  if (last && st.uid !== uid) throw new Error(`${p} isn't this account's`);
}

// Checks `dir` and every folder above it, making the missing ones (mode 700) when `make`.
export function checkPrivateDir(dir, { uid = process.getuid(), make = true } = {}) {
  const abs = resolve(dir);
  const parts = chain(abs);
  parts.forEach((p, i) => {
    let st = lstat(p);
    if (!st) {
      if (!make) throw new Error(`${p} isn't there`);
      try {
        mkdirSync(p, { mode: 0o700 });
      } catch (e) {
        if (e.code !== 'EEXIST') throw e; // someone made it first: checked below like any other
      }
      st = lstat(p);
    }
    checkEntry(p, st, uid, i === parts.length - 1);
  });
  // Then where any symlinks above really lead (/tmp is /private/tmp on a Mac).
  const real = realpathSync(abs);
  if (real !== abs) chain(real).forEach((p, i, all) => checkEntry(p, lstat(p), uid, i === all.length - 1));
  return real;
}

// Everything inside `dir` (an archive about to be reused) is this account's and can't be
// written by anyone else, so nothing was added or swapped by another account.
export function checkPrivateTree(dir, { uid = process.getuid() } = {}) {
  const root = checkPrivateDir(dir, { uid, make: false });
  const walk = (p) => {
    for (const name of readdirSync(p)) {
      const child = join(p, name);
      const st = lstatSync(child);
      if (st.uid !== uid) throw new Error(`${child} belongs to another account (uid ${st.uid})`);
      if (!st.isSymbolicLink() && (st.mode & WRITABLE_BY_OTHERS) !== 0) {
        throw new Error(`${child} can be written by other accounts`);
      }
      if (st.isDirectory()) walk(child);
    }
  };
  walk(root);
  return root;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  const tree = args[0] === '--tree';
  const dirs = tree ? args.slice(1) : args;
  if (dirs.length === 0) {
    console.error(`usage: ${process.argv[1].split(sep).pop()} [--tree] <dir>...`);
    process.exit(2);
  }
  try {
    for (const d of dirs) (tree ? checkPrivateTree : checkPrivateDir)(d);
  } catch (e) {
    console.error(`Not using it: ${e.message}`);
    process.exit(1);
  }
}
