// A picture of one thread in whichever app TB_APP says, for comparing them.
//
//   TB_APP=gpui node features/tools/shot.ts <thread title> <out.png> ['<steps>']
//
// <steps> is a JSON list of commands to run first, each as the app's `do`
// takes it (`{"cmd":"expand","row":"g:"}`, `{"sleep":500}`): for showing a row
// opened or a card pressed.
import { GpuiDriver } from '../support/gpui.ts';
import { SwiftDriver } from '../support/swift.ts';
import { sleep, type Driver } from '../support/driver.ts';

const [title, out, stepsJson] = process.argv.slice(2);
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
  await sleep(1200);
  for (const step of JSON.parse(stepsJson || '[]') as Record<string, unknown>[]) {
    if (typeof step.sleep === 'number') await sleep(step.sleep);
    else console.log(await app.do(step));
  }
  await sleep(500);
  await app.shot(out);
  console.log(`saved ${out}`);
} finally {
  await app.stop();
}
