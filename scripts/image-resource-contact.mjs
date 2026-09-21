// Reproducible nearest-neighbor review sheet from the frozen 1K Native probe.
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { deflateSync } from 'node:zlib';
import assert from 'node:assert/strict';
const [input, output, panels] = process.argv.slice(2);
assert.ok(input && output, 'node scripts/image-resource-contact.mjs <evidence-directory> <output.png> [panel-directory]');
const width = 1024, height = 512;
const pixels = Buffer.alloc(width * height * 4);
for (const [col, weight] of [0, 0.25, 0.5, 1].entries()) {
  for (const [row, channel] of ['height', 'normal'].entries()) {
    const source = readFileSync(join(input, `image-input-${weight}-${channel}.rgba`));
    assert.equal(source.length, 1024 * 1024 * 4);
    for (let y = 0; y < 256; y++) for (let x = 0; x < 256; x++) {
      const start = (y * 4 * 1024 + x * 4) * 4;
      source.copy(pixels, ((row * 256 + y) * width + col * 256 + x) * 4, start, start + 4);
    }
  }
}
function crc32(bytes) {
  let crc = 0xffffffff;
  for (const byte of bytes) {
    crc ^= byte;
    for (let i = 0; i < 8; i++) crc = (crc >>> 1) ^ ((crc & 1) ? 0xedb88320 : 0);
  }
  return (crc ^ 0xffffffff) >>> 0;
}
function chunk(type, data) {
  const content = Buffer.concat([Buffer.from(type), data]);
  const size = Buffer.alloc(4), crc = Buffer.alloc(4);
  size.writeUInt32BE(data.length); crc.writeUInt32BE(crc32(content));
  return Buffer.concat([size, content, crc]);
}
function save(path, width, height, pixels) {
  const header = Buffer.alloc(13);
  header.writeUInt32BE(width, 0); header.writeUInt32BE(height, 4); header[8] = 8; header[9] = 6;
  const scanlines = Buffer.alloc(height * (width * 4 + 1));
  for (let y = 0; y < height; y++) pixels.copy(scanlines, y * (width * 4 + 1) + 1, y * width * 4, (y + 1) * width * 4);
  writeFileSync(path, Buffer.concat([Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
    chunk('IHDR', header), chunk('IDAT', deflateSync(scanlines)), chunk('IEND', Buffer.alloc(0))]));
}
save(output, width, height, pixels);
if (panels) {
  mkdirSync(panels, { recursive: true });
  for (const [row, channel] of ['height', 'normal'].entries()) {
    for (const [col, weight] of [0, 0.25, 0.5, 1].entries()) {
      const panel = Buffer.alloc(256 * 256 * 4);
      for (let y = 0; y < 256; y++) {
        const start = ((row * 256 + y) * width + col * 256) * 4;
        pixels.copy(panel, y * 256 * 4, start, start + 256 * 4);
      }
      save(join(panels, `${channel}-${weight}.png`), 256, 256, panel);
    }
  }
}
