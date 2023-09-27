import {syncAll} from '../lib/index';
import {Request, Response} from 'express';
import {config} from '../lib/config';

export async function metabaseToSendInBlueConnector(_request: Request, response: Response) {
  return syncAll(config.metabase.collectionId, config.sendinblue.folderId)
    .then((output) => response.json(output))
    .catch((error) => {
      response.status(500).send(error);
    });
}
