import {map as mapP, mapSeries} from 'bluebird';
import {groupBy, identity, map, omit, pickBy, truncate} from 'lodash';
import {config} from './config';
import {onError} from './error';
import {logger} from './logger';
import {
  MetabaseAttribute,
  MetabaseClient,
  MetabaseContact,
  MetabaseQuestion,
  MetabaseAvailableAttributeTypes
} from './metabase';
import {
  SendinblueAvailableAttributeType,
  SendinblueClient,
  SendinblueContact,
  SendinblueContactUpdatePayload
} from './sendinblue';

type ApiClients = {
  metabase: MetabaseClient;
  sendinblue: SendinblueClient;
};

type SendinblueAttribute = {
  type: SendinblueAvailableAttributeType;
  fromMetabaseValue: (value: any) => any;
};

export type SyncMetabaseQuestionToSendinblueResult = {
  metabaseQuestion: MetabaseQuestion;
  sendInBlueTargetedList: {
    id: number;
    existed: boolean;
  };
  attributes: {
    sendinblueAttributesFromMetabase: Record<string, SendinblueAttribute>;
    sendinblueCreatedAttributes: Record<string, SendinblueAttribute>;
  };
  contacts: {
    upserted: SendinblueContactUpdatePayload[];
    removed: SendinblueContact[];
  };
};

export function diff<T, U>(firstArray: T[], secondArray: U[], key?: string): {added: U[]; removed: T[]} {
  const getValue = (el: any, key?: string) => (key ? el[key] : el);
  const firstArraySet = new Set(firstArray.map((el: any) => getValue(el, key)));
  const secondArraySet = new Set(secondArray.map((el: any) => getValue(el, key)));
  return {
    added: secondArray.filter((el: any) => !firstArraySet.has(getValue(el, key))),
    removed: firstArray.filter((el: any) => !secondArraySet.has(getValue(el, key)))
  };
}

export function fromMetabaseToSendinblueAttributesTypes(
  metabaseAttributes: MetabaseAttribute[]
): Record<string, SendinblueAttribute> {
  function toSendinblueAttributeType(metabaseType: MetabaseAvailableAttributeTypes): SendinblueAttribute {
    // https://github.com/metabase/metabase/blob/f342fe17bd897dd4940a2c23a150a78202fa6b72/src/metabase/driver/postgres.clj#LL568C29-L581C64
    switch (metabaseType) {
      case 'type/Boolean':
        return {
          type: 'boolean',
          fromMetabaseValue(value: string | null) {
            return Boolean(value) ? 'yes' : 'no';
          }
        };

      case 'type/Date': // "2022-12-08T00:00:00Z"
      case 'type/DateTime': // "2022-12-08T09:23:46.107648Z"
      case 'type/DateTimeWithTZ': // "2022-12-08T09:23:46.107648Z"
      case 'type/DateTimeWithLocalTZ': // "2022-12-08T09:23:46.107648Z"
        return {
          type: 'date',
          fromMetabaseValue(value: string | null) {
            if (!value) return null;
            return value.split('T')[0];
          }
        };

      case 'type/Decimal':
      case 'type/Float':
      case 'type/Integer':
        return {type: 'float', fromMetabaseValue: identity};

      case 'type/Time': // "09:23:46.107648Z"
      case 'type/TimeWithTZ': // "09:23:46.107648Z"
      case 'type/Text':
      case 'type/IPAddress':
      case 'type/UUID':
      default:
        return {type: 'text', fromMetabaseValue: identity};
    }
  }
  return metabaseAttributes.reduce((acc: Record<string, SendinblueAttribute>, attribute) => {
    acc[attribute.name] = toSendinblueAttributeType(attribute.base_type);
    return acc;
  }, {});
}

export function createSendinblueContactLists(
  clients: ApiClients,
  metabaseQuestion: MetabaseQuestion,
  sendinblueFolderId: number
) {
  logger.info(`creating list to sendinblue from metabase question : ${metabaseQuestion.name}`);
  // sendinblue list names are 50 characters maximum
  const sendinblueListName = truncate(`${metabaseQuestion.id}_${metabaseQuestion.name}`, {length: 50});
  return clients.sendinblue
    .createContactList(sendinblueListName, sendinblueFolderId)
    .then((createdList) => createdList.id)
    .catch(onError(`cannot create sendinblue list: ${sendinblueListName} in folder: ${sendinblueFolderId}`));
}

