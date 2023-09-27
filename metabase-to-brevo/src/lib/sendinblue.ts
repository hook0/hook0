import axios, {AxiosRequestConfig, AxiosResponse} from 'axios';
import {delay, Promise, map, mapSeries} from 'bluebird';
import {chunk, flatten, last, range} from 'lodash';
import {onAxiosError, onError} from './error';
import {logger} from './logger';

export type SendinblueContactList = {
  id: number;
  name: string;
  folderId: 1;
  uniqueSubscribers: number;
  totalBlacklisted: number;
  totalSubscribers: number;
};

export type SendinblueContact = {
  email: string;
  id: number;
  emailBlacklisted: boolean;
  smsBlacklisted: boolean;
  createdAt: string;
  modifiedAt: string;
  listIds: number[];
  attributes: Record<string, any>;
};

type SendinblueContactCreatePayload = Pick<SendinblueContact, 'email'> &
  Partial<Omit<SendinblueContact, 'id' | 'email' | 'createdAt' | 'modifiedAt'>>;

export type SendinblueContactUpdatePayload = SendinblueContactCreatePayload & {
  unlinkListIds?: number[];
};

export type SendinblueAvailableAttributeType = 'boolean' | 'date' | 'float' | 'text';

type ContactAttributes = {
  name: string;
  category: string;
  type: string;
  calculatedValue: string;
};

export type SendinblueConfig = {
  baseUrl: string;
  apiKey: any;
  folderId: number;
  testFolderId: number;
  attributeCategory: string;
  requestsConcurrency: number;
};

type SendinblueProcessStatus = {
  id: number;
  status: 'queued' | 'in_process' | 'completed';
  name: string;
};

export class SendinblueClient {
  constructor(private config: SendinblueConfig) {}

  private makeRequest(
    axiosConfig: AxiosRequestConfig,
    retries = 0,
    options = {logError: true}
  ): Promise<AxiosResponse> {
    return axios({
      ...axiosConfig,
      headers: {
        ...axiosConfig.headers,
        'api-key': this.config.apiKey
      }
    })
      .catch(onAxiosError('cannot make request on sendinblue', axiosConfig))
      .catch((error) => {
        if (options.logError) {
          logger.error(`error making request to sendinblue: ${error}`);
        }
        // no need to retry when we have these errors
        if (error.status >= 400 && error.status <= 404) {
          throw error;
        }
        if (retries > 5) {
          throw error;
        }
        return delay(retries * 1000).then(() => this.makeRequest(axiosConfig, retries + 1));
      });
  }

  private fetchProcessStatus(processId: number) {
    return this.makeRequest({
      method: 'GET',
      url: `${this.config.baseUrl}/processes/${processId}`
    })
      .then((response) => response.data as SendinblueProcessStatus)
      .catch(onError(`cannot fetch status of process ${processId} on sendinblue`));
  }

  private waitForProcessToComplete(processId: number, processesProgress: string = ''): Promise<void> {
    return this.fetchProcessStatus(processId).then((processStatus) => {
      logger.info(
        `${processesProgress} sendinblue process "${processStatus.name}" (${processId}) status: ${processStatus.status}`
      );
      if (processStatus.status === 'completed') {
        return Promise.resolve();
      }
      return delay(2_000).then(() => this.waitForProcessToComplete(processId, processesProgress));
    });
  }

  // https://developers.sendinblue.com/reference/getlists-1
  fetchListsOfFolder(folderId: number): Promise<SendinblueContactList[]> {
    logger.info(`fetching sendinblue lists of folder ${folderId}`);
    return this.makeRequest({
      method: 'GET',
      url: `${this.config.baseUrl}/contacts/folders/${folderId}/lists`
    })
      .then((response) => response.data.lists || [])
      .catch(onError('cannot fetch contact lists on sendinblue'));
  }

  // https://developers.sendinblue.com/reference/createlist-1
  createContactList(listName: string, folderId: number): Promise<{id: number}> {
    logger.info(`create sendinblue contact list ${listName} of folder ${folderId}`);
    return this.makeRequest({
      method: 'POST',
      url: `${this.config.baseUrl}/contacts/lists`,
      data: {
        name: listName,
        folderId: folderId
      }
    })
      .then((response) => response.data as {id: number})
      .catch(onError(`cannot create contact lists on sendinblue`));
  }

  // https://developers.sendinblue.com/reference/deletelist-1
  removeContactList(listId: number): Promise<void> {
    logger.info(`removing sendinblue contact list ${listId}`);
    return (
      this.makeRequest({
        method: 'DELETE',
        url: `${this.config.baseUrl}/contacts/lists/${listId}`
      })
        // res.data is empty, nothing returned from sendinblue
        .then(() => {})
        .catch(onError(`cannot remove contact list ${listId} on sendinblue`))
    );
  }

  removeAllContactListsOfFolder(folderId: number): Promise<void> {
    logger.info(`removing all sendinblue contact list of folder ${folderId}`);
    return this.fetchListsOfFolder(folderId)
      .then((lists) => {
        return map(lists, (list) => this.removeContactList(list.id), {concurrency: this.config.requestsConcurrency});
      })
      .then(() => {})
      .catch(onError(`cannot remove all contact lists from folder ${folderId} on sendinblue`));
  }

