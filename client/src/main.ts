import Phaser from 'phaser';
import './style.css';
import { TUNING } from './tuning';
import { DPR, VIEW_H, VIEW_W } from './ui';
import { BootScene } from './scenes/BootScene';
import { CharacterSelectScene } from './scenes/CharacterSelectScene';
import { CollectionScene } from './scenes/CollectionScene';
import { GameHudScene } from './scenes/GameHudScene';
import { GameScene } from './scenes/GameScene';
import { LevelSelectScene } from './scenes/LevelSelectScene';
import { ResultScene } from './scenes/ResultScene';

declare global {
  interface Window {
    /** Exposed for debugging and headless drive scripts. */
    game: Phaser.Game;
  }
}

window.game = new Phaser.Game({
  type: Phaser.AUTO,
  parent: 'game',
  // Backing store at device resolution; scenes lay out in 960×540 via a
  // per-scene camera zoom (ui.setupCamera). Phaser 4 has no DPR support.
  width: VIEW_W * DPR,
  height: VIEW_H * DPR,
  backgroundColor: '#33203c',
  physics: {
    default: 'matter',
    matter: {
      gravity: { x: 0, y: TUNING.GRAVITY_Y },
    },
  },
  scale: {
    mode: Phaser.Scale.FIT,
    autoCenter: Phaser.Scale.CENTER_BOTH,
  },
  scene: [
    BootScene,
    LevelSelectScene,
    CharacterSelectScene,
    GameScene,
    GameHudScene,
    ResultScene,
    CollectionScene,
  ],
});
