import Formbricks from '@formbricks/js';

type TFormbricks = Omit<Formbricks, 'track'> & {
  track: (code: string) => Promise<void>;
};

declare global {
  interface Window {
    formbricks: TFormbricks | undefined;
  }
}
