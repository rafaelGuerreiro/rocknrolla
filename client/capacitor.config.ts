import type { CapacitorConfig } from '@capacitor/cli';

const config: CapacitorConfig = {
  appId: 'dev.guerreiro.rocknrolla',
  appName: 'RocknRolla',
  webDir: 'dist',
  ios: {
    contentInset: 'never',
  },
};

export default config;
