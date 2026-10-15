// Stable download page/metadata only. APK bytes are served by GitHub Releases.
const headers={'Cache-Control':'no-store','X-Content-Type-Options':'nosniff'};
const unavailable=request=>new Response(request.method==='HEAD'?null:'Download is not available yet. Please try again later.',{status:503,headers});
const escape=value=>value.replace(/[&<>"']/g,char=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[char]));
export function validRelease(release){
 return release!=null && Object.keys(release).sort().join(',')==='apk_url,package,protocol,sha256,size_bytes,version_code,version_name' && release.protocol===1 && release.package==='app.together.v2' &&
  Number.isInteger(release.version_code) && release.version_code>0 && release.version_code<=2147483647 &&
  typeof release.version_name==='string' && release.version_name.length>=1 && release.version_name.length<=64 && !/[\x00-\x1f\x7f-\x9f]/.test(release.version_name) &&
  Number.isSafeInteger(release.size_bytes) && release.size_bytes>0 && release.size_bytes<=536870912 &&
  typeof release.sha256==='string' && /^[a-f0-9]{64}$/.test(release.sha256) &&
  release.apk_url===`https://github.com/jandev07/2getherstream/releases/download/android-v${release.version_code}/2gether.apk`;
}
export default {async fetch(request,env){
 const path=new URL(request.url).pathname;
 if(!['GET','HEAD'].includes(request.method))return new Response('Method not allowed',{status:405,headers});
 if(!['/','/release.json'].includes(path))return new Response('Not found',{status:404,headers});
 // Never forward authorization/cookies, and never fetch the APK on the Worker.
 let asset;
 try {asset=await env.ASSETS.fetch(new Request(new URL('/release.json',request.url)))}
 catch {return unavailable(request)}
 if(asset.status!==200)return unavailable(request);
 let release;try {release=await asset.json()}catch {return new Response('Release unavailable',{status:503,headers})}
 if(!validRelease(release))return new Response('Release unavailable',{status:503,headers});
 if(path==='/release.json')return new Response(request.method==='HEAD'?null:JSON.stringify(release),{headers:{...headers,'Content-Type':'application/json; charset=utf-8'}});
 const page=`<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>2gether for Android</title><style>body{font-family:system-ui;background:#111018;color:#f7f2ff;max-width:680px;margin:64px auto;padding:24px;line-height:1.6}a{display:inline-block;padding:14px 24px;background:#d0b4ff;color:#20122e;border-radius:30px;text-decoration:none}code{overflow-wrap:anywhere}</style><h1>2gether for Android</h1><p>Watch together with your Windows app.</p><p>Private Alpha ${escape(release.version_name)} · ${(release.size_bytes/1048576).toFixed(1)} MB</p><p><a href="${release.apk_url}" rel="noreferrer">Download Android app</a></p><p>Install over your existing 2gether app to keep your connection settings. Android will ask you to confirm installation.</p><p>Keep the Windows app open for pairing and playback. This download address stays the same when your watching connection restarts.</p><p>This is a test build. Physical-phone and two-hour playback acceptance remain open.</p><p>APK SHA-256: <code>${release.sha256}</code></p></html>`;
 return new Response(request.method==='HEAD'?null:page,{headers:{...headers,'Content-Type':'text/html; charset=utf-8','Content-Security-Policy':"default-src 'none'; style-src 'unsafe-inline'; base-uri 'none'; frame-ancestors 'none'",'Referrer-Policy':'no-referrer'}});
}};
