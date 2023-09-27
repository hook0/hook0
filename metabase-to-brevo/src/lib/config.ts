import {logger} from './logger';
import {MetabaseConfig} from './metabase';
import {SendinblueConfig} from './sendinblue';

const commonEnv = require('common-env/withLogger')(logger);

type BetteruptimeConfig = {
  heartbeatUrl: string;
};

type Config = {
  metabase: MetabaseConfig;
  sendinblue: SendinblueConfig;
  betteruptime: BetteruptimeConfig;
};

type configWithNamespace = {
  metabaseToSendinblue: Config;
};

const secureString = {
  $type: commonEnv.types.String,
  $secure: true,
  $default: ''
};

const defaultConfig: configWithNamespace = {
  // we use a namespace so that every environment variable will be prefixed with "METABASETOSENDINBLUE_",
  // this makes it easier to know which env var belongs to which project in a monorepo
  metabaseToSendinblue: {
    metabase: {
      host: '',
      username: '',
      password: secureString,
      collectionId: 0,
      testCollectionId: 0,
      testDatabaseId: 0
    },
    sendinblue: {
      baseUrl: 'https://api.brevo.com/v3',
      apiKey: secureString,
      folderId: 1,
      testFolderId: 1,
      attributeCategory: 'normal',
      requestsConcurrency: 10
    },
    betteruptime: {
      heartbeatUrl: ''
    }
  }
};

export const config: Config = commonEnv.getOrElseAll(defaultConfig).metabaseToSendinblue;