export function syncAvailableAttributes(
  clients: ApiClients,
  metabaseQuestionId: number
): Promise<{
  sendinblueAttributesFromMetabase: Record<string, SendinblueAttribute>;
  sendinblueCreatedAttributes: Record<string, SendinblueAttribute>;
}> {
  return Promise.all([
    clients.metabase.fetchQuestion(metabaseQuestionId),
    clients.sendinblue.fetchContactAttributes()
  ]).then(([metabaseDetailedQuestion, sendinblueContactAttributes]) => {
    const metabaseAttributes = metabaseDetailedQuestion.result_metadata;

    const diffContactsAttributes = diff(sendinblueContactAttributes, metabaseAttributes, 'name');
    const sendinblueAttributesFromMetabase = fromMetabaseToSendinblueAttributesTypes(metabaseAttributes);
    const diffContactsAttributesAddedNames = new Set(map(diffContactsAttributes.added, 'name'));

    const sendinblueAttributesToCreate = pickBy(sendinblueAttributesFromMetabase, (_value, key) => {
      return diffContactsAttributesAddedNames.has(key);
    });

    // since the sendinblue attributes are shared between list
    // we won't remove the sendinblue attributes that don't appear in the metabase question
    // because they might be used in other sendinblue contacts lists
    return mapP(
      Object.entries(omit(sendinblueAttributesToCreate, 'EMAIL')),
      ([attributeName, {type}]) => {
        return clients.sendinblue.createContactAttribute(attributeName, type);
      },
      {concurrency: config.sendinblue.requestsConcurrency}
    ).then(() => {
      return {
        sendinblueAttributesFromMetabase,
        sendinblueCreatedAttributes: sendinblueAttributesToCreate
      };
    });
  });
}

export function removeRemovedContacts(
  clients: ApiClients,
  sendinblueListId: number,
  metabaseContacts: MetabaseContact[],
  sendinblueContacts: SendinblueContact[]
): Promise<SendinblueContact[]> {
  const {added: contactsToRemoveOnSendinblue} = diff(metabaseContacts, sendinblueContacts, 'email');
  return mapP(
    contactsToRemoveOnSendinblue,
    (contact) => {
      // remove the contact from the current sendinblue list
      return clients.sendinblue
        .updateContacts([{email: contact.email, unlinkListIds: [sendinblueListId]}])
        .catch((error) => {
          logger.error(
            `encountered an error removing contact ${contact.email} from ${sendinblueListId} sendinblue list (keep going for next contacts): ${error}`
          );
        })
        .then(() => contact);
    },
    {concurrency: config.sendinblue.requestsConcurrency}
  );
}

export function syncContactWithAttributesValues(
  clients: ApiClients,
  metabaseContacts: MetabaseContact[],
  sendinblueContacts: SendinblueContact[],
  sendinblueListId: number,
  sendinblueAttributesFromMetabase: Record<string, SendinblueAttribute>
): Promise<SendinblueContactUpdatePayload[]> {
  function fromMetabaseToSendinblueAttributes(metabaseContact: MetabaseContact, attributesNames: string[]) {
    return attributesNames.reduce((acc: Record<string, any>, attributeName) => {
      const metabaseAttributeValue = metabaseContact[attributeName];
      const sendinblueAttribute = sendinblueAttributesFromMetabase[attributeName];
      if (sendinblueAttribute) {
        acc[attributeName] = sendinblueAttribute.fromMetabaseValue(metabaseAttributeValue);
      }
      return acc;
    }, {});
  }

  // format contacts found on metabase to sendinblue contacts
  const sendinblueContactsWithUpdatedAttributes = metabaseContacts.map((metabaseContact) => {
    const attributesNames = Object.keys(omit(metabaseContact, 'email'));
    return {
      email: metabaseContact.email,
      attributes: fromMetabaseToSendinblueAttributes(metabaseContact, attributesNames)
    } as SendinblueContactUpdatePayload;
  });

  // filter out existing sendinblue contacts already having the correct attributes values
  const existingSendinblueContactsByEmail = groupBy(sendinblueContacts, 'email');
  const sendinblueContactsToUpdate = sendinblueContactsWithUpdatedAttributes.filter((contactToUpdate) => {
    const existingContacts = existingSendinblueContactsByEmail[contactToUpdate.email || ''] || [];
    const existingContact = existingContacts[0];
    // the contact doesn't exist on sendinbllue: keep it
    if (!existingContact) {
      return true;
    }

    // keep the contact for update if at least one attribute needs to be updated
    const contactToUpdateAttributes = contactToUpdate.attributes || {};
    return Object.keys(contactToUpdateAttributes).some((attrName) => {
      return contactToUpdateAttributes[attrName] !== existingContact.attributes[attrName];
    });
  });

  // upsert the contacts on sendinblue with attributes
  return clients.sendinblue
    .importContactsInListBatch(sendinblueContactsToUpdate, [sendinblueListId])
    .catch((error) => {
      logger.error(`couldn't sync sendinblue contacts, reason: ${JSON.stringify(error)}`);
    })
    .then(() => sendinblueContactsToUpdate);
}

