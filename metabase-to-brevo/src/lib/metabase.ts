import axios, {AxiosRequestConfig, AxiosResponse} from 'axios';
import {delay, mapSeries, Promise} from 'bluebird';
import {onAxiosError, onError} from './error';
import {logger} from './logger';

export type MetabaseConfig = {
  host: string;
  username: string;
  password: any;
  collectionId: number;
  testCollectionId: number;
  testDatabaseId: number;
};

export type MetabaseQuestion = {
  id: number;
  collection_position: number | null;
  collection_preview: boolean;
  description: string;
  display: string;
  entity_id: string;
  fully_parametrized: boolean;
  model: string;
  moderated_status: string | null;
  name: string;
  'last-edit-info': {
    id: number;
    last_name: string;
    first_name: string;
    email: string;
    timestamp: string;
  };
};

export type MetabaseAvailableAttributeTypes =
  | 'type/Boolean'
  | 'type/Date'
  | 'type/DateTime'
  | 'type/DateTimeWithTZ'
  | 'type/DateTimeWithLocalTZ'
  | 'type/Decimal'
  | 'type/Float'
  | 'type/Integer'
  | 'type/Time'
  | 'type/TimeWithTZ'
  | 'type/Text'
  | 'type/IPAddress'
  | 'type/UUID';

export type MetabaseAttribute = {
  description: string | null;
  semantic_type: string | null;
  unit?: string;
  coercion_strategy: string | null;
  name: string;
  settings: string | null;
  field_ref: any[];
  effective_type: string;
  id: number;
  visibility_type: string;
  display_name: string;
  fingerprint: {
    global: any;
    type?: any;
  } | null;
  base_type: MetabaseAvailableAttributeTypes;
};

export type MetabaseDetailedQuestion = {
  description: string;
  archived: boolean;
  collection_position: number | null;
  table_id: number;
  result_metadata: MetabaseAttribute[];
  creator: {
    email: string;
    first_name: string;
    last_login: string;
    is_qbnewb: boolean;
    is_superuser: boolean;
    id: number;
    last_name: string;
    date_joined: string;
    common_name: string;
  };
  can_write: boolean;
  database_id: number;
  enable_embedding: boolean;
  collection_id: number;
  query_type: string;
  name: string;
  last_query_start: string;
  dashboard_count: 0;
  average_query_time: number;
  creator_id: number;
  moderation_reviews: any[];
  updated_at: string;
  made_public_by_id: null;
  embedding_params: null;
  cache_ttl: null;
  dataset_query: {
    database: number;
    query?: {
      'source-table': number;
      filter: any[];
    } | null;
    native?: {
      query: string;
      'template-tags': any;
    };
    type: string;
  };
  id: number;
  parameter_mappings: any[];
  display: string;
  entity_id: string;
  collection_preview: boolean;
  'last-edit-info': {
    id: number;
    email: string;
    first_name: string;
    last_name: string;
    timestamp: string;
  };
  visualization_settings: {
    'table.pivot_column'?: string;
    'table.cell_column'?: string;
  };
  collection: {
    authority_level: null;
    description: string;
    archived: boolean;
    slug: string;
    color: string;
    name: string;
    personal_owner_id: null;
    id: number;
    entity_id: string;
    location: string;
    namespace: null;
  };
  parameters: any[];
  dataset: boolean;
  created_at: string;
  public_uuid: string | null;
};

export type MetabaseCreateQuestionPayload =
  // these fields are required
  Pick<MetabaseDetailedQuestion, 'name' | 'dataset_query' | 'collection_id' | 'visualization_settings' | 'display'> &
    // and these ones are optional
    Partial<Pick<MetabaseDetailedQuestion, 'description' | 'parameters' | 'result_metadata'>>;

export type MetabaseUpdateQuestionPayload = {id: number} & MetabaseCreateQuestionPayload;

export type MetabaseContact = {
  email: string;
  [additionalProperties: string]: string | number | boolean;
};

export type MetabaseContactList = MetabaseQuestion & {
  contacts: MetabaseContact[];
};

export class MetabaseClient {
  private token: string | null = null;

  constructor(private config: MetabaseConfig) {}

