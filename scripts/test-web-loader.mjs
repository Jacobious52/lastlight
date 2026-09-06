import {readFileSync} from 'node:fs';
import vm from 'node:vm';
import test from 'node:test';
import assert from 'node:assert/strict';

const source=readFileSync(new URL('../web/index.html',import.meta.url),'utf8').match(/<script>([\s\S]*?)<\/script>/)[1];
function loader(fetch) {
  const element={setAttribute(){},addEventListener(){},focus(){}};
  const context={URL,Request,setTimeout:fn=>fn(),location:{href:'https://example.com/lastlight/'},
    document:{getElementById:()=>element},window:{fetch,addEventListener(){}}};
  vm.runInNewContext(source,context);
  return context.window.fetch;
}

test('asset requests recover from network failure and temporary server errors',async()=>{
  const calls=[];
  const fetch=loader(async resource=>{
    calls.push(String(resource));
    if(calls.length===1) throw new TypeError('Failed to fetch');
    return new Response('data',{status:calls.length===2?503:200});
  });
  const result=await fetch('./assets/art/terrain.png');
  assert.equal(result.status,200);
  assert.equal(calls.length,3);
  assert.ok(calls.every(url=>url.includes('/lastlight/assets/art/terrain.png?build=')));
});

test('permanent failure is bounded and missing assets are not retried',async()=>{
  for(const [status,count] of [[503,3],[404,1]]) {
    let calls=0;
    const fetch=loader(async()=>{calls++;return new Response('',{status});});
    assert.equal((await fetch('./assets/missing.png')).status,status);
    assert.equal(calls,count);
  }
});

test('aborts, writes and external requests are never replayed',async()=>{
  for(const [url,options,error] of [
    ['./assets/art/terrain.png',{},new DOMException('Cancelled','AbortError')],
    ['./assets/art/terrain.png',{method:'POST'},new TypeError('Disconnected')],
    ['https://elsewhere.example/image.png',{},new TypeError('Disconnected')],
  ]) {
    let calls=0;
    const fetch=loader(async()=>{calls++;throw error;});
    await assert.rejects(()=>fetch(url,options));
    assert.equal(calls,1);
  }
});

test('versioned Request inputs preserve their headers and signal',async()=>{
  const abort=new AbortController();
  let observed;
  const fetch=loader(async request=>{observed=request;return new Response('ok');});
  await fetch(new Request('https://example.com/lastlight/assets/art/terrain.png',{
    headers:{'X-Loader-Test':'preserved'},signal:abort.signal,
  }));
  assert.equal(observed.headers.get('X-Loader-Test'),'preserved');
  assert.ok(observed.url.includes('?build='));
  abort.abort();
  assert.equal(observed.signal.aborted,true);
});
