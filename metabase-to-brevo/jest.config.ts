import type {Config} from 'jest';

const config: Config = {
  testEnvironment: 'node',
  testTimeout: 30_000,
  testMatch: ['<rootDir>/**/?(*.)+(spec|test).ts'],
  transform: {
    '^.+\\.(t|j)sx?$': '@swc/jest'
  }
};

export default config;
