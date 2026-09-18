import { useEffect, useState } from 'react';
import type { Game } from '../lib/types';
import { useSettingsStore } from '../stores/useSettingsStore';

interface ArtworkProps {
  game: Game;
}

export function GameBackdrop({ game }: ArtworkProps) {
  const animatedBackgrounds = useSettingsStore((state) => state.global.animatedBackgrounds);
  const backgroundDim = useSettingsStore((state) => state.global.backgroundDim);
  const backgroundBlur = useSettingsStore((state) => state.global.backgroundBlur);
  const [imageAvailable, setImageAvailable] = useState(Boolean(game.assets?.background));
  const [videoAvailable, setVideoAvailable] = useState(Boolean(game.assets?.video));

  useEffect(() => {
    setImageAvailable(Boolean(game.assets?.background));
    setVideoAvailable(Boolean(game.assets?.video));
  }, [game.id, game.assets?.background, game.assets?.video]);

  const showVideo = animatedBackgrounds && videoAvailable && Boolean(game.assets?.video);
  const showImage = !showVideo && imageAvailable && Boolean(game.assets?.background);

  return (
    <div className="stage-artwork" aria-hidden="true">
      <div className="stage-background" />

      {showVideo && (
        <video
          key={`${game.id}-video`}
          className="stage-media stage-media-video"
          src={game.assets?.video}
          autoPlay
          muted
          loop
          playsInline
          preload="metadata"
          onError={() => setVideoAvailable(false)}
        />
      )}

      {showImage && (
        <img
          key={`${game.id}-image`}
          className="stage-media stage-media-image"
          src={game.assets?.background}
          alt=""
          draggable={false}
          onError={() => setImageAvailable(false)}
          style={{ filter: backgroundBlur > 0 ? `blur(${backgroundBlur}px) scale(1.03)` : undefined }}
        />
      )}

      <div className="stage-artwork-color" />
      <div
        className="stage-artwork-dim"
        style={{ '--background-dim': `${Math.min(90, Math.max(0, backgroundDim)) / 100}` } as React.CSSProperties}
      />
    </div>
  );
}

export function GameLogo({ game }: ArtworkProps) {
  const [available, setAvailable] = useState(Boolean(game.assets?.logo));

  useEffect(() => {
    setAvailable(Boolean(game.assets?.logo));
  }, [game.id, game.assets?.logo]);

  if (!available || !game.assets?.logo) return null;

  return (
    <img
      className="game-hero-logo"
      src={game.assets.logo}
      alt={`${game.name} Logo`}
      draggable={false}
      onError={() => setAvailable(false)}
    />
  );
}

export function GameIcon({ game }: ArtworkProps) {
  const [available, setAvailable] = useState(Boolean(game.assets?.icon));

  useEffect(() => {
    setAvailable(Boolean(game.assets?.icon));
  }, [game.id, game.assets?.icon]);

  if (!available || !game.assets?.icon) {
    return <span>{game.shortName}</span>;
  }

  return (
    <img
      className="rail-game-icon"
      src={game.assets.icon}
      alt=""
      draggable={false}
      onError={() => setAvailable(false)}
    />
  );
}
