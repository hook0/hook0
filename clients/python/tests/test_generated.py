"""What the generated request layer puts on the wire, and what it does with what comes back.

The generated half is handed a transport and nothing else, so these cases hand it the real ones and
watch a real API answer: the path it interpolated, the query it assembled, the credential it
carried, the type it read back, and the exception it raised when the answer was a problem.

Nothing here names an operation the API does not declare. What is exercised is reached through the
generated package, so an operation that stops being declared takes its case with it rather than
leaving one that compiles against nothing.
"""

from __future__ import annotations

import asyncio
import uuid
from typing import Any

import pytest
from conftest import FakeHook0Api, ScriptedResponse, TEST_TIMEOUT

from hook0 import AsyncHttpTransport, HttpTransport
from hook0.generated import ApplicationsApi, ApplicationsAsyncApi, ProblemError
from hook0.generated.errors import NotFoundError
from hook0.generated.models import ApplicationInfo

pytestmark = pytest.mark.timeout(TEST_TIMEOUT)

APPLICATION_ID = "3f2504e0-4f89-41d3-9a0c-0305e82c3301"
ORGANIZATION_ID = "3f2504e0-4f89-41d3-9a0c-0305e82c3302"


def an_application() -> dict[str, Any]:
    return {
        "application_id": APPLICATION_ID,
        "name": "an application",
        "organization_id": ORGANIZATION_ID,
        "consumption": {"events_per_day": 12},
        "onboarding_steps": {"event": "Done", "event_type": "ToDo", "subscription": "ToDo"},
        "quotas": {"days_of_events_retention_limit": 7, "events_per_day_limit": 100},
    }


def a_problem() -> dict[str, Any]:
    return {
        "id": "NotFound",
        "title": "Not found",
        "detail": "This application does not exist.",
        "status": 404,
        "type": "https://documentation.hook0.com/problems",
    }


class Group:
    """One of the two generated groups, asked the same things in the same words."""

    def __init__(self, base_url: str, awaiting: bool) -> None:
        self.awaiting = awaiting
        self.group: Any = (
            ApplicationsAsyncApi(AsyncHttpTransport(base_url, "token-xyz"))
            if awaiting
            else ApplicationsApi(HttpTransport(base_url, "token-xyz"))
        )

    def get(self, application_id: str) -> ApplicationInfo:
        if self.awaiting:
            return asyncio.run(self.group.get(application_id))
        return self.group.get(application_id)

    def listing(self, organization_id: str) -> list[Any]:
        if self.awaiting:
            return asyncio.run(self.group.list(organization_id))
        return self.group.list(organization_id)


@pytest.fixture
def group(api: FakeHook0Api, awaiting: bool) -> Group:
    return Group(api.base_url, awaiting)


def test_a_generated_method_reads_back_the_type_the_api_declares(api: FakeHook0Api, group: Group) -> None:
    api.will_answer(ScriptedResponse(200, an_application()))

    application = group.get(APPLICATION_ID)

    assert isinstance(application, ApplicationInfo)
    assert application.application_id == uuid.UUID(APPLICATION_ID)
    assert application.quotas.events_per_day_limit == 100
    assert application.onboarding_steps.event.value == "Done"


def test_a_generated_method_fills_the_path_and_carries_the_credential(api: FakeHook0Api, group: Group) -> None:
    api.will_answer(ScriptedResponse(200, an_application()))

    group.get(APPLICATION_ID)

    request = api.received[0]
    assert request.method == "GET"
    # The identifier lands in the path rather than staying as the placeholder that named it.
    assert request.target == f"/api/v1/applications/{APPLICATION_ID}"
    assert request.headers["authorization"] == "Bearer token-xyz"


def test_a_generated_method_assembles_the_query_the_operation_declares(api: FakeHook0Api, group: Group) -> None:
    api.will_answer(ScriptedResponse(200, []))

    applications = group.listing(ORGANIZATION_ID)

    assert applications == []
    assert api.received[0].target == f"/api/v1/applications/?organization_id={ORGANIZATION_ID}"


def test_a_generated_method_reads_a_list_of_the_type_the_api_declares(api: FakeHook0Api, group: Group) -> None:
    api.will_answer(
        ScriptedResponse(
            200,
            [{"application_id": APPLICATION_ID, "name": "an application", "organization_id": ORGANIZATION_ID}],
        )
    )

    applications = group.listing(ORGANIZATION_ID)

    assert len(applications) == 1
    assert applications[0].name == "an application"


def test_a_problem_the_api_reports_is_raised_as_the_exception_it_names(api: FakeHook0Api, group: Group) -> None:
    api.will_answer(ScriptedResponse(404, a_problem()))

    with pytest.raises(NotFoundError) as reported:
        group.get(APPLICATION_ID)

    assert reported.value.status == 404
    assert reported.value.problem is not None
    assert reported.value.problem.detail == "This application does not exist."
    # Every problem is a kind of the one exception a caller may catch instead of naming each.
    assert isinstance(reported.value, ProblemError)


def test_a_failure_that_is_not_a_problem_document_is_still_reported(api: FakeHook0Api, group: Group) -> None:
    api.will_answer(ScriptedResponse(502, "a gateway wrote this, and it is not a problem document"))

    with pytest.raises(ProblemError) as reported:
        group.get(APPLICATION_ID)

    assert reported.value.status == 502
    assert reported.value.problem is None
