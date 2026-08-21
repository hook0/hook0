"""The one failure this client reports.

Sending an event, upserting an event type and verifying a webhook all raise the same type, so a
caller has one thing to catch whatever it asked the client to do. The failures the *API* reports
are a different matter: those are the problems it names in its own error contract, and the
generated half of this package raises one exception per problem.
"""

from __future__ import annotations


class Hook0ClientError(Exception):
    """Something the client was asked to do that it could not do.

    The constructors below are what the client raises through; each one exists so that the same
    situation always reads the same way, whichever call it came out of.
    """

    @classmethod
    def event_sending(cls, event_id: str, detail: str) -> Hook0ClientError:
        """A send the API refused for a reason repeating it would not change."""
        return cls(f"Sending event {event_id} failed: {detail}")

    @classmethod
    def retries_exhausted(cls, event_id: str, attempts: int, waited: float, detail: str) -> Hook0ClientError:
        """A send that ran out of attempts, or out of the delay budget its attempts share.

        A send that gave up and a single refused request are otherwise indistinguishable to a
        caller, which is the difference between a transient outage and a request that will never
        be accepted.
        """
        return cls(
            f"Sending event {event_id} failed: gave up after {attempts} attempts "
            f"spread over {waited:.3f}s of retry delay; last failure: {detail}"
        )

    @classmethod
    def payload_too_large(cls, event_id: str, size: int, maximum: int) -> Hook0ClientError:
        """A payload above what the client agrees to send, refused before a socket is opened."""
        return cls(
            f"Sending event {event_id} failed: event payload is {size} bytes, which is more than "
            f"the {maximum} bytes this client sends at most; nothing was sent"
        )

    @classmethod
    def invalid_event_type(cls, event_type: str) -> Hook0ClientError:
        """An event type that does not read as `service.resource_type.verb`."""
        return cls(f"Provided event type '{event_type}' does not have a valid syntax (service.resource_type.verb)")

    @classmethod
    def available_event_types(cls, detail: str) -> Hook0ClientError:
        """The list of event types the application already declares could not be read."""
        return cls(f"Getting available event types failed: {detail}")

    @classmethod
    def creating_event_type(cls, event_type: str, detail: str) -> Hook0ClientError:
        """An event type that could not be created."""
        return cls(f"Creating event type '{event_type}' failed: {detail}")
