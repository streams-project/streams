// Writes a single track WebM file from encoded video chunks. Enough for a
// preview; the file has no seek index.

interface Chunk { data: Buffer; ms: number; key: boolean; }

function vint(size: number): Buffer {
  const b = Buffer.alloc(8);
  b.writeBigUInt64BE(BigInt(size) | (1n << 56n));
  return b;
}

function el(id: number[], body: Buffer): Buffer {
  return Buffer.concat([Buffer.from(id), vint(body.length), body]);
}

function uint(id: number[], value: number): Buffer {
  const b = Buffer.alloc(8);
  b.writeBigUInt64BE(BigInt(value));
  let i = 0;
  while (i < 7 && b[i] === 0) i++;
  return el(id, b.subarray(i));
}

function str(id: number[], value: string): Buffer {
  return el(id, Buffer.from(value, "ascii"));
}

export function webm(chunks: Chunk[], width: number, height: number, codec: string): Buffer {
  const header = el([0x1a, 0x45, 0xdf, 0xa3], Buffer.concat([
    uint([0x42, 0x86], 1), uint([0x42, 0xf7], 1), uint([0x42, 0xf2], 4), uint([0x42, 0xf3], 8),
    str([0x42, 0x82], "webm"), uint([0x42, 0x87], 2), uint([0x42, 0x85], 2)
  ]));
  const info = el([0x15, 0x49, 0xa9, 0x66], Buffer.concat([
    uint([0x2a, 0xd7, 0xb1], 1000000), str([0x4d, 0x80], "tv-landing"), str([0x57, 0x41], "tv-landing")
  ]));
  const video = el([0xe0], Buffer.concat([uint([0xb0], width), uint([0xba], height)]));
  const track = el([0xae], Buffer.concat([
    uint([0xd7], 1), uint([0x73, 0xc5], 1), uint([0x83], 1), str([0x86], codec), video
  ]));
  const tracks = el([0x16, 0x54, 0xae, 0x6b], track);
  const clusters: Buffer[] = [];
  let current: Buffer[] = [];
  let clusterMs = 0;
  const flush = (): void => {
    if (current.length) clusters.push(el([0x1f, 0x43, 0xb6, 0x75], Buffer.concat([uint([0xe7], clusterMs), ...current])));
    current = [];
  };
  for (const chunk of chunks) {
    const ms = Math.round(chunk.ms);
    if (!current.length || ms - clusterMs > 4000) {
      flush();
      clusterMs = ms;
    }
    const block = Buffer.alloc(4);
    block[0] = 0x81;
    block.writeInt16BE(ms - clusterMs, 1);
    block[3] = chunk.key ? 0x80 : 0;
    current.push(el([0xa3], Buffer.concat([block, chunk.data])));
  }
  flush();
  const segment = el([0x18, 0x53, 0x80, 0x67], Buffer.concat([info, tracks, ...clusters]));
  return Buffer.concat([header, segment]);
}
