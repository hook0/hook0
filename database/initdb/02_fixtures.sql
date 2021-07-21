BEGIN;

INSERT INTO event.organization(organization__id, name) VALUES ('8cf9a904-52c2-4ad9-a21a-4c12a7791334', 'Test orga');
INSERT INTO event.application(application__id, organization__id, name) VALUES ('dc8965e2-7fe4-4298-927c-e253e9c6f40d', '8cf9a904-52c2-4ad9-a21a-4c12a7791334', 'Test');
INSERT INTO event.application_secret(application__id, name, token) VALUES ('dc8965e2-7fe4-4298-927c-e253e9c6f40d', 'Test token', 'd426c1e1-7310-4b04-8698-0ea9068f898f');
INSERT INTO event.service(application__id, service__name) VALUES ('dc8965e2-7fe4-4298-927c-e253e9c6f40d', 'github');
INSERT INTO event.resource_type(application__id, service__name, resource_type__name) VALUES ('dc8965e2-7fe4-4298-927c-e253e9c6f40d', 'github', 'repo');
INSERT INTO event.verb(application__id, verb__name) VALUES ('dc8965e2-7fe4-4298-927c-e253e9c6f40d', 'starred');
INSERT INTO event.event_type(application__id, service__name, resource_type__name, verb__name) VALUES ('dc8965e2-7fe4-4298-927c-e253e9c6f40d', 'github', 'repo', 'starred');
INSERT INTO event.payload_content_type(payload_content_type__name, description) VALUES ('text/plain', 'Text');
INSERT INTO event.event(event__id, application__id, event_type__name, payload, payload_content_type__name, ip, occurred_at, application_secret__token, labels) VALUES ('073e238f-13d7-4040-97dd-7d348dd6555b', 'dc8965e2-7fe4-4298-927c-e253e9c6f40d', 'github.repo.starred', 'test', 'text/plain', '127.0.0.1', now() - interval '1 hour', 'd426c1e1-7310-4b04-8698-0ea9068f898f', '{"test": 0, "test2": 8}');

INSERT INTO webhook.subscription(subscription__id, application__id, label_key, label_value, target__id, metadata) VALUES ('91797663-e486-4f87-b427-1b4ab4849315', 'dc8965e2-7fe4-4298-927c-e253e9c6f40d', 'test', '0', 'a6c1604c-6a59-4c8d-83c1-df472233ebd9', '{"test": true}');
INSERT INTO webhook.subscription(subscription__id, application__id, label_key, label_value, target__id) VALUES ('3d8588e9-92c9-43b2-85a3-ab698e3e66de', 'dc8965e2-7fe4-4298-927c-e253e9c6f40d', 'test', '0', '0793594d-37f0-49dc-b87c-131428cf7eda');
INSERT INTO webhook.subscription(subscription__id, application__id, label_key, label_value, target__id) VALUES ('4bdbe960-5da7-4c6d-997f-1f9711f1b77b', 'dc8965e2-7fe4-4298-927c-e253e9c6f40d', 'test2', '8', '7bcc3ac1-5c6b-4cd1-86b4-6c554c4677eb');
INSERT INTO webhook.subscription(subscription__id, application__id, label_key, label_value, target__id) VALUES ('61cfe8b1-2089-4065-a8aa-6a2ba58b25f1', 'dc8965e2-7fe4-4298-927c-e253e9c6f40d', 'test2', '1', '3fee5405-7089-404a-bc3c-f3eeeeceff16');
INSERT INTO webhook.subscription__event_type(subscription__id, event_type__name) VALUES ('91797663-e486-4f87-b427-1b4ab4849315', 'github.repo.starred');
INSERT INTO webhook.target_http(target__id, method, url, headers) VALUES ('a6c1604c-6a59-4c8d-83c1-df472233ebd9', 'POST', 'http://localhost:7777/path', '{"headerName":"headerValue","headerName2":"headerValue2"}');
INSERT INTO webhook.target_http(target__id, method, url, headers) VALUES ('0793594d-37f0-49dc-b87c-131428cf7eda', 'POST', 'http://localhost:7777/path', '{"headerName":"headerValue","headerName2":"headerValue2"}');
INSERT INTO webhook.target_http(target__id, method, url, headers) VALUES ('7bcc3ac1-5c6b-4cd1-86b4-6c554c4677eb', 'POST', 'http://localhost:7777/path', '{"headerName":"headerValue","headerName2":"headerValue2"}');
INSERT INTO webhook.target_http(target__id, method, url, headers) VALUES ('3fee5405-7089-404a-bc3c-f3eeeeceff16', 'POST', 'http://localhost:7777/path', '{"headerName":"headerValue","headerName2":"headerValue2"}');
INSERT INTO webhook.response_error(response_error__name) VALUES ('E_UNKNOWN'), ('E_INVALID_TARGET'), ('E_CONNECTION'), ('E_TIMEOUT'), ('E_HTTP');
INSERT INTO webhook.request_attempt(event__id, subscription__id, request_attempt__id) VALUES ('073e238f-13d7-4040-97dd-7d348dd6555b', '91797663-e486-4f87-b427-1b4ab4849315', '8536a6a6-e7ec-4cea-b984-d7f377f394e4');
INSERT INTO webhook.request_attempt(event__id, subscription__id) VALUES ('073e238f-13d7-4040-97dd-7d348dd6555b', '91797663-e486-4f87-b427-1b4ab4849315');
INSERT INTO webhook.request_attempt(event__id, subscription__id) VALUES ('073e238f-13d7-4040-97dd-7d348dd6555b', '91797663-e486-4f87-b427-1b4ab4849315');

insert into event.event(application__id, event_type__name, payload, payload_content_type__name, ip, occurred_at, application_secret__token, labels) values ('dc8965e2-7fe4-4298-927c-e253e9c6f40d', 'github.repo.starred', 'test', 'text/plain', '127.0.0.1', now() - interval '1 hour', 'd426c1e1-7310-4b04-8698-0ea9068f898f', '{"test": 0, "test2": 8}');

COMMIT;
