// site/scripts/serve.mjs, the landing page's local preview: a bad request ends that request,
// never the server (Daybreak, 8 Nov: `Range: bytes=1-0` crashed it), and only this Mac can reach it.
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { request } from 'node:http';
import { networkInterfaces, tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { after, before, test } from 'node:test';
import { fileURLToPath } from 'node:url';
// @ts-expect-error plain .mjs without types
import { byteRange, createSiteServer } from '../../../site/scripts/serve.mjs';

const dist = mkdtempSync(join(tmpdir(), 'tb-site-')) + '/';
writeFileSync(join(dist, 'index.html'), '0123456789');
writeFileSync(join(dist, '404.html'), 'missing');
writeFileSync(join(dist, 'empty.txt'), '');
mkdirSync(join(dist, 'docs'));
const server = createSiteServer(dist);
let port = 0;
before(() => new Promise<void>((done) => server.listen(0, '127.0.0.1', () => ((port = server.address().port), done()))));
after(() => {
  server.close();
  rmSync(dist, { recursive: true, force: true });
});

function get(path: string, headers: Record<string, string> = {}, method = 'GET') {
  return new Promise<{ status: number; headers: Record<string, unknown>; body: string }>((done, fail) => {
    const req = request({ host: '127.0.0.1', port, path, method, headers }, (res) => {
      let body = '';
      res.setEncoding('utf8');
      res.on('data', (c) => (body += c));
      res.on('end', () => done({ status: res.statusCode!, headers: res.headers, body }));
    });
    req.on('error', fail);
    req.end();
  });
}

test('reads byte ranges the way Cloudflare does', () => {
  assert.deepEqual(byteRange('bytes=0-3', 10), [0, 3]);
  assert.deepEqual(byteRange('bytes=4-', 10), [4, 9]);
  assert.deepEqual(byteRange('bytes=2-99', 10), [2, 9]);
  assert.deepEqual(byteRange('bytes=-3', 10), [7, 9]);
  assert.deepEqual(byteRange('bytes=-99', 10), [0, 9]);
  assert.deepEqual(byteRange('bytes=0-99999999999999999999999', 10), [0, 9]);
  for (const bad of ['bytes=1-0', 'bytes=10-', 'bytes=99999999999999999999-', 'bytes=-0']) {
    assert.equal(byteRange(bad, 10), 'unsatisfiable', bad);
  }
  assert.equal(byteRange('bytes=0-', 0), 'unsatisfiable');
  assert.equal(byteRange('bytes=-1', 0), 'unsatisfiable');
  for (const ignored of [undefined, '', 'bytes=-', 'bytes=0-1,3-4', 'items=0-1']) assert.equal(byteRange(ignored, 10), undefined);
});

test('a range it can serve gets a 206 with just those bytes', async () => {
  const r = await get('/', { Range: 'bytes=2-4' });
  assert.equal(r.status, 206);
  assert.equal(r.body, '234');
  assert.equal(r.headers['content-range'], 'bytes 2-4/10');
  assert.equal((await get('/', { Range: 'bytes=-99' })).body, '0123456789');
});

test('a range outside the file gets a 416, and the server carries on', async () => {
  for (const range of ['bytes=1-0', 'bytes=10-', 'bytes=-0', 'bytes=99999999999999999999-']) {
    const r = await get('/', { Range: range });
    assert.equal(r.status, 416, range);
    assert.equal(r.headers['content-range'], 'bytes */10');
  }
  assert.equal((await get('/', { Range: 'bytes=0-' }, 'HEAD')).status, 206);
  assert.equal((await get('/empty.txt', { Range: 'bytes=0-' })).status, 416);
  assert.equal((await get('/empty.txt')).status, 200);
  assert.equal((await get('/')).body, '0123456789');
});

test('an address that does not decode gets a 400, and the server carries on', async () => {
  assert.equal((await get('/%E0%A4%A')).status, 400);
  assert.equal((await get('/%')).status, 400);
  assert.equal((await get('/')).status, 200);
});

test('anything else that goes wrong is a 500 for that request only', async () => {
  rmSync(join(dist, '404.html'));
  assert.equal((await get('/nowhere')).status, 500);
  writeFileSync(join(dist, '404.html'), 'missing');
  assert.equal((await get('/nowhere')).status, 404);
});

test('npm run serve only listens on this Mac', async (t) => {
  const others = Object.values(networkInterfaces())
    .flat()
    .filter((a) => a && a.family === 'IPv4' && !a.internal)
    .map((a) => a!.address);
  const script = resolve(fileURLToPath(import.meta.url), '../../../../site/scripts/serve.mjs');
  // A free port to give it.
  const free = await new Promise<number>((done) => {
    const probe = createSiteServer(dist).listen(0, '127.0.0.1', () => {
      const p = probe.address().port;
      probe.close(() => done(p));
    });
  });
  const env: NodeJS.ProcessEnv = { ...process.env, PORT: String(free) };
  delete env.HOST;
  const serve = spawn(process.execPath, [script], { env, stdio: 'pipe' });
  t.after(() => serve.kill());
  await new Promise<void>((done) => serve.stdout.once('data', () => done()));
  const reach = (host: string) =>
    new Promise<boolean>((done) => {
      const req = request({ host, port: free, path: '/', timeout: 500 }, (res) => (res.resume(), done(true)));
      req.on('error', () => done(false));
      req.on('timeout', () => (req.destroy(), done(false)));
      req.end();
    });
  assert.equal(await reach('127.0.0.1'), true);
  for (const address of others) assert.equal(await reach(address), false, address);
});
