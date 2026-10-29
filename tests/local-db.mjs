import EmbeddedPostgres from 'embedded-postgres';
import {mkdtemp} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
// Unique disposable data directory. Never touches the user's configured DATABASE_URL.
const dir=await mkdtemp(join(tmpdir(),'support-os-test-'));
const pg=new EmbeddedPostgres({databaseDir:dir,user:'postgres',password:'local-test-only',port:55432,persistent:true,onLog:()=>{},onError:message=>process.stderr.write(String(message)+'\n')});
await pg.initialise();await pg.start();await pg.createDatabase('support_os_test');
console.log('TEST_DATABASE_URL=postgres://postgres:local-test-only@127.0.0.1:55432/support_os_test');
console.log('Disposable Postgres running; Ctrl+C to stop. Directory: '+dir);
const stop=async()=>{await pg.stop();process.exit(0);};process.on('SIGINT',stop);process.on('SIGTERM',stop);
setInterval(()=>{},30000);
