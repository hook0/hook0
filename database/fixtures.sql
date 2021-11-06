begin;

insert into event.organization(organization__id, name) values ('8cf9a904-52c2-4ad9-a21a-4c12a7791334', 'test orga');
insert into event.application(application__id, organization__id, name) values ('dc8965e2-7fe4-4298-927c-e253e9c6f40d', '8cf9a904-52c2-4ad9-a21a-4c12a7791334', 'test');
insert into event.application_secret(application__id, name, token) values ('dc8965e2-7fe4-4298-927c-e253e9c6f40d', 'test token', 'd426c1e1-7310-4b04-8698-0ea9068f898f');
insert into event.service(application__id, service__name) values ('dc8965e2-7fe4-4298-927c-e253e9c6f40d', 'github');
insert into event.resource_type(application__id, service__name, resource_type__name) values ('dc8965e2-7fe4-4298-927c-e253e9c6f40d', 'github', 'repo');
insert into event.verb(application__id, verb__name) values ('dc8965e2-7fe4-4298-927c-e253e9c6f40d', 'starred');
insert into event.event_type(application__id, service__name, resource_type__name, verb__name) values ('dc8965e2-7fe4-4298-927c-e253e9c6f40d', 'github', 'repo', 'starred');
insert into event.payload_content_type(payload_content_type__name, description) values ('text/plain', 'text');
insert into event.event(event__id, application__id, event_type__name, payload, payload_content_type__name, ip, occurred_at, application_secret__token, labels) values ('073e238f-13d7-4040-97dd-7d348dd6555b', 'dc8965e2-7fe4-4298-927c-e253e9c6f40d', 'github.repo.starred', 'test', 'text/plain', '127.0.0.1', now() - interval '1 hour', 'd426c1e1-7310-4b04-8698-0ea9068f898f', '{"test": 0, "test2": 8}');

insert into webhook.subscription(subscription__id, application__id, label_key, label_value, target__id, metadata) values ('91797663-e486-4f87-b427-1b4ab4849315', 'dc8965e2-7fe4-4298-927c-e253e9c6f40d', 'test', '0', 'a6c1604c-6a59-4c8d-83c1-df472233ebd9', '{"test": true}');
insert into webhook.subscription(subscription__id, application__id, label_key, label_value, target__id) values ('3d8588e9-92c9-43b2-85a3-ab698e3e66de', 'dc8965e2-7fe4-4298-927c-e253e9c6f40d', 'test', '0', '0793594d-37f0-49dc-b87c-131428cf7eda');
insert into webhook.subscription(subscription__id, application__id, label_key, label_value, target__id) values ('4bdbe960-5da7-4c6d-997f-1f9711f1b77b', 'dc8965e2-7fe4-4298-927c-e253e9c6f40d', 'test2', '8', '7bcc3ac1-5c6b-4cd1-86b4-6c554c4677eb');
insert into webhook.subscription(subscription__id, application__id, label_key, label_value, target__id) values ('61cfe8b1-2089-4065-a8aa-6a2ba58b25f1', 'dc8965e2-7fe4-4298-927c-e253e9c6f40d', 'test2', '1', '3fee5405-7089-404a-bc3c-f3eeeeceff16');
insert into webhook.subscription__event_type(subscription__id, event_type__name) values ('91797663-e486-4f87-b427-1b4ab4849315', 'github.repo.starred');
insert into webhook.target_http(target__id, method, url, headers) values ('a6c1604c-6a59-4c8d-83c1-df472233ebd9', 'POST', 'http://localhost:7777/path', '{"headername":"headervalue","headername2":"headervalue2"}');
insert into webhook.target_http(target__id, method, url, headers) values ('0793594d-37f0-49dc-b87c-131428cf7eda', 'POST', 'http://localhost:7777/path', '{"headername":"headervalue","headername2":"headervalue2"}');
insert into webhook.target_http(target__id, method, url, headers) values ('7bcc3ac1-5c6b-4cd1-86b4-6c554c4677eb', 'POST', 'http://localhost:7777/path', '{"headername":"headervalue","headername2":"headervalue2"}');
insert into webhook.target_http(target__id, method, url, headers) values ('3fee5405-7089-404a-bc3c-f3eeeeceff16', 'POST', 'http://localhost:7777/path', '{"headername":"headervalue","headername2":"headervalue2"}');
insert into webhook.response_error(response_error__name) values ('E_UNKNOWN'), ('E_INVALID_TARGET'), ('E_CONNECTION'), ('E_TIMEOUT'), ('E_HTTP');
insert into webhook.request_attempt(event__id, subscription__id, request_attempt__id) values ('073e238f-13d7-4040-97dd-7d348dd6555b', '91797663-e486-4f87-b427-1b4ab4849315', '8536a6a6-e7ec-4cea-b984-d7f377f394e4');
insert into webhook.request_attempt(event__id, subscription__id) values ('073e238f-13d7-4040-97dd-7d348dd6555b', '91797663-e486-4f87-b427-1b4ab4849315');
insert into webhook.request_attempt(event__id, subscription__id) values ('073e238f-13d7-4040-97dd-7d348dd6555b', '91797663-e486-4f87-b427-1b4ab4849315');

insert into event.event(application__id, event_type__name, payload, payload_content_type__name, ip, occurred_at, application_secret__token, labels) values ('dc8965e2-7fe4-4298-927c-e253e9c6f40d', 'github.repo.starred', 'test', 'text/plain', '127.0.0.1', now() - interval '1 hour', 'd426c1e1-7310-4b04-8698-0ea9068f898f', '{"test": 0, "test2": 8}');

commit;
