import {mapSeries} from 'bluebird';
import {find, map, pick, merge} from 'lodash';
import {
  SyncMetabaseQuestionToSendinblueResult,
  createSendinblueContactLists,
  diff,
  fromMetabaseToSendinblueAttributesTypes,
  syncAll,
  syncAvailableAttributes,
  syncContactWithAttributesValues
} from '.';
import {config} from './config';
import {
  MetabaseAttribute,
  MetabaseAvailableAttributeTypes,
  MetabaseClient,
  MetabaseDetailedQuestion,
  MetabaseQuestion
} from './metabase';
import {SendinblueClient, SendinblueContact} from './sendinblue';

function getMetabaseQuestion(id: number, name: string): MetabaseQuestion {
  return {
    id,
    collection_position: null,
    collection_preview: false,
    description: '',
    display: '',
    entity_id: '',
    fully_parametrized: false,
    model: '',
    moderated_status: null,
    name,
    'last-edit-info': {
      id: 123,
      last_name: '',
      first_name: '',
      email: '',
      timestamp: ''
    }
  };
}

function getMetabaseAttribute(id: number, name: string, baseType: MetabaseAvailableAttributeTypes): MetabaseAttribute {
  return {
    id,
    description: 'description',
    semantic_type: 'semantic_type',
    coercion_strategy: 'coercion_strategy',
    name,
    settings: 'settings',
    field_ref: [],
    effective_type: 'effective_type',
    visibility_type: 'visibility_type',
    display_name: 'display_name',
    fingerprint: {
      global: 'global'
    },
    base_type: baseType
  };
}

function createTestMetabaseQuestion(name: string, query: string) {
  return clients.metabase.createQuestion({
    name,
    collection_id: config.metabase.testCollectionId,
    visualization_settings: {},
    display: 'table',
    dataset_query: {
      type: 'native',
      native: {
        query,
        'template-tags': {}
      },
      database: config.metabase.testDatabaseId
    }
  });
}

function sortOutput(output: SyncMetabaseQuestionToSendinblueResult[]) {
  return output.map((item) => {
    return {
      ...item,
      contacts: {
        removed: item.contacts.removed.sort(sortByEmail),
        upserted: item.contacts.upserted.sort(sortByEmail)
      }
    };
  });
}

function sortByEmail<T extends {email: string}>(a: T, z: T): number {
  return a.email.localeCompare(z.email);
}

function cleanFolderAndCollection(clients: {sendinblue: SendinblueClient; metabase: MetabaseClient}) {
  return Promise.all([
    clients.sendinblue.removeAllContactListsOfFolder(config.sendinblue.testFolderId).catch(() => {}),
    clients.metabase.removeAllQuestionsOfCollection(config.metabase.testCollectionId)
  ]);
}

const clients = {
  sendinblue: new SendinblueClient(config.sendinblue),
  metabase: new MetabaseClient(config.metabase)
};

