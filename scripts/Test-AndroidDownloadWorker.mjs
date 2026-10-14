import assert from 'node:assert/strict';
import test from 'node:test';
import worker,{validRelease} from '../cloud/android-download/worker.mjs';
const release=()=>({protocol:1,package:'app.together.v2',version_code:9,version_name:'0.1.8-private-alpha',size_bytes:5,sha256:'a'.repeat(64),apk_url:'https://github.com/jandev07/2getherstream/releases/download/android-v9/2gether.apk'});
function host(change=()=>{},status=200){
 const data=release();change(data);const calls=[];
 return {data,calls,run:(path='/',options={})=>worker.fetch(new Request('https://download.synthetic.invalid'+path,options),{ASSETS:{fetch:async request=>{calls.push(request);return status===200?Response.json(data):new Response(null,{status})}}})};
}
test('stable page points directly to exact GitHub asset, without streaming or forwarding credentials',async()=>{
 const h=host();const response=await h.run('/',{headers:{Authorization:'Bearer synthetic',Cookie:'private'}});
 assert.equal(response.status,200);assert.ok((await response.text()).includes(h.data.apk_url));
 assert.equal(h.calls.length,1);assert.equal(new URL(h.calls[0].url).pathname,'/release.json');
 assert.equal(h.calls[0].headers.get('Authorization'),null);assert.equal(h.calls[0].headers.get('Cookie'),null);
 assert.equal(response.headers.get('Cache-Control'),'no-store');assert.equal(response.headers.get('Referrer-Policy'),'no-referrer');
});
test('metadata and page HEAD return no body and metadata GET retains exact typed identity',async()=>{
 for(const path of ['/','/release.json']){const h=host();const response=await h.run(path,{method:'HEAD'});assert.equal(response.status,200);assert.equal(response.body,null)}
 const h=host();assert.deepEqual(await (await h.run('/release.json')).json(),h.data);
});
test('page escapes release labels and supplies restrictive page policy',async()=>{
 const h=host(p=>p.version_name='<script>bad</script>');const response=await h.run();const page=await response.text();
 assert.ok(!page.includes('<script>'));assert.ok(page.includes('&lt;script&gt;'));assert.match(response.headers.get('Content-Security-Policy'),/frame-ancestors 'none'/);
});
test('unpublished release fails with fixed unavailable response',async()=>{
 const h=host(()=>{},404);assert.equal((await h.run()).status,503);
});
test('unknown APK/internal/control paths and write methods are not served',async()=>{
 const h=host();for(const path of ['/releases/9/2gether.apk','/session','/pair','/index.html'])assert.equal((await h.run(path)).status,404);
 assert.equal((await h.run('/',{method:'POST',body:'synthetic'})).status,405);assert.equal(h.calls.length,0);
});
test('invalid/coerced/oversized/expiring/foreign manifests fail closed',async()=>{
 for(const change of [p=>p.protocol=true,p=>p.version_code='9',p=>p.version_code=2147483648,p=>p.size_bytes=536870913,p=>p.sha256='wrong',p=>p.package='legacy.app',p=>p.version_name='bad\nname',p=>p.apk_url+='?expires=1',p=>p.apk_url='https://evil.invalid/app.apk',p=>p.apk_url=p.apk_url.replace('android-v9','android-v8')]){
  const h=host(change);assert.equal((await h.run()).status,503);
 }
 assert.ok(validRelease(release()));
});
