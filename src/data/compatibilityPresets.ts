import type { RuntimeKind } from '../lib/settings';

export type CompatibilityState =
  | 'native'
  | 'playable'
  | 'playable-with-caveats'
  | 'experimental'
  | 'unsupported'
  | 'unknown';

export type ModBackendKind = 'xxmi' | 'library-only' | 'none';

export interface GameCompatibilityPreset {
  gameId: string;
  checkedAt: string;
  windows: {
    state: CompatibilityState;
    label: string;
  };
  linux: {
    state: CompatibilityState;
    label: string;
    recommendedRuntime: RuntimeKind;
    recommendedRunner: string;
    runnerCandidates: string[];
    blockedRunners: string[];
    launchArguments: string;
    environmentVariables: string;
    optionalEnvironmentVariables?: string;
    notes: string[];
    xxmiLaunchArguments?: string;
    xxmiNotes?: string[];
  };
  modding: {
    backend: ModBackendKind;
    importer?: string;
    label: string;
    note: string;
  };
}

export const compatibilityPresets: Record<string, GameCompatibilityPreset> = {
  arknights: {
    gameId: 'arknights',
    checkedAt: '2026-09-18',
    windows: { state: 'native', label: 'Nativer PC-Client' },
    linux: {
      state: 'playable-with-caveats',
      label: 'Community: läuft über Proton',
      recommendedRuntime: 'ge-proton',
      recommendedRunner: 'GE-Proton (aktuell) / Proton Experimental',
      runnerCandidates: ['GE-Proton (aktuell)', 'Proton Experimental', 'Valve Proton (aktuell)'],
      blockedRunners: [],
      launchArguments: '',
      environmentVariables: '',
      notes: [
        'Der neue offizielle PC-Client wurde von Linux-Nutzern mit Proton/GE-Proton erfolgreich gestartet.',
        'Gamescope kann bei träger Maus-/Fensterdarstellung helfen.',
        'Da es keine offizielle Linux-Unterstützung gibt, bleibt der Status community-basiert.',
      ],
    },
    modding: {
      backend: 'library-only',
      label: 'Library only',
      note: 'Kein XXMI-Importer hinterlegt. GachaHub kann Mods organisieren/importieren, aber keinen Loader automatisch injizieren.',
    },
  },
  endfield: {
    gameId: 'endfield',
    checkedAt: '2026-09-18',
    windows: { state: 'native', label: 'Nativer PC-Client' },
    linux: {
      state: 'playable-with-caveats',
      label: 'Läuft mit geeigneten Proton-Builds',
      recommendedRuntime: 'ge-proton',
      recommendedRunner: 'GE-Proton >= 10-30 oder aktuelles DWProton/CachyOS-Proton',
      runnerCandidates: ['GE-Proton >= 10-30', 'DWProton (aktuell)', 'Proton-CachyOS (aktuell)'],
      blockedRunners: ['Proton Vanilla', 'Proton UMU'],
      launchArguments: '',
      environmentVariables: '',
      notes: [
        'Neuere Proton-Builds enthalten Endfield-spezifische Wine/ACE-Fixes.',
        'Twintail warnt weiterhin davor, Proton Vanilla oder Proton UMU für Endfield zu verwenden.',
        'Bei Problemen zuerst den Runner wechseln, bevor Prefix oder Installation neu erstellt werden.',
      ],
      xxmiNotes: [
        'EFMI ist in XXMI vorhanden.',
        'GachaHub lässt XXMI für Endfield bewusst nur manuell aktivieren, da das Spiel als restricted markiert ist.',
      ],
    },
    modding: {
      backend: 'xxmi',
      importer: 'EFMI',
      label: 'XXMI / EFMI',
      note: 'Technisch vorhanden, in GachaHub standardmäßig eingeschränkt und nur bewusst manuell aktivierbar.',
    },
  },
  genshin: {
    gameId: 'genshin',
    checkedAt: '2026-09-18',
    windows: { state: 'native', label: 'Nativer Windows-Client' },
    linux: {
      state: 'playable',
      label: 'Community/Twintail: spielbar über Proton',
      recommendedRuntime: 'proton',
      recommendedRunner: 'aktuelles DWProton / Proton-CachyOS / Proton Hotfix',
      runnerCandidates: ['DWProton (aktuell)', 'Proton-CachyOS (aktuell)', 'Proton Hotfix', 'GE-Proton (aktuell)'],
      blockedRunners: [],
      launchArguments: '',
      environmentVariables: '',
      optionalEnvironmentVariables: 'WINE_ENABLE_TIMEOUT_FIX=1',
      notes: [
        'Aktuelle Community-Berichte zeigen Genshin auf Fedora/Bazzite/Mint über Proton als spielbar.',
        'Bei sporadischen Security-Token/Timeout-Crashes kann WINE_ENABLE_TIMEOUT_FIX=1 helfen; nicht pauschal erzwingen.',
        'Für Updates kann weiterhin der offizielle Launcher oder später der GachaHub-Provider verwendet werden.',
      ],
    },
    modding: {
      backend: 'xxmi',
      importer: 'GIMI',
      label: 'XXMI / GIMI',
      note: 'Direkt vom integrierten GachaHub Mod Manager als XXMI-Backend vorgesehen.',
    },
  },
  hsr: {
    gameId: 'hsr',
    checkedAt: '2026-09-18',
    windows: { state: 'native', label: 'Nativer Windows-Client' },
    linux: {
      state: 'playable',
      label: 'Twintail: aktuelle Proton-Builds funktionieren',
      recommendedRuntime: 'proton',
      recommendedRunner: 'aktueller Proton/CachyOS/GE Build',
      runnerCandidates: ['Proton-CachyOS (aktuell)', 'Proton Experimental', 'GE-Proton (aktuell)'],
      blockedRunners: [],
      launchArguments: '',
      environmentVariables: '',
      notes: [
        'Twintail meldete im Mai 2026, dass Star Rail wieder mit aktuellen Proton-Familien läuft.',
        'Ältere Proton-10-Builds hatten zeitweise Regressionen; deshalb nicht blind eine alte Version pinnen.',
        'Wenn ein Update Startprobleme verursacht, zuerst einen aktuellen alternativen Runner testen.',
      ],
    },
    modding: {
      backend: 'xxmi',
      importer: 'SRMI',
      label: 'XXMI / SRMI',
      note: 'XXMI unterstützt Star Rail über SRMI.',
    },
  },
  zzz: {
    gameId: 'zzz',
    checkedAt: '2026-09-18',
    windows: { state: 'native', label: 'Nativer Windows-Client' },
    linux: {
      state: 'playable-with-caveats',
      label: 'Grundsätzlich spielbar; aktuelle 3.2-Regressionsberichte beachten',
      recommendedRuntime: 'proton',
      recommendedRunner: 'Proton Experimental / aktueller Proton-Build',
      runnerCandidates: ['Proton Experimental', 'Valve Proton (aktuell)', 'GE-Proton (aktuell)'],
      blockedRunners: [],
      launchArguments: '',
      environmentVariables: '',
      notes: [
        'ZZZ wird von Twintail im Multi-Plattform-Launcher unterstützt, aber nach Update 3.2 wurden im September 2026 einzelne sofortige Start-Crashes unter Proton gemeldet.',
        'DX12/RTX/Frame-Generation hat auf einigen Linux-Systemen zusätzliche Fehlerberichte; Runner und Rendering-Modus deshalb nicht hart verdrahten.',
        'Bei Problemen nach Spielupdates zuerst Runner/Prefix prüfen und nicht automatisch Schutzmechanismen umgehen.',
      ],
      xxmiNotes: [
        'ZZMI kann die Spieleinstellungen konfigurieren.',
        'Für ZZMI müssen Character Quality = High und High-Precision Character Animation = Disabled sein, falls die automatische Konfiguration nicht genutzt wird.',
      ],
    },
    modding: {
      backend: 'xxmi',
      importer: 'ZZMI',
      label: 'XXMI / ZZMI',
      note: 'Direkter XXMI-Importer vorhanden.',
    },
  },
  wuwa: {
    gameId: 'wuwa',
    checkedAt: '2026-09-18',
    windows: { state: 'native', label: 'Nativer Windows-/Steam-Client' },
    linux: {
      state: 'playable-with-caveats',
      label: 'Spielbar; aktueller Proton empfohlen',
      recommendedRuntime: 'proton',
      recommendedRunner: 'aktuelles Proton-CachyOS / GE-Proton / Experimental',
      runnerCandidates: ['Proton-CachyOS (aktuell)', 'GE-Proton (aktuell)', 'Proton Experimental'],
      blockedRunners: ['alte Proton-9-Builds'],
      launchArguments: '',
      environmentVariables: '',
      notes: [
        'Wuthering Waves ist unter Proton spielbar; aktuelle Proton-Versionen beheben ältere Media-/WebView-Probleme.',
        'Bei NVIDIA wurden einzelne VRAM/UI-Probleme gemeldet; deshalb Runner und Gamescope separat konfigurierbar lassen.',
      ],
      xxmiLaunchArguments: '-DisableModule=streamline -dx11 -d3d11',
      xxmiNotes: [
        'WWMI benötigt DX11. XXMI dokumentiert -DisableModule=streamline -dx11 -d3d11 als Startargumente.',
        'Diese Argumente sollten nur mit aktiviertem WWMI automatisch ergänzt werden.',
      ],
    },
    modding: {
      backend: 'xxmi',
      importer: 'WWMI',
      label: 'XXMI / WWMI',
      note: 'XXMI-Backend vorhanden; beim Mod-Start DX11-Anforderungen berücksichtigen.',
    },
  },
  gfl2: {
    gameId: 'gfl2',
    checkedAt: '2026-09-18',
    windows: { state: 'native', label: 'Nativer Windows-/Steam-Client' },
    linux: {
      state: 'playable-with-caveats',
      label: 'Proton funktioniert, Runner-Regressions beachten',
      recommendedRuntime: 'proton',
      recommendedRunner: 'Proton 10.0-4 / Proton Experimental; GE 10-34 als Video-Fallback',
      runnerCandidates: ['Proton 10.0-4', 'Proton Experimental', 'GE-Proton 10-34'],
      blockedRunners: ['Proton 11.0 Beta (bekannte Startregression)', 'GE-Proton 11.1–11.3 (Video-Regressionsberichte)'],
      launchArguments: '',
      environmentVariables: '',
      optionalEnvironmentVariables: 'UMU_ID=0',
      notes: [
        'Die Steam-Version läuft bei vielen Nutzern mit Proton; Valve führt SteamOS aktuell trotzdem als unsupported.',
        'Proton 11 Beta hatte eine Startregression; Proton 10.0-4 funktionierte im zugehörigen Valve-Report.',
        'GE-Proton 11.1–11.3 hatte gemeldete Videoprobleme; GE 10-34 war dort der letzte funktionierende Stand.',
        'Der Launcher kann WebView2 benötigen. GachaHub sollte das später als Prefix-Dependency prüfen.',
        'UMU_ID=0 ist ein optionaler Steam-Workaround und soll nicht global erzwungen werden.',
      ],
    },
    modding: {
      backend: 'library-only',
      label: 'Library only',
      note: 'Kein XXMI-Importer hinterlegt. Mod-Library/Import kann genutzt werden; automatisches Deployment benötigt später einen eigenen Adapter.',
    },
  },
};

export function getCompatibilityPreset(gameId: string): GameCompatibilityPreset | undefined {
  return compatibilityPresets[gameId];
}