export function syncAll(
  metabaseCollectionId: number,
  sendinblueFolderId: number
): Promise<SyncMetabaseQuestionToSendinblueResult[]> {
  const clients = {
    sendinblue: new SendinblueClient(config.sendinblue),
    metabase: new MetabaseClient(config.metabase)
  };

  logger.info('fetching questions from metabase and sendinblue contacts lists');
  return Promise.all([
    clients.sendinblue.fetchListsOfFolder(sendinblueFolderId),
    clients.metabase.fetchQuestionsFromCollection(metabaseCollectionId)
  ]).then(([sendinblueLists, metabaseQuestions]) => {
    // 1. for each metabase question...
    return mapSeries(metabaseQuestions, async (metabaseQuestion, i) => {
      logger.info(`👉 syncing metabase question to sendinblue list ${i + 1}/${metabaseQuestions.length}`);

      const questionPrefix = `${metabaseQuestion.id}_`;
      const sendinblueTargetedList = sendinblueLists.find((list) => list.name.startsWith(questionPrefix));

      // 2. ...create its sendinblue list equivalent (if it doesn't exist already)
      const sendinblueListId = sendinblueTargetedList
        ? await Promise.resolve(sendinblueTargetedList.id)
        : await createSendinblueContactLists(clients, metabaseQuestion, sendinblueFolderId);

      // 3. ...sync the attributes, they are global on sendinblue (not linked to a list)
      // here we only sync their names & types, not the values they'll have for each contact
      const {sendinblueAttributesFromMetabase, sendinblueCreatedAttributes} = await syncAvailableAttributes(
        clients,
        metabaseQuestion.id
      );

      // 4. ...fetch the contacts from both side, so we can...
      const [metabaseContacts, sendinblueContacts] = await Promise.all([
        clients.metabase.runQuestion(metabaseQuestion.id),
        clients.sendinblue.fetchContactsFromList(sendinblueListId)
      ]);

      // 5. ...remove contacts not present on metabase question but in sendinblue list...
      const sendinblueRemovedContacts = await removeRemovedContacts(
        clients,
        sendinblueListId,
        metabaseContacts,
        sendinblueContacts
      );

      // 6. ...and sync the contacts with the attributes values on sendinblue contacts to match the
      // values fetched from metabase question
      const sendinblueContactsWithUpdatedAttributes = await syncContactWithAttributesValues(
        clients,
        metabaseContacts,
        sendinblueContacts,
        sendinblueListId,
        sendinblueAttributesFromMetabase
      );

      return {
        metabaseQuestion: metabaseQuestion,
        sendInBlueTargetedList: {
          id: sendinblueListId,
          existed: Boolean(sendinblueTargetedList)
        },
        attributes: {
          sendinblueAttributesFromMetabase,
          sendinblueCreatedAttributes
        },
        contacts: {
          upserted: sendinblueContactsWithUpdatedAttributes,
          removed: sendinblueRemovedContacts
        }
      };
    });
  });
}
