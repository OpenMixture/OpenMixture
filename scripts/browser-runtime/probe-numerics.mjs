// Diagnostic only: execute the production WGSL and capture intermediate f16 bytes.
// Usage: node probe-numerics.mjs <playwright-module> <fresh-output> [native-noise.rgba16]
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { createServer } from 'node:http';
import { createHash } from 'node:crypto';
const [modulePath, destination, replayPath] = process.argv.slice(2);
if (!modulePath || !destination) throw Error('playwright module and fresh output required');
const { chromium } = await import(pathToFileURL(resolve(modulePath)));
const output = resolve(destination);
await mkdir(output);
const precision = await readFile('crates/mixture-wgpu/shaders/precision.wgsl', 'utf8');
const shaders = {};
for (const name of ['fractal-noise', 'height-to-normal']) {
  shaders[name] = precision + '\n' + await readFile(`crates/mixture-wgpu/shaders/nodes/${name}.wgsl`, 'utf8');
}
const replay = replayPath ? (await readFile(replayPath)).toString('base64') : null;
const parameters = JSON.parse(process.env.MIXTURE_NUMERICAL_PARAMETERS ?? '[29,32,4,0,1056964608,0,0,0]');
if (!Array.isArray(parameters) || parameters.length !== 8 || parameters.some(v=>!Number.isInteger(v) || v<0 || v>0xffffffff)
    || parameters[1]<1 || parameters[1]>128 || parameters[2]<1 || parameters[2]>6 || parameters[3]>2
    || parameters[4]>0x3f800000) throw Error('invalid diagnostic parameters');
if (process.env.MIXTURE_NUMERICAL_SHADER) shaders['fractal-noise'] = await readFile(process.env.MIXTURE_NUMERICAL_SHADER, 'utf8');
const server = createServer((_req, res) => res.end('<!doctype html><title>Mixture numerical probe</title>'));
await new Promise(r => server.listen(0, '127.0.0.1', r));
let browser;
try {
  browser = await chromium.launch({ channel: 'chrome', headless: true, args: ['--enable-unsafe-webgpu', '--ignore-gpu-blocklist'] });
  const page = await browser.newPage();
  await page.goto(`http://127.0.0.1:${server.address().port}`);
  const result = await page.evaluate(async ({ shaders, replay, parameters }) => {
    const adapter = await navigator.gpu.requestAdapter({ powerPreference: 'high-performance' });
    if (!adapter) throw Error('No hardware adapter');
    const device = await adapter.requestDevice();
    device.pushErrorScope('validation');
    const texture = () => device.createTexture({ size: [1024,1024], format: 'rgba16float', usage: GPUTextureUsage.STORAGE_BINDING | GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST });
    const noise = texture(), normal = texture();
    const files = {};
    async function capture(name, shader, entryPoint, words, input) {
      const uniform = device.createBuffer({ size: words.byteLength, usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST });
      device.queue.writeBuffer(uniform, 0, words);
      const module = device.createShaderModule({ code: shader });
      const info = await module.getCompilationInfo();
      if (info.messages.some(m => m.type === 'error')) throw Error(JSON.stringify(info.messages));
      const pipeline = await device.createComputePipelineAsync({ layout: 'auto', compute: { module, entryPoint } });
      const entries = [{ binding: 0, resource: { buffer: uniform } }, { binding: 1, resource: (input ? normal : noise).createView() }];
      if (input) entries.push({ binding: 2, resource: noise.createView() });
      const group = device.createBindGroup({ layout: pipeline.getBindGroupLayout(0), entries });
      const buffer = device.createBuffer({ size: 1024*1024*8, usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ });
      const encoder = device.createCommandEncoder(), pass = encoder.beginComputePass();
      pass.setPipeline(pipeline); pass.setBindGroup(0, group); pass.dispatchWorkgroups(128,128); pass.end();
      encoder.copyTextureToBuffer({ texture: input ? normal : noise }, { buffer, bytesPerRow: 8192, rowsPerImage: 1024 }, [1024,1024]);
      device.queue.submit([encoder.finish()]);
      await buffer.mapAsync(GPUMapMode.READ);
      const bytes = new Uint8Array(buffer.getMappedRange());
      let binary = '';
      for (let i=0;i<bytes.length;i+=8192) binary += String.fromCharCode(...bytes.subarray(i,i+8192));
      files[name] = btoa(binary);
      buffer.unmap(); buffer.destroy(); uniform.destroy();
    }
    try {
      await capture('noise', shaders['fractal-noise'], 'fractal_noise', new Uint32Array(parameters), false);
      await capture('normal', shaders['height-to-normal'], 'height_to_normal', new Uint32Array([0x3f800000,0,0,0]), true);
      if (replay) {
        const data = Uint8Array.from(atob(replay), c => c.charCodeAt(0));
        if (data.length !== 1024*1024*8) throw Error('invalid replay length');
        device.queue.writeTexture({ texture: noise }, data, { bytesPerRow: 8192, rowsPerImage: 1024 }, [1024,1024]);
        await capture('replay-normal', shaders['height-to-normal'], 'height_to_normal', new Uint32Array([0x3f800000,0,0,0]), true);
      }
      const error = await device.popErrorScope(); if (error) throw Error(error.message);
      const info = adapter.info;
      return { files, adapter: Object.fromEntries(['vendor','architecture','device','description','isFallbackAdapter'].map(k=>[k,info[k]])) };
    } finally { device.destroy(); }
  }, { shaders, replay, parameters });
  const digests = {};
  for (const [name, base64] of Object.entries(result.files)) {
    const bytes = Buffer.from(base64, 'base64');
    await writeFile(`${output}/${name}.rgba16`, bytes);
    digests[name] = createHash('sha256').update(bytes).digest('hex');
  }
  await writeFile(`${output}/receipt.json`, JSON.stringify({ parameters, adapter: result.adapter, browser: browser.version(), replaySha256: replay ? createHash('sha256').update(Buffer.from(replay,'base64')).digest('hex') : null, shaders: Object.fromEntries(Object.entries(shaders).map(([k,v])=>[k,createHash('sha256').update(v).digest('hex')])), digests }, null, 2)+'\n');
} finally { await browser?.close(); await new Promise(r=>server.close(r)); }
