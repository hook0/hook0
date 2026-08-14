export {
  DEFAULT_MAX_PAYLOAD_BYTES,
  DEFAULT_MAX_RESPONSE_BYTES,
  DEFAULT_REQUEST_TIMEOUT_MS,
  MAX_ATTEMPTS_CAP,
  MAX_HEAD_BYTES,
  MAX_HEADER_BYTES,
  MAX_RESPONSE_HEADERS,
  Hook0ClientError,
  Hook0Client,
  Hook0ClientOptions,
  RetryPolicy,
  Event,
  EventType,
  verifyWebhookSignature,
  verifyWebhookSignatureWithCurrentTime,
} from './lib';
export { Signature } from './signature';

/**
 * Everything the API document describes, written by the SDK generator and never by hand.
 *
 * It is handed out as one namespace rather than flattened into this module on purpose. The document
 * declares schemas called `Event` and `EventType`, which are the API's own resources and not the
 * `Event` an emitter fills in here; re-exporting them side by side would either be refused outright
 * or, under a star, let whichever one lost disappear from this contract without a word. Under a
 * namespace, every name the document declares is reachable, unambiguous, and safe for the API to
 * add to.
 */
import * as generated from './generated';

export { generated };