  private makeRequest(axiosConfig: AxiosRequestConfig, retries = 0): Promise<AxiosResponse> {
    logger.info(`making request on metabase: ${JSON.stringify(axiosConfig)}`);
    return (this.token ? Promise.resolve() : this.authenticate())
      .then(() => {
        return axios({
          ...axiosConfig,
          headers: {
            ...axiosConfig.headers,
            'X-Metabase-Session': this.token
          }
        });
      })
      .catch(onAxiosError('cannot make request on metabase', axiosConfig))
      .catch((error) => {
        logger.error(`error making request to metabase: ${error}`);
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

  // https://www.metabase.com/docs/latest/api/session.html#post-apisession
  private authenticate(): Promise<void> {
    logger.info(`authenticating on metabase on ${this.config.host}`);
    return axios({
      method: 'POST',
      url: `${this.config.host}/api/session`,
      data: {
        username: this.config.username,
        password: this.config.password
      }
    })
      .then((res) => {
        this.token = res.data.id;
      })
      .catch(onError('could not authenticate on metabase'));
  }

  // https://www.metabase.com/docs/latest/api/card#get-apicard
  fetchQuestionsFromCollection(collectionId: number): Promise<MetabaseQuestion[]> {
    logger.info(`fetching metabase questions from collection ${collectionId}`);
    return this.makeRequest({
      method: 'GET',
      url: `${this.config.host}/api/collection/${collectionId}/items?models=card`
    })
      .then((response) => response.data.data as MetabaseQuestion[])
      .catch(onError(`cannot fetch questions from collection ${collectionId} on metabase`));
  }

  // // https://www.metabase.com/docs/latest/api/card#get-apicard
  fetchQuestion(questionId: number): Promise<MetabaseDetailedQuestion> {
    logger.info(`fetching metabase details of question ${questionId}`);
    return this.makeRequest({
      method: 'GET',
      url: `${this.config.host}/api/card/${questionId}`
    })
      .then((response) => {
        const question = response.data as MetabaseDetailedQuestion;
        return {
          ...question,
          result_metadata: (question.result_metadata || []).map((attribute) => {
            return {
              ...attribute,
              // on sendinblue created attributes are uppercased,
              // we do this here to avoid comparing with lowercased attributes latter
              name: attribute.name.toUpperCase()
            };
          })
        };
      })
      .catch(onError(`cannot fetch question ${questionId} on metabase`));
  }

  // https://www.metabase.com/docs/latest/api/card#post-apicardcard-idqueryexport-format
  runQuestion(questionId: number): Promise<MetabaseContact[]> {
    logger.info(`running metabase question ${questionId}`);
    return this.makeRequest({
      method: 'POST',
      url: `${this.config.host}/api/card/${questionId}/query/json`
    })
      .then((response) => {
        if (response.data.error) {
          throw new Error(`${response.data.error}`);
        }
        return response.data;
      })
      .then((contacts) => {
        return contacts.map((contact: MetabaseContact) => {
          return Object.keys(contact).reduce((acc, key) => {
            if (key === 'email') {
              acc.email = contact.email.toLowerCase();
            } else {
              const formattedKey = key
                .toUpperCase()
                .replace(/^.*→\s/, '')
                .replaceAll(' ', '_');
              acc[formattedKey] = contact[key];
            }
            return acc;
          }, {} as Partial<MetabaseContact>);
        });
      })
      .catch(onError(`cannot run question ${questionId} on metabase`));
  }

  // https://www.metabase.com/docs/latest/api/card#post-apicard
  createQuestion(questionPayload: MetabaseCreateQuestionPayload): Promise<MetabaseDetailedQuestion> {
    logger.info(`creating metabase question ${questionPayload.name}`);
    return this.makeRequest({
      method: 'POST',
      url: `${this.config.host}/api/card`,
      data: questionPayload
    })
      .then((response) => response.data as MetabaseDetailedQuestion)
      .catch(onError(`cannot create question ${questionPayload.name} on metabase`));
  }

  // https://www.metabase.com/docs/latest/api/card#put-apicardid
  updateQuestion(questionPayload: MetabaseUpdateQuestionPayload): Promise<MetabaseDetailedQuestion> {
    logger.info(`creating metabase question ${questionPayload.name}`);
    return this.makeRequest({
      method: 'PUT',
      url: `${this.config.host}/api/card/${questionPayload.id}`,
      data: questionPayload
    })
      .then((response) => response.data as MetabaseDetailedQuestion)
      .catch(onError(`cannot update question ${questionPayload.name} on metabase`));
  }

  // https://www.metabase.com/docs/latest/api/card#post-apicard
  removeQuestion(questionId: number): Promise<void> {
    logger.info(`removing metabase question ${questionId}`);
    return (
      this.makeRequest({
        method: 'DELETE',
        url: `${this.config.host}/api/card/${questionId}`
      })
        // res.data is empty
        .then(() => {})
        .catch(onError(`cannot remove question ${questionId} on metabase`))
    );
  }

  // https://www.metabase.com/docs/latest/api/card#post-apicard
  removeAllQuestionsOfCollection(collectionId: number): Promise<void> {
    logger.info(`removing all metabase questions of collection ${collectionId}`);
    return this.fetchQuestionsFromCollection(collectionId)
      .then((questions) => {
        return mapSeries(questions, (question) => {
          return this.removeQuestion(question.id);
        });
      })
      .then(() => {})
      .catch(onError(`cannot all metabase questions of collection ${collectionId}`));
  }
}
