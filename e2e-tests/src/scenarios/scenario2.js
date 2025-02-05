import create_application from '../services/applications/create_application.js';
import create_service_token from '../services/service_token/create_service_token.js';
import list_subscription, {
  list_subscriptions_fail,
} from '../services/subscriptions/list_subscription.js';
import attenuate_dashboard_service_token from '../services/service_token/attenuate/dashboard.js';
import { isNotNull } from '../utils/function.js';
import delete_application from '../services/applications/delete_application.js';

export function scenario_2(config) {
  const h = config.apiOrigin;
  const s = config.serviceToken;
  const o = config.organizationId;

  let application_id = null;
  let label_key = 'test';
  let label_value = 'true';

  let scenario_name = '[E2E-SCENARIO-2]';

  try {
    let service_token = create_service_token(h, s, o);
    if (!isNotNull(service_token)) {
      throw new Error(`${scenario_name} Failed to create service token`);
    }

    application_id = create_application(h, o, service_token);
    if (!isNotNull(application_id)) {
      throw new Error(`${scenario_name} Failed to create application`);
    }

    let attenuated_service_token = attenuate_dashboard_service_token(
      h,
      service_token,
      application_id,
      label_key,
      label_value
    );
    if (!isNotNull(attenuated_service_token)) {
      throw new Error(`${scenario_name} Failed to attenuate service token`);
    }

    let list_subscriptions = list_subscription(
      h,
      service_token,
      `?application_id=${application_id}`
    );
    if (!isNotNull(list_subscriptions)) {
      throw new Error(
        `${scenario_name} Failed to list subscriptions with unattenuated service token`
      );
    }

    let list_subscriptions_with_labels = list_subscription(
      h,
      attenuated_service_token,
      `?application_id=${application_id}&label_key=${label_key}&label_value=${label_value}`
    );
    if (!isNotNull(list_subscriptions_with_labels)) {
      throw new Error(
        `${scenario_name} Failed to list subscriptions attenuated service token and labels`
      );
    }

    let list_subscriptions_without_labels_fail = list_subscriptions_fail(
      h,
      attenuated_service_token,
      `?application_id=${application_id}`
    );
    if (!isNotNull(list_subscriptions_without_labels_fail)) {
      throw new Error(
        `${scenario_name} Expected failure when listing subscriptions with attenuated service token and no labels`
      );
    }

    let list_subscriptions_with_different_labels_fail = list_subscriptions_fail(
      h,
      attenuated_service_token,
      `?application_id=${application_id}&label_key=other_label&label_value=other_label`
    );
    if (!isNotNull(list_subscriptions_with_different_labels_fail)) {
      throw new Error(
        `${scenario_name} Expected failure when listing subscriptions with attenuated service token and different labels`
      );
    }

    if (application_id && !config.keepTestApplication) {
      // TODO: wait for the request attempts to be successful instead of waiting 3s
      setTimeout(() => {
        delete_application(h, application_id, s);
      }, 3000);
    }
  } catch (error) {
    console.error(error.message);
    if (application_id && !config.keepTestApplication) {
      delete_application(h, application_id, s);
    }
    throw error;
  }
}
