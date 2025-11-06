alter table webhook.response alter column body type text using (convert_from(body, 'UTF8'));
