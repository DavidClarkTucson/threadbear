// Little web sites for the web panel scenarios (features/web-panels.feature),
// on this machine, made before the app starts as its environment names one:
//
// - the panel's own site: a home page with links to a page of its own and to
//   the other site, plain and in new windows;
// - another site (another port is another site, as WebPanelPool.sameSite
//   has it), where a link that leaves the panel goes;
// - a stand-in for Convex's embedded dashboard, which asks for its
//   credentials as the real one does (docs.convex.dev/platform-apis/embedded-dashboard)
//   and says in its title whose it got, never the key itself. The GPUI app
//   is pointed at it with THREADBEAR_CONVEX_EMBEDDED, which only a scripted
//   build reads.
import http from 'node:http';
import type { AddressInfo } from 'node:net';

export type Sites = { home: string; other: string; dashboard: string };

let servers: http.Server[] = [];

function serve(pages: (base: string) => Record<string, string>): Promise<string> {
  return new Promise((resolve, reject) => {
    let base = '';
    const server = http.createServer((req, res) => {
      const page = pages(base)[(req.url ?? '/').split('?')[0]];
      res.writeHead(page === undefined ? 404 : 200, { 'content-type': 'text/html; charset=utf-8', 'cache-control': 'no-store' });
      res.end(page ?? '<title>Not here</title>');
    });
    server.on('error', reject);
    server.listen(0, '127.0.0.1', () => {
      base = `http://127.0.0.1:${(server.address() as AddressInfo).port}`;
      servers.push(server);
      resolve(base);
    });
  });
}

export async function startSites(): Promise<Sites> {
  const other = await serve(() => ({ '/video': '<!doctype html><title>Video</title><p>A video on another site.</p>', '/docs': '<!doctype html><title>Docs</title>' }));
  const home = await serve((base) => ({
    '/': `<!doctype html><title>Panel home</title><body style="font:14px -apple-system,system-ui;padding:16px">
<p><a id="other" href="${other}/video">A video on another site</a></p>
<p><a id="next" href="${base}/second">Another of this site's pages</a></p>
<p><a id="next-window" target="_blank" href="${base}/second">The same, in a new window</a></p>
<p><a id="other-window" target="_blank" href="${other}/video">The video, in a new window</a></p>`,
    '/second': '<!doctype html><title>Second page</title><p>The site\'s second page.</p>',
  }));
  const dashboard = await serve(() => ({
    '/': `<!doctype html><title>Waiting for credentials</title><body style="font:14px -apple-system,system-ui;padding:16px">
<p><a id="docs" href="${other}/docs">Convex's docs</a></p>
<script>
addEventListener('message', (e) => {
  if (e.data && e.data.type === 'dashboard-credentials') {
    document.title = 'Credentials for ' + e.data.deploymentName + (e.data.adminKey ? ' with a key' : ' with no key');
  }
});
postMessage({ type: 'dashboard-credentials-request' }, location.origin);
</script>`,
  }));
  return { home: `${home}/`, other, dashboard: `${dashboard}/` };
}

export function stopSites() {
  for (const server of servers.splice(0)) server.close();
}
