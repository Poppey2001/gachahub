import type { Game, GameAssets } from '../lib/types';

const artwork = (id: string): GameAssets => ({
  background: `/game-assets/${id}/background.webp`,
  video: `/game-assets/${id}/background.mp4`,
  logo: `/game-assets/${id}/logo.webp`,
  icon: `/game-assets/${id}/icon.webp`,
});

export const fallbackGames: Game[] = [
  {
    id: 'arknights', name: 'Arknights', shortName: 'AK', subtitle: 'Rhodes Island tactical operations', publisher: 'Yostar / Hypergryph', provider: 'hypergryph',
    platforms: ['Windows', 'Linux (community Proton)'], installModes: ['Official PC Client'], status: 'not_installed',
    modPolicy: 'restricted', accent: '#55d7ff', accent2: '#536dff', version: 'PC Client', notes: 'Official PC client; Linux community reports via GE-Proton/Proton Experimental.', xxmiImporter: null, assets: artwork('arknights')
  },
  {
    id: 'endfield', name: 'Arknights: Endfield', shortName: 'EF', subtitle: 'Explore Talos-II', publisher: 'GRYPHLINE', provider: 'gryphline',
    platforms: ['Windows', 'Linux (Proton with compatible runner)'], installModes: ['Official PC Client'], status: 'not_installed',
    modPolicy: 'restricted', accent: '#f4cd58', accent2: '#ff7a45', version: 'Stable', notes: 'Use Proton builds with Endfield/ACE fixes; restricted mod policy by default.', xxmiImporter: 'EFMI', assets: artwork('endfield')
  },
  {
    id: 'genshin', name: 'Genshin Impact', shortName: 'GI', subtitle: 'Step into a vast magical world', publisher: 'HoYoverse', provider: 'hoyoverse',
    platforms: ['Windows', 'Linux (compatibility layer)'], installModes: ['Direct provider', 'Official launcher import'], status: 'not_installed',
    modPolicy: 'supported', accent: '#70cfff', accent2: '#7f7bff', version: 'Latest', xxmiImporter: 'GIMI', assets: artwork('genshin')
  },
  {
    id: 'hsr', name: 'Honkai: Star Rail', shortName: 'HSR', subtitle: 'Board the Astral Express', publisher: 'HoYoverse', provider: 'hoyoverse',
    platforms: ['Windows', 'Linux (current Proton builds)'], installModes: ['Direct provider', 'Official launcher import'], status: 'not_installed',
    modPolicy: 'restricted', accent: '#b78cff', accent2: '#6574ff', version: 'Latest', xxmiImporter: 'SRMI', assets: artwork('hsr')
  },
  {
    id: 'zzz', name: 'Zenless Zone Zero', shortName: 'ZZZ', subtitle: 'Welcome to New Eridu', publisher: 'HoYoverse', provider: 'hoyoverse',
    platforms: ['Windows', 'Linux (compatibility layer)'], installModes: ['Direct provider', 'Official launcher import'], status: 'not_installed',
    modPolicy: 'supported', accent: '#f4e75a', accent2: '#8cff6b', version: 'Latest', xxmiImporter: 'ZZMI', assets: artwork('zzz')
  },
  {
    id: 'wuwa', name: 'Wuthering Waves', shortName: 'WW', subtitle: 'Wake and journey as a Rover', publisher: 'Kuro Games', provider: 'kuro',
    platforms: ['Windows', 'Linux (compatibility layer)'], installModes: ['Direct provider', 'Official launcher import'], status: 'not_installed',
    modPolicy: 'supported', accent: '#58e5dc', accent2: '#3998ff', version: 'Latest', xxmiImporter: 'WWMI', assets: artwork('wuwa')
  },
  {
    id: 'gfl2', name: "GIRLS' FRONTLINE 2: EXILIUM", shortName: 'GFL2', subtitle: 'Command your tactical dolls', publisher: 'Sunborn', provider: 'sunborn',
    platforms: ['Windows', 'Linux (Proton with runner caveats)'], installModes: ['Steam', 'Standalone import'], status: 'not_installed',
    modPolicy: 'restricted', accent: '#ff8d72', accent2: '#ff4f7a', version: 'Latest', xxmiImporter: null, assets: artwork('gfl2')
  }
];
