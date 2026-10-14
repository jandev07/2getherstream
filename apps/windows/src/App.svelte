<script lang="ts">
import {invoke} from '@tauri-apps/api/core';
import {onMount} from 'svelte';
type Player={initialized:boolean;state:string;position_ms:number;duration_ms:number;buffering:boolean;error:string|null;generation:number;drift_ms:number;rate:number};
type Snapshot={media:{provider:string}|null;ended:boolean;prepare:{ready:string[]}|null;participants:Record<string,{connected:boolean;buffering:boolean;rtt_ms:number}>;fault:string|null};
let player=$state<Player>({initialized:false,state:'Starting player',position_ms:0,duration_ms:0,buffering:false,error:null,generation:0,drift_ms:0,rate:1});
let session=$state<Snapshot|null>(null);
let error=$state(''); let surface:HTMLDivElement;
let pairingCode=$state('');let pairingBusy=$state(false);let pairingOpened=$state(0);
let page=$state<'discover'|'watch'|'connection'>('discover');
type Connection={error?:string|null;emulator_address:string;phone_address:string|null;tunnel:{mode:string;process?:{state:string}}};
type AndroidDownload={url:string;qr_svg:string};
let androidDownload=$state<AndroidDownload|null>(null);let downloadError=$state('');
async function openAndroidDownload(){try{await invoke('open_android_download');downloadError=''}catch{downloadError='Could not open the download page. Scan the code or copy the address below.'}}
let sourceNoticeError=$state('');
async function openSourceNotices(){try{await invoke('open_source_notices');sourceNoticeError=''}catch{sourceNoticeError='Could not open the source and licenses page. Visit github.com/jandev07/2getherstream.'}}
let connection=$state<Connection|null>(null);let connectionBusy=$state(false);let connectionPolled=0;let connectionRequest=0;
let connectionRefresh:Promise<void>|null=null;let connectionRefreshAgain=false;
let visibleKind=$state('all');
const phoneConnected=()=>Boolean(session?.participants.phone?.connected);
async function refreshConnection(){
 ++connectionRequest;
 if(connectionRefresh){connectionRefreshAgain=true;return connectionRefresh}
 const refresh=(async()=>{
  try {
   do {
    connectionRefreshAgain=false;const request=connectionRequest;
    try {const next=await invoke<Connection>('pairing_connection');if(request===connectionRequest)connection=next}
    catch(problem){if(request===connectionRequest)throw problem}
   } while(connectionRefreshAgain);
  } finally {connectionRefresh=null}
 })();
 connectionRefresh=refresh;
 try {await refresh}finally{if(connectionRefresh===refresh)connectionRefresh=null}
}
async function startPhoneConnection(){if(connectionBusy || pairingBusy)return;connectionBusy=true;try{await invoke('start_temporary_connection');await refreshConnection();error=''}catch{error='Could not start the phone connection. Check your internet connection and try again.'}finally{connectionBusy=false}}
async function stopPhoneConnection(){if(connectionBusy || pairingBusy)return;connectionBusy=true;try{await invoke('stop_temporary_connection');pairingCode='';await invoke('cancel_pairing');await refreshConnection();error=''}catch{error='Could not stop the temporary connection.'}finally{connectionBusy=false}}

