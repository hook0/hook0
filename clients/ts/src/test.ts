import { Hook0Client, Event } from './lib';

async function main() {
  let client = new Hook0Client(
    'http://localhost:8080/api/v1/',
    'f23c5bd2-8145-486e-a68c-2afb7ce94ddc',
    'e9ba139e-f9d3-4904-9a10-89ef3dca6585'
  );

  let event = new Event('t.t.t', '{"test": true}', 'application/json', {});

  try {
    console.log('Sending event...');
    await client.sendEvent(event);
    console.log('Event sent successfully');
  } catch (error) {
    console.error('Error sending event:', error);
  }

  let eventTypes = ['test.t.c', 'test.t.d'];

  try {
    await client.upsertEventTypes(eventTypes);
  } catch (error) {
    console.error('Error getting available event types:', error);
  }
}

main();
