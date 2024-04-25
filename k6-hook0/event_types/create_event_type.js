import http from "k6/http";
import { check } from "k6";
import {generateRandomLetters} from "../utils/random_letters.js";

export default function (baseUrl, application_secret, application_id) {
  const url = `${baseUrl}api/v1/event_types/`;
  const payload = JSON.stringify({
    application_id: application_id,
    service: "test_k6_"+generateRandomLetters(5),
    resource_type: generateRandomLetters(5),
    verb: generateRandomLetters(5),
  });

  const params = {
    headers: {
      "Content-Type": "application/json",
      Authorization: application_secret,
    },
  };

  let res = http.post(url, payload, params);

  if(!check(res, {
    "Create event type is successful": (r) => r.status === 201 && r.body && r.body.includes('resource_type_name') && r.body.includes('service_name') && r.body.includes('verb_name') && r.body.includes('event_type_name'),
  })) {
    return null;
  }

  return JSON.parse(res.body).event_type_name;
}