type Video={torrent_id:number;file_id:number;filename?:string;filesize?:number};
let videos=$state<Video[]>([]);let torboxBusy=$state(false);let listed=$state(false);let selectedTitle=$state('');
let accountLinks=$state(false);
type Title={id:string;kind:string;name:string};
type Catalog={name:string;kind:string;searchable:boolean;search_required:boolean};
type Browser={connected:boolean;revision:number;catalogs:Catalog[];titles:Title[];episode_mode:boolean};
type Ready={index:number;filename:string;filesize:number|null};
type Options={revision:number;candidates:number;unavailable_providers:number;cache_available:boolean;ready:Ready[];cached:Ready[]};
let browser=$state<Browser|null>(null);let options=$state<Options|null>(null);let stremioBusy=$state(false);let stremioCancelable=$state(false);let stremioRequest=0;
let catalogIndex=$state('0');let search=$state('');let movieName=$state('');
let copiedLink=$state('');
let prepared=$state<Set<number>>(new Set());let preparingMessage=$state('');
function sourceFailure(problem:unknown,retry='Select the video again to retry.'){
 const message=String(problem);
 if(['Selection changed; choose the movie again','Movie selection replaced'].includes(message))return 'Your movie selection changed. Refresh Stremio and choose the title again.';
 if(['stale media resolution','TorBox selection replaced'].includes(message))return `Playback or your source selection changed while this request was running. ${retry}`;
 if(message==='server stopping')return 'Windows playback is closing. Reopen 2gether before choosing a video.';
 return message;
}
async function prepareStremio(option:Ready){if(!options || stremioBusy || torboxBusy)return;stremioBusy=true;try{const status=await invoke<{ready:boolean;filename:string;filesize:number}>('stremio_prepare',{revision:options.revision,index:option.index});prepared=new Set([...prepared,option.index]);if(status.ready){options.ready=[...options.ready.filter(row=>row.index!==option.index),{index:option.index,filename:status.filename,filesize:status.filesize}];options.cached=options.cached.filter(row=>row.index!==option.index);preparingMessage='Your video is ready. Select Watch to prepare both players.'}else{preparingMessage='TorBox is still preparing this video. Check preparation again shortly. You can keep watching the current movie.'}error=''}catch(e){error=sourceFailure(e)}finally{stremioBusy=false}}
async function importStremio(){if(stremioBusy || torboxBusy)return;const link=copiedLink;copiedLink='';stremioBusy=true;try{await invoke('stremio_copied_source',{link});selectedTitle='Imported Stremio video';page='watch';error=''}catch(e){error=sourceFailure(e,'Paste the source again to retry.')}finally{stremioBusy=false}}
async function stremioAction(action:(request:number)=>Promise<void>,cancelable=false){
 if(stremioBusy || torboxBusy)return;
 const request=++stremioRequest;
 stremioBusy=true;stremioCancelable=cancelable;options=null;prepared=new Set();preparingMessage='';
 try {await action(request);if(request===stremioRequest)error=''}
 catch(e){
  if(request!==stremioRequest)return;
  error=String(e);
  try {const next=await invoke<Browser>('stremio_browser');if(request===stremioRequest)browser=next}
  catch {if(request===stremioRequest)browser=null}
 } finally {if(request===stremioRequest){stremioBusy=false;stremioCancelable=false}}
}
async function cancelStremioLookup(){
 if(!stremioCancelable)return;
 const request=++stremioRequest;stremioCancelable=false;
 try {
  const next=await invoke<Browser>('stremio_cancel_lookup');
  if(request===stremioRequest){browser=next;options=null;movieName='';error=''}
 } catch {if(request===stremioRequest){stremioCancelable=true;error='Could not cancel the lookup. Try again.'}}
 finally {if(request===stremioRequest)stremioBusy=false}
}
async function connectStremio(){await stremioAction(async(request)=>{const next=await invoke<Browser>('stremio_connect');if(request===stremioRequest){browser=next;catalogIndex='0';movieName=''}})}
async function browseStremio(){if(!browser?.connected)return;await stremioAction(async(request)=>{const next=await invoke<Browser>('stremio_browse',{revision:browser!.revision,index:Number(catalogIndex),search:search.trim() || null});if(request===stremioRequest){browser=next;movieName=''}},true)}
async function movieOptions(index:number){if(!browser?.connected)return;await stremioAction(async(request)=>{movieName=browser!.titles[index].name;const next=await invoke<Options>('stremio_options',{revision:browser!.revision,index});if(request===stremioRequest){options=next;browser!.revision=next.revision}},true)}
async function chooseTitle(index:number){if(!browser?.connected)return;if(browser.titles[index].kind==='series' && !browser.episode_mode){await stremioAction(async(request)=>{const next=await invoke<Browser>('stremio_episodes',{revision:browser!.revision,index});if(request===stremioRequest){browser=next;movieName=''}},true)}else{await movieOptions(index)}}
async function backToSeries(){if(!browser?.connected)return;await stremioAction(async(request)=>{const next=await invoke<Browser>('stremio_back',{revision:browser!.revision});if(request===stremioRequest){browser=next;movieName=''}})}
function chooseAnotherTitle(){if(stremioBusy || torboxBusy)return;options=null;movieName='';preparingMessage=''}
async function watchStremio(option:Ready){if(!options || stremioBusy || torboxBusy)return;stremioBusy=true;try{await invoke('stremio_select',{revision:options.revision,index:option.index});selectedTitle=movieName;page='watch';error=''}catch(e){error=sourceFailure(e)}finally{stremioBusy=false}}
async function loadTorbox(){if(stremioBusy || torboxBusy)return;torboxBusy=true;try{videos=await invoke<Video[]>('torbox_videos');listed=true;error=''}catch(e){error=String(e)}finally{torboxBusy=false}}
async function selectTorbox(video:Video){if(stremioBusy || torboxBusy)return;torboxBusy=true;try{await invoke('torbox_select',{torrentId:video.torrent_id,fileId:video.file_id});selectedTitle=video.filename || 'Selected movie';page='watch';error=''}catch(e){error=sourceFailure(e)}finally{torboxBusy=false}}
async function refreshTorbox(){if(stremioBusy || torboxBusy)return;torboxBusy=true;try{await invoke('torbox_refresh');error=''}catch(e){error=sourceFailure(e,'Refresh playback links again to retry.')}finally{torboxBusy=false}}
type DiagnosticHealth={active:boolean;dropped_records:number;io_errors:number};
let journal=$state<DiagnosticHealth|null>(null);
let exportPath=$state('');let exporting=$state(false);
async function exportLog(){exporting=true;try{exportPath=await invoke<string>('export_session_log');error=''}catch{error='Could not export the session log. Check diagnostic status and try again.'}finally{exporting=false}}
async function pairPhone(){if(pairingBusy || connectionBusy)return;if(phoneConnected()){page='connection';return}pairingBusy=true;page='connection';try{await refreshConnection();pairingCode=await invoke<string>('open_pairing');pairingOpened=Date.now();error=''}catch{error='Could not open pairing. Check that the Windows session is running.'}finally{pairingBusy=false}}
async function closePairing(){if(pairingBusy || connectionBusy)return;pairingBusy=true;try{await invoke('cancel_pairing');pairingCode='';error=''}catch{error='Could not cancel the pairing code. Try again.'}finally{pairingBusy=false}}
const time=(ms:number)=>`${Math.floor(ms/60000)}:${String(Math.floor(ms/1000)%60).padStart(2,'0')}`;
let intentRequest=0;
async function intent(action:string,positionMs?:number){
 const request=++intentRequest;
 try{await invoke('player_intent',{intent:action,positionMs});if(request===intentRequest)error=''}
 catch(e){if(request===intentRequest)error=String(e)}
}
const canSeek=()=>player.initialized && Boolean(session?.media) && player.duration_ms>0;
async function seek(deltaMs:number){
 if(!canSeek())return;
 const lastPosition=Math.max(0,player.duration_ms-500);
 await intent('seek',Math.min(lastPosition,Math.max(0,player.position_ms+deltaMs)));
}
onMount(()=>{
 void invoke<AndroidDownload>('android_download_info').then(info=>{androidDownload=info}).catch(()=>{downloadError='Could not prepare the Android download code.'});
 let stopped=false;let refreshing=false;
 let startupProof=false;let startupReported=false;
 void refreshConnection().catch(()=>{error='Could not read connection setup.'});
 void invoke<boolean>('startup_proof_enabled').then(value=>{startupProof=value}).catch(()=>{});
 void invoke<boolean>('private_alpha_account_links').then(value=>{if(!stopped)accountLinks=value}).catch(()=>{error='Could not determine this build’s playback delivery policy.'});
 async function refresh(){if(stopped || refreshing)return;refreshing=true;try{
 if(page==='connection' && !connectionBusy && Date.now()-connectionPolled>=2000){connectionPolled=Date.now();void refreshConnection().catch(()=>{})}const [next,snapshot,health]=await Promise.all([invoke<Player>('player_status'),invoke<Snapshot>('session_snapshot'),invoke<DiagnosticHealth|null>('diagnostic_health')]);if(!stopped){player=next;session=snapshot;journal=health;
 if(pairingCode && (phoneConnected() || Date.now()-pairingOpened>=300000))pairingCode='';
 if(startupProof && !startupReported && next.initialized && !next.error && !error){const text=document.querySelector('main')?.textContent ?? '';const rendered=document.querySelectorAll('main button').length>=8 && text.includes('Stremio') && text.includes('TorBox') && text.includes('gether');startupReported=await invoke<boolean>('startup_frontend_ready',{rendered})}
 }}catch(e){if(!stopped)error=String(e)}finally{refreshing=false}}
 const bounds=()=>{const r=surface.getBoundingClientRect();void invoke('video_bounds',{x:Math.max(0,r.x),y:Math.max(0,r.y),width:r.width,height:r.height}).catch(e=>{error=String(e)})};
 const observer=new ResizeObserver(bounds);observer.observe(surface);window.addEventListener('resize',bounds);window.addEventListener('scroll',bounds);
 bounds();void refresh();const timer=setInterval(()=>void refresh(),250);
 return ()=>{stopped=true;clearInterval(timer);observer.disconnect();window.removeEventListener('resize',bounds);window.removeEventListener('scroll',bounds)};
});
</script>
<svelte:head><title>2gether · Watch together</title></svelte:head>
<main>
<nav><strong><span>2</span>gether</strong><div class="nav-links"><button class:chosen={page==='discover'} onclick={()=>page='discover'}>Discover</button><button class:chosen={page==='watch'} onclick={()=>page='watch'}>Watch</button><button class:chosen={page==='connection'} onclick={()=>{page='connection';void refreshConnection().catch(()=>{error='Could not read connection setup.'})}}>Connection <span class:online={phoneConnected()} class="dot"></span></button></div></nav>
<p class="intro">Connect your phone, choose a movie, and watch together. Play, pause and seek controls are shared.</p>
{#if error || player.error}<p role="alert" class="notice error">{error || player.error}</p>{/if}
<div class="watch-page" class:hidden={page!=='watch'}><h1>{selectedTitle || 'Your shared cinema'}</h1>
<div bind:this={surface} class="video" aria-label="Native libmpv video surface"></div>
<section><strong>{player.state}</strong><span>{time(player.position_ms)} / {time(player.duration_ms)}</span>
{#if player.buffering}<p>Waiting for media…</p>{/if}
<p>Generation {player.generation} · Drift {player.drift_ms} ms · Rate {player.rate.toFixed(3)}×</p>
{#if session}
<p>{#each Object.entries(session.participants) as [id,p]}{id === 'pc' ? 'Windows' : 'Android'}: {p.connected ? (p.buffering ? 'Buffering' : 'Connected') : 'Waiting'} · {/each}
{!session.participants.phone?.connected ? 'Waiting for your phone. Playback starts when both players are ready.' : session.prepare ? 'Getting both players ready…' : session.media ? 'Ready to watch together' : 'Choose a movie to get started.'}</p>
{#if session.fault}<p class="error">Playback needs to recover. Select Retry playback to try again.</p>{/if}
{/if}

<div class="controls">
<button disabled={pairingBusy || connectionBusy || phoneConnected()} onclick={pairPhone}>Pair phone</button>
<button disabled={!player.initialized} onclick={()=>intent('load_test')}>Try sample video</button>
<button disabled={!player.initialized || !session?.media} onclick={()=>intent('play')}>{session?.fault ? 'Retry playback' : session?.ended ? 'Replay' : 'Play'}</button>
<button disabled={!player.initialized || !session?.media} onclick={()=>intent('pause')}>Pause</button>
<button disabled={!canSeek()} onclick={()=>seek(-10000)}>−10 seconds</button>
<button disabled={!canSeek()} onclick={()=>seek(10000)}>+10 seconds</button>
</div></section></div>
<div class="discover-page" class:hidden={page!=='discover'}><header class="hero"><small>YOUR NEXT MOVIE NIGHT</small><h1>Choose a movie or episode.</h1><p>Browse your Stremio catalog, choose an episode, or play a ready video from TorBox.</p></header>
{#if accountLinks}<p class="error">Private Alpha delivery: your paired phone receives playback links containing your TorBox account key. Keep this session private.</p>{/if}
<section aria-label="Stremio movies"><strong>Stremio movies</strong>
<p>Use your saved PC account and configured add-ons. Choose a movie, then a ready TorBox video. Both players will receive the same file.</p>
<button disabled={stremioBusy || torboxBusy} onclick={connectStremio}>{stremioBusy ? 'Loading…' : browser?.connected ? 'Refresh Stremio library' : 'Browse Stremio'}</button>
{#if stremioCancelable}<button onclick={cancelStremioLookup}>Cancel lookup</button>{/if}
{#if browser?.connected && !options}
{#if browser.catalogs.length}
<div class="controls"><label>Catalog <select bind:value={catalogIndex} disabled={stremioBusy || torboxBusy}>{#each browser.catalogs as catalog,index}<option value={String(index)}>{catalog.name} · {catalog.kind}</option>{/each}</select></label>
{#if browser.catalogs[Number(catalogIndex)]?.searchable}<label>Find a movie <input bind:value={search} maxlength="160" placeholder="Movie title" disabled={stremioBusy || torboxBusy}/></label>{/if}
<button disabled={stremioBusy || torboxBusy || (browser.catalogs[Number(catalogIndex)]?.search_required && search.trim().length < 2)} onclick={browseStremio}>Show titles</button></div>
{/if}
{#if !browser.titles.length}<p>Your library or catalog is empty. Choose a catalog above to find movies.</p>{/if}
{#if browser.episode_mode}<p>Choose the episode you want to watch.</p><button disabled={stremioBusy || torboxBusy} onclick={backToSeries}>Back to series titles</button>{/if}
<div class="controls"><button class:chosen={visibleKind==='all'} onclick={()=>visibleKind='all'}>All titles</button><button class:chosen={visibleKind==='movie'} onclick={()=>visibleKind='movie'}>Movies</button><button class:chosen={visibleKind==='series'} onclick={()=>visibleKind='series'}>{browser.episode_mode ? 'Episodes' : 'Series'}</button></div>
<div class="movie-list">{#each browser.titles as title,index}{#if visibleKind==='all' || title.kind===visibleKind}<button class="title-card" disabled={stremioBusy || torboxBusy} onclick={()=>chooseTitle(index)}><span class="cover" aria-hidden="true">{title.name.slice(0,1)}</span><strong>{title.name}</strong><small>{title.kind === 'series' ? (browser.episode_mode ? 'Choose source' : 'Choose episode') : 'Movie · choose source'}</small></button>{/if}{/each}</div>
{/if}
{#if options}<div class="source-panel"><h2>{movieName}</h2><button disabled={stremioBusy || torboxBusy} onclick={chooseAnotherTitle}>Choose another title</button><p>Choose a source to prepare the same file on both devices.</p><p>{movieName}: {options.ready.length} ready video{options.ready.length === 1 ? '' : 's'}.</p>
{#if !options.ready.length && !options.cached.length}<p>No exact ready or provider-cached TorBox file matched. Try another title or prepare the movie in Stremio first, then use its copied source.</p>{/if}
{#if !options.cache_available}<p>TorBox cached-source checks were unavailable. Choose the movie again to retry; ready account videos remain available.</p>{/if}
{#if options.unavailable_providers}<p>Some add-ons did not respond. Choose the movie again to retry.</p>{/if}
{#each options.ready as option}<p><button disabled={stremioBusy || torboxBusy || !player.initialized} onclick={()=>watchStremio(option)}>Watch {option.filename}</button></p>{/each}
{#if options.cached.length}<p>These exact files are cached by TorBox. Prepare one in your account, then select Watch when it is ready. Your account limits still apply.</p>{/if}
{#each options.cached as option}<p><button disabled={stremioBusy || torboxBusy} onclick={()=>prepareStremio(option)}>{prepared.has(option.index) ? 'Check preparation' : 'Prepare in TorBox'} · {option.filename}</button></p>{/each}
{#if preparingMessage}<p role="status">{preparingMessage}</p>{/if}
</div>{/if}
</section>
<section aria-label="TorBox playback"><strong>TorBox playback</strong>
<p>Choose a video from your TorBox account. Keep this Windows app open while you watch.</p>
<button disabled={stremioBusy || torboxBusy} onclick={loadTorbox}>{torboxBusy ? 'Connecting…' : 'Show available TorBox videos'}</button>
{#if session?.media?.provider === 'torbox' && !session.ended}
<p>If the movie's links stop working, refresh them to retry from your current position.</p>
<button disabled={stremioBusy || torboxBusy || !player.initialized} onclick={refreshTorbox}>Refresh playback links</button>
{/if}
{#if listed && !videos.length && !torboxBusy}<p>No ready videos were found. Add a video to TorBox and check again once it has finished processing.</p>{/if}
{#each videos as video}<p><button disabled={stremioBusy || torboxBusy || !player.initialized} onclick={()=>selectTorbox(video)}>{video.filename || `Video ${video.torrent_id}/${video.file_id}`}</button></p>{/each}
</section>
<details class="copied-source"><summary>Use a copied Stremio source</summary><section aria-label="Copied Stremio source">
<p>Alternatively, start a movie or episode in Stremio on this PC and paste its Torrentio/TorBox source link here. Once that exact file is ready in TorBox, 2gether can prepare both players.</p>
<div class="controls"><label>Stremio source <input type="password" bind:value={copiedLink} maxlength="8192" autocomplete="off" placeholder="Paste copied source link" disabled={stremioBusy || torboxBusy}/></label><button disabled={stremioBusy || torboxBusy || !player.initialized || !copiedLink.trim()} onclick={importStremio}>Use Stremio source</button></div>
<p>The source stays on this PC. Localhost and unrecognized links are not forwarded to your phone.</p>
</section></details></div>
<details><summary>Playback diagnostics</summary><section aria-label="Session diagnostics"><strong>Playback diagnostics</strong>
<p>Generation {player.generation} · Drift {player.drift_ms} ms · Rate {player.rate.toFixed(3)}×</p>
<p>Phone logs sent from Android are included here. Use Export session log when reporting a problem.</p>
<p>{journal?.active ? (journal.io_errors ? 'Recording has storage errors' : 'Recording') : 'Recording unavailable'}{#if journal} · Dropped records: {journal.dropped_records} · Storage errors: {journal.io_errors}{/if}</p>
<button disabled={exporting || !journal?.active} onclick={exportLog}>{exporting ? 'Exporting…' : 'Export session log'}</button>
{#if exportPath}<p class="export-path">Saved to: {exportPath}</p>{/if}
</section></details>
<section class="connection-page" class:hidden={page!=='connection'}><h1>Connect your phone</h1><p class="notice">{phoneConnected() ? 'Android is connected to this Windows session.' : 'Keep Windows open. On Android, enter the server address and a fresh pairing code.'}</p>
<article class="android-download"><h2>Get the Android app</h2><p>Scan this code on your phone or open the download page. This address stays the same when Windows or the phone connection restarts.</p>{#if androidDownload}<img src={"data:image/svg+xml,"+encodeURIComponent(androidDownload.qr_svg)} alt="QR code for the 2gether Android download page" width="200" height="200"/><code>{androidDownload.url}</code><button onclick={openAndroidDownload}>Open Android download</button>{/if}{#if downloadError}<p class="error">{downloadError}</p>{/if}<p>Install over your existing 2gether app to keep pairing. Future versions are available from App updates on Android.</p></article>
<div class="connection-grid"><article><h2>Test on this PC</h2><p>Open the Android emulator shortcut on your Desktop. Enter this address in Android:</p><code>{connection?.emulator_address || 'http://127.0.0.1:47820'}</code><p>This local address works through the emulator launcher. A physical phone cannot use it to reach this PC.</p></article><article><h2>Connect a physical phone</h2>{#if connection?.phone_address}<code>{connection.phone_address}</code><p>Control tunnel: {connection.tunnel.process?.state || connection.tunnel.mode}. A running process still needs a reachable network address.</p>{:else}<p class="error">Remote phone connection is not set up on this PC.</p><p>Start a temporary phone connection below. It creates an HTTPS address for this watching session.</p>{/if}</article></div>
{#if connection?.tunnel.mode!=='configured'}<div class="notice"><h2>Phone connection for this Alpha</h2><p>Keep this app open while you watch. Temporary addresses change when you start a new connection; Android may need the new address and a fresh pairing code next time.</p>{#if connection?.error}<p class="error">{connection.error}</p>{/if}{#if connection?.tunnel.mode==='temporary'}<p role="status">{connection.phone_address ? 'Address ready. Enter it on Android, then generate a pairing code.' : 'Creating your HTTPS address…'}</p><button disabled={connectionBusy || pairingBusy} onclick={stopPhoneConnection}>Stop phone connection</button>{:else}<button disabled={connectionBusy || pairingBusy} onclick={startPhoneConnection}>{connectionBusy ? 'Starting…' : 'Start phone connection'}</button>{/if}</div>{/if}
<button disabled={pairingBusy || connectionBusy || phoneConnected()} onclick={pairPhone}>{pairingBusy ? 'Opening…' : pairingCode ? 'Generate a new code' : 'Generate pairing code'}</button>
{#if phoneConnected()}<p role="status">An Android device is already connected. Close the emulator or disconnect that device before pairing another phone.</p>{/if}
{#if pairingCode}<div class="pairing-card"><p>Enter this code in Android:</p><strong class="pairing-code">{pairingCode}</strong><p>Works once and expires in five minutes. The code disappears here after the phone connects.</p><button disabled={pairingBusy || connectionBusy} onclick={closePairing}>Cancel code</button></div>{/if}<p>After pairing, Android saves the connection securely and reconnects automatically. To change PCs, use “Connect to a different PC” on Android.</p></section>
<p><button onclick={openSourceNotices}>Source code and licenses</button></p>{#if sourceNoticeError}<p role="status">{sourceNoticeError}</p>{/if}
<footer>Private Alpha · Physical-phone and two-hour stability testing remain outstanding.</footer>
</main>
<style>
.export-path{overflow-wrap:anywhere;font-size:13px}
.movie-list{display:grid;grid-template-columns:repeat(auto-fill,minmax(150px,1fr));gap:18px;margin-top:20px}.movie-list button{overflow-wrap:anywhere}input,select{background:#16131d;color:#eee;border:1px solid #51405f;border-radius:999px;padding:10px;max-width:320px}label{display:flex;align-items:center;gap:8px}
:global(body){margin:0;background:#07090e;color:#f1f1f5;font-family:Segoe UI,system-ui,sans-serif}main{max-width:1200px;margin:auto;padding:24px 32px}nav{display:flex;justify-content:space-between;align-items:center}nav strong{font-size:25px;letter-spacing:-1px}nav span{color:#bf9efc;margin-left:5px}small,.intro,footer{color:#a9adbb}.intro{font-size:14px;margin:20px 0}.video{width:100%;aspect-ratio:16/9;background:#000}section{padding:20px 0}section>span{float:right;color:#a9adbb}.controls{display:flex;gap:10px;flex-wrap:wrap;margin-top:18px}button{background:#292135;color:#eee;border:1px solid #51405f;padding:11px 16px;border-radius:999px;cursor:pointer}button:disabled{opacity:.4;cursor:default}.error{color:#ffb1ad}footer{font-size:12px;line-height:1.6}

.hidden{display:none}.nav-links{display:flex;gap:10px}.chosen{background:#60468b;border-color:#bfa0fc}.dot{display:inline-block;width:8px;height:8px;border-radius:50%;background:#aab0bc}.dot.online{background:#76deb5}.hero{padding:24px;background:linear-gradient(120deg,#33234c,#171d30);border-radius:24px;margin:20px 0}.hero h1{font-size:clamp(28px,3.4vw,40px);line-height:1.15;margin:16px 0}.hero p{max-width:580px;color:#c4c8d5}.notice{padding:16px;background:#211d2b;border-radius:16px}.title-card{text-align:left;border-radius:18px;padding:12px;display:flex;flex-direction:column;gap:10px;background:#161923}.cover{width:100%;aspect-ratio:2/3;border-radius:12px;background:linear-gradient(145deg,#624680,#192941);display:grid;place-items:center;font-size:64px;color:#ceb6ec}.title-card small{font-size:12px}.source-panel{margin-top:20px;padding:24px;background:#181a25;border-radius:20px}.connection-grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(260px,1fr));gap:20px;margin:24px 0}article,.pairing-card{padding:24px;border:1px solid #373344;border-radius:20px}code{display:block;overflow-wrap:anywhere;color:#d1baff;padding:12px;background:#13121a;border-radius:12px}.pairing-code{font-size:32px;letter-spacing:3px}.pairing-card{margin-top:20px}.connection-page p{line-height:1.6}h1{letter-spacing:-1px}h2{font-size:20px}button:hover:enabled{border-color:#ba9de5}.video{border-radius:16px;overflow:hidden}@media(max-width:760px){main{padding:20px}nav{align-items:flex-start;gap:14px;flex-direction:column}.nav-links{flex-wrap:wrap}.hero{padding:24px}}
.android-download img{display:block;background:white;padding:8px;margin:16px 0}.android-download{margin:24px 0}
</style>
