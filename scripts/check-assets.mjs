import { existsSync } from 'node:fs';
import { join } from 'node:path';

const games = ['arknights', 'endfield', 'genshin', 'hsr', 'zzz', 'wuwa', 'gfl2'];
const recommended = ['background.webp', 'logo.webp', 'icon.webp'];
const optional = ['background.mp4'];

let present = 0;
let missing = 0;

for (const game of games) {
  console.log(`\n${game}`);
  for (const file of recommended) {
    const path = join('public', 'game-assets', game, file);
    const ok = existsSync(path);
    console.log(`  ${ok ? '✓' : '·'} ${file}${ok ? '' : ' (fallback will be used)'}`);
    ok ? present++ : missing++;
  }
  for (const file of optional) {
    const path = join('public', 'game-assets', game, file);
    console.log(`  ${existsSync(path) ? '✓' : '·'} ${file} (optional)`);
  }
}

console.log(`\nArtwork files: ${present} present, ${missing} using fallbacks.`);
process.exit(0);
