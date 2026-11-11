import { test } from 'node:test';
import assert from 'node:assert/strict';
import { evaluateRun } from '../../scripts/acceptance-017.mjs';

const run = () => ({ samples: Array.from({ length: 1800 }, (_, second) => ({ second, kind: 'measured', delayMs: 300 })), sourceFps: 30, decodedFrames: 48600, interruptionSeconds: 18, maxPauseSeconds: 1, textCorrect: 19, baselineGameFps: 100, candidateGameFps: 95 });
test('fixed denominator and inclusive quality boundaries', () => { assert.equal(evaluateRun(run()).status, 'pass'); });
test('1782 instrumented and 1710 successful slots are accepted', () => {
 const r = run();
 for (let i=0;i<18;i++) r.samples[i*10] = { second:i*10, kind:'instrument-gap' };
 for (let i=180;i<252;i++) r.samples[i].delayMs = 900;
 assert.equal(evaluateRun(r).latency, 'pass');
 r.samples[252].delayMs=900;
 assert.equal(evaluateRun(r).latency,'fail');
});
test('insufficient coverage and three consecutive gaps cannot pass', () => {
 const r=run(); for(let i=0;i<19;i++) r.samples[i*10]={second:i*10,kind:'instrument-gap'};
 assert.equal(evaluateRun(r).latency,'pending');
 const s=run(); for(let i=0;i<3;i++) s.samples[i]={second:i,kind:'instrument-gap'};
 assert.equal(evaluateRun(s).latency,'pending');
});
test('video outage is a delivery failure and known failure outranks missing measurements', () => {
 const r=run(); for(let i=0;i<5;i++) r.samples[i]={second:i,kind:'no-video'};
 for(let i=20;i<23;i++) r.samples[i]={second:i,kind:'instrument-gap'};
 assert.equal(evaluateRun(r).latency,'fail');
});
test('quality thresholds are enforceable and missing evidence stays pending', () => {
 for(const change of [{decodedFrames:48420},{interruptionSeconds:18.1},{maxPauseSeconds:1.1},{candidateGameFps:94.9},{textCorrect:18}]) assert.equal(evaluateRun({...run(),...change}).quality,'fail');
 const r=run(); delete r.sourceFps; assert.equal(evaluateRun(r).quality,'pending');
});
test('rejects identifiers, duplicate slots and fabricated values', () => {
 assert.throws(()=>evaluateRun({...run(),ip:'private'}));
 const r=run(); r.samples[1].second=0; assert.throws(()=>evaluateRun(r));
 const s=run(); delete s.samples[0].delayMs; assert.throws(()=>evaluateRun(s));
});
