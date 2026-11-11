import { readFile } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';

const keys = ['samples','sourceFps','decodedFrames','interruptionSeconds','maxPauseSeconds','textCorrect','baselineGameFps','candidateGameFps'];
const obj = v => typeof v === 'object' && v !== null && !Array.isArray(v);
const number = v => typeof v === 'number' && Number.isFinite(v) && v >= 0;
const only = (v, allowed) => obj(v) && Object.keys(v).every(k => allowed.includes(k));
const consecutive = (rows, kind) => { let current=0,max=0; for(const row of rows) {current=row.kind===kind?current+1:0;max=Math.max(current,max);} return max; };

export function evaluateRun(input) {
 if(!only(input,keys) || !Array.isArray(input.samples)) throw Error('Formato inválido: somente métricas anônimas são aceitas.');
 for(const key of keys.slice(1)) if(input[key]!==undefined && !number(input[key])) throw Error(`Métrica inválida: ${key}`);
 const slots = Array.from({length:1800}, (_,second)=>({second,kind:'instrument-gap'}));
 const seen = new Set();
 for(const row of input.samples) {
  if(!only(row,['second','kind','delayMs','avOffsetMs']) || !Number.isSafeInteger(row.second) || row.second<0 || row.second>=1800 || seen.has(row.second)) throw Error('Instante inválido ou repetido.');
  if(!['measured','no-video','instrument-gap'].includes(row.kind) || (row.kind==='measured' && !number(row.delayMs)) || (row.kind!=='measured' && row.delayMs!==undefined)) throw Error('Medição inválida.');
  if(row.avOffsetMs!==undefined && (typeof row.avOffsetMs!=='number' || !Number.isFinite(row.avOffsetMs))) throw Error('Medição A/V inválida.');
  seen.add(row.second); slots[row.second]=row;
 }
 const measured=slots.filter(r=>r.kind==='measured');
 const delays=measured.map(r=>r.delayMs).sort((a,b)=>a-b);
 const instrumented=1800-slots.filter(r=>r.kind==='instrument-gap').length;
 const successful=measured.filter(r=>r.delayMs<=500).length;
 const knownFailure=delays.some(v=>v>=5000) || consecutive(slots,'no-video')>=5;
 const coverageOk=instrumented>=1782 && consecutive(slots,'instrument-gap')<=2;
 const latency=knownFailure?'fail':!coverageOk?'pending':successful>=1710?'pass':'fail';
 const available=keys.slice(1).every(k=>input[k]!==undefined) && input.sourceFps>=30 && input.baselineGameFps>0;
 const qualityFailure=(input.decodedFrames!==undefined && input.decodedFrames/1800<27) || (input.interruptionSeconds!==undefined && input.interruptionSeconds>18) || (input.maxPauseSeconds!==undefined && input.maxPauseSeconds>1) || (input.textCorrect!==undefined && input.textCorrect<19) || (input.baselineGameFps>0 && input.candidateGameFps!==undefined && input.candidateGameFps<input.baselineGameFps*.95);
 const quality=qualityFailure?'fail':available?'pass':'pending';
 return {status:latency==='fail'||quality==='fail'?'fail':latency==='pending'||quality==='pending'?'pending':'pass',latency,quality,expected:1800,instrumented,successful,gaps:1800-instrumented,maxGap:consecutive(slots,'instrument-gap'),p95Ms:delays.length?delays[Math.ceil(.95*delays.length)-1]:null,maxMs:delays.at(-1)??null,over5s:delays.filter(v=>v>=5000).length,avMaxMs:measured.some(r=>r.avOffsetMs!==undefined)?Math.max(...measured.flatMap(r=>r.avOffsetMs===undefined?[]:[Math.abs(r.avOffsetMs)])):null};
}

if(process.argv[1] && import.meta.url===pathToFileURL(process.argv[1]).href) {
 try { if(!process.argv[2]) throw Error('Uso: node scripts/acceptance-017.mjs arquivo.json'); console.log(JSON.stringify(evaluateRun(JSON.parse(await readFile(process.argv[2],'utf8'))),null,2)); }
 catch(error) { console.error(error.message);process.exitCode=1; }
}
