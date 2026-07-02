import Phaser from 'phaser';
import './style.css';

class GameScene extends Phaser.Scene {
  private player!: Phaser.Physics.Matter.Image;

  create(): void {
    this.cameras.main.setBackgroundColor('#10141f');

    const texture = this.add.graphics();
    texture.fillStyle(0xf5c451).fillCircle(24, 24, 24);
    texture.lineStyle(4, 0x5b4516).lineBetween(24, 24, 44, 24);
    texture.generateTexture('rocknrolla', 48, 48);
    texture.destroy();

    this.player = this.matter.add.image(150, 180, 'rocknrolla', undefined, {
      shape: 'circle',
      friction: 0.8,
      frictionAir: 0.01,
      restitution: 0.1,
    });

    this.matter.add.rectangle(480, 360, 900, 40, {
      isStatic: true,
      angle: Phaser.Math.DegToRad(12),
    });
    this.matter.add.rectangle(610, 285, 28, 100, {
      isStatic: true,
      angle: Phaser.Math.DegToRad(12),
    });

    this.add.text(24, 24, 'Press or tap to escape the obstacle', {
      color: '#ffffff',
      fontSize: '20px',
    });

    this.input.on('pointerdown', () => {
      this.player.applyForce(new Phaser.Math.Vector2(0.003, -0.025));
    });
  }
}

new Phaser.Game({
  type: Phaser.AUTO,
  parent: 'game',
  width: 960,
  height: 540,
  backgroundColor: '#10141f',
  physics: {
    default: 'matter',
    matter: {
      gravity: { x: 0, y: 1 },
    },
  },
  scale: {
    mode: Phaser.Scale.FIT,
    autoCenter: Phaser.Scale.CENTER_BOTH,
  },
  scene: GameScene,
});