describe('tests metabase to sendinblue connector', () => {
  describe('difference between 2 arrays (diff)', () => {
    it('should make a diff on primitive types without a key', () => {
      expect(diff([1, 2, 3], [3, 4, 5])).toMatchInlineSnapshot(`
    {
      "added": [
        4,
        5,
      ],
      "removed": [
        1,
        2,
      ],
    }
    `);
    });
    it('should make a diff on objects with a key', () => {
      const key = 'email';
      expect(diff([{[key]: 1}, {[key]: 2}, {[key]: 3}], [{[key]: 3}, {[key]: 4}, {[key]: 5}], key))
        .toMatchInlineSnapshot(`
    {
      "added": [
        {
          "email": 4,
        },
        {
          "email": 5,
        },
      ],
      "removed": [
        {
          "email": 1,
        },
        {
          "email": 2,
        },
      ],
    }
    `);
    });
  });

  describe('format metabase attributes types to sendinblue attributes types (fromMetabaseToSendinblueAttributesTypes)', () => {
    const sendinblueAttributesTypes = fromMetabaseToSendinblueAttributesTypes([
      getMetabaseAttribute(1, 'email', 'type/Text'),
      getMetabaseAttribute(2, 'createdAt', 'type/DateTimeWithTZ'),
      getMetabaseAttribute(3, 'age', 'type/Decimal')
    ]);
    expect(sendinblueAttributesTypes).toMatchInlineSnapshot(`
    {
      "age": {
        "fromMetabaseValue": [Function],
        "type": "float",
      },
      "createdAt": {
        "fromMetabaseValue": [Function],
        "type": "date",
      },
      "email": {
        "fromMetabaseValue": [Function],
        "type": "text",
      },
    }
    `);
  });

  describe('create a sendinblue contacts list (createSendinblueContactLists)', () => {
    afterEach(() => {
      return cleanFolderAndCollection(clients);
    });

    it('should create the list', () => {
      const folderId = config.sendinblue.testFolderId;
      return createSendinblueContactLists(clients, getMetabaseQuestion(101, 'Metabase test question'), folderId).then(
        (createdListId) => {
          expect(createdListId).toEqual(expect.any(Number));
          return clients.sendinblue.fetchListsOfFolder(folderId).then((lists) => {
            const createdContactList = find(lists, {id: createdListId});
            expect(createdContactList).toMatchInlineSnapshot(`
    {
      "id": ${createdListId},
      "name": "101_Metabase test question",
      "totalBlacklisted": 0,
      "totalSubscribers": 0,
      "uniqueSubscribers": 0,
    }
    `);
          });
        }
      );
    });
  });

  describe('sync available attributes from metabase to sendinblue (syncAvailableAttributes)', () => {
    const testPrefix = 'TEST_ATTRS_NAMES';
    const sendinblueExistingAttributeName = `${testPrefix}_DUMMY`;
    const metabaseAttributes = [
      getMetabaseAttribute(1, `${testPrefix}_FAVORITE_COLOR`, 'type/Text'),
      getMetabaseAttribute(2, `${testPrefix}_CREATEDAT`, 'type/DateTimeWithTZ')
    ];
    const metabaseAttributesNames = map(metabaseAttributes, 'name');

    let metabaseTestQuestion: MetabaseDetailedQuestion | null;

    beforeEach(() => {
      return cleanFolderAndCollection(clients).then(() => {
        return createTestMetabaseQuestion(
          'tests-e2e-attributes',
          `select 'noop' as "${testPrefix}_FAVORITE_COLOR", now()::timestamptz as "${testPrefix}_CREATEDAT"`
        ).then((createdQuestion) => {
          metabaseTestQuestion = createdQuestion;
        });
      });
    });

    afterEach(() => {
      const attributesToRemove = [sendinblueExistingAttributeName, ...metabaseAttributesNames];
      return Promise.all([
        cleanFolderAndCollection(clients),
        mapSeries(attributesToRemove, (attributeName) => {
          return clients.sendinblue.removeContactAttribute(attributeName);
        })
      ]);
    });

    it('should sync attributes', () => {
      if (!metabaseTestQuestion) {
        throw new Error('no metabaseTestQuestion, beforeEach() went wrong');
      }
      // we first add an attribute on sendinblue (that is not present in metabase -> we want to ensure it'll be deleted)
      return clients.sendinblue
        .createContactAttribute(sendinblueExistingAttributeName, 'text')
        .then(() => {
          // then we sync the attributes from metabase to sendinblue
          return syncAvailableAttributes(clients, metabaseTestQuestion!.id);
        })
        .then(() => {
          return clients.sendinblue.fetchContactAttributes().then((sendinblueAttributes) => {
            const attributesFromMetabase = sendinblueAttributes.filter((sendinblueAttribute) => {
              return metabaseAttributesNames.includes(sendinblueAttribute.name);
            });
            const existingSendinblueAttributeNotInMetabase = find(sendinblueAttributes, {
              name: sendinblueExistingAttributeName.toUpperCase()
            });

            // the attributes that were on metabase but not sendinblue should have been created,
            // and thus are present here:
            expect(attributesFromMetabase.sort((a, z) => z.name.localeCompare(a.name))).toMatchInlineSnapshot(`
  [
    {
      "category": "normal",
      "name": "${testPrefix}_FAVORITE_COLOR",
      "type": "text",
    },
    {
      "category": "normal",
      "name": "${testPrefix}_CREATEDAT",
      "type": "date",
    },
  ]
  `);
            // the attributes already on sendinblue but not on metabase shouldn't be removed,
            // because they are global and might be used by something else
            // (so this shouldn't be empty)
            expect(existingSendinblueAttributeNotInMetabase).toMatchInlineSnapshot(`
  {
    "category": "normal",
    "name": "${testPrefix}_DUMMY",
    "type": "text",
  }
  `);
          });
        });
    });
  });

  describe('sync contacts with attributes values (syncContactWithAttributesValues)', () => {
    const testPrefix = 'TEST_ATTRS_VALUES';
    let metabaseTestQuestion: MetabaseDetailedQuestion;
    let metabaseTestQuestionWithUpdatedAttributes: MetabaseDetailedQuestion;
    let sendinblueTestList: number;

    beforeEach(() => {
      return cleanFolderAndCollection(clients).then(() => {
        return Promise.all([
          clients.sendinblue.createContactList(testPrefix, config.sendinblue.testFolderId).then(({id}) => {
            sendinblueTestList = id;
          }),

          createTestMetabaseQuestion(
            'tests-e2e-all',
            `select *
              from (
                values
                ('A@hey.com', 'A', 1),
                ('B@hey.com', 'B', 2),
                ('C@hey.com', 'C', 3)
                ) as q ("email", "${testPrefix}_ATTR1", "${testPrefix}_ATTR2")
                `
          ).then((createdQuestion) => {
            metabaseTestQuestion = createdQuestion;
            return syncAvailableAttributes(clients, metabaseTestQuestion.id);
          }),

          createTestMetabaseQuestion(
            'tests-e2e-all',
            `select *
              from (
                values
                ('A@hey.com', 'A', null),
                ('B@hey.com', 'B', 4), -- previous is 2, should be updated
                ('C@hey.com', 'C', 3) -- no changes, should not appear in the contacts to update
                ) as q ("email", "${testPrefix}_ATTR1", "${testPrefix}_ATTR2")
                `
          ).then((createdQuestion) => {
            metabaseTestQuestionWithUpdatedAttributes = createdQuestion;
          })
        ]);
      });
    });

    afterEach(() => {
      const attributesToRemove = [`${testPrefix}_ATTR1`, `${testPrefix}_ATTR2`];
      return Promise.all([
        cleanFolderAndCollection(clients),
        // remove the test attributes on sendinblue
        mapSeries(attributesToRemove, (attributeName) => {
          return clients.sendinblue.removeContactAttribute(attributeName);
        })
      ]);
    });

    it('should sync the available contacts with attributes to update', () => {
      return clients.metabase
        .runQuestion(metabaseTestQuestion.id)
        .then((metabaseContacts) => {
          expect(metabaseContacts.sort(sortByEmail)).toMatchInlineSnapshot(`
  [
    {
      "TEST_ATTRS_VALUES_ATTR1": "A",
      "TEST_ATTRS_VALUES_ATTR2": 1,
      "email": "a@hey.com",
    },
    {
      "TEST_ATTRS_VALUES_ATTR1": "B",
      "TEST_ATTRS_VALUES_ATTR2": 2,
      "email": "b@hey.com",
    },
    {
      "TEST_ATTRS_VALUES_ATTR1": "C",
      "TEST_ATTRS_VALUES_ATTR2": 3,
      "email": "c@hey.com",
    },
  ]
  `);
          return syncAvailableAttributes(clients, metabaseTestQuestion.id).then(
            ({sendinblueAttributesFromMetabase}) => {
              expect(sendinblueAttributesFromMetabase).toMatchInlineSnapshot(`
  {
    "EMAIL": {
      "fromMetabaseValue": [Function],
      "type": "text",
    },
    "TEST_ATTRS_VALUES_ATTR1": {
      "fromMetabaseValue": [Function],
      "type": "text",
    },
    "TEST_ATTRS_VALUES_ATTR2": {
      "fromMetabaseValue": [Function],
      "type": "float",
    },
  }
  `);
              return syncContactWithAttributesValues(
                clients,
                metabaseContacts,
                [], // the sendinblue list is empty, no need to fetch the contacts
                sendinblueTestList,
                sendinblueAttributesFromMetabase
              ).then((sendinblueContactsWithUpdatedAttributes) => {
                expect(sendinblueContactsWithUpdatedAttributes.sort(sortByEmail)).toMatchInlineSnapshot(`
  [
    {
      "attributes": {
        "TEST_ATTRS_VALUES_ATTR1": "A",
        "TEST_ATTRS_VALUES_ATTR2": 1,
      },
      "email": "a@hey.com",
    },
    {
      "attributes": {
        "TEST_ATTRS_VALUES_ATTR1": "B",
        "TEST_ATTRS_VALUES_ATTR2": 2,
      },
      "email": "b@hey.com",
    },
    {
      "attributes": {
        "TEST_ATTRS_VALUES_ATTR1": "C",
        "TEST_ATTRS_VALUES_ATTR2": 3,
      },
      "email": "c@hey.com",
    },
  ]
  `);
              });
            }
          );
        })
        .then(() => {
          return clients.metabase.runQuestion(metabaseTestQuestionWithUpdatedAttributes.id).then((metabaseContacts) => {
            expect(metabaseContacts.sort(sortByEmail)).toMatchInlineSnapshot(`
  [
    {
      "TEST_ATTRS_VALUES_ATTR1": "A",
      "TEST_ATTRS_VALUES_ATTR2": null,
      "email": "a@hey.com",
    },
    {
      "TEST_ATTRS_VALUES_ATTR1": "B",
      "TEST_ATTRS_VALUES_ATTR2": 4,
      "email": "b@hey.com",
    },
    {
      "TEST_ATTRS_VALUES_ATTR1": "C",
      "TEST_ATTRS_VALUES_ATTR2": 3,
      "email": "c@hey.com",
    },
  ]
  `);
            return syncAvailableAttributes(clients, metabaseTestQuestionWithUpdatedAttributes.id).then(
              ({sendinblueAttributesFromMetabase}) => {
                expect(sendinblueAttributesFromMetabase).toMatchInlineSnapshot(`
  {
    "EMAIL": {
      "fromMetabaseValue": [Function],
      "type": "text",
    },
    "TEST_ATTRS_VALUES_ATTR1": {
      "fromMetabaseValue": [Function],
      "type": "text",
    },
    "TEST_ATTRS_VALUES_ATTR2": {
      "fromMetabaseValue": [Function],
      "type": "float",
    },
  }
  `);
                return clients.sendinblue.fetchContactsFromList(sendinblueTestList).then((sendinblueContacts) => {
                  expect(
                    sendinblueContacts
                      .map((c) => {
                        return pick(c, ['email', 'attributes']);
                      })
                      .sort(sortByEmail)
                  ).toMatchInlineSnapshot(`
  [
    {
      "attributes": {
        "TEST_ATTRS_VALUES_ATTR1": "A",
        "TEST_ATTRS_VALUES_ATTR2": 1,
      },
      "email": "a@hey.com",
    },
    {
      "attributes": {
        "TEST_ATTRS_VALUES_ATTR1": "B",
        "TEST_ATTRS_VALUES_ATTR2": 2,
      },
      "email": "b@hey.com",
    },
    {
      "attributes": {
        "TEST_ATTRS_VALUES_ATTR1": "C",
        "TEST_ATTRS_VALUES_ATTR2": 3,
      },
      "email": "c@hey.com",
    },
  ]
  `);
                  return syncContactWithAttributesValues(
                    clients,
                    metabaseContacts,
                    sendinblueContacts,
                    sendinblueTestList,
                    sendinblueAttributesFromMetabase
                  ).then((sendinblueContactsWithUpdatedAttributes) => {
                    expect(sendinblueContactsWithUpdatedAttributes.sort(sortByEmail)).toMatchInlineSnapshot(`
  [
    {
      "attributes": {
        "TEST_ATTRS_VALUES_ATTR1": "A",
        "TEST_ATTRS_VALUES_ATTR2": null,
      },
      "email": "a@hey.com",
    },
    {
      "attributes": {
        "TEST_ATTRS_VALUES_ATTR1": "B",
        "TEST_ATTRS_VALUES_ATTR2": 4,
      },
      "email": "b@hey.com",
    },
  ]
  `);
                  });
                });
              }
            );
          });
        });
    });
  });

  describe('sync contacts & ensure removed metabase contacts are removed on sendinblue', () => {
    beforeEach(() => {
      return cleanFolderAndCollection(clients);
    });

    afterEach(() => {
      return cleanFolderAndCollection(clients);
    });

    it('should remove contacts on sendinblue that were removed from metabase', () => {
      return createTestMetabaseQuestion(
        'tests-e2e-removed-contacts',
        `select * from (values('1@hey.com'), ('2@hey.com')) as q ("email")`
      ).then((metabaseQuestion) => {
        return syncAll(config.metabase.testCollectionId, config.sendinblue.testFolderId)
          .then((syncOutput) => {
            const contacts = sortOutput(syncOutput).map((o) => o.contacts);
            expect(contacts).toMatchInlineSnapshot(`
[
  {
    "removed": [],
    "upserted": [
      {
        "attributes": {},
        "email": "1@hey.com",
      },
      {
        "attributes": {},
        "email": "2@hey.com",
      },
    ],
  },
]
`);
          })
          .then(() => {
            return clients.metabase.updateQuestion(
              merge(metabaseQuestion, {
                dataset_query: {
                  native: {
                    query: `select * from (values('2@hey.com')) as q ("email")` // no more 1@hey.com
                  }
                }
              })
            );
          })
          .then(() => {
            return syncAll(config.metabase.testCollectionId, config.sendinblue.testFolderId);
          })
          .then((syncOutput) => {
            const outputSorted = sortOutput(syncOutput).map((o) => o.contacts);
            expect(outputSorted).toMatchInlineSnapshot(`
[
  {
    "removed": [
      {
        "attributes": {},
        "createdAt": "${outputSorted[0]?.removed[0]?.createdAt}",
        "email": "1@hey.com",
        "emailBlacklisted": false,
        "id": ${outputSorted[0]?.removed[0]?.id},
        "listIds": [
          ${outputSorted[0]?.removed[0]?.listIds[0]},
        ],
        "modifiedAt": "${outputSorted[0]?.removed[0]?.modifiedAt}",
        "smsBlacklisted": false,
      },
    ],
    "upserted": [],
  },
]
`);
          });
      });
    });
  });

  describe('sync everything (lists, attributes & contacts)', () => {
    const testPrefix = 'TEST_ALL';
    const sendinblueExistingAttributeName = `${testPrefix}_DUMMY`;
    const metabaseAttributes = [
      getMetabaseAttribute(1, `${testPrefix}_FAVORITE_COLOR`, 'type/Text'),
      getMetabaseAttribute(2, `${testPrefix}_CREATEDAT`, 'type/DateTimeWithTZ')
    ];
    const metabaseAttributesNames = map(metabaseAttributes, 'name');

    let metabaseTestQuestion: MetabaseDetailedQuestion | null;

    beforeEach(() => {
      // ensure we start with a clean state
      return cleanFolderAndCollection(clients).then(() => {
        return createTestMetabaseQuestion(
          'tests-e2e-all',
          `select *
                from (
                  values
                  ('1@hey.com', 'blue', '2022-11-03 16:02:12.056659+0'::timestamptz),
                  ('2@hey.com', 'red', '2022-12-09 17:32:12.056659+0'::timestamptz)
                ) as q ("email", "${testPrefix}_FAVORITE_COLOR", "${testPrefix}_CREATEDAT")
            `
        ).then((createdQuestion) => {
          console.log('createdQuestion', createdQuestion);
          metabaseTestQuestion = createdQuestion;
        });
      });
    });

    afterEach(() => {
      const attributesToRemove = [sendinblueExistingAttributeName, ...metabaseAttributesNames];
      return Promise.all([
        cleanFolderAndCollection(clients),
        // remove the test attributes on sendinblue
        mapSeries(attributesToRemove, (attributeName) => {
          return clients.sendinblue.removeContactAttribute(attributeName);
        })
      ]);
    });

    it('should sync all', () => {
      if (!metabaseTestQuestion) {
        throw new Error('no metabaseTestQuestion, beforeEach() went wrong');
      }

      return (
        clients.sendinblue
          // we create some attributes to ensure they'll be deleted
          .createContactAttribute(sendinblueExistingAttributeName, 'text')
          .then(() => syncAll(config.metabase.testCollectionId, config.sendinblue.testFolderId))
          .then((syncOutput) => {
            const outputSorted = sortOutput(syncOutput);
            expect(outputSorted).toMatchInlineSnapshot(`
[
  {
    "attributes": {
      "sendinblueAttributesFromMetabase": {
        "EMAIL": {
          "fromMetabaseValue": [Function],
          "type": "text",
        },
        "TEST_ALL_CREATEDAT": {
          "fromMetabaseValue": [Function],
          "type": "date",
        },
        "TEST_ALL_FAVORITE_COLOR": {
          "fromMetabaseValue": [Function],
          "type": "text",
        },
      },
      "sendinblueCreatedAttributes": {
        "EMAIL": {
          "fromMetabaseValue": [Function],
          "type": "text",
        },
        "TEST_ALL_CREATEDAT": {
          "fromMetabaseValue": [Function],
          "type": "date",
        },
        "TEST_ALL_FAVORITE_COLOR": {
          "fromMetabaseValue": [Function],
          "type": "text",
        },
      },
    },
    "contacts": {
      "removed": [],
      "upserted": [
        {
          "attributes": {
            "TEST_ALL_CREATEDAT": "2022-11-03",
            "TEST_ALL_FAVORITE_COLOR": "blue",
          },
          "email": "1@hey.com",
        },
        {
          "attributes": {
            "TEST_ALL_CREATEDAT": "2022-12-09",
            "TEST_ALL_FAVORITE_COLOR": "red",
          },
          "email": "2@hey.com",
        },
      ],
    },
    "metabaseQuestion": {
      "app_id": null,
      "collection_position": null,
      "collection_preview": true,
      "description": null,
      "display": "table",
      "entity_id": "${syncOutput[0]!.metabaseQuestion.entity_id}",
      "fully_parametrized": true,
      "id": ${syncOutput[0]!.metabaseQuestion.id},
      "last-edit-info": {
        "email": "${syncOutput[0]!.metabaseQuestion['last-edit-info'].email}",
        "first_name": "${syncOutput[0]!.metabaseQuestion['last-edit-info'].first_name}",
        "id": ${syncOutput[0]!.metabaseQuestion['last-edit-info'].id},
        "last_name": "${syncOutput[0]!.metabaseQuestion['last-edit-info'].last_name}",
        "timestamp": "${syncOutput[0]!.metabaseQuestion['last-edit-info'].timestamp}",
      },
      "model": "card",
      "moderated_status": null,
      "name": "tests-e2e-all",
    },
    "sendInBlueTargetedList": {
      "existed": false,
      "id": ${syncOutput[0]!.sendInBlueTargetedList.id},
    },
  },
]
`);
          })
      );
    });
  });
});
