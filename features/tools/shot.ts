// A picture of one thread in whichever app TB_APP says, for comparing them.
//
//   TB_APP=gpui node features/tools/shot.ts <thread title> <out.png> [JSON command]...
//
// Commands after the file name run before the picture (the same JSON the
// apps' automation takes), such as {"cmd":"collapse","section":"inbox"} to
// bring a section into view or {"cmd":"hover","id":"..."} to show a row's
// buttons. "{id}" in one is the opened thread's id.
import { GpuiDriver } from '../support/gpui.ts';
import { SwiftDriver } from '../support/swift.ts';
import { sleep, type Driver } from '../support/driver.ts';

const [title, out, ...commands] = process.argv.slice(2);
const app: Driver = (process.env.TB_APP ?? 'gpui') === 'swift' ? new SwiftDriver() : new GpuiDriver();
await app.start();
try {
  let id = '';
  for (let i = 0; i < 300 && !id; i += 1) {
    const d = await app.dump();
    id = d.sidebar.sections.flatMap((s) => s.threads).find((t) => t.title === title)?.id ?? '';
    if (!id) await sleep(50);
  }
  await app.do({ cmd: 'open', id });
  for (let i = 0; i < 200; i += 1) {
    const d = await app.dump();
    if (d.transcript.state === 'loaded' && d.selection?.id === id) break;
    await sleep(50);
  }
  for (const command of commands) {
    await app.do(JSON.parse(command.replaceAll('{id}', id)) as Record<string, unknown>);
    await sleep(200);
  }
  await sleep(1200);
  await app.shot(out);
  console.log(`saved ${out}`);
} finally {
  await app.stop();
}