  // https://developers.sendinblue.com/reference/getcontactsfromlist
  fetchContactsFromList(listId: number): Promise<SendinblueContact[]> {
    const howManyPagesToFetchAtOnce = this.config.requestsConcurrency;
    const limit = 500;

    // we fetch several pages at once concurrently because on some big contacts list,
    // fetching serially 500 per 500 contacts is too slow
    const fetchSeveralPages = (acc: SendinblueContact[] = [], fromPage: number = 0): Promise<SendinblueContact[]> => {
      return Promise.map(range(fromPage, fromPage + howManyPagesToFetchAtOnce), (page) => {
        const offset = fromPage + 500 * page;

        logger.info(`fetching sendinblue contacts from list ${listId}, offset ${offset} limit ${limit}`);
        return this.makeRequest({
          method: 'GET',
          url: `${this.config.baseUrl}/contacts/lists/${listId}/contacts`,
          params: {
            limit,
            offset
          }
        })
          .then((response) => response.data.contacts as SendinblueContact[])
          .catch(onError(`cannot fetch contacts from list: ${listId}`));
      }).then((pages) => {
        const lastPage = last(pages) || [];
        const gotItAll = lastPage.length === 0;
        const newAcc = acc.concat(flatten(pages));
        if (gotItAll) {
          logger.info(`fetched all sendinblue contacts from list ${listId}`);
          return newAcc;
        }
        return fetchSeveralPages(newAcc, fromPage + howManyPagesToFetchAtOnce);
      });
    };

    return fetchSeveralPages();
  }

  // https://developers.sendinblue.com/reference/getattributes-1
  fetchContactAttributes(): Promise<ContactAttributes[]> {
    logger.info(`fetching sendinblue contacts attributes`);
    return this.makeRequest({
      method: 'GET',
      url: `${this.config.baseUrl}/contacts/attributes`
    })
      .then((response) => response.data.attributes as ContactAttributes[])
      .catch(onError('cannot fetch contact attributes on sendinblue'));
  }

  // https://developers.sendinblue.com/reference/createattribute-1
  createContactAttribute(
    attributeName: string,
    attributeType: SendinblueAvailableAttributeType,
    attributeCategory = 'normal'
  ): Promise<void> {
    logger.info(`create sendinblue contacts attribute ${attributeName} of type ${attributeType}`);
    return (
      this.makeRequest({
        method: 'POST',
        url: `${this.config.baseUrl}/contacts/attributes/${attributeCategory}/${attributeName}`,
        data: {
          type: attributeType
        }
      })
        // res.data is empty, nothing returned from sendinblue
        .then(() => {})
        .catch(onError(`cannot create contact attribute on sendinblue, attributeName: ${attributeName}`))
    );
  }

  removeContactAttribute(attributeName: string, attributeCategory = 'normal'): Promise<void> {
    logger.info(`removing sendinblue contacts attribute ${attributeName}`);
    return (
      this.makeRequest({
        method: 'DELETE',
        url: `${this.config.baseUrl}/contacts/attributes/${attributeCategory}/${attributeName}`
      })
        // res.data is empty, nothing returned from sendinblue
        .then(() => {})
        .catch(onError(`cannot remove contact attribute ${attributeName} on sendinblue`))
    );
  }

  // https://developers.sendinblue.com/reference/createcontact
  upsertContact(payload: SendinblueContactCreatePayload): Promise<{id: number}> {
    logger.info(`create sendinblue contact ${payload.email} belonging to list ${payload.listIds}`);
    return this.makeRequest(
      {
        method: 'POST',
        url: `${this.config.baseUrl}/contacts`,
        data: payload
      },
      0,
      {logError: false}
    )
      .then((response) => response.data)
      .catch((error) => {
        const errorMessage = `couldn't create sendinblue contact ${payload.email}`;
        // if the contact already exists, update it
        if (error.message && error.message.includes('duplicate_parameter')) {
          return this.updateContacts([payload]);
        }
        return onError(errorMessage, error.response.status)(error);
      });
  }

  // https://developers.sendinblue.com/reference/updatebatchcontacts
  updateContacts(contacts: SendinblueContactUpdatePayload[]): Promise<void> {
    // we cannot create more than 100 contacts per request
    const contactsChunks = chunk(contacts, 100);
    const totalChunks = contactsChunks.length;
    return Promise.map(
      contactsChunks,
      (contactsChunk, i) => {
        logger.info(`updating sendinblue contacts, chunk ${i + 1}/${totalChunks}`);
        return this.makeRequest({
          method: 'POST',
          url: `${this.config.baseUrl}/contacts/batch`,
          data: {
            contacts: contactsChunk
          }
          // res.data is empty, nothing returned from sendinblue
        }).catch(onError(`cannot update contacts chunk ${i + 1}/${totalChunks} on sendinblue`));
      },
      {concurrency: this.config.requestsConcurrency}
    ).then(() => {});
  }

  importContactsInListBatch(contacts: SendinblueContactUpdatePayload[], listIds: number[]): Promise<void> {
    const contactsChunks = chunk(contacts, 5000);
    const totalChunks = contactsChunks.length;
    // concurrent requests is of no use here since they are queued on sendinblue side
    return mapSeries(contactsChunks, (contactsChunk, i) => {
      logger.info(`importing sendinblue contacts, chunk ${i + 1}/${totalChunks}`);
      // https://developers.brevo.com/reference/importcontacts-1
      return this.makeRequest({
        method: 'POST',
        url: `${this.config.baseUrl}/contacts/import`,
        data: {
          listIds,
          jsonBody: contactsChunk,
          updateExistingContacts: true,
          emptyContactsAttributes: false
        }
      })
        .then((response) => {
          const processId = response.data.processId;
          return this.waitForProcessToComplete(processId, `${i + 1}/${contactsChunks.length}`);
        })
        .catch(onError(`cannot import contacts chunk ${i + 1}/${totalChunks} on sendinblue`));
    }).then(() => {});
  }
}
